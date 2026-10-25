import { nativePainterComponent } from './bridge/zui-native-painter-role';
import featherIcons from 'feather-icons/dist/icons.json';
import { recolorIconShape } from './penpot-prefab-icons';
import { assertExportShapeGuard } from './penpot-export-guard';
import { guardedPathContent } from './penpot-icon-path-content';
import { resolveCanonicalComponent } from './penpot-canonical-library';
import { inheritedTreeRowPainterForCopy } from './penpot-native-component-copy';
import type { Board, Shape, Text } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import type { ProjectionText } from './bridge/penpot-projection-model';
import {
  applyPenpotTextStyle,
  resolvePenpotTextStyle,
  type PenpotTextStyle,
} from './penpot-text-style';
import { PENPOT_PALETTE } from './bridge/zui-prefab-system';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
} from './metadata';
import {
  nativePainterContent,
  type NativePainterIcon,
  type NativePainterPanel,
  type NativePainterText,
  type NativePainterTone,
} from './penpot-native-painter-content';

type ContentPlacement = {
  x: number;
  y: number;
  width: number;
  height: number;
  exactBounds?: boolean;
};

const NATIVE_PAINTER_COLORS: Record<NativePainterTone, string> = {
  accent: PENPOT_PALETTE.accentStrong,
  danger: PENPOT_PALETTE.danger,
  info: PENPOT_PALETTE.info,
  muted: PENPOT_PALETTE.textMuted,
  primary: PENPOT_PALETTE.text,
  selected: PENPOT_PALETTE.surfaceSelected,
  success: PENPOT_PALETTE.accent,
  surface: PENPOT_PALETTE.surfaceRaised,
  warning: PENPOT_PALETTE.warning,
};

function addText(
  board: Board,
  value: string,
  placement: ContentPlacement,
  selected = false,
  style?: PenpotTextStyle,
  tone?: NativePainterTone,
  color?: string,
): Text {
  const text = penpot.createText(value);
  if (!text) throw new Error('Unable to create control content');
  applyPenpotTextStyle(
    text,
    style ?? {
      family: 'Fira Sans',
      id: 'firasans',
      size: 14,
      weight: selected ? '600' : '400',
      lineHeight: 1.4,
    },
  );
  text.growType = 'fixed';
  text.verticalAlign = 'center';
  text.fills = [
    {
      fillColor:
        color ??
        (tone
          ? NATIVE_PAINTER_COLORS[tone]
          : selected
            ? PENPOT_PALETTE.accent
            : PENPOT_PALETTE.text),
      fillOpacity: 1,
    },
  ];
  text.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  text.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-placement',
    JSON.stringify(placement),
  );
  board.appendChild(text);
  if (text.layoutChild) text.layoutChild.absolute = true;
  return text;
}

function addPanel(board: Board, panel: NativePainterPanel): Board {
  const child = penpot.createBoard();
  if (!child) throw new Error('Unable to create native painter content');
  child.name = `Penpot · ${panel.kind}`;
  child.resize(
    Math.max(1, board.width * panel.placement.width),
    Math.max(1, board.height * panel.placement.height),
  );
  child.x = board.x + board.width * panel.placement.x;
  child.y = board.y + board.height * panel.placement.y;
  child.fills = [
    {
      fillColor: panel.color?.slice(0, 7) ?? NATIVE_PAINTER_COLORS[panel.tone],
      fillOpacity: panel.opacity ?? (panel.tone === 'surface' ? 1 : 0.9),
    },
  ];
  child.borderRadius = panel.radius ?? 4;
  child.strokes = panel.borderColor
    ? [
        {
          strokeColor: panel.borderColor,
          strokeWidth: panel.borderWidth ?? 1,
          strokeOpacity: 1,
          strokeStyle: 'solid',
          strokeAlignment: 'inner',
        },
      ]
    : [];
  child.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  child.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-panel-placement',
    JSON.stringify(panel.placement),
  );
  board.appendChild(child);
  if (child.layoutChild) child.layoutChild.absolute = true;
  if (panel.placement.horizontalConstraint)
    child.constraintsHorizontal = panel.placement.horizontalConstraint;
  return child;
}

const NATIVE_ICON_EXTENTS = {
  s: 16,
  m: 20,
  l: 24,
  xl: 32,
} as const;

function addIcon(board: Board, icon: NativePainterIcon): void {
  if (typeof penpot.createShapeFromSvg !== 'function') return;
  const path = featherIcons[icon.name as keyof typeof featherIcons];
  if (!path && !icon.svg) return;
  const color = icon.color ?? NATIVE_PAINTER_COLORS[icon.tone];
  const shape = penpot.createShapeFromSvg(
    icon.svg ??
      `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${path}</svg>`,
  );
  if (!shape) return;
  if (icon.svg) recolorIconShape(shape, color);
  shape.name = `Penpot · icon · ${icon.name}`;
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-icon-placement',
    JSON.stringify(icon.placement),
  );
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-icon-size',
    icon.size,
  );
  board.appendChild(shape);
  if (shape.layoutChild) shape.layoutChild.absolute = true;
  if (icon.placement.horizontalConstraint)
    shape.constraintsHorizontal = icon.placement.horizontalConstraint;
  resizeNativeIcon(board, shape, icon.placement, icon.size);
}

function resizeNativeIcon(
  board: Board,
  shape: Shape,
  placement: ContentPlacement,
  size: NativePainterIcon['size'],
): void {
  if (placement.exactBounds) {
    shape.resize(
      Math.max(1, board.width * placement.width),
      Math.max(1, board.height * placement.height),
    );
    shape.x = board.x + board.width * placement.x;
    shape.y = board.y + board.height * placement.y;
    return;
  }
  const availableWidth = Math.max(1, board.width * placement.width - 8);
  const availableHeight = Math.max(1, board.height * placement.height - 8);
  const extent = Math.max(
    8,
    Math.min(NATIVE_ICON_EXTENTS[size], availableWidth, availableHeight),
  );
  shape.resize(extent, extent);
  shape.x =
    board.x +
    board.width * placement.x +
    (board.width * placement.width - extent) / 2;
  shape.y =
    board.y +
    board.height * placement.y +
    (board.height * placement.height - extent) / 2;
}

function nativeTextStyle(
  style: PenpotTextStyle | undefined,
  text: NativePainterText,
): PenpotTextStyle {
  const base =
    style ??
    ({
      family: 'Fira Sans',
      id: 'firasans',
      size: 14,
      weight: '400',
      lineHeight: 1.4,
    } satisfies PenpotTextStyle);
  return {
    ...base,
    size: text.size ?? base.size,
    weight: text.weight ?? base.weight,
    lineHeight: text.lineHeight ?? base.lineHeight,
  };
}

const TREE_ROW_PAINTER = 'tree-row-authoring-painter';
type TreeRowPaint = { fills: Shape['fills']; strokes: Shape['strokes']; opacity: Shape['opacity']; radius?: number };
function withoutUnsetPaintFields<T extends object>(value: T, fields: readonly string[]): T {
  const result = { ...value } as Record<string, unknown>;
  for (const key of fields) if (result[key] === null || result[key] === undefined) delete result[key];
  return result as T;
}
/** Match the SDK parser's parse-hex only for the paint color fields we write. */
function canonicalTreeRowPaint(paint: TreeRowPaint): TreeRowPaint {
  return { ...paint,
    fills: Array.isArray(paint.fills) ? paint.fills.map((fill) => {
      const result = withoutUnsetPaintFields(fill, ['fillColorGradient', 'fillColorRefFile', 'fillColorRefId', 'fillImage']);
      return typeof result.fillColor === 'string' ? { ...result, fillColor: result.fillColor.toLowerCase() } : result;
    }) : paint.fills,
    strokes: paint.strokes.map((stroke) => {
      const result = withoutUnsetPaintFields(stroke, ['strokeColorRefFile', 'strokeColorRefId', 'strokeCapStart', 'strokeCapEnd', 'strokeColorGradient', 'strokeImage']);
      return typeof result.strokeColor === 'string' ? { ...result, strokeColor: result.strokeColor.toLowerCase() } : result;
    }),
  };
}
type TreeRowGeometry = { x: number; y: number; width: number; height: number };
type TreeRowPartState = {
  type: string; parent: string | null; children: string[];
  paint?: TreeRowPaint; geometry?: TreeRowGeometry; content?: string;
  hidden?: boolean; rotation?: number; flipX?: boolean; flipY?: boolean;
};
type TreeRowSvgSource = { width: number; height: number; drawables: number; widths: number[]; viewportWidths: number[] };
interface TreeRowPainterState {
  version: 1;
  parts: string[];
  size: { width: number; height: number };
  panels: NativePainterPanel[];
  icons: NativePainterIcon[];
  iconSources: TreeRowSvgSource[];
  desired?: string;
  generated?: Record<string, TreeRowPartState>;
  originalPaint?: Record<string, TreeRowPaint>;
  originalGeometry?: Record<string, TreeRowGeometry>;
  originalBounds?: Record<string, { width: number; height: number }>;
  originalWidths?: Record<string, number[]>;
}

function sameTreeRowValue(left: unknown, right: unknown): boolean {
  if (left === right) return true;
  if (!left || !right || typeof left !== 'object' || typeof right !== 'object') return false;
  if (Array.isArray(left) || Array.isArray(right))
    return Array.isArray(left) && Array.isArray(right) && left.length === right.length &&
      left.every((value, index) => sameTreeRowValue(value, right[index]));
  const l = left as Record<string, unknown>, r = right as Record<string, unknown>;
  const keys = Object.keys(l);
  return keys.length === Object.keys(r).length && keys.every((key) =>
    Object.prototype.hasOwnProperty.call(r, key) && sameTreeRowValue(l[key], r[key]));
}

function treeRowPainterSize(board: Board): { width: number; height: number } {
  const raw = board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-layout-allocation');
  if (raw) {
    const allocation = JSON.parse(raw) as { parent: string | null; width: number; height: number };
    if (allocation.parent !== (board.parent?.id ?? null))
      throw new Error('TreeRow native layout allocation belongs to a different parent.');
    if (!Number.isFinite(allocation.width) || !Number.isFinite(allocation.height) ||
        allocation.width < 0 || allocation.height < 0)
      throw new Error('TreeRow has invalid native layout allocation.');
    return { width: allocation.width, height: allocation.height };
  }
  // Initial materialization can precede the first published native allocation.
  return { width: board.width, height: board.height };
}

function treeRowParts(board: Board): Shape[] {
  return board.children.filter((child) =>
    child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-panel-placement') ||
    child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-icon-placement'));
}

function treeRowGeometry(shape: Shape): TreeRowGeometry {
  return { x: shape.x - (shape.parent?.x ?? 0), y: shape.y - (shape.parent?.y ?? 0),
    width: shape.width, height: shape.height };
}

function guardedTreeRowGeometry(geometry: TreeRowGeometry): TreeRowGeometry {
  // Use the same parent-relative coordinates and 2dp representation as
  // penpot-export-guard guardedState; no additional tolerance is introduced.
  const round = (value: number) => Math.round(value * 100) / 100;
  return { x: round(geometry.x), y: round(geometry.y), width: round(geometry.width), height: round(geometry.height) };
}

function treeRowState(board: Board): Record<string, TreeRowPartState> {
  const result: Record<string, TreeRowPartState> = {};
  const visit = (shape: Shape, painted: boolean): void => {
    const children = 'children' in shape ? shape.children : [];
    result[shape.id] = {
      type: shape.type, parent: shape.parent?.id ?? null, children: children.map((child) => child.id),
      ...(painted ? {
        geometry: guardedTreeRowGeometry(treeRowGeometry(shape)),
        hidden: shape.hidden, rotation: shape.rotation, flipX: shape.flipX, flipY: shape.flipY,
        paint: JSON.parse(JSON.stringify({
          fills: shape.fills, strokes: shape.strokes, opacity: shape.opacity,
          ...(shape.type === 'board' ? { radius: shape.borderRadius } : {}),
        })) as TreeRowPaint,
      } : {}),
      ...(painted && shape.type === 'path' ? { content: guardedPathContent(shape) } : {}),
    };
    for (const child of children)
      visit(child, painted || !!child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-panel-placement') ||
        !!child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-icon-placement'));
  };
  visit(board, false);
  return JSON.parse(JSON.stringify(result)) as Record<string, TreeRowPartState>;
}

function treeRowAsset(board: Board): Shape {
  let asset: Shape = board;
  while (asset.parent && asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !== 'asset') asset = asset.parent;
  return asset;
}

function sameTreeRowState(board: Board, before: Record<string, TreeRowPartState>, after: Record<string, TreeRowPartState>): boolean {
  // The SDK exposes absent optional paint fields as either missing or nil.
  // Compare those documented empty representations without changing the
  // captured metadata; nonempty references, paint and unknown fields remain exact.
  const canonical = (state: Record<string, TreeRowPartState>) => Object.fromEntries(
    Object.entries(state).map(([id, part]) => [id, part.paint ? { ...part, paint: canonicalTreeRowPaint(part.paint) } : part]));
  before = canonical(before); after = canonical(after);
  if (sameTreeRowValue(before, after)) return true;
  const withoutPaths = (state: Record<string, TreeRowPartState>) => Object.fromEntries(
    Object.entries(state).map(([id, { content: _content, ...part }]) => [id, part]));
  if (!sameTreeRowValue(withoutPaths(before), withoutPaths(after))) return false;
  // Only the existing export guard can accept its established generated-path
  // serialization equivalence. The captured guard remains unchanged.
  const asset = treeRowAsset(board);
  if (asset.type !== 'board' || !asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard')) return false;
  assertExportShapeGuard(asset);
  return true;
}

/** Bounded failure evidence only; never accepts a change or updates a baseline. */
function treeRowStateDiagnostic(board: Board, before: Record<string, TreeRowPartState>, current: Record<string, TreeRowPartState>): string {
  const limits = { differences: 32, depth: 8, text: 160, visits: 4096 };
  const differences: Array<Record<string, unknown>> = [];
  let truncated = false, visits = 0;
  const text = (value: string): string => {
    if (value.length > limits.text) truncated = true;
    return value.slice(0, limits.text);
  };
  const value = (input: unknown): unknown => {
    if (input === undefined) return { kind: 'missing' };
    if (typeof input === 'string') return text(input);
    if (typeof input === 'number' && !Number.isFinite(input)) return String(input);
    if (input && typeof input === 'object') {
      if (Array.isArray(input)) return { kind: 'array', length: input.length };
      const keys = Object.keys(input);
      if (keys.length > limits.differences) truncated = true;
      return { kind: 'object', keys: keys.slice(0, limits.differences).map(text) };
    }
    return input;
  };
  const visit = (id: string, path: string[], left: unknown, right: unknown): void => {
    if (left === right) return;
    if (differences.length >= limits.differences || ++visits > limits.visits) { truncated = true; return; }
    if (left && right && typeof left === 'object' && typeof right === 'object' &&
        Array.isArray(left) === Array.isArray(right) && path.length < limits.depth) {
      if (Array.isArray(left) && Array.isArray(right) && left.length !== right.length)
        visit(id, [...path, 'length'], left.length, right.length);
      const l = left as Record<string, unknown>, r = right as Record<string, unknown>;
      const keys = new Set([...Object.keys(l), ...Object.keys(r)]);
      for (const key of keys) {
        if (differences.length >= limits.differences || visits >= limits.visits) { truncated = true; break; }
        visit(id, [...path, key], l[key], r[key]);
      }
      return;
    }
    if (path.length >= limits.depth && left && right && typeof left === 'object' && typeof right === 'object') truncated = true;
    const field = path[0];
    const category = field === 'geometry' || field === 'paint' || field === 'content' ? field :
      !field || ['type', 'parent', 'children'].includes(field) ? 'topology' : 'state';
    const shape = current[id] ?? before[id];
    differences.push({ shape: { id: text(id), type: shape ? text(shape.type) : null,
      parent: shape?.parent === undefined || shape.parent === null ? null : text(shape.parent) },
    category, path: path.map(text), before: value(left), current: value(right) });
  };
  for (const id of new Set([...Object.keys(before), ...Object.keys(current)])) {
    if (differences.length >= limits.differences || visits >= limits.visits) { truncated = true; break; }
    visit(id, [], before[id], current[id]);
  }
  return JSON.stringify({ row: { id: text(board.id), name: text(board.name),
    nodeId: text(board.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID)) },
  differences, truncated, limits });
}

function treeRowSvgSource(icon: NativePainterIcon): TreeRowSvgSource {
  const svg = icon.svg ?? '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2">' +
    featherIcons[icon.name as keyof typeof featherIcons] + '</svg>';
  const root = /<svg\b([^>]*)>/i.exec(svg)?.[1];
  if (root === undefined) throw new Error('TreeRow icon has no SVG source: ' + icon.name);
  const attribute = (attributes: string, name: string) => new RegExp('(?:^|\\s)' + name + '\\s*=\\s*(["\x27])(.*?)\\1', 'i').exec(attributes)?.[2];
  const viewBox = attribute(root, 'viewBox')?.trim().split(/[\s,]+/).map(Number);
  const width = viewBox ? viewBox[2] : Number.parseFloat(attribute(root, 'width') ?? '');
  const height = viewBox ? viewBox[3] : Number.parseFloat(attribute(root, 'height') ?? '');
  if ((viewBox && viewBox.length !== 4) || !Number.isFinite(width) || !Number.isFinite(height) || width! <= 0 || height! <= 0)
    throw new Error('TreeRow icon has an invalid original SVG viewport.');
  const inherited: Array<{ stroke: string; width: number }> = [{ stroke: 'none', width: 1 }];
  const source: TreeRowSvgSource = { width: width!, height: height!, drawables: 0, widths: [], viewportWidths: [] };
  for (const [, closing, tag, attributes] of svg.matchAll(/<(\/?)([a-z][\w:-]*)\b([^>]*)>/gi)) {
    if (closing) { inherited.pop(); continue; }
    const parent = inherited[inherited.length - 1]!;
    const style = attribute(attributes!, 'style');
    const property = (name: string) => style && new RegExp('(?:^|;)\\s*' + name + '\\s*:\\s*([^;]+)', 'i').exec(style)?.[1]?.trim();
    const stroke = property('stroke') || attribute(attributes!, 'stroke') || parent.stroke;
    const rawWidth = property('stroke-width') || attribute(attributes!, 'stroke-width');
    let strokeWidth = parent.width;
    if (rawWidth !== undefined && rawWidth !== '') {
      if (!/^(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?(?:px)?$/i.test(rawWidth.trim()))
        throw new Error('TreeRow icon has an unsupported source stroke width: ' + rawWidth);
      strokeWidth = Number.parseFloat(rawWidth);
      if (!Number.isFinite(strokeWidth) || strokeWidth < 0)
        throw new Error('TreeRow icon has an invalid source stroke width.');
    }
    if (tag!.toLowerCase() === 'svg' && inherited.length === 1 && stroke.toLowerCase() !== 'none') source.viewportWidths = [strokeWidth];
    if (/^(?:path|line|polyline|polygon|circle|ellipse|rect)$/i.test(tag!)) {
      source.drawables++;
      if (stroke.toLowerCase() !== 'none') source.widths.push(strokeWidth);
    }
    if (!/\/\s*$/.test(attributes!)) inherited.push({ stroke, width: strokeWidth });
  }
  return source;
}

function treeRowSvgViewport(shape: Shape): Shape | undefined {
  if (!('children' in shape)) return undefined;
  const first = shape.children[0];
  if (first?.type !== 'rectangle') return undefined;
  // The official SVG importer prepends its root viewport rectangle. Preserve
  // its captured identity/paint; root SVG stroke attributes also apply to it.
  return sameTreeRowValue(guardedTreeRowGeometry(treeRowGeometry(first)),
    guardedTreeRowGeometry({ x: 0, y: 0, width: shape.width, height: shape.height })) ? first : undefined;
}

/** Failure evidence only; never changes SVG readiness or captures a baseline. */
function treeRowSvgDiagnostic(
  shape: Shape,
  source: TreeRowSvgSource,
  observed: {
    stage: 'children' | 'strokes';
    actualDrawables?: number;
    geometryReady?: boolean;
    actualStrokeWidths?: number[];
    expectedStrokeWidths?: number[];
  },
): string {
  const limit = 32, maxDepth = 8;
  let truncated = false;
  const text = (value: string | undefined) => {
    if (value === undefined) return null;
    if (value.length > 160) truncated = true;
    return value.slice(0, 160);
  };
  const number = (value: number | undefined) =>
    value === undefined ? 'undefined' : Number.isFinite(value) ? value : String(value);
  const widths = (values: Array<number | undefined>) => {
    if (values.length > limit) truncated = true;
    return values.slice(0, limit).map(number);
  };
  const topology: Array<Record<string, unknown>> = [];
  const visit = (child: Shape, depth: number): void => {
    if (topology.length >= limit) { truncated = true; return; }
    const children = 'children' in child ? child.children : [];
    if (children.length > limit) truncated = true;
    topology.push({
      id: text(child.id), parent: text(child.parent?.id), type: text(child.type),
      name: text(child.name), hidden: child.hidden,
      relativeBounds: Object.fromEntries(Object.entries(treeRowGeometry(child))
        .map(([key, value]) => [key, number(value)])),
      childCount: children.length, childIds: children.slice(0, limit).map((nested) => text(nested.id)),
      strokeWidths: widths(child.strokes.map((stroke) => stroke.strokeWidth)),
    });
    if (depth >= maxDepth) { if (children.length) truncated = true; return; }
    for (const nested of children) {
      if (topology.length >= limit) { truncated = true; break; }
      visit(nested, depth + 1);
    }
  };
  visit(shape, 0);
  const first = 'children' in shape ? shape.children[0] : undefined;
  const viewport = treeRowSvgViewport(shape);
  const details = {
    owner: { id: text(shape.parent?.id), name: text(shape.parent?.name),
      nodeId: text(shape.parent?.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID)) },
    source: { width: number(source.width), height: number(source.height), drawables: source.drawables,
      strokeWidths: widths(source.widths), viewportStrokeWidths: widths(source.viewportWidths) },
    observed: { stage: observed.stage,
      ...(observed.actualDrawables !== undefined ? { actualDrawables: observed.actualDrawables } : {}),
      ...(observed.geometryReady !== undefined ? { geometryReady: observed.geometryReady } : {}),
      ...(observed.actualStrokeWidths ? { actualStrokeWidths: widths(observed.actualStrokeWidths) } : {}),
      ...(observed.expectedStrokeWidths ? { expectedStrokeWidths: widths(observed.expectedStrokeWidths) } : {}) },
    firstChildId: text(first?.id), firstChildIsRectangle: first?.type === 'rectangle',
    viewportMatched: viewport !== undefined, viewportId: text(viewport?.id),
    topology,
  };
  return JSON.stringify({ ...details, truncated, limits: { nodes: limit, depth: maxDepth, text: 160 } });
}

function treeRowIconReady(shape: Shape, source: TreeRowSvgSource): boolean {
  if (!Number.isFinite(shape.width) || !Number.isFinite(shape.height) || shape.width <= 0 || shape.height <= 0) return false;
  const viewport = treeRowSvgViewport(shape);
  let actual = 0, geometryReady = true;
  const visit = (child: Shape): void => {
    const geometry = treeRowGeometry(child);
    if (Object.values(geometry).some((value) => !Number.isFinite(value)) || geometry.width < 0 || geometry.height < 0) geometryReady = false;
    if ('children' in child && child.children.length) for (const nested of child.children) visit(nested);
    // Penpot returns a fresh proxy on each children read; IDs identify shapes.
    else if ((!viewport || child.id !== viewport.id) && child.type !== 'group' && child.type !== 'board') actual++;
  };
  visit(shape);
  if (actual > source.drawables) throw new Error('TreeRow SVG has added or unsupported children. SVG diagnostic: ' +
    treeRowSvgDiagnostic(shape, source, { stage: 'children', actualDrawables: actual, geometryReady }));
  return geometryReady && source.drawables > 0 && actual === source.drawables;
}

/** Reflow an authored TreeRow without replacing any captured native part. */
export function refreshTreeRowPainterContent(board: Board, node: ZuiNode, document: ZuiDocument): boolean {
  const raw = board.getSharedPluginData(ZUI_METADATA_NAMESPACE, TREE_ROW_PAINTER);
  if (!raw) throw new Error('TreeRow has no original painter metadata; reimport it.');
  let state = JSON.parse(raw) as TreeRowPainterState;
  if (state.version !== 1 || !Array.isArray(state.parts) || !Array.isArray(state.panels) || !Array.isArray(state.icons) ||
      !Array.isArray(state.iconSources) || state.iconSources.length !== state.icons.length)
    throw new Error('TreeRow has invalid original painter metadata; reimport it.');
  if (board.isComponentCopyInstance?.() && !state.generated)
    throw new Error('TreeRow linked copy requires the immutable original painter baseline.');
  const parts = treeRowParts(board);
  let inherited = false;
  if (!sameTreeRowValue(state.parts, parts.map((part) => part.id)) && board.isComponentCopyInstance?.()) {
    const component = board.component?.(), source = node['penpot_prefab_source'];
    const canonical = component && typeof source === 'string' ? resolveCanonicalComponent(source,
      [{ id: component.libraryId, components: [component] }]) : null;
    if (!canonical || !canonical.record.sourcePath ||
        node['penpot_review_source_path'] !== canonical.record.sourcePath ||
        node['penpot_review_source_node_id'] !==
          canonical.component.mainInstance().getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID))
      throw new Error('TreeRow linked copy has no canonical source owner in this projection.');
    state = JSON.parse(inheritedTreeRowPainterForCopy(canonical.component.mainInstance(), board)) as TreeRowPainterState;
    inherited = true;
  }
  if (!sameTreeRowValue(state.parts, parts.map((part) => part.id)))
    throw new Error('TreeRow children changed without a ZUI source mapping.');
  const current = treeRowState(board);
  if (state.generated && !sameTreeRowState(board, state.generated, current))
    throw new Error('TreeRow subtree, geometry or paint changed without a ZUI source mapping. State diagnostic: ' +
      treeRowStateDiagnostic(board, state.generated, current));
  const panels = parts.filter((part) => part.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-panel-placement'));
  const icons = parts.filter((part) => part.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-icon-placement'));
  if (panels.length !== state.panels.length || icons.length !== state.icons.length)
    throw new Error('TreeRow painter parts changed without a ZUI source mapping.');
  const size = treeRowPainterSize(board);
  if (!state.generated) {
    if (icons.some((part, index) => !treeRowIconReady(part, state.iconSources[index]!))) return false;
    const asset = treeRowAsset(board);
    if (asset.type === 'board' && asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard')) assertExportShapeGuard(asset);
    state.generated = current;
    state.originalPaint = Object.fromEntries(Object.entries(current).flatMap(([id, value]) => value.paint ? [[id, value.paint]] : []));
    state.originalGeometry = {};
    state.originalBounds = {};
    state.originalWidths = {};
    const capture = (part: Shape): void => {
      state.originalGeometry![part.id] = treeRowGeometry(part);
      state.originalWidths![part.id] = part.strokes.map((stroke) => stroke.strokeWidth!);
      if ('children' in part) for (const child of part.children) capture(child);
    };
    for (const part of panels) {
      capture(part);
      const { width, height } = state.originalGeometry[part.id]!;
      state.originalBounds[part.id] = { width, height };
    }
    for (const [index, part] of icons.entries()) {
      const widths: number[] = [];
      const inspect = (child: Shape): void => {
        widths.push(...child.strokes.map((stroke) => stroke.strokeWidth!));
        if ('children' in child) for (const nested of child.children) inspect(nested);
      };
      inspect(part);
      const source = state.iconSources[index]!;
      const expected = [...source.widths, ...(treeRowSvgViewport(part) ? source.viewportWidths : [])];
      if (widths.length < expected.length || widths.some((width) => width === undefined)) return false;
      if (widths.some((width) => !Number.isFinite(width)) ||
          !sameTreeRowValue([...widths].sort((a, b) => a - b), [...expected].sort((a, b) => a - b)))
        throw new Error('TreeRow original strokes do not match its SVG source. SVG diagnostic: ' +
          treeRowSvgDiagnostic(part, source, { stage: 'strokes', actualStrokeWidths: widths,
            expectedStrokeWidths: expected }));
      capture(part);
      // SDK resize scales descendants from the actual root selection bounds.
      const { width, height } = state.originalGeometry[part.id]!;
      state.originalBounds[part.id] = { width, height };
    }
  }
  if (!state.originalPaint || !state.originalGeometry || !state.originalBounds || !state.originalWidths)
    throw new Error('TreeRow has no immutable original SVG geometry and paint; reimport it.');
  const content = nativePainterContent(node, { document, ...size });
  if (!content || content.panels.length !== panels.length || (content.icons ?? []).length !== icons.length ||
      content.panels.some((panel, index) => panel.kind !== state.panels[index]!.kind) ||
      (content.icons ?? []).some((icon, index) => icon.name !== state.icons[index]!.name || icon.svg !== state.icons[index]!.svg))
    throw new Error('TreeRow painter topology changed; reimport the source for new SVG parts.');
  const desired = JSON.stringify({ size, panels: content.panels, icons: content.icons ?? [] });
  if (state.desired === desired) {
    if (!inherited) return false;
    // All original state, allocation and source-painter checks above still
    // apply. Persist only the derived IDs after that preflight succeeds.
    board.setSharedPluginData(ZUI_METADATA_NAMESPACE, TREE_ROW_PAINTER, JSON.stringify(state));
    return true;
  }

  // Preflight and plan the complete native tree before any geometry or paint
  // mutation. SVG widths and geometry references never use last SDK readback.
  const generated = JSON.parse(JSON.stringify(state.generated)) as Record<string, TreeRowPartState>;
  const paintWrites = new Map<Shape, TreeRowPaint>();
  const planGeometry = (part: Shape, placement: ContentPlacement): void => {
    const bounds = state.originalBounds![part.id];
    const width = size.width * placement.width, height = size.height * placement.height;
    if (!bounds || [bounds.width, bounds.height, width, height].some((value) => !Number.isFinite(value) || value <= 0))
      throw new Error('TreeRow native part has invalid original or desired bounds.');
    const scaleX = width / bounds.width, scaleY = height / bounds.height;
    const visit = (child: Shape, root: boolean): void => {
      const original = state.originalGeometry![child.id];
      if (!original) throw new Error('TreeRow original geometry does not cover its native tree.');
      generated[child.id]!.geometry = guardedTreeRowGeometry(root
        ? { x: size.width * placement.x, y: size.height * placement.y, width, height }
        : { x: original.x * scaleX, y: original.y * scaleY, width: original.width * scaleX, height: original.height * scaleY });
      if ('children' in child) for (const nested of child.children) visit(nested, false);
    };
    visit(part, true);
  };
  for (const [index, panel] of content.panels.entries()) {
    const part = panels[index]!;
    planGeometry(part, panel.placement);
    paintWrites.set(part, canonicalTreeRowPaint({
      opacity: current[part.id]!.paint!.opacity,
      fills: [{ fillColor: panel.color?.slice(0, 7) ?? NATIVE_PAINTER_COLORS[panel.tone], fillOpacity: panel.opacity ?? (panel.tone === 'surface' ? 1 : 0.9) }],
      strokes: panel.borderColor ? [{ strokeColor: panel.borderColor, strokeWidth: panel.borderWidth ?? 1, strokeOpacity: 1, strokeStyle: 'solid', strokeAlignment: 'inner' }] : [],
      radius: panel.radius ?? 4,
    }));
  }
  for (const [index, icon] of (content.icons ?? []).entries()) {
    const part = icons[index]!, source = state.iconSources[index]!;
    planGeometry(part, icon.placement);
    const scale = Math.min(size.width * icon.placement.width / source.width, size.height * icon.placement.height / source.height);
    const color = icon.color ?? NATIVE_PAINTER_COLORS[icon.tone];
    const visit = (child: Shape): void => {
      const original = state.originalPaint![child.id], widths = state.originalWidths![child.id];
      if (!original || !widths || widths.length !== original.strokes.length)
        throw new Error('TreeRow original paint does not cover its SVG tree.');
      const visible = (value: string | undefined) => !!value && !['none', 'transparent', '#00000000'].includes(value.toLowerCase());
      paintWrites.set(child, canonicalTreeRowPaint({ ...original,
        fills: Array.isArray(original.fills) ? original.fills.map((fill) => visible(fill.fillColor) ? { ...fill, fillColor: color } : fill) : original.fills,
        strokes: original.strokes.map((stroke, ordinal) => ({ ...stroke, strokeWidth: widths[ordinal]! * scale,
          ...(visible(stroke.strokeColor) ? { strokeColor: color } : {}) })),
      }));
      if ('children' in child) for (const nested of child.children) visit(nested);
    };
    visit(part);
  }
  for (const [part, paint] of paintWrites) generated[part.id]!.paint = paint;
  const applyGeometry = (part: Shape, before: ContentPlacement, after: ContentPlacement): void => {
    const original = state.originalBounds![part.id]!;
    // The first resize must use the same captured root bounds as its plan.
    const previousWidth = state.desired ? state.size.width * before.width : original.width;
    const previousHeight = state.desired ? state.size.height * before.height : original.height;
    const width = size.width * after.width, height = size.height * after.height;
    if (width !== previousWidth || height !== previousHeight) part.resize(width, height);
    const x = size.width * after.x, y = size.height * after.y;
    if (x !== state.size.width * before.x || (!state.desired && current[part.id]!.geometry!.x !== generated[part.id]!.geometry!.x)) part.x = board.x + x;
    if (y !== state.size.height * before.y || (!state.desired && current[part.id]!.geometry!.y !== generated[part.id]!.geometry!.y)) part.y = board.y + y;
  };
  for (const [index, panel] of content.panels.entries()) {
    const part = panels[index]!;
    applyGeometry(part, state.panels[index]!.placement, panel.placement);
    part.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-panel-placement', JSON.stringify(panel.placement));
  }
  for (const [index, icon] of (content.icons ?? []).entries()) {
    const part = icons[index]!;
    applyGeometry(part, state.icons[index]!.placement, icon.placement);
    part.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-icon-placement', JSON.stringify(icon.placement));
  }
  for (const [part, paint] of paintWrites) {
    const currentPaint = canonicalTreeRowPaint({fills: part.fills, strokes: part.strokes, opacity: part.opacity});
    if (Array.isArray(paint.fills) && !sameTreeRowValue(currentPaint.fills, paint.fills)) part.fills = paint.fills;
    if (!sameTreeRowValue(currentPaint.strokes, paint.strokes)) part.strokes = paint.strokes;
    if (part.type === 'board' && paint.radius !== undefined && part.borderRadius !== paint.radius) part.borderRadius = paint.radius;
    if (part.type === 'path') generated[part.id]!.content = guardedPathContent(part);
  }
  state.generated = generated;
  state.desired = desired;
  state.size = size;
  state.panels = content.panels;
  state.icons = content.icons ?? [];
  board.setSharedPluginData(ZUI_METADATA_NAMESPACE, TREE_ROW_PAINTER, JSON.stringify(state));
  return true;
}

function createNativePainterContent(
  board: Board,
  node: ZuiNode,
  style: PenpotTextStyle | undefined,
  document?: ZuiDocument,
): boolean {
  const size = nativePainterComponent(node) === 'TreeRow' ? treeRowPainterSize(board) : board;
  const content = nativePainterContent(
    node,
    document
      ? { document, width: size.width, height: size.height }
      : undefined,
  );
  if (!content) return false;
  for (const panel of content.panels) addPanel(board, panel);
  for (const icon of content.icons ?? []) addIcon(board, icon);
  const treeRow = nativePainterComponent(node) === 'TreeRow';
  if (treeRow) board.setSharedPluginData(ZUI_METADATA_NAMESPACE, TREE_ROW_PAINTER, JSON.stringify({
    version: 1, parts: treeRowParts(board).map((part) => part.id),
    size: { width: size.width, height: size.height }, panels: content.panels, icons: content.icons ?? [],
    iconSources: (content.icons ?? []).map(treeRowSvgSource),
  } satisfies TreeRowPainterState));
  // TreeRow uses the visible source-owned semantic text, not an auxiliary clone.
  for (const text of treeRow ? [] : content.texts)
    addText(
      board,
      text.value,
      text.placement,
      false,
      nativeTextStyle(style, text),
      text.tone,
      text.color,
    );
  return !treeRow;
}

export function createControlContent(
  board: Board,
  node: ZuiNode,
  document?: ZuiDocument,
  projected?: ProjectionText | null,
): boolean {
  const style = document
    ? resolvePenpotTextStyle(document, node, projected)
    : undefined;
  const component = node.component.toLowerCase();
  const options = node.props?.['options'];
  const items = node.props?.['items'];
  let replacesText = createNativePainterContent(board, node, style, document);
  // Empty slots have no product pixels. Their review hosts must supply a real
  // instance; drawing a label or dashed guide would turn missing content into
  // a misleading visual substitute and hide slot contract failures.
  if (
    !replacesText &&
    (component === 'table' || component === 'segmentedcontrol') &&
    Array.isArray(options) &&
    options.length
  ) {
    const count = options.length;
    options.forEach((value, index) => {
      const label = String(value).split('|')[0];
      addText(
        board,
        label,
        { x: index / count, y: 0, width: 1 / count, height: 1 },
        label === node.props?.['value'],
        style,
      );
    });
    replacesText = true;
  } else if (
    !replacesText &&
    ['virtuallist', 'pagedlist', 'list'].includes(component) &&
    Array.isArray(items)
  ) {
    const rowHeight = Number(node.props?.['item_extent'] ?? 24);
    const count = Math.min(
      items.length,
      Math.max(1, Math.floor(board.height / rowHeight)),
    );
    for (let index = 0; index < count; index++) {
      const value = items[index];
      if (typeof value === 'string')
        addText(
          board,
          value,
          {
            x: 0,
            y: index / count,
            width: 1,
            height: 1 / count,
          },
          false,
          style,
        );
    }
    replacesText = count > 0;
  } else if (
    !replacesText &&
    component === 'slider' &&
    typeof node.props?.['value'] === 'number'
  ) {
    addText(
      board,
      Number(node.props['value']).toFixed(2),
      { x: 0.78, y: 0, width: 0.22, height: 1 },
      false,
      style,
    );
    replacesText = true;
  } else if (
    !replacesText &&
    !projected &&
    (component.includes('number') ||
      component.includes('vector') ||
      component.includes('range'))
  ) {
    const value = node.props?.['value'] ?? node.props?.['value_number'];
    if (typeof value === 'number' || Array.isArray(value)) {
      addText(
        board,
        Array.isArray(value) ? value.join('    ') : String(value),
        { x: 0, y: 0, width: 1, height: 1 },
        false,
        style,
      );
      replacesText = true;
    }
  }
  if (nativePainterComponent(node) !== 'TreeRow') refreshControlContent(board);
  return replacesText;
}

/**
 * Rebuild only the design-time content painted by a native Runtime control.
 *
 * A retained host review can change a popup/painter state without changing
 * the authored component tree.  Keeping this operation separate from the
 * generic control-content path lets the review host update a cloned board
 * without rematerializing every native component in a large Workbench.
 */
export function refreshNativePainterContent(
  board: Board,
  node: ZuiNode,
  document?: ZuiDocument,
  projected?: ProjectionText | null,
): boolean {
  const painterChildren = board.children.filter(
    (child) =>
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-placement') ||
      child.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'content-panel-placement',
      ) ||
      child.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'content-icon-placement',
      ),
  );
  for (const child of painterChildren) child.remove();
  const style = document
    ? resolvePenpotTextStyle(document, node, projected)
    : undefined;
  const painted = createNativePainterContent(board, node, style, document);
  if (painted) refreshControlContent(board);
  return painted;
}

export function refreshControlContent(board: Board): void {
  for (const child of board.children) {
    const decoration = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'control-decoration',
    );
    if (decoration) {
      const sliderValueLane =
        decoration === 'slider' && board.width >= 96 ? 56 : 0;
      const railWidth = Math.max(16, board.width - 16 - sliderValueLane);
      if (child.name.endsWith(' rail')) {
        child.resize(railWidth, 4);
        child.x = board.x + 8;
        child.y = board.y + (board.height - child.height) / 2;
      } else {
        const fraction = Number(
          child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'control-fraction'),
        );
        const thumbWidth = decoration === 'splitter' ? 8 : 12;
        const thumbHeight =
          decoration === 'splitter' ? Math.max(16, board.height - 10) : 12;
        child.resize(thumbWidth, thumbHeight);
        child.x =
          board.x +
          8 +
          (railWidth - thumbWidth) * (Number.isFinite(fraction) ? fraction : 0);
        child.y = board.y + (board.height - child.height) / 2;
      }
      continue;
    }
    const panelStored = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'content-panel-placement',
    );
    if (panelStored) {
      const placement = JSON.parse(panelStored) as ContentPlacement;
      child.resize(
        Math.max(1, board.width * placement.width),
        Math.max(1, board.height * placement.height),
      );
      child.x = board.x + board.width * placement.x;
      child.y = board.y + board.height * placement.y;
      continue;
    }
    const iconStored = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'content-icon-placement',
    );
    if (iconStored) {
      const placement = JSON.parse(iconStored) as ContentPlacement;
      const size = child.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'content-icon-size',
      );
      if (size === 's' || size === 'm' || size === 'l' || size === 'xl') {
        resizeNativeIcon(board, child as Shape, placement, size);
      }
      continue;
    }
    const stored = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'content-placement',
    );
    if (!stored) continue;
    const placement = JSON.parse(stored) as {
      x: number;
      y: number;
      width: number;
      height: number;
      exactBounds?: boolean;
    };
    child.resize(
      Math.max(
        1,
        board.width * placement.width - (placement.exactBounds ? 0 : 16),
      ),
      Math.max(1, board.height * placement.height),
    );
    child.x =
      board.x + board.width * placement.x + (placement.exactBounds ? 0 : 8);
    child.y = board.y + board.height * placement.y;
  }
}
