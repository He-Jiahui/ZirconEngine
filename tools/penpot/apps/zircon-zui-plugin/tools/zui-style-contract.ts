import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import type { CatalogManifest } from './zui-layout-catalog.js';
import {
  parseZuiDocument,
  type ZuiDocument,
  type ZuiNode,
} from '../src/bridge/zui-document.js';
import { parseZuiLayoutSource } from '../src/bridge/zui-layout-optimizer.js';

export type StyleIssue = {
  source: string;
  node: string;
  path: string;
  message: string;
};
export type StyleContractResult = {
  source: string;
  exemptTheme: boolean;
  issues: StyleIssue[];
};

// Zero is an intentional compact-group value; 2px is the documented xsmall
// token.  Larger values remain explicit group/panel spacing tokens.
const spacing = new Set([0, 2, 4, 8, 12, 16, 24]);
const dialogComponents = /(dialog|modal|popover|popup)/i;
const ordinaryControlHeights = new Set([28, 32]);
const labeledSegmentedHeight = new Set([48]);
const baseControlNames = new Set([
  'button',
  'iconbutton',
  'inputfield',
  'numberfield',
  'searchinput',
  'dropdown',
  'checkbox',
  'radio',
  'toggle',
  'togglebutton',
  'slider',
  'rangeslider',
  'segmentedcontrol',
  'tab',
  'textfield',
  'workbenchbutton',
  'workbenchiconbutton',
  'workbenchfield',
  'workbenchnumberfield',
  'workbenchsearchinput',
  'workbenchdropdown',
  'workbenchcheckbox',
  'workbenchradio',
  'workbenchtoggle',
  'workbenchtogglebutton',
  'workbenchslider',
  'workbenchrangeslider',
  'workbenchsegmentedcontrol',
  'workbenchtab',
]);

function sourceExemption(source: string): boolean {
  // Fixtures preserve their original parser/runtime contract and WoC keeps
  // its game-specific visual scale.  Neither is an Editor Workbench source.
  const normalized = source.replaceAll('\\', '/');
  return /(?:examples\/woc|\/tests\/fixtures\/|\/ui\/runtime\/fixtures\/|roundtrip-fixture\.zui$)/i.test(
    normalized,
  );
}

/** Public scope predicate used by the catalog audit and downstream reports. */
export function isStyleContractExempt(source: string): boolean {
  const normalized = source.replaceAll('\\', '/');
  return (
    sourceExemption(normalized) || /(?:examples\/woc|\bwoc\b)/i.test(normalized)
  );
}

export function isBaseControlComponent(component: string): boolean {
  return baseControlNames.has(component.replace(/[._\s-]/g, '').toLowerCase());
}

function allowedControlHeights(node: ZuiNode): Set<number> {
  const component = node.component.replace(/[._\s-]/g, '').toLowerCase();
  const label = ['label', 'label_text', 'group_label']
    .map((key) => node.props?.[key])
    .find((value) => typeof value === 'string' && value.trim());
  // SegmentedControl paints the authored group label above the ordinary 28px
  // options row. Only that explicitly labeled composite reserves 48px; an
  // unlabeled control still has to follow the regular 28/32px scale.
  return label &&
    ['segmentedcontrol', 'workbenchsegmentedcontrol'].includes(component)
    ? labeledSegmentedHeight
    : ordinaryControlHeights;
}

function numberAt(value: unknown, key: string): number | undefined {
  if (!value || typeof value !== 'object') return undefined;
  const candidate = (value as Record<string, unknown>)[key];
  return typeof candidate === 'number' && Number.isFinite(candidate)
    ? candidate
    : undefined;
}

function numberAtPath(
  value: unknown,
  path: readonly string[],
): number | undefined {
  let current: unknown = value;
  for (const segment of path) {
    if (!current || typeof current !== 'object') return undefined;
    current = (current as Record<string, unknown>)[segment];
  }
  return typeof current === 'number' && Number.isFinite(current)
    ? current
    : undefined;
}

function issue(
  issues: StyleIssue[],
  source: string,
  node: string,
  path: string,
  message: string,
) {
  issues.push({ source, node, path, message });
}

function checkValue(
  issues: StyleIssue[],
  source: string,
  nodeId: string,
  component: string,
  value: unknown,
  path: string,
  allowed: Set<number>,
  label: string,
) {
  const n = typeof value === 'number' ? value : undefined;
  if (n !== undefined && !allowed.has(n))
    issue(
      issues,
      source,
      nodeId,
      path,
      `${label} must use ${[...allowed].join('/')}px; got ${n}`,
    );
}

/**
 * A large radius is meaningful for a pill, round icon, or a viewport-only
 * light/shadow shape. These are not ordinary editor controls and should not
 * be flattened by the 4px control rule.
 */
export function isIntentionalRoundGeometry(
  nodeId: string,
  node: ZuiNode,
  radius: number,
): boolean {
  if (radius >= 100) return true;
  const identity = [
    nodeId,
    node.control_id ?? '',
    ...(node.classes ?? []),
    node.component ?? '',
  ].join(' ');
  if (
    /badge|chip|avatar|dot|indicator|pill|circle|fab|lightwash|shadow|reflection|glow|timeline/i.test(
      identity,
    )
  )
    return true;
  if (node.component !== 'Space') return false;
  const props = node.props;
  const background = props?.['background_color'];
  const border = numberAt(props, 'border_width');
  return (
    typeof background === 'string' && (border === undefined || border <= 0)
  );
}

function checkNode(
  issues: StyleIssue[],
  source: string,
  nodeId: string,
  node: ZuiNode,
  exemptTheme: boolean,
) {
  const component = node.component ?? '';
  if (exemptTheme || sourceExemption(source)) return;
  const baseControl = isBaseControlComponent(component);
  const controlHeights = allowedControlHeights(node);
  const root = `nodes.${nodeId}`;
  const fields: Array<[string, Record<string, unknown> | undefined]> = [
    ['props', node.props],
    ['style', node.style],
    ['layout', node.layout],
  ];
  for (const [field, values] of fields) {
    if (!values) continue;
    const prefix = `${root}.${field}`;
    for (const key of ['font_size', 'fontSize']) {
      const size = numberAt(values, key);
      if (size !== undefined && ![12, 14, 16, 20].includes(size))
        issue(
          issues,
          source,
          nodeId,
          `${prefix}.${key}`,
          `Editor/plugin text size must be 12/14/16/20px; got ${size}`,
        );
    }
    for (const key of ['height', 'min_height', 'minHeight']) {
      const height = numberAt(values, key);
      if (height !== undefined && baseControl)
        checkValue(
          issues,
          source,
          nodeId,
          component,
          height,
          `${prefix}.${key}`,
          controlHeights,
          'control height',
        );
    }
    for (const key of [
      'gap',
      'padding',
      'padding_x',
      'padding_y',
      'padding_left',
      'padding_right',
      'padding_top',
      'padding_bottom',
    ]) {
      const value = numberAt(values, key);
      if (value !== undefined)
        checkValue(
          issues,
          source,
          nodeId,
          component,
          value,
          `${prefix}.${key}`,
          spacing,
          'spacing',
        );
    }
    for (const key of [
      'corner_radius',
      'border_radius',
      'radius',
      'cornerRadius',
    ]) {
      const radius = numberAt(values, key);
      if (radius === undefined) continue;
      // Panel surfaces are deliberately flat; pills and circular icons have
      // explicit geometry semantics and are outside the ordinary-control cap.
      const max = dialogComponents.test(component) ? 8 : 4;
      if (!isIntentionalRoundGeometry(nodeId, node, radius) && radius > max)
        issue(
          issues,
          source,
          nodeId,
          `${prefix}.${key}`,
          `corner radius must be <=${max}px; got ${radius}`,
        );
    }
    const nestedHeight = numberAtPath(values, ['height', 'preferred']);
    if (nestedHeight !== undefined && baseControl)
      checkValue(
        issues,
        source,
        nodeId,
        component,
        nestedHeight,
        `${prefix}.height.preferred`,
        controlHeights,
        'control height',
      );
    for (const [nestedPath, label] of [
      [['height', 'min'], 'control height'],
      [['height', 'max'], 'control height'],
    ] as const) {
      const nested = numberAtPath(values, nestedPath);
      if (nested !== undefined && baseControl)
        checkValue(
          issues,
          source,
          nodeId,
          component,
          nested,
          `${prefix}.${nestedPath.join('.')}`,
          controlHeights,
          label,
        );
    }
    const container = values['container'];
    const gap = numberAtPath(container, ['gap']);
    if (gap !== undefined)
      checkValue(
        issues,
        source,
        nodeId,
        component,
        gap,
        `${prefix}.container.gap`,
        spacing,
        'spacing',
      );
    const padding = values['padding'];
    if (padding && typeof padding === 'object') {
      for (const side of ['top', 'right', 'bottom', 'left']) {
        const value = numberAtPath(padding, [side]);
        if (value !== undefined)
          checkValue(
            issues,
            source,
            nodeId,
            component,
            value,
            `${prefix}.padding.${side}`,
            spacing,
            'spacing',
          );
      }
    }
  }
}

export function checkZuiStyleContract(
  sourceText: string,
  source = '<memory>',
): StyleContractResult {
  let parsed: ZuiDocument;
  try {
    parsed = parseZuiDocument(sourceText).document;
  } catch {
    // Legacy/layout assets are still audited after the same migration used by
    // the catalog; the source bytes remain untouched.
    parsed = parseZuiLayoutSource(sourceText, source).document;
  }
  const exemptTheme = isStyleContractExempt(source);
  const issues: StyleIssue[] = [];
  for (const [nodeId, node] of Object.entries(parsed.nodes ?? {}))
    checkNode(issues, source, nodeId, node, exemptTheme);
  return { source, exemptTheme, issues };
}

export async function checkZuiStyleFile(
  path: string,
): Promise<StyleContractResult> {
  const absolute = resolve(path);
  try {
    return checkZuiStyleContract(await readFile(absolute, 'utf8'), path);
  } catch (error) {
    return {
      source: path,
      exemptTheme: false,
      issues: [
        {
          source: path,
          node: '<document>',
          path: 'document',
          message: `Unable to parse source for style audit: ${String(error)}`,
        },
      ],
    };
  }
}

export async function checkZuiStyleFiles(
  paths: string[],
): Promise<StyleContractResult[]> {
  return Promise.all(paths.map(checkZuiStyleFile));
}

async function main() {
  const args = process.argv.slice(2);
  let paths = args;
  const catalogIndex = args.indexOf('--catalog');
  if (catalogIndex >= 0) {
    const catalogPath = args[catalogIndex + 1];
    if (!catalogPath) throw new Error('--catalog requires catalog.json');
    const manifest = JSON.parse(
      await readFile(resolve(catalogPath), 'utf8'),
    ) as CatalogManifest;
    const catalogRoot = resolve(manifest.repoRoot);
    paths = manifest.entries.map((entry) =>
      resolveLayoutCatalogPath(catalogRoot, entry.sourcePath),
    );
    paths = paths.concat(args.slice(catalogIndex + 2));
  }
  if (!paths.length)
    throw new Error(
      'Usage: tsx tools/zui-style-contract.ts [--catalog catalog.json] <file.zui> [...]',
    );
  const results = await checkZuiStyleFiles(paths);
  const issues = results.flatMap((result) => result.issues);
  for (const item of issues)
    console.error(`${item.source} ${item.node} ${item.path}: ${item.message}`);
  console.log(JSON.stringify({ files: results.length, issues: issues.length }));
  if (issues.length) process.exitCode = 1;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
)
  void main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
