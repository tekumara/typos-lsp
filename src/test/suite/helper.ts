import * as vscode from "vscode";
import * as path from "path";

export let doc: vscode.TextDocument;
export let editor: vscode.TextEditor;

/**
 * Activates the extension
 */
export async function activate(docUri: vscode.Uri) {
  const ext = vscode.extensions.getExtension("tekumara.typos-vscode")!;

  await ext.activate();
  try {
    doc = await vscode.workspace.openTextDocument(docUri);
    editor = await vscode.window.showTextDocument(doc);
    await sleep(100); // Wait for server activation
  } catch (e) {
    console.error(e);
  }
}

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export const getDocPath = (p: string) => {
  return path.resolve(__dirname, "../../../src/test/fixture", p);
};

export const getDocUri = (p: string) => {
  return vscode.Uri.file(getDocPath(p));
};
