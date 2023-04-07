import { workspace, ExtensionContext } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	Executable
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {

    const serverId = "typos";
	const serverName = "Typos Language Server";

	const env = { ...process.env };

	// TODO: move into config
    env.RUST_LOG = "trace";

	let config = workspace.getConfiguration(serverId);
	let command = config.get<null | string>("path");

	if (!command) {
		throw new Error("Please specify typos.path setting.");
	}

	const run: Executable = {
		command: command,
		options: { env: env },
	};

	const serverOptions: ServerOptions = {
		run: run,
		// used when launched in debug mode
		debug: run
	};

	const clientOptions: LanguageClientOptions = {
		// Register the server for all documents
		documentSelector: [{ scheme: 'file', pattern: '**' }],
	};

	client = new LanguageClient(
		serverId,
		serverName,
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
