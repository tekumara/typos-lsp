import * as assert from "assert";
import { chmod, mkdtemp, mkdir, realpath, rm, writeFile } from "fs/promises";
import * as os from "os";
import * as path from "path";

import { isExecutableName, resolveExecutable } from "../../extension";

suite("server path resolution", () => {
  let root: string;
  let bin: string;

  setup(async () => {
    root = await mkdtemp(path.join(os.tmpdir(), "typos-lsp-test-"));
    bin = path.join(root, "bin");
    await mkdir(bin);
  });

  teardown(async () => {
    await rm(root, { recursive: true, force: true });
  });

  test("recognizes executable names", () => {
    assert.equal(isExecutableName("typos-lsp"), true);
    assert.equal(isExecutableName("typos-lsp.exe"), true);
    assert.equal(isExecutableName("./typos-lsp"), false);
    assert.equal(isExecutableName("bin/typos-lsp"), false);
    assert.equal(isExecutableName("bin\\typos-lsp.exe"), false);
    assert.equal(isExecutableName("C:\\tools\\typos-lsp.exe"), false);
    assert.equal(isExecutableName("C:typos-lsp.exe"), false);
  });

  test("resolves an executable from an absolute PATH entry", async () => {
    const executable = await createExecutable("typos-lsp");
    assert.equal(
      await resolveExecutable("typos-lsp", bin, process.platform, [], true),
      executable,
    );
  });

  test("ignores empty and relative PATH entries", async () => {
    const executable = await createExecutable("typos-lsp");
    const searchPath = ["", ".", bin].join(path.delimiter);
    assert.equal(
      await resolveExecutable(
        "typos-lsp",
        searchPath,
        process.platform,
        [],
        true,
      ),
      executable,
    );
  });

  test("accepts a quoted absolute PATH entry", async () => {
    const executable = await createExecutable("typos-lsp");
    assert.equal(
      await resolveExecutable(
        "typos-lsp",
        `"${bin}"`,
        process.platform,
        [],
        true,
      ),
      executable,
    );
  });

  test("resolves Windows .com and .exe suffixes", async () => {
    const com = await createExecutable("typos-lsp.com");
    const exe = await createExecutable("typos-lsp.exe");
    assert.equal(
      await resolveExecutable("typos-lsp", bin, "win32", [], true),
      com,
    );
    await rm(com);
    assert.equal(
      await resolveExecutable("typos-lsp", bin, "win32", [], true),
      exe,
    );
  });

  test("fails when the command is missing", async () => {
    await assert.rejects(
      resolveExecutable("missing", bin, process.platform, [], true),
      /missing was not found in PATH\./,
    );
  });

  test("rejects an untrusted workspace executable", async () => {
    await createExecutable("typos-lsp");
    await assert.rejects(
      resolveExecutable("typos-lsp", bin, process.platform, [root], false),
      /typos-lsp was not found in PATH\./,
    );
  });

  async function createExecutable(name: string): Promise<string> {
    const executable = path.join(bin, name);
    await writeFile(executable, "");
    await chmod(executable, 0o755);
    return await realpath(executable);
  }
});
