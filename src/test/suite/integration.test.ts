import * as vscode from "vscode";
import * as assert from "assert";
import { activate, getDocUri, sleep } from "./helper";

suite("VS Code Integration Tests", async () => {
  const docUri = getDocUri("diagnostics.txt");

  suiteSetup(async () => {
    await activate(docUri);
  });

  test("Diagnoses typo", async () => {
    await testDiagnostics(docUri, [
      {
        message: "`apropriate` should be `appropriate`",
        range: toRange(0, 11, 0, 21),
        severity: vscode.DiagnosticSeverity.Warning,
        source: "ex",
      },
      {
        message: "`fo` should be `of`, `for`, `do`, `go`, `to`",
        range: toRange(1, 0, 1, 2),
        severity: vscode.DiagnosticSeverity.Warning,
        source: "ex",
      },
    ]);
  });

  test("Auto fix applies corrections", async () => {
    let editor = vscode.window.activeTextEditor;

    if (!editor) {
      throw new Error("no active editor");
    }

    // place cursor on the spelling mistake
    let position = new vscode.Position(0, 13);
    let selection = new vscode.Selection(position, position);
    editor.selection = selection;

    // for GHA CI to work, we need to wait long enough for the
    // cursor to move to the spelling mistake, otherwise the
    // autofix won't trigger
    await sleep(1000);

    // trigger correction
    await vscode.commands.executeCommand("editor.action.autoFix");
    await sleep(100);

    let contents = vscode.window.activeTextEditor?.document.getText();
    assert.equal(contents, "this is an appropriate test\nfo typos\n");
  });
});

function toRange(sLine: number, sChar: number, eLine: number, eChar: number) {
  const start = new vscode.Position(sLine, sChar);
  const end = new vscode.Position(eLine, eChar);
  return new vscode.Range(start, end);
}

async function testDiagnostics(
  docUri: vscode.Uri,
  expectedDiagnostics: vscode.Diagnostic[],
) {
  const actualDiagnostics = vscode.languages.getDiagnostics(docUri);
  assert.equal(
    actualDiagnostics.length,
    expectedDiagnostics.length,
    "Missing diagnostics",
  );

  expectedDiagnostics.forEach((expectedDiagnostic, i) => {
    const actualDiagnostic = actualDiagnostics[i];
    assert.equal(actualDiagnostic.message, expectedDiagnostic.message);
    assert.deepEqual(actualDiagnostic.range, expectedDiagnostic.range);
    assert.equal(actualDiagnostic.severity, expectedDiagnostic.severity);
  });
}
