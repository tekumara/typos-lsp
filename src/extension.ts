import {
    window,
    workspace,
    commands,
    ConfigurationChangeEvent,
    ExtensionContext,
	OutputChannel,
} from "vscode";

import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from "vscode-languageclient/node";

let client: LanguageClient;


export function activate(context: ExtensionContext)  {
	let name = "Typos";

    const outputChannel = window.createOutputChannel(name);
    context.subscriptions.push(outputChannel);

    client = createClient(name, outputChannel);

    context.subscriptions.push(
        workspace.onDidChangeConfiguration(
            async (e: ConfigurationChangeEvent) => {
                const restartTriggeredBy = ["typos.path"].find((s) =>
                    e.affectsConfiguration(s)
                );

                if (restartTriggeredBy) {
                    await commands.executeCommand("typos.restart");
                }
            }
        )
    );

    context.subscriptions.push(
        commands.registerCommand("typos.restart", async () => {
            //void window.showInformationMessage("Restarting typos...");

			// don't stop if the client has previously failed to start
			if (client.needsStop()) {
            	await client.stop();
			}

			client = createClient(name, outputChannel);
            await client.start();
        })
    );

    // Start the client. This will also launch the server
    client.start();
}

function createClient(name: string, outputChannel: OutputChannel): LanguageClient {
    const env = { ...process.env };

    // TODO: move into config
    env.RUST_LOG = "trace";

    let config = workspace.getConfiguration("typos");
    let path = config.get<null | string>("path");

    if (!path) {
        throw new Error(`Please specify the typos.path setting.`);
    }

    const run: Executable = {
        command: path,
        options: { env: env },
    };

    const serverOptions: ServerOptions = {
        run: run,
        // used when launched in debug mode
        debug: run,
    };

    const clientOptions: LanguageClientOptions = {
        // Register the server for all documents
        // We use scheme = file to ignore Untitled documents because that generates
        // too much request chatter
        documentSelector: [{ scheme: "file", pattern: "**" }],
		outputChannel: outputChannel,
		traceOutputChannel: outputChannel,
    };

    return new LanguageClient(
        "typos",
        name,
        serverOptions,
        clientOptions
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
