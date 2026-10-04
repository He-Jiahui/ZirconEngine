import { resolveLayoutCatalogPath } from './zui-layout-paths';
import assert from 'node:assert/strict';
import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseZuiDocument, type ZuiTable } from '../src/bridge/zui-document';
import { inline } from './zui-material-source-refinement';
import { bytesSha256 } from './zui-layout-evidence';
import type { CatalogManifest } from './zui-layout-catalog';

const fixed = (value: number) => ({
  min: value,
  preferred: value,
  max: value,
  stretch: 'Fixed',
});
const table = (value: unknown) => value as ZuiTable;

export function refineMaterialSamples(source: string) {
  const original = parseZuiDocument(source).document;
  const document = structuredClone(original);
  const nodes = document.nodes!;
  const sample = nodes['sample'];
  const root = nodes['root'];
  const changes: string[] = [];
  if (sample?.component !== 'HorizontalBox' || !sample.children?.length)
    return { source, changes };
  if (
    sample.props?.['background_color'] === 'transparent' &&
    sample.props?.['text'] === ''
  )
    return { source, changes };
  if (typeof sample.props?.['value_text'] === 'string')
    return { source, changes };
  assert.equal(nodes['title'].component, 'WorkbenchLabel');
  if (sample.props?.['text'] !== undefined)
    assert.equal(
      sample.props['text'],
      nodes['title'].props?.['text'],
      'Only the duplicated container caption may be removed; all unique content stays visible',
    );
  assert.ok(
    !sample['bindings'],
    'Bound sample captions require an explicit source migration',
  );
  sample.props!['text'] = '';
  sample.props!['background_color'] = 'transparent';
  sample.props!['border_width'] = 0;
  sample.props!['corner_radius'] = 0;
  if (sample.style?.['self']) {
    const self = table(sample.style['self']);
    self['background'] = { ...table(self['background']), color: 'transparent' };
    self['border'] = { ...table(self['border']), width: 0, radius: 0 };
  }
  const attached = sample.props?.['button_group_segment_count'] !== undefined;
  const floating = sample.children.some(
    ({ node }) => nodes[node].props?.['fab_style'] !== undefined,
  );
  const columns = sample.children.length > 5 ? 2 : 1;
  let sampleHeight = 32;
  if (floating) {
    const extended = sample.children
      .map(({ node }) => nodes[node])
      .find((node) => node.props?.['button_shape'] === 'extended');
    if (extended) {
      Object.assign(extended.props!, {
        fab_style: 'standard',
        button_horizontal_padding: 12,
        button_vertical_padding: 12,
        min_layout_width: 128,
        min_layout_height: 48,
        icon_size: 20,
        corner_radius: '$editor.control.radius.control',
      });
      extended.layout!['width'] = fixed(128);
      extended.layout!['height'] = fixed(48);
    }
    sampleHeight = 56;
    sample.layout!['container'] = {
      ...table(sample.layout!['container']),
      gap: '$editor.density.gap.medium',
    };
    changes.push(
      'sample: give the three FAB specimens a 56px row; extended command uses 128x48 with Workbench padding and radius',
    );
  } else {
    for (const { node: id } of sample.children) {
      const node = nodes[id];
      node.layout!['width'] = { min: 0, stretch: 'Stretch' };
      node.layout!['height'] = fixed(32);
      if (node.props?.['corner_radius'] !== undefined && !attached)
        node.props['corner_radius'] = '$editor.control.radius.control';
      if (node.style?.['self'] && !attached) {
        const self = table(node.style['self']);
        if (self['border'])
          table(self['border'])['radius'] = '$editor.control.radius.control';
      }
    }
    if (attached) {
      sample.layout!['container'] = { kind: 'HorizontalBox', gap: 0 };
      changes.push(
        'sample: equal-width attached button segments retain their ordering, individual states and click routes',
      );
    } else {
      const rows = Math.ceil(sample.children.length / columns);
      sampleHeight = rows * 32 + (rows - 1) * 8;
      sample.layout!['container'] = {
        kind: 'GridBox',
        columns,
        rows,
        column_gap: '$editor.density.gap.medium',
        row_gap: '$editor.density.gap.medium',
      };
      changes.push(
        `sample: ${columns}-column grid of ${rows} 32px rows with 8px gaps; every specimen remains present`,
      );
    }
  }
  sample.layout!['height'] = fixed(sampleHeight);
  const beforeHeight = Number(
    table(original.nodes!['sample'].layout!['height'])['preferred'],
  );
  const rootHeight =
    Number(table(root.layout!['height'])['preferred']) +
    sampleHeight -
    beforeHeight;
  root.layout!['height'] = fixed(rootHeight);
  changes.push(
    'sample.text: duplicate caption is displayed once by the existing Workbench title; container no longer paints on top of its children',
  );
  for (const [id, node] of Object.entries(nodes)) {
    if (node.component !== 'WorkbenchCaption') continue;
    node.props!['background_color'] = 'transparent';
    if (node.style?.['self'])
      table(node.style['self'])['background'] = { color: 'transparent' };
    changes.push(
      `${id}: use a flat caption without a decorative tile background`,
    );
  }
  let section = '';
  const output = source
    .split(/\r?\n/)
    .map((line) => {
      const header = line.match(/^\[([^\]]+)\]$/);
      if (header) section = header[1];
      const assignment = line.match(/^(props|layout|style)\s*=/);
      if (!assignment || !section.startsWith('nodes.')) return line;
      const id = section.slice(6),
        key = assignment[1];
      if (
        JSON.stringify(original.nodes![id]?.[key]) ===
        JSON.stringify(nodes[id]?.[key])
      )
        return line;
      return `${key} = ${inline(nodes[id][key])}`;
    })
    .join(source.includes('\r\n') ? '\r\n' : '\n');
  assert.deepEqual(parseZuiDocument(output).document, document);
  for (const [id, before] of Object.entries(original.nodes!)) {
    assert.deepEqual(nodes[id].events, before.events);
    assert.deepEqual(nodes[id].children, before.children);
    assert.deepEqual(nodes[id].state, before.state);
    assert.equal(nodes[id].component, before.component);
  }
  return { source: output, changes, sampleHeight, rootHeight };
}

async function main() {
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
  for (const entry of catalog.entries) {
    if (
      !entry.sourcePath.startsWith(
        'zircon_editor/assets/ui/editor/material_components/inputs/',
      )
    )
      continue;
    const path = resolve(repoRoot, entry.sourcePath);
    const before = await readFile(path, 'utf8');
    const result = refineMaterialSamples(before);
    if (!result.changes.length) continue;
    if (apply) {
      const directory = resolveLayoutCatalogPath(
        catalogRoot,
        dirname(entry.outputPath),
        'evidence/before-20260907-sample-reflow',
      );
      await mkdir(directory, { recursive: true });
      await writeFile(resolve(directory, 'source.zui'), before);
      for (const evidence of entry.penpotEvidence ?? [])
        if (evidence.screenshotPath)
          await cp(
            resolveLayoutCatalogPath(catalogRoot, evidence.screenshotPath),
            resolve(directory, `${evidence.caseId}.png`),
          );
      await writeFile(path, result.source);
    }
    records.push({
      sourcePath: entry.sourcePath,
      beforeSha256: bytesSha256(before),
      afterSha256: bytesSha256(result.source),
      changes: result.changes,
      sampleHeight: result.sampleHeight,
      rootHeight: result.rootHeight,
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
      `evidence/material-sample-${apply ? 'applied' : 'proposed'}.json`,
    ),
    `${JSON.stringify(report, null, 2)}\n`,
  );
  console.log(JSON.stringify({ ...report, records: undefined }));
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  await main();
