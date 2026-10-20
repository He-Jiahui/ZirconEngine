import { runCurrentSourceRefinement, type SourceRefinementContexts } from './zui-source-refinement-current';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseZuiDocument, type ZuiTable } from '../src/bridge/zui-document';

const label =
  'res://ui/editor/components/workbench/primitives/data/label.zui#WorkbenchLabel';
const caption =
  'res://ui/editor/components/workbench/primitives/data/caption.zui#WorkbenchCaption';
const tokens = 'res://ui/editor/theme/editor_tokens.zui';
const fixed = (height: number) => ({
  min: height,
  preferred: height,
  max: height,
  stretch: 'Fixed',
});

function tableValue(value: unknown): ZuiTable {
  assert.ok(
    value &&
      typeof value === 'object' &&
      !Array.isArray(value) &&
      !(value instanceof Date),
    'Expected a TOML table',
  );
  return value as ZuiTable;
}

export function inline(value: unknown): string {
  if (value instanceof Date) return value.toISOString();
  if (Array.isArray(value)) return `[${value.map(inline).join(', ')}]`;
  if (value && typeof value === 'object')
    return `{ ${Object.entries(value)
      .map(([key, child]) => `${JSON.stringify(key)} = ${inline(child)}`)
      .join(', ')} }`;
  assert.ok(
    ['string', 'boolean', 'number'].includes(typeof value),
    'Unsupported inline TOML value',
  );
  return JSON.stringify(value);
}

export function refineMaterialPrototype(source: string) {
  const original = parseZuiDocument(source).document;
  const document = structuredClone(original);
  const nodes = document.nodes!;
  const root = nodes['root'],
    meta = nodes['meta'],
    title = nodes['title'],
    sample = nodes['sample'];
  assert.deepEqual(
    root.children?.map((child) => child.node),
    ['title', 'meta', 'sample', 'state_strip'],
  );
  assert.equal(meta.children?.length, 4);
  assert.equal(nodes['state_strip'].children?.length, 8);
  if (title.component === 'WorkbenchLabel') return { source, changed: false };
  document.imports ??= {};
  for (const [key, additions] of [
    ['widgets', [label, caption]],
    ['styles', [tokens]],
  ] as const) {
    const existing: unknown = document.imports[key] ?? [];
    assert.ok(Array.isArray(existing), 'Expected an import array');
    document.imports[key] = [...new Set([...existing, ...additions])];
  }
  const longest = Math.max(
    ...meta.children!.map(
      (child) => String(nodes[child.node].props?.['text'] ?? '').length,
    ),
  );
  const metaRowHeight = longest > 32 ? 56 : longest > 18 ? 40 : 24;
  const titleHeight = String(title.props?.['text'] ?? '').length > 20 ? 56 : 28;
  title.component = 'WorkbenchLabel';
  Object.assign(title.props!, {
    font_size: '$editor.typography.title.size',
    font_weight: '$editor.typography.strong.weight',
    wrap: 'word',
    text_overflow: 'clip',
  });
  title.layout!['height'] = fixed(titleHeight);
  for (const [id, node] of Object.entries(nodes)) {
    if (
      (!id.startsWith('meta_') && !id.startsWith('state_')) ||
      id === 'state_strip'
    )
      continue;
    node.component = 'WorkbenchCaption';
    Object.assign(node.props!, {
      font_size: '$editor.typography.caption.size',
      font_weight: '$editor.typography.body.weight',
      surface_variant: 'frame_only',
      corner_radius: 0,
      border_width: 0,
      text_align: 'left',
      wrap: 'word',
      text_overflow: 'clip',
    });
    const self = node.style?.['self'];
    if (
      self &&
      typeof self === 'object' &&
      !Array.isArray(self) &&
      !(self instanceof Date)
    ) {
      self['background'] = { color: 'transparent' };
      self['border'] = {
        ...(self['border'] ? tableValue(self['border']) : {}),
        width: 0,
        radius: 0,
      };
    }
    node.layout!['width'] = { stretch: 'Stretch' };
    node.layout!['height'] = fixed(id.startsWith('meta_') ? metaRowHeight : 24);
  }
  for (const [id, rows, rowHeight] of [
    ['meta', 2, metaRowHeight],
    ['state_strip', 4, 24],
  ] as const) {
    nodes[id].component = 'GridBox';
    nodes[id].layout!['height'] = fixed(rows * rowHeight + (rows - 1) * 4);
    nodes[id].layout!['container'] = {
      kind: 'GridBox',
      columns: 2,
      rows,
      column_gap: '$editor.density.gap.medium',
      row_gap: '$editor.density.gap.small',
    };
  }
  root.layout!['container'] = {
    ...tableValue(root.layout!['container']),
    gap: '$editor.density.gap.large',
  };
  const height =
    titleHeight +
    metaRowHeight * 2 +
    4 +
    Number(tableValue(sample.layout!['height'])['preferred']) +
    108 +
    36;
  root.layout!['height'] = fixed(height);
  const lines = source.split(/\r?\n/);
  let table = '';
  const updated = lines
    .map((line) => {
      const header = line.match(/^\[([^\]]+)\]$/);
      if (header) table = header[1];
      const assignment = line.match(
        /^(widgets|styles|component|props|layout|style)\s*=/,
      );
      if (!assignment) return line;
      const key = assignment[1];
      if (table === 'imports' && (key === 'widgets' || key === 'styles'))
        return `${key} = ${inline(document.imports![key]!)}`;
      if (!table.startsWith('nodes.')) return line;
      const id = table.slice(6);
      if (!nodes[id]) return line;
      const before = original.nodes![id][key],
        after = nodes[id][key];
      return JSON.stringify(before) === JSON.stringify(after)
        ? line
        : `${key} = ${inline(after!)}`;
    })
    .join(source.includes('\r\n') ? '\r\n' : '\n');
  assert.deepEqual(
    parseZuiDocument(updated).document,
    document,
    'Refinement must preserve all unmodified contracts',
  );
  assert.deepEqual(
    sample,
    original.nodes!['sample'],
    'Sample controls retain their behavior and geometry',
  );
  return { source: updated, changed: true, titleHeight, metaRowHeight, height };
}


export function runCurrentMaterialSourceRefinement(args: readonly string[] = [], contexts?: SourceRefinementContexts) {
  return runCurrentSourceRefinement({
    id: "material-source", select: entry => entry.sourcePath.startsWith('zircon_editor/assets/ui/editor/'),
    transform: (source) => refineMaterialPrototype(source),
  }, args, contexts);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  void runCurrentMaterialSourceRefinement(process.argv.slice(2)).then((report) => {
    console.log(JSON.stringify({ ...report, records: undefined }));
  }).catch((error: unknown) => { console.error(error); process.exitCode = 1; });
}
