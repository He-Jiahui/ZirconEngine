import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { lstat, readFile, realpath, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const app = resolve(repo, 'tools/penpot/apps/zircon-zui-plugin');
const cliArgs = process.argv.slice(2);
assert.ok(cliArgs.length === 2 && cliArgs[0] === '--output-dir' && cliArgs[1],
  'build-normal-workbench-contracts.mjs --output-dir <existing approved artifact directory>');
const output = resolve(cliArgs[1]);
const node = 'D:/cargo-targets/zircon-local/penpot-public/tools/node/node-v24.19.0-win-x64/node.exe';
const nodeSha = '3602f2bb1a10f2cbab4c36886218a33c1ab3db87290e73b033c46c77147d0237';
const esbuildPath = 'D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot/node_modules/.pnpm/esbuild@0.28.1/node_modules/esbuild/lib/main.js';
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const key = (path) => path.replaceAll('/', '\\').replace(/^\\\\\?\\/, '').replace(/[\\]+$/, '').toLowerCase();
async function physical(path) {
  const absolute = resolve(path);
  let ancestor = absolute;
  for (;;) {
    assert.equal((await lstat(ancestor)).isSymbolicLink(), false, `Reparse alias: ${path}`);
    const parent = dirname(ancestor);
    if (parent === ancestor) break;
    ancestor = parent;
  }
  assert.equal(key(await realpath(absolute)), key(absolute), `Nonphysical path: ${path}`);
  return absolute;
}
const exports = [
  ['tools/zui-layout-review-contract.ts', ['validateLayoutReviewCase', 'caseSha256']],
  ['tools/zui-layout-workbench-presentation.ts', ['validateWorkbenchPresentation']],
  ['tools/zui-layout-workbench-managed-inputs.ts', ['verifyWorkbenchManagedSceneFingerprint', 'assertWorkbenchManagedProjectPath']],
  ['tools/zui-layout-artifact-path.ts', ['requireApprovedPenpotArtifactDirectory', 'requireApprovedPenpotArtifactFile']],
  ['tools/zui-layout-component-hosts.ts', ['prepareComponentReviewHost']],
  ['tools/zui-layout-dependencies.ts', ['LayoutDependencies']],
  ['src/bridge/zui-layout-optimizer.ts', ['parseZuiLayoutSource']],
  ['src/bridge/zui-viewport-toolbar-publication.ts', ['readViewportToolbarPublication']],
];
const entry = exports.map(([path, names]) => `export { ${names.join(', ')} } from ${JSON.stringify(resolve(app, path))};`).join('\n');
async function main() {
  assert.match(key(output), /^[def]:\\cargo-targets\\zircon-local\\/);
  await physical(output);
  await physical(app);
  await physical(node);
  assert.equal(sha(await readFile(node)), nodeSha, 'Current verified Node binary changed');
  const esbuild = createRequire(import.meta.url)(await physical(esbuildPath));
  const inputs = new Map();
  const builderSource = await physical(fileURLToPath(import.meta.url));
  const builderBytes = await readFile(builderSource);
  inputs.set(key(builderSource), { path: builderSource, sha256: sha(builderBytes) });
  const captureInputs = { name: 'read-canonical-inputs-once', setup(build) {
    build.onLoad({ filter: /\.(?:[cm]?[jt]sx?|json)$/ }, async ({ path }) => {
      const actual = await physical(path);
      const bytes = await readFile(actual);
      inputs.set(key(actual), { path: actual, sha256: sha(bytes) });
      const extension = actual.match(/\.([^.]+)$/)[1];
      const loader = extension === 'json' ? 'json' : extension === 'tsx' ? 'tsx' : extension === 'jsx' ? 'jsx' : /^(?:m|c)?ts$/.test(extension) ? 'ts' : 'js';
      return { contents: bytes, loader, resolveDir: dirname(actual) };
    });
  } };
  const outfile = resolve(output, 'normal-workbench-contracts.mjs');
  const result = await esbuild.build({ absWorkingDir: repo, stdin: { contents: entry, resolveDir: app, sourcefile: 'normal-contract-entry.ts', loader: 'ts' },
    bundle: true, platform: 'node', format: 'esm', target: 'node24', packages: 'bundle', write: false, metafile: true, outfile,
    nodePaths: ['D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot/apps/zircon-zui-plugin/node_modules',
      'D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot/node_modules'], plugins: [captureInputs] });
  assert.equal(result.outputFiles.length, 1, 'Expected one compiled contract module');
  const producerSource = await physical(resolve(repo, 'tools/analysis/visual/normal-workbench-case-producer.mjs'));
  const producerPath = resolve(output, 'normal-workbench-case-producer.mjs');
  const producerResult = await esbuild.build({ absWorkingDir: repo, entryPoints: [producerSource],
    bundle: true, platform: 'node', format: 'esm', target: 'node24', write: false,
    metafile: true, outfile: producerPath, plugins: [captureInputs] });
  assert.equal(producerResult.outputFiles.length, 1, 'Expected one compiled Main producer');
  const compiledInputs = [...new Set([
    ...Object.keys(result.metafile.inputs).filter((path) => path !== '<stdin>' && !path.endsWith('normal-contract-entry.ts')),
    ...Object.keys(producerResult.metafile.inputs), builderSource,
  ])];
  const rows = [];
  for (const path of compiledInputs) {
    const actual = await physical(resolve(repo, path));
    assert.ok(!key(actual).startsWith(key(resolve(repo, 'dev/penpot/plugins')) + '\\'),
      `Reference-tree module cannot own a current contract: ${path}`);
    const input = inputs.get(key(actual));
    assert.ok(input, `Compiled input was not captured: ${path}`);
    assert.equal(sha(await readFile(actual)), input.sha256, `Source changed during compilation: ${path}`);
    rows.push(input);
  }
  rows.sort((a, b) => a.path.localeCompare(b.path, 'en'));
  assert.equal(new Set(rows.map((item) => key(item.path))).size, rows.length, 'Duplicate compiled input');
  const audit = { schema: 'dev.zircon.editor.normal-workbench-contract-inputs', version: 1,
    repoRoot: repo, canonicalAppRoot: app, outputPath: outfile, outputSha256: sha(result.outputFiles[0].contents),
    inputs: rows, sourceCopiesCreated: false, entrySha256: sha(Buffer.from(entry)), esbuildVersion: esbuild.version };
  const inputPath = resolve(output, 'normal-workbench-contract-inputs.json');
  const configuration = { schema: 'dev.zircon.editor.normal-frame-case-producer', version: 1,
    node: { path: node, sha256: nodeSha }, producer: { path: producerPath, sha256: sha(producerResult.outputFiles[0].contents) },
    contract: { path: outfile, sha256: audit.outputSha256 },
    contractInputs: { path: inputPath, sha256: sha(Buffer.from(JSON.stringify(audit, null, 2) + '\n')) } };
  await writeFile(producerPath, producerResult.outputFiles[0].contents, { flag: 'wx', flush: true });
  await writeFile(outfile, result.outputFiles[0].contents, { flag: 'wx', flush: true });
  await writeFile(inputPath, JSON.stringify(audit, null, 2) + '\n', { flag: 'wx', flush: true });
  const configurationPath = resolve(output, 'normal-workbench-case-producer-config.json');
  await writeFile(configurationPath, JSON.stringify(configuration, null, 2) + '\n', { flag: 'wx', flush: true });
  console.log(JSON.stringify({ status: 'compiled-current-contracts', inputCount: rows.length,
    configurationPath, outputSha256: audit.outputSha256, sourceCopiesCreated: false, wholeUiAccepted: false }));
}
main().catch((error) => { console.error(error.stack ?? String(error)); process.exitCode = 1; });
