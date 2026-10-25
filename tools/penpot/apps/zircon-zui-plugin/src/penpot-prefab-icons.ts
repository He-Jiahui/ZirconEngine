import icons from 'feather-icons/dist/icons.json';
import type { Board, Shape, Text } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { localizedText } from './bridge/penpot-projection';
import { recordNativeSemanticLayoutGeometry } from './penpot-native-semantic-layout-geometry';
import { buttonStatePreviewNode } from './bridge/zui-button-state-projection';
import { styledPreviewNode } from './bridge/zui-style-projection';
import {
  editorButtonPalette,
  editorButtonState,
  editorTabCloseStyle,
} from './bridge/zui-editor-button-style';
import { editorButtonTabKind, isEditorTabCloseIconButton, isEditorWorkbenchButton, usesWorkbenchLanguage } from './bridge/zui-editor-button-identity';
import { hasEditorButtonPainter, hostButtonPreviewNode } from './bridge/zui-editor-button-projection';
import { PENPOT_PALETTE, resolveDesignColor, resolveDesignNumber } from './bridge/zui-prefab-system';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_TEXT,
} from './metadata';

const DETAILS_SOURCE = 'zircon_editor/assets/ui/editor/unreal/components/details.zui';
const WINDOW_SOURCE = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const SHELL_SOURCE = 'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
const HEADER_SOURCE = 'zircon_editor/assets/ui/editor/unreal/components/dock_header.zui';
const STATUS_SOURCE = 'zircon_editor/assets/ui/editor/unreal/components/status_bar.zui';
const TAB_ASSETS: Record<string, string> = {
  console: 'ionicons/terminal-outline.svg',
  'runtime-diagnostics': 'ionicons/grid-outline.svg',
  'build-export': 'ionicons/share-outline.svg',
};
type RecordValue = Record<string, unknown>;
function object(value: unknown): RecordValue {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? value as RecordValue : {};
}
function sourcePath(value: unknown): string | null {
  if (typeof value !== 'string') return null;
  if (value.startsWith('res://ui/')) return 'zircon_editor/assets/' + value.slice(6);
  return value.startsWith('zircon_editor/assets/ui/') ? value : null;
}
function sourceSteps(node: ZuiNode): Array<{ sourcePath: string; sourceNodeId: string }> | null {
  try {
    const steps: unknown = JSON.parse(String(node['penpot_review_instance_path']));
    if (!Array.isArray(steps) || steps.length > 3) return null;
    const result = [];
    for (const raw of steps) {
      const step = object(raw), path = sourcePath(step['sourcePath']);
      if (!path || typeof step['sourceNodeId'] !== 'string' || !step['sourceNodeId'] ||
        Object.keys(step).sort().join(',') !== 'sourceNodeId,sourcePath') return null;
      result.push({ sourcePath: path, sourceNodeId: step['sourceNodeId'] });
    }
    return result;
  } catch { return null; }
}
function detailsInvocation(steps: NonNullable<ReturnType<typeof sourceSteps>>): boolean {
  // Direct component review, the actual owner, and its established shell mount.
  return steps.length === 0 ||
    JSON.stringify(steps) === JSON.stringify([{ sourcePath: WINDOW_SOURCE, sourceNodeId: 'details' }]) ||
    JSON.stringify(steps) === JSON.stringify([{ sourcePath: SHELL_SOURCE, sourceNodeId: 'pane_surface' },
      { sourcePath: WINDOW_SOURCE, sourceNodeId: 'details' }]);
}
interface NativeSvgGlyph {
  family: 'add-component' | 'section-transform' | 'section-mesh' | 'bottom-tab';
  assetPath: string;
  sourceScale: number;
}
export type NativeSemanticGlyph = NativeSvgGlyph | {
  family: 'status-signal';
  assetPath: null;
  sourceScale: 1;
  signal: 'success' | 'warning' | 'info';
};

/** Painter admission includes the complete real Source invocation, not a caption/control guess. */
export function nativeSemanticGlyph(document: ZuiDocument, node: ZuiNode): NativeSemanticGlyph | null {
  if (typeof document['penpot_host_theme_source'] !== 'string') return null;
  const path = sourcePath(node['penpot_review_source_path']);
  const id = node['penpot_review_source_node_id'], steps = sourceSteps(node);
  if (!path || !steps) return null;
  if (node.component === 'Label' && path === STATUS_SOURCE) {
    const owner = [{ sourcePath: WINDOW_SOURCE, sourceNodeId: 'status_bar' }];
    const invocations = [[], owner, [{ sourcePath: SHELL_SOURCE, sourceNodeId: 'pane_surface' }, ...owner]];
    const values = styledPreviewNode(document, { ...node, props: { ...node.props, ...node.state } }).props;
    const signals = [
      ['ready', 'WorkbenchStatusReady', 'success'], ['errors', 'WorkbenchStatusErrors', 'success'],
      ['warnings', 'WorkbenchStatusWarnings', 'warning'], ['messages', 'WorkbenchStatusMessages', 'info'],
    ] as const;
    const match = signals.find(([sourceId, controlId]) => sourceId === id && controlId === node.control_id);
    if (match && values?.['component_variant'] === 'semantic_status_signal' &&
        invocations.some((invocation) => JSON.stringify(invocation) === JSON.stringify(steps)))
      return { family: 'status-signal', assetPath: null, sourceScale: 1, signal: match[2] };
  }
  const buttonIdentity = [node.control_id, ...['text', 'value_text', 'button_variant',
    'surface_variant', 'validation_level'].map((key) => node.props?.[key])]
    .map((value) => String(value ?? '').toLowerCase());
  const differentButtonGlyph = ['delete', 'trash', 'danger', 'dropdown', 'drop-down', 'menu']
    .some((needle) => buttonIdentity.some((value) => value.includes(needle)));
  if (node.control_id === 'WorkbenchAddComponent' && isEditorWorkbenchButton(node) &&
    String(node.props?.['button_variant'] ?? '').toLowerCase() !== 'text' &&
    !differentButtonGlyph &&
    !node.props?.['icon'] && !node.props?.['icon_name'] &&
    path === 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui' &&
    id === 'root' && steps.at(-1)?.sourcePath === DETAILS_SOURCE && steps.at(-1)?.sourceNodeId === 'add' &&
    detailsInvocation(steps.slice(0, -1)))
    return { family: 'add-component', assetPath: 'zircon_editor_shell/controls/add.svg', sourceScale: 1 };
  if (node.component === 'Label' && path === DETAILS_SOURCE && detailsInvocation(steps)) {
    if (id === 'transform_title' && node.control_id === 'WorkbenchTransformLabel')
      return { family: 'section-transform', assetPath: 'zircon_editor_shell/inspector/transform.svg', sourceScale: 1 };
    if (id === 'component_title' && node.control_id === 'WorkbenchMeshLabel')
      return { family: 'section-mesh', assetPath: 'zircon_editor_shell/inspector/mesh-renderer.svg', sourceScale: 1 };
  }
  // WorkbenchInspectorTitle is not admitted by Native section_titles/identity.
  const tab = object(node['penpot_review_native_tab_icon']);
  if (tab['authority'] !== 'actual-workbench-bottom-tab') return null;
  const review = object(document['penpot_actual_normal_review_case']);
  const dpi = review['dpi'], viewport = object(review['viewport']), lane = object(tab['sourceViewport']);
  const presentation = object(object(review['data'])['workbenchPresentation']);
  const owner = object(presentation['sourceFingerprint']);
  const headers = presentation['dockHeaders'];
  const bottomHeaders = Array.isArray(headers) ? headers.filter((item) =>
    object(item)['kind'] === 'bottom' && object(item)['surfaceKey'] === 'bottom') : [];
  const sourceViewport = object(object(bottomHeaders[0])['viewport']);
  if (review['host'] !== 'editor' || sourcePath(review['sourcePath']) !== WINDOW_SOURCE ||
    typeof dpi !== 'number' || ![1, 1.25, 1.5].includes(dpi) ||
    typeof viewport['width'] !== 'number' || lane['width'] !== viewport['width'] * dpi ||
    typeof lane['height'] !== 'number' || !Number.isFinite(lane['height']) || lane['height'] <= 0 ||
    tab['sourceUnits'] !== 'physical-pixels' ||
    bottomHeaders.length !== 1 || sourcePath(object(bottomHeaders[0])['sourcePath']) !== HEADER_SOURCE ||
    sourceViewport['width'] !== lane['width'] || sourceViewport['height'] !== lane['height'] ||
    typeof owner['sha256'] !== 'string' || !/^[0-9a-f]{64}$/.test(owner['sha256']) ||
    typeof presentation['stateFingerprint'] !== 'string' || !/^[0-9a-f]{64}$/.test(presentation['stateFingerprint']) ||
    sourcePath(owner['sourcePath']) !== WINDOW_SOURCE || owner['sha256'] !== tab['ownerSha256'] ||
    presentation['stateFingerprint'] !== tab['stateFingerprint']) return null; // Missing actual unit binding stays unknown.
  const drawers = presentation['drawers'];
  const bottom = Array.isArray(drawers) ? drawers.filter((item) => object(item)['slot'] === 'bottom') : [];
  const tabs = object(bottom[0])['tabs'];
  const businessTabs = Array.isArray(tabs) ? tabs : [];
  const matches = businessTabs.filter((item) => object(item)['id'] === tab['tabId']);
  const business = object(matches[0]);
  const slot = typeof id === 'string' ? /^dock_tab_([0-2])$/.exec(id) : null;
  if (bottom.length !== 1 || object(bottom[0])['visible'] !== true || businessTabs.length > 3 ||
    typeof tab['tabId'] !== 'string' || !tab['tabId'] || typeof tab['descriptorId'] !== 'string' || !tab['descriptorId'] ||
    matches.length !== 1 || !slot ||
    businessTabs[Number(slot[1])] !== matches[0] || node.component !== 'Label' || path !== HEADER_SOURCE ||
    steps.length !== 0 || tab['sourcePath'] !== path || tab['sourceNodeId'] !== id || tab['instancePath'] !== '[]' ||
    tab['surfaceKey'] !== 'bottom' || node['penpot_review_runtime_surface_key'] !== 'bottom' ||
    tab['drawerSlot'] !== 'bottom' || node.control_id !== 'DockTab' + slot[1] ||
    tab['title'] !== node.props?.['text'] || tab['selected'] !== node.props?.['selected'] ||
    business['title'] !== tab['title'] || business['descriptorId'] !== tab['descriptorId'] ||
    business['iconKey'] !== tab['iconKey'] ||
    (object(bottom[0])['activeTabId'] === tab['tabId']) !== tab['selected'] ||
    typeof tab['iconKey'] !== 'string' || TAB_ASSETS[tab['iconKey']] !== tab['assetPath']) return null;
  return { family: 'bottom-tab', assetPath: tab['assetPath'] as string, sourceScale: dpi };
}

function hostMetric(document: ZuiDocument, key: string, positive = false): number {
  const value = resolveDesignNumber(document, document.tokens?.[key]);
  if (value === null || !Number.isFinite(value) || value < 0 || (positive && value === 0))
    throw new Error('Native semantic glyph requires host token ' + key);
  return value;
}
interface ContentRect { x: number; y: number; width: number; height: number }
const NATIVE_TAB_STROKES = 'native-semantic-glyph-source-strokes';
interface NativeTabStrokeSource {
  version: 1;
  width: number;
  height: number;
  widths: number[];
  originalWidths?: Record<string, number[]>;
  appliedScale?: number;
}

function nativeTabStrokeSource(svg: string): NativeTabStrokeSource {
  const root = /<svg\b([^>]*)>/i.exec(svg)?.[1];
  const viewBox = root && /\bviewBox\s*=\s*(["'])(.*?)\1/i.exec(root)?.[2]
    ?.trim().split(/[\s,]+/).map(Number);
  if (!viewBox || viewBox.length !== 4 || !viewBox.every(Number.isFinite) ||
    viewBox[2]! <= 0 || viewBox[3]! <= 0)
    throw new Error('Native tab icon has no valid source SVG viewport.');
  const widths = [...svg.matchAll(/\bstroke-width\s*(?:=\s*(["'])(.*?)\1|:\s*([^;"']+))/gi)].map((match) => {
    const raw = (match[2] ?? match[3] ?? '').trim();
    const width = Number.parseFloat(raw);
    if (!/^(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?(?:px)?$/i.test(raw) ||
      !Number.isFinite(width) || width < 0)
      throw new Error('Native tab icon has an unsupported source SVG stroke width.');
    return width;
  });
  if (widths.length === 0 || widths.length > 128)
    throw new Error('Native tab icon has no bounded source SVG strokes.');
  return { version: 1, width: viewBox[2]!, height: viewBox[3]!, widths };
}

/** Penpot resize scales SVG geometry but leaves strokes in document pixels. */
function scaleNativeTabStrokes(icon: Shape, rect: ContentRect): boolean {
  const raw = icon.getSharedPluginData(ZUI_METADATA_NAMESPACE, NATIVE_TAB_STROKES);
  if (!raw) throw new Error('Native tab icon has no original SVG stroke metadata; reimport it.');
  const source = JSON.parse(raw) as NativeTabStrokeSource;
  if (source.version !== 1 || !Number.isFinite(source.width) || source.width <= 0 ||
    !Number.isFinite(source.height) || source.height <= 0 || !Array.isArray(source.widths) ||
    source.widths.length === 0 || source.widths.length > 128 ||
    source.widths.some((width) => !Number.isFinite(width) || width < 0))
    throw new Error('Native tab icon has invalid original SVG stroke metadata.');
  const shapes: Shape[] = [];
  const visit = (shape: Shape): void => {
    shapes.push(shape);
    if ('children' in shape) for (const child of shape.children) visit(child);
  };
  visit(icon);
  const currentWidths = shapes.flatMap((shape) => shape.strokes.map((stroke) => stroke.strokeWidth));
  if (!source.originalWidths) {
    // The official SVG importer can return a proxy before its children exist.
    // Keep it hidden until every actual source stroke is available.
    if (currentWidths.length < source.widths.length) return false;
    if (currentWidths.length !== source.widths.length ||
      currentWidths.some((width) => width == null || !Number.isFinite(width)) ||
      JSON.stringify([...currentWidths].sort((a, b) => a! - b!)) !==
        JSON.stringify([...source.widths].sort((a, b) => a - b)))
      throw new Error('Native tab icon imported strokes differ from its SVG source.');
    source.originalWidths = Object.fromEntries(shapes.map((shape) => [shape.id,
      shape.strokes.map((stroke) => stroke.strokeWidth!)]));
  }
  const previousScale = source.appliedScale ?? 1;
  if (!Number.isFinite(previousScale) || previousScale <= 0 ||
    new Set(shapes.map((shape) => shape.id)).size !== shapes.length ||
    Object.keys(source.originalWidths).length !== shapes.length)
    throw new Error('Native tab icon SVG stroke identity changed.');
  // Validate the complete tree before any writes; fresh SDK proxies retain IDs.
  for (const shape of shapes) {
    const widths = source.originalWidths[shape.id];
    if (!Array.isArray(widths) || widths.length !== shape.strokes.length ||
      widths.some((width, index) => !Number.isFinite(width) || width < 0 ||
        !Number.isFinite(shape.strokes[index]!.strokeWidth) ||
        Math.abs(shape.strokes[index]!.strokeWidth! - width * previousScale) > 1e-6))
      throw new Error('Native tab icon stroke changed without a ZUI source mapping.');
  }
  const scale = Math.min(rect.width / source.width, rect.height / source.height);
  for (const shape of shapes) shape.strokes = shape.strokes.map((stroke, index) => ({
    ...stroke, strokeWidth: source.originalWidths![shape.id]![index]! * scale,
  }));
  source.appliedScale = scale;
  icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, NATIVE_TAB_STROKES, JSON.stringify(source));
  return true;
}

export interface NativeSemanticGlyphGeometry {
  icon: ContentRect | null;
  text: ContentRect;
  pendingMeasurement: boolean;
}

/** Only the regular retained Button content family shares these metrics. */
function nativeLeadingAssetButton(document: ZuiDocument, node: ZuiNode): ZuiNode | null {
  if (typeof document['penpot_host_theme_source'] !== 'string') return null;
  const styled = styledPreviewNode(document, { ...node, props: { ...node.props, ...node.state } });
  const asset = styled.props?.['icon'] ?? styled.props?.['icon_name'];
  if (!isEditorWorkbenchButton(styled) || editorButtonTabKind(styled) !== null ||
      String(styled.props?.['component_variant'] ?? '').split(/[\t\n\v\f\r ]+/)
        .some((token) => token.toLowerCase() === 'compact_icon_text') ||
      typeof asset !== 'string' || !asset.trim() ||
      previewIconPlacement(styled) !== 'leading') return null;
  // A trailing built-in chevron reserves another lane. Shrink also changes the
  // actual font measurement; neither belongs to this regular leading-only path.
  const identity = [styled.control_id, ...['text', 'value_text', 'button_variant',
    'surface_variant', 'validation_level'].map((key) => styled.props?.[key])]
    .map((value) => String(value ?? '').toLowerCase());
  const has = (needles: string[]) => identity.some((value) => needles.some((needle) => value.includes(needle)));
  if (String(styled.props?.['overflow'] ?? '').toLowerCase() === 'shrink' ||
      (String(styled.props?.['button_variant'] ?? '').toLowerCase() !== 'text' &&
       !has(['delete', 'trash', 'danger']) && has(['dropdown', 'drop-down', 'menu']))) return null;
  return styled;
}

function measuredButtonContentGeometry(document: ZuiDocument, node: ZuiNode,
  width: number, height: number, textHeight: number, size: number,
  measuredTextWidth?: number, withinPadding = false): NativeSemanticGlyphGeometry {
  if (![width, height, textHeight, size].every((value) => Number.isFinite(value) && value >= 0) || size === 0)
    throw new Error('Invalid native button content dimensions');
  if (measuredTextWidth !== undefined && (!Number.isFinite(measuredTextWidth) || measuredTextWidth < 0))
    throw new Error('Invalid actual native button font measurement');
  const large = hostMetric(document, 'editor.density.gap.large');
  const medium = hostMetric(document, 'editor.density.gap.medium');
  const border = hostMetric(document, 'editor.control.border_width');
  const pad = (key: string, fallback: number) => {
    const value = resolveDesignNumber(document, node.props?.[key]);
    return value !== null && Number.isFinite(value) && value >= 0 ? value : fallback;
  };
  const left = pad('layout_padding_left', large), right = pad('layout_padding_right', large);
  const top = pad('layout_padding_top', 0), bottom = pad('layout_padding_bottom', 0);
  const contentHeight = Math.max(0, height - top - bottom);
  const props = { ...node.props, ...node.state };
  const reference = props['text'];
  // Use the projection's translation owner while retaining the authored key.
  const localized = reference !== null && typeof reference === 'object' &&
    !Array.isArray(reference) && !(reference instanceof Date) && typeof reference['text_key'] === 'string'
    ? localizedText(document, reference) : undefined;
  const hasText = localized !== undefined ? localized.trim().length > 0 :
    ['text', 'value_text', 'value', 'label'].some((key) =>
      typeof props[key] === 'string' && String(props[key]).trim().length > 0);
  const declaredGap = ['layout_content_offset_x', 'layout_gap', 'layout_spacing', 'track_offset_x']
    .map((key) => resolveDesignNumber(document, node.props?.[key]))
    .find((value) => value !== null);
  const gap = typeof declaredGap === 'number' && declaredGap > 0 ? declaredGap : Math.max(0, medium - border);
  const glyphWidth = size + (hasText ? gap : 0);
  const room = Math.max(0, width - left - right), maxText = Math.max(0, room - glyphWidth);
  const pendingMeasurement = hasText && measuredTextWidth === undefined;
  const ink = hasText ? Math.min(maxText, measuredTextWidth ?? maxText) : 0;
  const start = left + Math.max(0, room - Math.min(room, ink + glyphWidth)) / 2;
  const offset = editorButtonState(node).interaction === 'pressed' ? border : 0;
  const icon = { x: start, y: top + Math.max(0, contentHeight - size) / 2 + offset, width: size, height: size };
  // Native tests the painted glyph against the outer button, not its text padding.
  const paintable = !pendingMeasurement && icon.x >= 0 && icon.y >= 0 &&
    icon.x + size <= width && icon.y + size <= height &&
    (!withinPadding || (icon.x + size <= width - right && icon.y >= top &&
      icon.y + size <= top + contentHeight));
  return { icon: paintable ? icon : null, pendingMeasurement,
    text: { x: start + glyphWidth,
      y: top + Math.max(0, contentHeight - textHeight) / 2 + offset,
      width: Math.min(maxText, ink + Math.max(0, medium - border * 2)),
      height: Math.min(contentHeight, textHeight) } };
}

/** Existing Native resolved metrics; no new authored icon-size value is introduced. */
export function nativeSemanticGlyphGeometry(document: ZuiDocument, node: ZuiNode,
  width: number, height: number, textHeight: number, measuredTextWidth?: number): NativeSemanticGlyphGeometry | null {
  const glyph = nativeSemanticGlyph(document, node);
  if (!glyph) {
    const button = nativeLeadingAssetButton(document, node);
    if (!button) return null;
    const declared = resolveDesignNumber(document, button.props?.['layout_icon_size']);
    const size = declared !== null && Number.isFinite(declared) && declared > 0 ? declared :
      Math.max(1, hostMetric(document, 'editor.density.row_height', true) -
        hostMetric(document, 'editor.density.gap.large'));
    return measuredButtonContentGeometry(document, button, width, height, textHeight, size, measuredTextWidth);
  }
  const small = hostMetric(document, 'editor.density.gap.small');
  const medium = hostMetric(document, 'editor.density.gap.medium');
  const border = hostMetric(document, 'editor.control.border_width');
  const body = hostMetric(document, 'editor.typography.body.size', true);
  let icon: ContentRect | null, text: ContentRect, pendingMeasurement = false;
  if (glyph.family === 'status-signal') {
    const offset = (key: string) => resolveDesignNumber(document, node.props?.[key]) ?? 0;
    const x = medium + offset('layout_offset_x');
    const y = Math.max(0, height - medium) / 2 + offset('layout_offset_y');
    const paintable = x >= 0 && y >= 0 && x + medium <= width && y + medium <= height;
    icon = paintable ? { x, y, width: medium, height: medium } : null;
    const declaredGap = resolveDesignNumber(document, node.props?.['layout_content_offset_x']);
    const gap = declaredGap !== null && declaredGap > 0 ? declaredGap : medium;
    text = { x: x + medium + gap,
      y: Math.max(0, height - textHeight) / 2 + offset('layout_offset_y'),
      width: Math.max(0, width - x - medium - gap), height: textHeight };
  } else if (glyph.family === 'add-component') {
    const large = hostMetric(document, 'editor.density.gap.large');
    const size = Math.max(1, hostMetric(document, 'editor.density.row_height', true) - large);
    // Keep the primitive's established inner-padding paint guard unchanged.
    return measuredButtonContentGeometry(document, node, width, height, textHeight, size, measuredTextWidth, true);
  } else if (glyph.family === 'bottom-tab') {
    // Header Source viewport inputs are physical; only this proven lane is
    // normalized once to DIP. Source/board geometry itself is never rewritten.
    const w = width / glyph.sourceScale, h = height / glyph.sourceScale;
    const inset = Math.min(small + border, Math.min(w, h) * 0.25);
    const size = Math.min(Math.max(0, Math.min(w, h) - inset * 2),
      Math.max(1 / glyph.sourceScale, Math.min(h - inset * 2, w * 0.28)));
    const horizontal = Math.min(small, w * 0.25);
    const physicalCaption = hostMetric(document, 'editor.typography.caption.size', true) * glyph.sourceScale;
    const minimumText = Math.max(Math.round(physicalCaption *
      hostMetric(document, 'editor.typography.line_height', true)), Math.ceil(physicalCaption)) / glyph.sourceScale;
    const vertical = Math.min(small, Math.min(Math.max(border, (h - minimumText) / 2), h / 2));
    icon = size > 0 ? { x: inset, y: (h - size) / 2, width: size, height: size } : null;
    const leading = Math.min(Math.max(0, w - horizontal * 2), size + small);
    text = { x: horizontal + leading, y: vertical,
      width: Math.max(0, w - horizontal * 2 - leading), height: Math.max(0, h - vertical * 2) };
  } else {
    const paintable = width >= medium + body && height >= body;
    icon = paintable ? { x: medium, y: (height - body) / 2, width: body, height: body } : null;
    const x = medium + (paintable ? body + medium : 0);
    text = { x, y: Math.max(0, height - textHeight) / 2,
      width: Math.max(0, width - x - medium), height: Math.min(height, textHeight) };
  }
  return { icon, text, pendingMeasurement };
}

export function layoutNativeSemanticText(board: Board, text: Text, node: ZuiNode,
  document: ZuiDocument, measuredTextWidth?: number): boolean {
  const lineHeight = Math.max(Number(text.fontSize), Number(text.fontSize) * Number(text.lineHeight));
  const geometry = nativeSemanticGlyphGeometry(document, node, board.width, board.height, lineHeight, measuredTextWidth);
  if (!geometry) return false;
  if (nativeSemanticGlyph(document, node)?.family === 'status-signal')
    text.hidden = !geometry.icon || geometry.text.x < 0 || geometry.text.y < 0 ||
      geometry.text.width <= 0 || geometry.text.y + geometry.text.height > board.height;
  text.resize(Math.max(0.01, geometry.text.width), Math.max(0.01, geometry.text.height));
  // Preserve editable authored alignment for no-edit Source export. Native
  // centers measured ink as a glyph+label group, then adds a right clip guard;
  // compensate Penpot's alignment inside that guard rather than rewriting it.
  const guardedInk = (nativeSemanticGlyph(document, node)?.family === 'add-component' ||
    nativeLeadingAssetButton(document, node) !== null) &&
    measuredTextWidth !== undefined ? Math.min(geometry.text.width, measuredTextWidth) : geometry.text.width;
  const alignmentOffset = Math.max(0, geometry.text.width - guardedInk) *
    (text.align === 'center' ? 0.5 : text.align === 'right' ? 1 : 0);
  text.x = board.x + geometry.text.x - alignmentOffset;
  text.y = board.y + geometry.text.y;
  text.verticalAlign = 'center';
  recordNativeSemanticLayoutGeometry(text);
  return true;
}

export function refreshNativeSemanticIcons(board: Board, node: ZuiNode | undefined,
  document: ZuiDocument, measuredTextWidth?: number): void {
  if (!node) {
    for (const icon of board.children)
      if (icon.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph')) icon.hidden = true;
    return;
  }
  const text = board.children.find((shape): shape is Text => shape.type === 'text' &&
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT);
  const height = text ? Math.max(Number(text.fontSize), Number(text.fontSize) * Number(text.lineHeight)) : 0;
  const geometry = nativeSemanticGlyphGeometry(document, node, board.width, board.height, height, measuredTextWidth);
  const glyph = nativeSemanticGlyph(document, node);
  for (const icon of board.children) {
    if (!icon.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph')) {
      const placement = icon.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'icon-placement');
      // Only our ordinary icon leaf participates. Semantic glyph ownership and
      // the independent Close brush continue through their established paths.
      if (icon.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) === ZUI_ROLE_AUXILIARY &&
          ['center', 'leading', 'trailing'].includes(placement) &&
          !(typeof document['penpot_host_theme_source'] === 'string' && isEditorTabCloseIconButton(node)))
        layoutPrefabIcon(board, icon, node, document, placement as PrefabIconPlacement, geometry);
      continue;
    }
    if (!geometry?.icon) {
      icon.hidden = true;
      continue;
    }
    if (glyph?.family === 'bottom-tab') {
      icon.hidden = true;
      if (!scaleNativeTabStrokes(icon, geometry.icon)) continue;
    }
    icon.resize(geometry.icon.width, geometry.icon.height);
    icon.x = board.x + geometry.icon.x;
    icon.y = board.y + geometry.icon.y;
    recolorIconShape(icon, previewIconTint(document, node));
    icon.hidden = false;
    recordNativeSemanticLayoutGeometry(icon);
  }
}

export function previewIconName(node: ZuiNode): string | null {
  const value =
    node.props?.['icon'] ?? node.props?.['icon_name'] ?? node.props?.['source'];
  if (typeof value !== 'string' || value.length === 0) return null;
  const name = value
    .split('/')
    .pop()!
    .replace(/\.svg$/, '')
    .replace(/-outline$/, '')
    .replaceAll('_', '-');
  if (name in icons) return name;
  const aliases: Record<string, string> = {
    close: 'x',
    settings: 'settings',
    'chevron-down': 'chevron-down',
    cube: 'box',
    'folder-open': 'folder',
    'save-outline': 'save',
    'play-arrow': 'play',
    'more-horiz': 'more-horizontal',
    add: 'plus',
    'minus-circle': 'minus-circle',
    document: 'file',
    refresh: 'refresh-cw',
    undo: 'corner-up-left',
    redo: 'corner-up-right',
    translate: 'move',
    rotate: 'rotate-cw',
    scale: 'maximize-2',
    filter: 'filter',
    AddCircle: 'plus-circle',
    'add-circle': 'plus-circle',
    alert: 'alert-circle',
    button: 'square',
    cancel: 'x-circle',
    capsule: 'sliders',
    chart: 'bar-chart-2',
    component: 'layers',
    error: 'alert-octagon',
    gear: 'settings',
    history: 'clock',
    leaf: 'feather',
    light: 'sun',
    locate: 'crosshair',
    more: 'more-horizontal',
    navigate: 'navigation',
    options: 'sliders',
    'pulse-rifle': 'crosshair',
    puzzle: 'grid',
    renderer: 'monitor',
    resize: 'maximize-2',
    route: 'git-branch',
    sparkles: 'star',
    timeline: 'activity',
    tree: 'git-branch',
    warning: 'alert-triangle',
  };
  return aliases[name] ?? null;
}

export function previewIconPlacement(
  node: ZuiNode,
  iconOnly = false,
): 'none' | 'center' | 'leading' | 'trailing' {
  if (iconOnly) return 'center';
  const authored = node.props?.['icon_placement'];
  if (authored === undefined || authored === '') return 'leading';
  if (typeof authored !== 'string')
    throw new Error(`Unsupported icon placement: ${String(authored)}`);
  switch (authored.toLowerCase()) {
    case 'start':
    case 'before':
    case 'leading':
      return 'leading';
    case 'end':
    case 'after':
    case 'trailing':
      return 'trailing';
    case 'icon_only':
    case 'icon-only':
    case 'only':
      return 'center';
    case 'none':
      return 'none';
    default:
      throw new Error(`Unsupported icon placement: ${authored}`);
  }
}

/** Editor button painters tint SVG geometry using their resolved glyph color. */
export function previewIconTint(
  document: ZuiDocument,
  node: ZuiNode,
): string | null {
  const semantic = nativeSemanticGlyph(document, node);
  if (semantic) {
    const color = editorButtonPalette(document);
    if (semantic.family === 'status-signal') {
      const state = editorButtonState(node);
      return color(state.disabled || state.loading ? 'text_disabled' : semantic.signal);
    }
    if (semantic.family === 'section-transform') return color('text_disabled');
    if (semantic.family === 'section-mesh') return color('text_secondary');
    if (semantic.family === 'add-component')
      return resolveDesignColor(document, hostButtonPreviewNode(document, node).props?.['icon_color']);
    const state = editorButtonState(node);
    if (state.disabled) return color('text_disabled');
    const tone = String(node.props?.['text_tone'] ?? '').toLowerCase();
    const validation = String(node.props?.['validation_level'] ?? '').toLowerCase();
    if (tone === 'error' || validation === 'error') return color('error');
    if (tone === 'warning' || validation === 'warning') return color('warning');
    const styled = styledPreviewNode(document, { ...node, props: { ...node.props, ...node.state } });
    return resolveDesignColor(document, styled.props?.['foreground_color']) ??
      color(state.selected || state.pressed || state.open ? 'accent' : 'text_primary');
  }
  const styled = styledPreviewNode(document, { ...node, props: { ...node.props, ...node.state } });
  if (node.component !== 'IconButton')
    return hasEditorButtonPainter(document, styled)
      ? resolveDesignColor(document, hostButtonPreviewNode(document, styled).props?.['icon_color'])
      : null;
  if (typeof document['penpot_host_theme_source'] === 'string' && isEditorTabCloseIconButton(node))
    return editorTabCloseStyle(node).glyph;
  if (typeof document['penpot_host_theme_source'] !== 'string') {
    return (
      resolveDesignColor(
        document,
        buttonStatePreviewNode(node).props?.['icon_color'],
      ) ?? null
    );
  }
  if (node.classes?.includes('unreal-transform-mode')) {
    return resolveDesignColor(document, styled.props?.['foreground_color']);
  }
  const palette = editorButtonPalette(document);
  if (!usesWorkbenchLanguage(node)) {
    // These nodes use retained node_images, not the Workbench IconButton
    // selector. Its resolved foreground wins over the active fallback.
    const state = editorButtonState(styled);
    if (state.disabled) return palette('text_disabled');
    const tone = String(styled.props?.['text_tone'] ?? '').toLowerCase();
    const validation = String(styled.props?.['validation_level'] ?? '').toLowerCase();
    if (tone === 'error' || validation === 'error') return palette('error');
    if (tone === 'warning' || validation === 'warning') return palette('warning');
    return resolveDesignColor(document, styled.props?.['foreground_color']) ??
      palette(state.selected || state.pressed || state.open ? 'accent' : 'text_primary');
  }
  const state = editorButtonState(node);
  const danger = [
    node.control_id,
    node.props?.['icon'],
    node.props?.['validation_level'],
  ]
    .map((value) => String(value ?? '').toLowerCase())
    .some((value) =>
      ['delete', 'trash', 'danger', 'error'].some((key) => value.includes(key)),
    );
  if (state.disabled || state.loading) return palette('text_disabled');
  if (danger) return palette('error');
  if (state.pressed || state.selected || state.open || state.dragging)
    return palette('accent');
  if (state.hovered) return palette('text_primary');
  return (
    resolveDesignColor(document, node.props?.['icon_color']) ??
    palette('text_primary')
  );
}

export function recolorIconShape(shape: Shape, tint: string | null): void {
  const painted = (color: string | undefined) =>
    !!color &&
    !['transparent', 'none', '#00000000'].includes(color.toLowerCase());
  const black = (color: string | undefined) =>
    !!color &&
    ['#000000', '#000', 'black', 'currentcolor'].includes(color.toLowerCase());
  if (Array.isArray(shape.fills))
    shape.fills = shape.fills.map((fill) =>
      tint
        ? painted(fill.fillColor)
          ? { ...fill, fillColor: tint }
          : fill
        : black(fill.fillColor)
          ? { ...fill, fillColor: PENPOT_PALETTE.text }
          : fill,
    );
  shape.strokes = shape.strokes.map((stroke) =>
    tint
      ? painted(stroke.strokeColor)
        ? { ...stroke, strokeColor: tint }
        : stroke
      : black(stroke.strokeColor)
        ? { ...stroke, strokeColor: PENPOT_PALETTE.text }
        : stroke,
  );
  if ('children' in shape)
    for (const child of shape.children) recolorIconShape(child, tint);
}

type PrefabIconPlacement = 'center' | 'leading' | 'trailing';

function prefabIconSize(board: Board, node: ZuiNode, document: ZuiDocument): number | null {
  if (![board.width, board.height].every((value) => Number.isFinite(value) && value > 0)) return null;
  const host = typeof document['penpot_host_theme_source'] === 'string';
  if (host && isEditorTabCloseIconButton(node)) return Math.min(16, board.width, board.height);
  const styled = styledPreviewNode(document, { ...node, props: { ...node.props, ...node.state } });
  const declared = resolveDesignNumber(document, styled.props?.['layout_icon_size']);
  const declaredSlot = isEditorWorkbenchButton(styled) || (node.component === 'IconButton' &&
    usesWorkbenchLanguage(node) && !(node.control_id ?? '').startsWith('WorkbenchStatus'));
  if (host && declaredSlot && declared !== null && Number.isFinite(declared) && declared > 0)
    return declared <= Math.min(board.width, board.height) ? declared : null;
  if (host && node.component === 'IconButton' && !usesWorkbenchLanguage(node)) {
    // The generic retained node_images path uses a bounded 16% inset, not
    // the Workbench IconButton density slot or a fixed 18 DIP cap.
    const side = Math.min(board.width, board.height);
    const inset = Math.max(0, Math.min(side * 0.16, hostMetric(document, 'editor.density.gap.small')));
    return Math.max(1, side - inset * 2);
  }
  return Math.max(8, Math.min(18, board.width - 8, board.height - 8));
}

function layoutPrefabIcon(board: Board, icon: Shape, node: ZuiNode, document: ZuiDocument,
  placement: PrefabIconPlacement, content?: NativeSemanticGlyphGeometry | null): void {
  const size = prefabIconSize(board, node, document);
  if (size === null) {
    icon.hidden = true;
    return;
  }
  const geometry = content === undefined
    ? nativeSemanticGlyphGeometry(document, node, board.width, board.height, 0) : content;
  const targetSize = geometry?.icon?.width ?? size;
  if (icon.width !== targetSize || icon.height !== targetSize) icon.resize(targetSize, targetSize);
  icon.x = board.x + (placement === 'center' ? (board.width - size) / 2 :
    placement === 'trailing' ? board.width - size - 10 : 10);
  icon.y = board.y + (board.height - size) / 2;
  const tint = previewIconTint(document, node);
  if (tint !== null) recolorIconShape(icon, tint);
  if (geometry) {
    icon.hidden = !geometry.icon;
    if (geometry.icon) {
      icon.x = board.x + geometry.icon.x;
      icon.y = board.y + geometry.icon.y;
    }
  } else icon.hidden = false;
}

export function createPrefabIcon(
  board: Board,
  node: ZuiNode,
  document: ZuiDocument,
  iconOnly = true,
): Shape | null {
  const semantic = nativeSemanticGlyph(document, node);
  if (semantic) {
    if (semantic.family === 'status-signal') {
      if (typeof penpot.createEllipse !== 'function') throw new Error('Penpot ellipse creation is unavailable');
      const icon = penpot.createEllipse();
      icon.name = 'Native semantic status signal';
      icon.fills = [{ fillColor: previewIconTint(document, node)! }];
      icon.strokes = [];
      icon.hidden = true;
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph', semantic.family);
      board.appendChild(icon);
      if (icon.layoutChild) icon.layoutChild.absolute = true;
      refreshNativeSemanticIcons(board, node, document);
      return icon;
    }
    const svg = semantic.family === 'bottom-tab' ? node['penpot_icon_svg'] ??
      object(document['penpot_native_icon_svgs'])[semantic.assetPath] :
      object(document['penpot_native_icon_svgs'])[semantic.assetPath];
    if (typeof svg !== 'string' || !svg.trim()) throw new Error('Missing actual native semantic SVG: ' + semantic.assetPath);
    const strokeSource = semantic.family === 'bottom-tab' ? nativeTabStrokeSource(svg) : null;
    if (typeof penpot.createShapeFromSvg !== 'function') throw new Error('Penpot SVG creation is unavailable');
    const icon = penpot.createShapeFromSvg(svg);
    if (!icon) throw new Error('Could not render actual native semantic SVG: ' + semantic.assetPath);
    icon.name = 'Native glyph: ' + semantic.assetPath;
    icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
    icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph', semantic.family);
    icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph-asset', semantic.assetPath);
    if (strokeSource)
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, NATIVE_TAB_STROKES, JSON.stringify(strokeSource));
    board.appendChild(icon);
    if (icon.layoutChild) icon.layoutChild.absolute = true;
    refreshNativeSemanticIcons(board, node, document);
    return icon;
  }
  const placement = previewIconPlacement(node, iconOnly);
  if (placement === 'none') return null;
  const name = previewIconName(node);
  const authoredSvg = node['penpot_icon_svg'];
  if (
    (!name && typeof authoredSvg !== 'string') ||
    typeof penpot.createShapeFromSvg !== 'function'
  )
    return null;
  const svg =
    typeof authoredSvg === 'string'
      ? authoredSvg
      : `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${PENPOT_PALETTE.textMuted}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">${icons[name as keyof typeof icons]}</svg>`;
  const icon = penpot.createShapeFromSvg(svg);
  if (!icon) throw new Error(`Could not render icon ${name}`);
  // An authored, untinted Runtime SVG retains its original colors, including
  // black paint. Placeholder icons still receive the existing fallback ink.
  if (typeof authoredSvg !== 'string' && previewIconTint(document, node) === null)
    recolorIconShape(icon, null);
  const tabClose = typeof document['penpot_host_theme_source'] === 'string' &&
    isEditorTabCloseIconButton(node);
  if (tabClose) icon.opacity *= editorTabCloseStyle(node).opacity;
  icon.name = `Icon: ${String(node.props?.['label'] ?? name)}`;
  icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'icon-placement', placement);
  icon.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  board.appendChild(icon);
  if (icon.layoutChild) icon.layoutChild.absolute = true;
  layoutPrefabIcon(board, icon, node, document, placement);
  return icon;
}
