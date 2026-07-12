import * as assert from "assert";

import { isExecutableName } from "../../extension";

suite("server path resolution", () => {
  test("recognizes executable names", () => {
    assert.equal(isExecutableName("typos-lsp"), true);
    assert.equal(isExecutableName("typos-lsp.exe"), true);
    assert.equal(isExecutableName("./typos-lsp"), false);
    assert.equal(isExecutableName("bin/typos-lsp"), false);
    assert.equal(isExecutableName("bin\\typos-lsp.exe"), false);
    assert.equal(isExecutableName("C:\\tools\\typos-lsp.exe"), false);
    assert.equal(isExecutableName("C:typos-lsp.exe"), false);
  });
});
