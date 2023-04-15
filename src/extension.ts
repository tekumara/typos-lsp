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

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext) {
    let name = "Typos";

    const outputChannel = window.createOutputChannel(name);
    context.subscriptions.push(outputChannel);

    context.subscriptions.push(
        workspace.onDidChangeConfiguration(
            async (e: ConfigurationChangeEvent) => {
                const restartTriggeredBy = [
                    "typos.path",
                    "typos.logLevel",
                ].find((s) => e.affectsConfiguration(s));

                if (restartTriggeredBy) {
                    await commands.executeCommand("typos.restart");
                }
            }
        )
    );

    context.subscriptions.push(
        commands.registerCommand("typos.restart", async () => {
            // can't stop if the client has previously failed to start
            if (client && client.needsStop()) {
                await client.stop();
            }

            try {
                client = createClient(name, outputChannel);
            } catch (err) {
                window.showErrorMessage(
                    `Typos: ${err instanceof Error ? err.message : err}`
                );
                return;
            }
            await client.start();
        })
    );

    client = createClient(name, outputChannel);

    // Start the client. This will also launch the server
    client.start();
}

function createClient(
    name: string,
    outputChannel: OutputChannel
): LanguageClient {
    const env = { ...process.env };

    let config = workspace.getConfiguration("typos");
    let path = config.get<null | string>("path");

    if (!path) {
        throw new Error(`Please specify the typos.path setting.`);
    }

    env.RUST_LOG = config.get("logLevel");

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

    return new LanguageClient("typos", name, serverOptions, clientOptions);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
