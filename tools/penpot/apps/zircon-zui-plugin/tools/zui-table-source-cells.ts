import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseZuiDocument, type ZuiDocument } from '../src/bridge/zui-document';
import { tableCells } from '../src/bridge/zui-table-cells';
import { bytesSha256 } from './zui-layout-evidence';

export function explicitTableCells(source: string): {
  source: string;
  changes: Array<{ nodeId: string; options: string[] }>;
  unresolved: Array<{ nodeId: string; reason: string }>;
} {
  const document = parseZuiDocument(source).document;
  const expected: ZuiDocument = structuredClone(document);
  const newline = source.includes('\r\n') ? '\r\n' : '\n';
  const lines = source.split(/\r?\n/);
  const changes: Array<{ nodeId: string; options: string[] }> = [];
  const unresolved: Array<{ nodeId: string; reason: string }> = [];
  for (const [nodeId, node] of Object.entries(document.nodes ?? {})) {
    if (node.component !== 'WorkbenchTableRow' || node.props?.['options'] !== undefined) continue;
    const props = node.props ?? {};
    const text = props['text'];
    const detail = props['value_text'];
    if (typeof text !== 'string' || !text.trim()) {
      unresolved.push({ nodeId, reason: 'No authored table label' });
      continue;
    }
    if (detail !== undefined && typeof detail !== 'string') {
      unresolved.push({ nodeId, reason: 'Non-string detail requires a model adapter' });
      continue;
    }
    const options = typeof detail === 'string' && detail.trim()
      ? [text.trim(), ...detail.trim().split(/ {2,}|\t+/)]
      : tableCells(props).map((cell) => cell.value);
    if (options.length > 4 || options.some((value) => !value || /[\r\n]/.test(value))) {
      unresolved.push({ nodeId, reason: `Authored content requires ${options.length} columns or multiline cells` });
      continue;
    }
    // The native archived-row heuristic must still recognize these as cells.
    if (tableCells({ options }).length !== options.length) {
      unresolved.push({ nodeId, reason: 'Native archived-row heuristic rejects declared cell values' });
      continue;
    }
    const start = lines.findIndex((line) => line.trim() === `[nodes.${nodeId}]`);
    const next = lines.findIndex((line, index) => index > start && /^\s*\[/.test(line));
    const end = next < 0 ? lines.length : next;
    const lineIndex = lines.findIndex((line, index) => index > start && index < end && /^\s*props\s*=\s*\{.*\}\s*$/.test(line));
    if (start < 0 || lineIndex < 0) {
      unresolved.push({ nodeId, reason: 'Props are not a single inline table; edit manually' });
      continue;
    }
    const line = lines[lineIndex];
    const close = line.lastIndexOf('}');
    lines[lineIndex] = `${line.slice(0, close).trimEnd()}, options = ${JSON.stringify(options)} ${line.slice(close)}`;
    expected.nodes![nodeId].props!['options'] = options;
    changes.push({ nodeId, options });
  }
  const updated = lines.join(newline);
  assert.deepEqual(parseZuiDocument(updated).document, expected, 'Table migration must only add explicit options');
  return { source: updated, changes, unresolved };
}

async function main(): Promise<void> {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const catalog = JSON.parse(await readFile(resolve(root, 'docs/_data/layout/catalog.json'), 'utf8')) as {
    entries: Array<{ sourcePath: string }>;
  };
  const apply = process.argv.includes('--apply');
  const records = [];
  for (const entry of catalog.entries) {
    if (!entry.sourcePath.startsWith('zircon_editor/assets/')) continue;
    const path = resolve(root, entry.sourcePath);
    const original = await readFile(path, 'utf8');
    const result = explicitTableCells(original);
    if (!result.changes.length && !result.unresolved.length) continue;
    const beforePath = `docs/_data/layout/evidence/before-table-cells/${entry.sourcePath}`;
    if (apply && result.changes.length) {
      await mkdir(dirname(resolve(root, beforePath)), { recursive: true });
      await writeFile(resolve(root, beforePath), original);
      await writeFile(path, result.source);
    }
    records.push({
      sourcePath: entry.sourcePath,
      beforePath,
      beforeSha256: bytesSha256(original),
      afterSha256: bytesSha256(result.source),
      changes: result.changes,
      unresolved: result.unresolved,
    });
  }
  const report = {
    generatedAt: new Date().toISOString(),
    applied: apply,
    changedFiles: records.filter((record) => record.changes.length).length,
    changedRows: records.reduce((sum, record) => sum + record.changes.length, 0),
    unresolvedRows: records.reduce((sum, record) => sum + record.unresolved.length, 0),
    records,
  };
  const path = resolve(root, `docs/_data/layout/evidence/table-source-cells-${apply ? 'applied' : 'proposed'}.json`);
  await writeFile(path, `${JSON.stringify(report, null, 2)}\n`);
  console.log(JSON.stringify({ ...report, records: undefined, report: path }));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
}
