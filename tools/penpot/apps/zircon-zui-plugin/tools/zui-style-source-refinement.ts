import { resolveLayoutCatalogPath } from './zui-layout-paths';
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import {
  parseZuiDocument,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
  type ZuiValue,
} from '../src/bridge/zui-document';
import type { CatalogManifest } from './zui-layout-catalog';
import { bytesSha256 } from './zui-layout-evidence';
import {
  isBaseControlComponent,
  isIntentionalRoundGeometry,
  isStyleContractExempt,
} from './zui-style-contract';

const allowedFontSizes = [12, 14, 16, 20];
const allowedSpacing = [0, 2, 4, 8, 12, 16, 24];
const dialogComponents = /(dialog|modal|popover|popup)/i;
const directSpacingKeys = [
  'gap',
  'padding',
  'padding_x',
  'padding_y',
  'padding_left',
  'padding_right',
  'padding_top',
  'padding_bottom',
];
const radiusKeys = ['corner_radius', 'border_radius', 'radius', 'cornerRadius'];

export interface StyleSourceRefinement {
  source: string;
  changed: boolean;
  changes: string[];
}

function table(value: unknown): ZuiTable | undefined {
  return value && typeof value === 'object' && !Array.isArray(value) && !(value instanceof Date)
    ? (value as ZuiTable)
    : undefined;
}

function number(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
}

function normalized(value: number, allowed: readonly number[]): number {
  return allowed.reduce((best, candidate) => {
    const bestDistance = Math.abs(value - best);
    const distance = Math.abs(value - candidate);
    return distance < bestDistance || (distance === bestDistance && candidate > best)
      ? candidate
      : best;
  });
}

function normalizeNumeric(
  values: ZuiTable,
  key: string,
  allowed: readonly number[],
  path: string,
  changes: string[],
): void {
  const before = number(values[key]);
  if (before === undefined || allowed.includes(before)) return;
  const after = normalized(before, allowed);
  values[key] = after;
  changes.push(`${path}: ${before} -> ${after}`);
}

function normalizeControlHeight(
  values: ZuiTable,
  path: string,
  changes: string[],
): void {
  const height = table(values['height']);
  if (!height) return;
  const keys = ['min', 'preferred', 'max'];
  const before = keys
    .map((key) => number(height[key]))
    .filter((value): value is number => value !== undefined);
  if (!before.length || before.every((value) => value === 28 || value === 32)) return;
  const reference = number(height['preferred']) ?? before[0];
  const target = reference <= 30 ? 28 : 32;
  for (const key of keys) {
    if (number(height[key]) !== undefined) height[key] = target;
  }
  changes.push(`${path}: ${before.join('/')} -> ${target}`);
}

function normalizeDirectControlHeight(
  values: ZuiTable,
  path: string,
  changes: string[],
): void {
  for (const key of ['height', 'min_height', 'minHeight']) {
    const before = number(values[key]);
    if (before === undefined || before === 28 || before === 32) continue;
    const target = before <= 30 ? 28 : 32;
    values[key] = target;
    changes.push(`${path}.${key}: ${before} -> ${target}`);
  }
}

function normalizeNode(nodeId: string, node: ZuiNode, changes: string[]): void {
  const component = node.component ?? '';
  const baseControl = isBaseControlComponent(component);
  for (const field of ['props', 'style', 'layout'] as const) {
    const values = table(node[field]);
    if (!values) continue;
    const prefix = `nodes.${nodeId}.${field}`;
    for (const key of ['font_size', 'fontSize'])
      normalizeNumeric(values, key, allowedFontSizes, `${prefix}.${key}`, changes);
    for (const key of directSpacingKeys)
      normalizeNumeric(values, key, allowedSpacing, `${prefix}.${key}`, changes);
    for (const key of radiusKeys) {
      const radius = number(values[key]);
      if (radius === undefined || isIntentionalRoundGeometry(nodeId, node, radius))
        continue;
      const cap = dialogComponents.test(component) ? 8 : 4;
      if (radius <= cap) continue;
      values[key] = cap;
      changes.push(`${prefix}.${key}: ${radius} -> ${cap}`);
    }
    if (baseControl) {
      normalizeDirectControlHeight(values, prefix, changes);
      normalizeControlHeight(values, `${prefix}.height`, changes);
    }
    const container = table(values['container']);
    if (container)
      normalizeNumeric(
        container,
        'gap',
        allowedSpacing,
        `${prefix}.container.gap`,
        changes,
      );
    const padding = table(values['padding']);
    if (padding) {
      for (const side of ['top', 'right', 'bottom', 'left'])
        normalizeNumeric(
          padding,
          side,
          allowedSpacing,
          `${prefix}.padding.${side}`,
          changes,
        );
    }
  }
}

function inline(value: ZuiValue): string {
  if (typeof value === 'bigint') return value.toString();
  if (value instanceof Date) return value.toISOString();
  if (Array.isArray(value)) return `[${value.map(inline).join(', ')}]`;
  if (value && typeof value === 'object')
    return `{ ${Object.entries(value)
      .map(([key, child]) => `${JSON.stringify(key)} = ${inline(child)}`)
      .join(', ')} }`;
  return JSON.stringify(value);
}

function bareKeys(value: ZuiValue): string {
  return inline(value).replace(/"([A-Za-z_][A-Za-z0-9_]*)" = /g, '$1 = ');
}

function rewriteChangedNodeFields(
  originalSource: string,
  before: ZuiDocument,
  after: ZuiDocument,
): string {
  let section = '';
  const changedFields = new Set<string>();
  for (const [nodeId, node] of Object.entries(after.nodes ?? {})) {
    for (const field of ['props', 'style', 'layout'] as const) {
      if (JSON.stringify(before.nodes?.[nodeId]?.[field]) !== JSON.stringify(node[field]))
        changedFields.add(`${nodeId}.${field}`);
    }
  }
  const rewritten = originalSource
    .split(/\r?\n/)
    .map((line) => {
      const header = line.match(/^\[([^\]]+)\]$/);
      if (header) section = header[1];
      const assignment = line.match(/^(props|style|layout)\s*=/);
      if (!assignment || !section.startsWith('nodes.')) return line;
      const nodeId = section.slice('nodes.'.length);
      const field = assignment[1] as 'props' | 'style' | 'layout';
      if (!changedFields.delete(`${nodeId}.${field}`)) return line;
      const value = after.nodes?.[nodeId]?.[field];
      assert.ok(value, `Missing rewritten ${nodeId}.${field}`);
      return `${field} = ${bareKeys(value)}`;
    })
    .join(originalSource.includes('\r\n') ? '\r\n' : '\n');
  assert.deepEqual(
    [...changedFields],
    [],
    `Style source has a changed multi-line field that cannot be rewritten safely: ${[...changedFields].join(', ')}`,
  );
  assert.deepEqual(
    parseZuiDocument(rewritten).document,
    after,
    'Style refinement must preserve all unknown runtime fields',
  );
  return rewritten;
}

export function refineZuiStyleSource(
  source: string,
  sourcePath: string,
): StyleSourceRefinement {
  if (isStyleContractExempt(sourcePath))
    return { source, changed: false, changes: [] };
  const before = parseZuiDocument(source).document;
  const after = structuredClone(before);
  const changes: string[] = [];
  for (const [nodeId, node] of Object.entries(after.nodes ?? {}))
    normalizeNode(nodeId, node, changes);
  if (!changes.length) return { source, changed: false, changes };
  return {
    source: rewriteChangedNodeFields(source, before, after),
    changed: true,
    changes,
  };
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const apply = args.includes('--apply');
  const sourceFilters = args.flatMap((argument, index) =>
    argument === '--source' && args[index + 1] ? [args[index + 1]] : [],
  );
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const catalogRoot = resolve(repoRoot, 'docs/_data/layout');
  const catalog = JSON.parse(
    await readFile(resolveLayoutCatalogPath(catalogRoot, 'catalog.json'), 'utf8'),
  ) as CatalogManifest;
  const records: Array<{
    sourcePath: string;
    beforeSha256: string;
    afterSha256: string;
    changes: string[];
  }> = [];
  for (const entry of catalog.entries) {
    if (
      sourceFilters.length &&
      !sourceFilters.some((filter) => entry.sourcePath.includes(filter))
    )
      continue;
    const path = resolve(repoRoot, entry.sourcePath);
    const before = await readFile(path, 'utf8');
    const result = refineZuiStyleSource(before, entry.sourcePath);
    if (!result.changed) continue;
    if (apply) await writeFile(path, result.source, 'utf8');
    records.push({
      sourcePath: entry.sourcePath,
      beforeSha256: bytesSha256(before),
      afterSha256: bytesSha256(result.source),
      changes: result.changes,
    });
  }
  const report = {
    generatedAt: new Date().toISOString(),
    applied: apply,
    changedFiles: records.length,
    changes: records.reduce((count, record) => count + record.changes.length, 0),
    records,
  };
  if (apply)
    await writeFile(
      resolveLayoutCatalogPath(catalogRoot, 'evidence/style-source-normalization-applied.json'),
      `${JSON.stringify(report, null, 2)}\n`,
      'utf8',
    );
  console.log(JSON.stringify({ ...report, records: undefined }));
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
)
  void main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
