import { parse } from 'smol-toml';
import { migrateLegacyDocument } from './zui-legacy-source';
import {
  ZuiDocumentError,
  cloneZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
  type ZuiValue,
} from './zui-document';
import {
  applyPenpotPrefabFoundation,
  resolveDesignNumber,
} from './zui-prefab-system';
import {
  fitAutoLayoutContainersToContent,
  fitComponentRootsToContent,
} from './zui-component-layout-optimizer';
import { refineLayoutPresentation } from './zui-layout-refinement';

type AnyRecord = Record<string, unknown>;

export interface ZuiLayoutCatalogEntry {
  category: string;
  name: string;
}

export interface PreparedZuiLayoutAsset {
  document: ZuiDocument;
  sourceFormat: 'v2' | 'legacy';
  changes: string[];
}

const COLOR_KEYS = new Set([
  'background_color',
  'border_color',
  'foreground_color',
  'color',
  'text_color',
  'fill_color',
]);
const NON_NEGATIVE_PROP_KEYS = new Set([
  'border_width',
  'corner_radius',
  'font_size',
]);

/**
 * Prepare one source asset for the Penpot semantic bridge. The returned
 * document is a catalog copy; source assets remain owned by their product
 * modules and are never rewritten by the catalog job.
 */
export function prepareZuiLayoutAsset(
  source: string,
  sourcePath: string,
): PreparedZuiLayoutAsset {
  const parsed = parseZuiLayoutSource(source, sourcePath);
  const optimized = optimizeV2Document(parsed.document);
  return {
    ...optimized,
    sourceFormat: parsed.sourceFormat,
    changes: uniqueChanges([...parsed.changes, ...optimized.changes]),
  };
}

/** Parse before resolving dependencies, parameters or measuring a preview. */
export function parseZuiLayoutSource(
  source: string,
  sourcePath: string,
): PreparedZuiLayoutAsset {
  const raw = parseToml(source);
  const rawAsset = asRecord(raw['asset']);
  const rawVersion = finiteNumber(rawAsset?.['version']);
  const rawKind = stringValue(rawAsset?.['kind']);

  if (rawVersion === 2 && rawKind !== 'layout' && rawKind !== 'widget') {
    const parsed = parseZuiDocument(source).document;
    return { document: parsed, sourceFormat: 'v2', changes: [] };
  }

  if (rawVersion === 1 || rawKind === 'layout' || rawKind === 'widget') {
    const migrated = migrateLegacyDocument(raw, sourcePath);
    const validated = parseZuiDocument(serializeZuiDocument(migrated)).document;
    return {
      document: validated,
      sourceFormat: 'legacy',
      changes: ['migrate-legacy-document'],
    };
  }

  // Let the canonical parser produce its structured diagnostics for malformed
  // or unknown documents instead of silently making a lossy migration.
  const parsed = parseZuiDocument(source).document;
  return { document: parsed, sourceFormat: 'v2', changes: [] };
}

export function catalogEntryForSource(
  sourcePath: string,
): ZuiLayoutCatalogEntry {
  const normalized = sourcePath.replaceAll('\\', '/');
  const sourceName = normalized.split('/').pop() ?? 'zui-asset.zui';
  const base = sourceName.replace(/\.zui$/i, '');
  const readable = slug(base) || 'zui-asset';
  return {
    category: categoryForSource(normalized),
    name: `${readable}-${stableHash(normalized)}`,
  };
}

export function optimizeV2Document(
  document: ZuiDocument,
): PreparedZuiLayoutAsset {
  const optimized = cloneZuiDocument(document);
  const changes = new Set<string>();
  const asset = asRecord(optimized.asset);
  if (asset) {
    if (
      typeof asset['display_name'] !== 'string' ||
      asset['display_name'].trim() === ''
    ) {
      asset['display_name'] = asset['id'];
      changes.add('normalize-display-name');
    }
    if (asset['version'] !== 2) {
      asset['version'] = 2;
      changes.add('normalize-schema-version');
    }
  }

  const nodes = asRecord(optimized.nodes);
  if (nodes) {
    for (const [nodeId, value] of Object.entries(nodes)) {
      const node = asRecord(value);
      if (!node) continue;
      normalizeNode(node, changes, nodeId);
    }
  }
  refineLayoutPresentation(optimized, changes);
  normalizeTokenValues(asRecord(optimized.tokens), changes);
  normalizeStyleSheets(optimized.stylesheets, changes);
  resolveOverlayActionCollisions(optimized, changes);
  for (const change of applyPenpotPrefabFoundation(optimized)) {
    changes.add(change);
  }
  fitAutoLayoutContainersToContent(optimized, changes);
  fitComponentRootsToContent(optimized, changes);

  return {
    document: optimized,
    sourceFormat: 'v2',
    changes: uniqueChanges([...changes]),
  };
}

function normalizeNode(
  node: AnyRecord,
  changes: Set<string>,
  nodeId: string,
): void {
  const props = asRecord(node['props']);
  if (props) {
    for (const [key, value] of Object.entries(props)) {
      if (COLOR_KEYS.has(key) && typeof value === 'string') {
        const normalized = normalizeColor(value);
        if (normalized !== value) {
          props[key] = normalized;
          changes.add('normalize-paint');
        }
      }
      if (
        key === 'opacity' &&
        typeof value === 'number' &&
        Number.isFinite(value)
      ) {
        const normalized = clamp(value, 0, 1);
        if (normalized !== value) {
          props[key] = normalized;
          changes.add('normalize-paint');
        }
      }
      if (
        NON_NEGATIVE_PROP_KEYS.has(key) &&
        typeof value === 'number' &&
        Number.isFinite(value)
      ) {
        const normalized = Math.max(0, value);
        if (normalized !== value) {
          props[key] = normalized;
          changes.add('normalize-paint');
        }
      }
      if (key === 'text_align' && typeof value === 'string') {
        const normalized = normalizeTextAlign(value);
        if (normalized !== value) {
          props[key] = normalized;
          changes.add('normalize-text-alignment');
        }
      }
    }
  }

  const layout = asRecord(node['layout']);
  if (layout && normalizeLayout(layout)) changes.add('normalize-geometry');
  const style = asRecord(node['style']);
  if (style && normalizeStyleBlock(style, changes))
    changes.add('normalize-style');

  // Keep a stable human-readable node marker for the Penpot layer tree. This
  // is metadata only and does not alter runtime component semantics.
  if (node['control_id'] === undefined && nodeId.trim() !== '') {
    node['control_id'] = nodeId;
    changes.add('add-penpot-node-label');
  }
}

function normalizeLayout(layout: AnyRecord): boolean {
  let changed = false;
  for (const key of ['position', 'padding']) {
    const table = asRecord(layout[key]);
    if (!table) continue;
    for (const [field, value] of Object.entries(table)) {
      if (typeof value !== 'number' || !Number.isFinite(value)) continue;
      const normalized = key === 'position' ? value : Math.max(0, value);
      if (normalized !== value) {
        table[field] = normalized;
        changed = true;
      }
    }
  }
  for (const key of ['width', 'height']) {
    const table = asRecord(layout[key]);
    if (!table) continue;
    const min = finiteNumber(table['min']);
    const max = finiteNumber(table['max']);
    const preferred = finiteNumber(table['preferred']);
    const normalizedMin = min === null ? null : Math.max(0, min);
    let normalizedMax = max === null ? null : Math.max(0, max);
    let normalizedPreferred =
      preferred === null ? null : Math.max(0, preferred);
    if (
      normalizedMin !== null &&
      normalizedMax !== null &&
      normalizedMax < normalizedMin
    ) {
      normalizedMax = normalizedMin;
    }
    if (normalizedPreferred !== null && normalizedMin !== null) {
      normalizedPreferred = Math.max(normalizedMin, normalizedPreferred);
    }
    if (normalizedPreferred !== null && normalizedMax !== null) {
      normalizedPreferred = Math.min(normalizedMax, normalizedPreferred);
    }
    if (normalizedMin !== null && normalizedMin !== min) {
      table['min'] = normalizedMin;
      changed = true;
    }
    if (normalizedMax !== null && normalizedMax !== max) {
      table['max'] = normalizedMax;
      changed = true;
    }
    if (normalizedPreferred !== null && normalizedPreferred !== preferred) {
      table['preferred'] = normalizedPreferred;
      changed = true;
    }
  }
  const container = asRecord(layout['container']);
  if (container) {
    const gap = finiteNumber(container['gap']);
    if (gap !== null && gap < 0) {
      container['gap'] = 0;
      changed = true;
    }
    for (const key of ['column_gap', 'row_gap']) {
      const value = finiteNumber(container[key]);
      if (value !== null && value < 0) {
        container[key] = 0;
        changed = true;
      }
    }
  }
  return changed;
}

function normalizeStyleBlock(style: AnyRecord, changes: Set<string>): boolean {
  let changed = false;
  for (const value of Object.values(style)) {
    const table = asRecord(value);
    if (!table) continue;
    for (const [key, item] of Object.entries(table)) {
      if (COLOR_KEYS.has(key) && typeof item === 'string') {
        const normalized = normalizeColor(item);
        if (normalized !== item) {
          table[key] = normalized;
          changed = true;
          changes.add('normalize-paint');
        }
      }
    }
  }
  return changed;
}

function normalizeTokenValues(
  tokens: AnyRecord | undefined,
  changes: Set<string>,
): void {
  if (!tokens) return;
  for (const [key, value] of Object.entries(tokens)) {
    if (typeof value !== 'string') continue;
    const normalized = normalizeColor(value);
    if (normalized !== value) {
      tokens[key] = normalized;
      changes.add('normalize-token-colors');
    }
  }
}

function normalizeStyleSheets(value: unknown, changes: Set<string>): void {
  if (!Array.isArray(value)) return;
  for (const sheet of value) {
    const sheetRecord = asRecord(sheet);
    const rules = sheetRecord ? sheetRecord['rules'] : undefined;
    if (!Array.isArray(rules)) continue;
    for (const rule of rules) {
      const ruleRecord = asRecord(rule);
      const set = ruleRecord ? asRecord(ruleRecord['set']) : undefined;
      if (set) normalizeStyleBlock(set, changes);
    }
  }
}

const OVERLAY_ACTION_GAP = 12;

interface AnchoredBox {
  id: string;
  node: AnyRecord;
  layout: AnyRecord;
  width: number;
  height: number;
  anchorX: number;
  anchorY: number;
  pivotX: number;
  pivotY: number;
  positionX: number;
  positionY: number;
}

function resolveOverlayActionCollisions(
  document: ZuiDocument,
  changes: Set<string>,
): void {
  const nodes = asRecord(document.nodes);
  if (!nodes) return;
  const parentByChild = new Map<string, string>();
  for (const [parentId, value] of Object.entries(nodes)) {
    const parent = asRecord(value);
    if (!parent || !Array.isArray(parent['children'])) continue;
    for (const mount of parent['children']) {
      const childId = stringValue(asRecord(mount)?.['node']);
      if (childId) parentByChild.set(childId, parentId);
    }
  }

  for (const [parentId, value] of Object.entries(nodes)) {
    const parent = asRecord(value);
    if (!parent || !isOverlayContainer(parent)) continue;
    const children = childBoxes(document, parent, nodes);
    const leftActions = children.filter((box) => isCornerAction(box, 0));
    const rightActions = children.filter((box) => isCornerAction(box, 1));
    if (leftActions.length === 0 || rightActions.length === 0) continue;

    const parentWidth = inheritedDimension(
      document,
      parentId,
      nodes,
      parentByChild,
      'width',
      new Set<string>(),
    );
    const parentHeight = inheritedDimension(
      document,
      parentId,
      nodes,
      parentByChild,
      'height',
      new Set<string>(),
    );

    for (const rail of children.filter(isCenteredBottomRail)) {
      const left = widestBox(leftActions);
      const right = widestBox(rightActions);
      if (rail.width < left.width + right.width + OVERLAY_ACTION_GAP * 4) {
        continue;
      }
      const cornerTop = Math.min(
        ...[...leftActions, ...rightActions].map(
          (action) => action.positionY - action.height,
        ),
      );
      const cornerBottom = Math.max(
        ...[...leftActions, ...rightActions].map((action) => action.positionY),
      );
      const railTop = rail.positionY - rail.height;
      if (!intervalsOverlap(railTop, rail.positionY, cornerTop, cornerBottom)) {
        continue;
      }

      const targetBottom = cornerTop - OVERLAY_ACTION_GAP;
      const position = asRecord(rail.layout['position']);
      if (!position || rail.positionY <= targetBottom) continue;
      position['y'] = targetBottom;
      rail.positionY = targetBottom;
      changes.add('separate-overlay-bottom-actions');

      if (parentWidth === null || parentHeight === null) continue;
      constrainOverlaySidePanels(
        document,
        children,
        rail,
        parentWidth,
        parentHeight,
        changes,
      );
    }
  }
}

function constrainOverlaySidePanels(
  document: ZuiDocument,
  children: AnchoredBox[],
  rail: AnchoredBox,
  parentWidth: number,
  parentHeight: number,
  changes: Set<string>,
): void {
  const railLeft = anchoredStart(
    parentWidth,
    rail.width,
    rail.anchorX,
    rail.pivotX,
    rail.positionX,
  );
  const railTop = anchoredStart(
    parentHeight,
    rail.height,
    rail.anchorY,
    rail.pivotY,
    rail.positionY,
  );
  for (const panel of children) {
    if (
      panel.id === rail.id ||
      panel.anchorY !== 0 ||
      panel.pivotY !== 0 ||
      !isSidePanel(panel)
    ) {
      continue;
    }
    const panelLeft = anchoredStart(
      parentWidth,
      panel.width,
      panel.anchorX,
      panel.pivotX,
      panel.positionX,
    );
    if (
      !intervalsOverlap(
        panelLeft,
        panelLeft + panel.width,
        railLeft,
        railLeft + rail.width,
      )
    ) {
      continue;
    }
    const panelTop = anchoredStart(
      parentHeight,
      panel.height,
      panel.anchorY,
      panel.pivotY,
      panel.positionY,
    );
    const safeHeight = railTop - OVERLAY_ACTION_GAP - panelTop;
    if (
      safeHeight <= 0 ||
      panelTop + panel.height <= railTop - OVERLAY_ACTION_GAP
    ) {
      continue;
    }
    const height = asRecord(panel.layout['height']);
    if (!height) continue;
    const minimum = resolveDesignNumber(document, height['min']) ?? 0;
    if (safeHeight < minimum) continue;
    const constrained = Math.max(minimum, safeHeight);
    height['preferred'] = constrained;
    const maximum = resolveDesignNumber(document, height['max']);
    if (maximum !== null && maximum > constrained) height['max'] = constrained;
    changes.add('constrain-overlay-side-panel');
  }
}

function childBoxes(
  document: ZuiDocument,
  parent: AnyRecord,
  nodes: AnyRecord,
): AnchoredBox[] {
  if (!Array.isArray(parent['children'])) return [];
  const boxes: AnchoredBox[] = [];
  for (const mount of parent['children']) {
    const id = stringValue(asRecord(mount)?.['node']);
    const node = id ? asRecord(nodes[id]) : undefined;
    const layout = node ? asRecord(node['layout']) : undefined;
    const anchor = layout ? asRecord(layout['anchor']) : undefined;
    const pivot = layout ? asRecord(layout['pivot']) : undefined;
    const position = layout ? asRecord(layout['position']) : undefined;
    const width = layout ? layoutDimension(document, layout, 'width') : null;
    const height = layout ? layoutDimension(document, layout, 'height') : null;
    if (
      !id ||
      !node ||
      !layout ||
      !anchor ||
      !pivot ||
      !position ||
      width === null ||
      height === null
    ) {
      continue;
    }
    boxes.push({
      id,
      node,
      layout,
      width,
      height,
      anchorX: resolveDesignNumber(document, anchor['x']) ?? 0,
      anchorY: resolveDesignNumber(document, anchor['y']) ?? 0,
      pivotX: resolveDesignNumber(document, pivot['x']) ?? 0,
      pivotY: resolveDesignNumber(document, pivot['y']) ?? 0,
      positionX: resolveDesignNumber(document, position['x']) ?? 0,
      positionY: resolveDesignNumber(document, position['y']) ?? 0,
    });
  }
  return boxes;
}

function inheritedDimension(
  document: ZuiDocument,
  nodeId: string,
  nodes: AnyRecord,
  parentByChild: Map<string, string>,
  axis: 'width' | 'height',
  visited: Set<string>,
): number | null {
  if (visited.has(nodeId)) return null;
  visited.add(nodeId);
  const node = asRecord(nodes[nodeId]);
  const layout = node ? asRecord(node['layout']) : undefined;
  if (layout) {
    const explicit = layoutDimension(document, layout, axis);
    if (explicit !== null) return explicit;
  }
  if (document.asset.kind === 'view' && document.root?.node === nodeId) {
    return axis === 'width' ? 960 : 640;
  }
  const dimension = layout ? asRecord(layout[axis]) : undefined;
  const parentId = parentByChild.get(nodeId);
  if (dimension?.['stretch'] === 'Stretch' && parentId) {
    return inheritedDimension(
      document,
      parentId,
      nodes,
      parentByChild,
      axis,
      visited,
    );
  }
  return null;
}

function layoutDimension(
  document: ZuiDocument,
  layout: AnyRecord,
  axis: 'width' | 'height',
): number | null {
  const dimension = asRecord(layout[axis]);
  if (!dimension) return null;
  return (
    resolveDesignNumber(document, dimension['preferred']) ??
    resolveDesignNumber(document, dimension['min']) ??
    resolveDesignNumber(document, dimension['max'])
  );
}

function isOverlayContainer(node: AnyRecord): boolean {
  const layout = asRecord(node['layout']);
  const container = layout ? asRecord(layout['container']) : undefined;
  const kind =
    stringValue(container?.['kind']) ?? stringValue(node['component']);
  return kind?.toLowerCase() === 'overlay';
}

function isCornerAction(box: AnchoredBox, horizontalAnchor: 0 | 1): boolean {
  const component = stringValue(box.node['component'])?.toLowerCase() ?? '';
  const classes = Array.isArray(box.node['classes'])
    ? box.node['classes'].filter(
        (value): value is string => typeof value === 'string',
      )
    : [];
  return (
    box.anchorX === horizontalAnchor &&
    box.pivotX === horizontalAnchor &&
    box.anchorY === 1 &&
    box.pivotY === 1 &&
    (component.includes('button') ||
      classes.some((value) => value.includes('action')))
  );
}

function isCenteredBottomRail(box: AnchoredBox): boolean {
  const component = stringValue(box.node['component'])?.toLowerCase() ?? '';
  return (
    box.anchorX === 0.5 &&
    box.pivotX === 0.5 &&
    box.anchorY === 1 &&
    box.pivotY === 1 &&
    /(?:horizontal|grid|stack|rail)/.test(component)
  );
}

function isSidePanel(box: AnchoredBox): boolean {
  const component = stringValue(box.node['component'])?.toLowerCase() ?? '';
  return /(?:scroll|panel|drawer|vertical)/.test(component);
}

function widestBox(boxes: AnchoredBox[]): AnchoredBox {
  return boxes.reduce((widest, box) =>
    box.width > widest.width ? box : widest,
  );
}

function anchoredStart(
  parentSize: number,
  childSize: number,
  anchor: number,
  pivot: number,
  position: number,
): number {
  return parentSize * anchor - childSize * pivot + position;
}

function intervalsOverlap(
  leftStart: number,
  leftEnd: number,
  rightStart: number,
  rightEnd: number,
): boolean {
  return leftStart < rightEnd && rightStart < leftEnd;
}

function parseToml(source: string): AnyRecord {
  try {
    return parse(source, {
      integersAsBigInt: 'asNeeded',
    }) as unknown as AnyRecord;
  } catch (error) {
    throw new ZuiDocumentError(
      `Failed to parse .zui TOML: ${errorMessage(error)}`,
      [
        {
          severity: 'error',
          code: 'toml-parse-failed',
          message: errorMessage(error),
        },
      ],
    );
  }
}

function normalizeColor(value: string): string {
  const trimmed = value.trim();
  if (trimmed.toLowerCase() === 'transparent') return '#00000000';
  if (trimmed.startsWith('$') || trimmed.startsWith('@')) return trimmed;
  const match = trimmed.match(/^#([0-9a-fA-F]{3,4}|[0-9a-fA-F]{6,8})$/);
  if (!match) return trimmed;
  const digits =
    match[1].length <= 4
      ? [...match[1]].map((digit) => `${digit}${digit}`).join('')
      : match[1];
  return `#${digits.toLowerCase()}`;
}

function normalizeTextAlign(value: string): string {
  const normalized = value.trim().toLowerCase().replaceAll('_', '-');
  return ['left', 'center', 'right', 'justify'].includes(normalized)
    ? normalized
    : value;
}

function categoryForSource(path: string): string {
  if (path.startsWith('examples/woc/')) return 'examples-woc';
  if (path.startsWith('zircon_editor/assets/ui/editor/')) return 'editor-ui';
  if (path.startsWith('zircon_editor/assets/ui/theme/')) return 'editor-theme';
  if (path.startsWith('zircon_editor/src/tests/fixtures/ui_zui/editor/'))
    return 'editor-fixtures';
  if (path.startsWith('zircon_editor/src/tests/fixtures/ui_zui/theme/'))
    return 'theme-fixtures';
  if (path.startsWith('zircon_runtime/assets/ui/')) return 'runtime-ui';
  if (path.startsWith('zircon_runtime/tests/fixtures/ui/'))
    return 'runtime-fixtures';
  if (path.startsWith('zircon_plugins/')) {
    const owner = path.split('/')[1] || 'other';
    return `plugin-${slug(owner)}`;
  }
  return slug(path.split('/')[0] || 'other') || 'other';
}

function slug(value: string): string {
  return value
    .replace(/\.drawer$/i, '')
    .replace(/[^a-zA-Z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .toLowerCase()
    .slice(0, 64);
}

function stableHash(value: string): string {
  let hash = 2_166_136_261;
  for (const character of value) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 16_777_619);
  }
  return (hash >>> 0).toString(16).padStart(8, '0');
}

function uniqueChanges(values: string[]): string[] {
  return [...new Set(values)];
}

function asRecord(value: unknown): AnyRecord | undefined {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as AnyRecord)
    : undefined;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' ? value : undefined;
}

function finiteNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

// Keep the imported value aliases visible to TypeScript's structural checker;
// smol-toml accepts the same recursive value shape as the ZUI document model.
void (undefined as unknown as ZuiValue);
