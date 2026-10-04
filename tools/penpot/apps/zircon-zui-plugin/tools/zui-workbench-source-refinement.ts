import { resolveLayoutCatalogPath } from './zui-layout-paths';
import assert from 'node:assert/strict';
import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { CatalogManifest } from './zui-layout-catalog';
import { parseZuiDocument, type ZuiTable } from '../src/bridge/zui-document';
import { bytesSha256 } from './zui-layout-evidence';
import { inline } from './zui-material-source-refinement';

const dense = '$editor.control.height.dense';
const fixed = (value: string | number) => ({
  min: value,
  preferred: value,
  max: value,
  stretch: 'Fixed',
});
const table = (value: unknown) => value as ZuiTable;
const targets = [
  'components/showcase/showcase_collections_section.zui',
  'components/showcase/showcase_input_section.zui',
  'components/showcase/showcase_selection_section.zui',
  'components/workbench/composites/animation/workbench_transport_controls.zui',
  'components/workbench/composites/animation/workbench_sample_weights.zui',
  'components/workbench/floating/workbench_command_palette.zui',
];

function refine(source: string) {
  const original = parseZuiDocument(source).document;
  const document = structuredClone(original);
  const nodes = document.nodes!;
  const rootId = Object.values(document.components!)[0].root;
  const root = nodes[rootId];
  const changes: string[] = [];
  if (document.components?.['WorkbenchTransportControls']) {
    for (const [id, node] of Object.entries(nodes)) {
      if (node.component !== 'WorkbenchIconButton') continue;
      node.layout!['height'] = { min: dense, max: dense, stretch: 'Stretch' };
      node.props!['layout_min_height'] = dense;
      node.props!['layout_min_width'] = dense;
      changes.push(
        `${id}: override inherited 32px minimum with the 28px dense lane bounds; preserve the 20px icon, 2px padding and click route`,
      );
    }
  } else if (document.components?.['WorkbenchSampleWeights']) {
    for (const [id, node] of Object.entries(nodes)) {
      if (id.endsWith('_row') && node.component === 'HorizontalGroup') {
        node.layout!['height'] = fixed(dense);
        changes.push(`${id}: align weight labels and track on a 28px row`);
      }
      if (node.component === 'WorkbenchProgressBar') {
        node.layout!['height'] = { min: dense, max: dense, stretch: 'Stretch' };
        node.props!['background_color'] = 'transparent';
        changes.push(
          `${id}: constrain the inherited progress control to its row and keep only the track surface`,
        );
      }
    }
  } else if (rootId === 'palette') {
    root.layout!['width'] = {
      ...table(root.layout!['width']),
      min: 0,
      stretch: 'Stretch',
    };
    const result = nodes['first_result'];
    assert.equal(result.props?.['label'], 'Open command');
    result.props!['text'] = result.props!['label'];
    changes.push(
      'palette: retain preferred/max width while allowing the actual host width below the previous 520px minimum',
    );
    changes.push(
      'first_result.text: display the existing Open command label instead of inheriting the generic List item text',
    );
  } else {
    assert.ok(rootId.endsWith('_section_root'));
    assert.ok(
      root.component === 'VerticalGroup' || root.component === 'ScrollableBox',
    );
    root.component = 'ScrollableBox';
    root.layout = {
      ...root.layout,
      clip: true,
      input_policy: 'Receive',
      container: {
        kind: 'ScrollableBox',
        axis: 'Vertical',
        gap: '$editor.density.gap.medium',
        scrollbar_visibility: 'Auto',
      },
      width: { min: 0, stretch: 'Stretch' },
      height: { min: 0, stretch: 'Stretch' },
    };
    changes.push(
      `${rootId}: make the existing full ordered content a real vertical scroll container constrained by its host; no child is removed or hidden`,
    );
    changes.push(
      'All original controls and event routes remain; scroll-before/after and native interaction evidence are still required',
    );
  }
  let section = '';
  const bareKeys = (value: unknown) =>
    inline(value).replace(/"([A-Za-z_][A-Za-z0-9_]*)" = /g, '$1 = ');
  const output = source
    .split(/\r?\n/)
    .map((line) => {
      const header = line.match(/^\[([^\]]+)\]$/);
      if (header) section = header[1];
      const assignment = line.match(/^(component|props|layout)\s*=/);
      if (!assignment || !section.startsWith('nodes.')) return line;
      const id = section.slice(6),
        key = assignment[1];
      if (
        JSON.stringify(original.nodes![id]?.[key]) ===
        JSON.stringify(nodes[id]?.[key])
      )
        return line;
      return `${key} = ${bareKeys(nodes[id][key])}`;
    })
    .join(source.includes('\r\n') ? '\r\n' : '\n');
  assert.deepEqual(parseZuiDocument(output).document, document);
  for (const [id, before] of Object.entries(original.nodes!)) {
    assert.deepEqual(nodes[id].events, before.events);
    assert.deepEqual(nodes[id].children, before.children);
    assert.deepEqual(nodes[id]['bindings'], before['bindings']);
    assert.deepEqual(nodes[id].state, before.state);
    assert.equal(nodes[id].control_id, before.control_id);
  }
  return { output, changes: output === source ? [] : changes };
}

const repoRoot = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../..',
);
const catalogRoot = resolve(repoRoot, 'docs/_data/layout');
const catalog: CatalogManifest = JSON.parse(
  await readFile(resolveLayoutCatalogPath(catalogRoot, 'catalog.json'), 'utf8'),
);
const apply = process.argv.includes('--apply');
const records = [];
for (const suffix of targets) {
  const sourcePath = `zircon_editor/assets/ui/editor/${suffix}`;
  const entry = catalog.entries.find(
    (entry) => entry.sourcePath === sourcePath,
  )!;
  assert.ok(entry, `Missing source ${sourcePath}`);
  const path = resolve(repoRoot, sourcePath);
  const before = await readFile(path, 'utf8');
  assert.equal(
    bytesSha256(before),
    entry.sourceSha256,
    `Regenerate catalog for ${sourcePath}`,
  );
  const { output, changes } = refine(before);
  if (!changes.length) continue;
  if (apply) {
    const baseline = resolveLayoutCatalogPath(
      catalogRoot,
      dirname(entry.outputPath),
      'evidence/before-20260907-composite-reflow',
    );
    await mkdir(baseline, { recursive: true });
    await writeFile(resolve(baseline, 'source.zui'), before);
    for (const evidence of entry.penpotEvidence ?? [])
      if (evidence.screenshotPath)
        await cp(
          resolveLayoutCatalogPath(catalogRoot, evidence.screenshotPath),
          resolve(baseline, `${evidence.caseId}.png`),
        );
    await writeFile(path, output);
  }
  records.push({
    sourcePath,
    beforeSha256: bytesSha256(before),
    afterSha256: bytesSha256(output),
    changes,
  });
}
const report = {
  generatedAt: new Date().toISOString(),
  applied: apply,
  changedFiles: records.length,
  records,
};
await writeFile(
  resolveLayoutCatalogPath(
    catalogRoot,
    `evidence/workbench-composite-${apply ? 'applied' : 'proposed'}.json`,
  ),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(JSON.stringify({ ...report, records: undefined }));
