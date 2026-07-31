import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const sourceCommit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const expectedCatalogHash = 'adfcb65467451e98d5b8ff14bd2fca5deee1c06215798d900428d25d4a584d52';
const expectedSourceHashes = {
  'src/sim/lockpick.ts': 'c3d3e3a8b1dd5382d8ab0870f2a95f65c60596d08791fea9277bbfae720c090b',
  'src/sim/content/delves/lockpick_tiers.ts': 'b9b2ff24883243b8011fd78986dd337516b773c641a001bfd2a69b7747b532a7',
  'src/sim/delves/lockpick_controller.ts': '4f44c2522299a732738f26f8d2523b317808e6e166d1acb9ddfea9b61de6ae4f',
  'src/sim/types.ts': '303321fb109f7bcce51a6871597be5b7b05f54ed1c6b86185387390cc47ccd90',
};
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_lockpick_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_lockpick_content.json');
const contentPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_lockpick_content.zr');
const check = process.argv.includes('--check');

const extract = () => {
  const result = spawnSync(process.execPath, [extractor], {
    encoding: 'utf8',
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: sourceCommit },
  });
  if (result.status !== 0) throw new Error(result.stderr || 'lockpick extractor failed');
  return JSON.parse(result.stdout);
};
const sha256 = (text) => createHash('sha256').update(text).digest('hex');
const sourceText = (path) =>
  execFileSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${path}`], { encoding: 'utf8' });
const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};
const writeOrCheck = (path, text) => {
  if (check) {
    assert(existsSync(path), `generated file missing: ${path}`);
    assert(readFileSync(path, 'utf8') === text, `generated file drifted: ${path}`);
  } else {
    writeFileSync(path, text, 'utf8');
  }
};
const uint = (value) => `<uint>${value}`;

function renderIndexLookup(name, items, valueFor, error) {
  const upperBound = Math.max(...items.map((item) => item.index)) + 1;
  const lines = [`pub ${name}(index: int, required: bool): int {`,
    `    if (!required || index < 0 || index >= ${upperBound}) { throw "${error}"; }`];
  for (const item of items) lines.push(`    if (index == ${item.index}) { return ${valueFor(item)}; }`);
  lines.push(`    throw "${error}";`, `}`, ``);
  return lines;
}

function renderContent(content) {
  const lines = ['// Generated source-locked Delve lockpick content contract.', ''];
  lines.push(...renderIndexLookup('actionDelta', content.actions, (action) => action.delta, 'woc lockpick action is invalid'));
  lines.push('pub actionName(index: int, required: bool): string {',
    `    if (!required || index < 0 || index >= ${content.actions.length}) { throw "woc lockpick action is invalid"; }`);
  for (const action of content.actions) lines.push(`    if (index == ${action.index}) { return "${action.id}"; }`);
  lines.push('    throw "woc lockpick action is invalid";', '}', '');
  lines.push(...renderIndexLookup('antePages', content.antes.map((ante, index) => ({ ...ante, index: index + 1 })), (ante) => ante.pages, 'woc lockpick ante is invalid'));
  lines.push(...renderIndexLookup('anteTries', content.antes.map((ante, index) => ({ ...ante, index: index + 1 })), (ante) => ante.tries, 'woc lockpick ante is invalid'));
  lines.push(...renderIndexLookup('anteStepTimeoutMilliseconds', content.antes.map((ante, index) => ({ ...ante, index: index + 1 })), (ante) => ante.step_timeout_ms, 'woc lockpick ante is invalid'));
  lines.push('pub anteLootTier(index: int, required: bool): string {',
    '    if (!required || index < 1 || index > 3) { throw "woc lockpick ante is invalid"; }');
  for (const ante of content.antes) lines.push(`    if (index == ${ante.ante}) { return "${ante.loot_tier}"; }`);
  lines.push('    throw "woc lockpick ante is invalid";', '}', '');
  lines.push('pub tierIndex(tierId: string, required: bool): int {',
    '    if (!required) { throw "woc lockpick tier is required"; }',
    '    return tierId == "heroic" ? 1 : 0;', '}', '');
  for (const [name, property] of [
    ['tierColumns', 'cols'], ['tierRows', 'rows'], ['tierWidth', 'width'],
    ['tierGateCount', 'gateCount'], ['tierVisibilityWindow', 'visibilityWindow'], ['tierTrapCount', 'trapCount'],
  ]) {
    lines.push(...renderIndexLookup(name, content.presets, (preset) => preset[property], 'woc lockpick tier is invalid'));
  }
  lines.push('pub tierActionAllowed(tierIndexValue: int, actionIndex: int, required: bool): bool {',
    `    if (!required || tierIndexValue < 0 || tierIndexValue >= ${content.presets.length} || actionIndex < 0 || actionIndex >= ${content.actions.length}) {`,
    '        throw "woc lockpick action availability is invalid";', '    }', '    return true;', '}', '');
  lines.push('pub rewardBonusMarksForAnte(index: int, required: bool): int {',
    '    if (!required || index < 1 || index > 3) { throw "woc lockpick ante is invalid"; }');
  for (const ante of content.antes) lines.push(`    if (index == ${ante.ante}) { return ${content.rewards[ante.loot_tier].bonusMarks}; }`);
  lines.push('    throw "woc lockpick ante is invalid";', '}', '');
  lines.push('pub rewardCopperNumeratorForAnte(index: int, required: bool): int {',
    '    if (!required || index < 1 || index > 3) { throw "woc lockpick ante is invalid"; }');
  for (const ante of content.antes) {
    const multiplier = content.rewards[ante.loot_tier].copperMult;
    lines.push(`    if (index == ${ante.ante}) { return ${Number.isInteger(multiplier) ? multiplier : multiplier * 2}; }`);
  }
  lines.push('    throw "woc lockpick ante is invalid";', '}', '');
  lines.push('pub rewardCopperDenominatorForAnte(index: int, required: bool): int {',
    '    if (!required || index < 1 || index > 3) { throw "woc lockpick ante is invalid"; }');
  for (const ante of content.antes) {
    const multiplier = content.rewards[ante.loot_tier].copperMult;
    lines.push(`    if (index == ${ante.ante}) { return ${Number.isInteger(multiplier) ? 1 : 2}; }`);
  }
  lines.push('    throw "woc lockpick ante is invalid";', '}', '');
  lines.push(`pub tickMilliseconds(required: bool): int { if (!required) { throw "woc lockpick tick is required"; } return ${content.tick_milliseconds}; }`,
    `pub baseSeedMultiplier(required: bool): uint { if (!required) { throw "woc lockpick seed is required"; } return ${uint(content.base_seed_multiplier)}; }`,
    `pub pageSeedMultiplier(required: bool): uint { if (!required) { throw "woc lockpick seed is required"; } return ${uint(content.page_seed_multiplier)}; }`,
    `pub retrySeedMultiplier(required: bool): uint { if (!required) { throw "woc lockpick seed is required"; } return ${uint(content.retry_seed_multiplier)}; }`, '');
  lines.push('pub contractTest(): int {',
    '    if (actionDelta(0, true) != -2 || actionDelta(4, true) != 2 ||',
    '        antePages(1, true) != 3 || anteTries(3, true) != 3 ||',
    '        anteStepTimeoutMilliseconds(2, true) != 6000 || anteLootTier(1, true) != "premium" ||',
    '        tierColumns(0, true) != 12 || tierColumns(1, true) != 16 ||',
    '        tierVisibilityWindow(0, true) != 4 || tierTrapCount(1, true) != 5 ||',
    `        tickMilliseconds(true) != ${content.tick_milliseconds} || baseSeedMultiplier(true) != ${uint(content.base_seed_multiplier)} ||`,
    `        retrySeedMultiplier(true) != ${uint(content.retry_seed_multiplier)}) { return -1; }`,
    '    return 1;', '}', '');
  return lines.join('\n');
}

const content = extract();
assert(content.actions.length === 5 && content.antes.length === 3 && content.presets.length === 2,
  'lockpick catalog shape drifted');
assert(content.presets[0].cols === 12 && content.presets[1].cols === 16 && content.tick_milliseconds === 50,
  'lockpick fixed contracts drifted');
const catalogHash = sha256(JSON.stringify(content));
assert(catalogHash === expectedCatalogHash, `lockpick catalog drifted: ${catalogHash}`);
const sourceHashes = {};
for (const [path, expected] of Object.entries(expectedSourceHashes)) {
  const actual = sha256(sourceText(path));
  assert(actual === expected, `lockpick source drifted: ${path}`);
  sourceHashes[path] = actual;
}
const catalog = {
  schema_version: 1,
  source_commit: sourceCommit,
  generated_by: 'examples/woc/tools/m7_lockpick_content_codegen.mjs',
  source_sha256: sourceHashes,
  content,
  catalog_sha256: catalogHash,
};
writeOrCheck(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
writeOrCheck(contentPath, renderContent(content));
