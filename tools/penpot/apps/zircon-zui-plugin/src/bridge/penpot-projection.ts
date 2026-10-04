import { projectTreeRowText } from './zui-tree-row-visual';
import {
  ZuiDocumentError,
  validateZuiDocument,
  zuiNodes,
  zuiRootNodeIds,
  type ZuiDiagnostic,
  type ZuiChildMount,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
} from './zui-document';
import type {
  PenpotAssetSnapshot,
  ProjectedZuiNode,
  ProjectionContainer,
  ProjectionEditableState,
  ProjectionGeometry,
  ProjectionPaint,
  ProjectionText,
  ZuiAssetProjection,
} from './penpot-projection-model';
import {
  PENPOT_PALETTE,
  prefabRoleForNode,
  prefabVisualDefaults,
  resolveDesignColor,
  resolveDesignNumber,
  responsiveDirection,
  responsivePreviewValue,
  type ZuiPrefabRole,
} from './zui-prefab-system';
import { styledPreviewNode } from './zui-style-projection';
import { hostButtonPreviewNode } from './zui-editor-button-projection';
import { authoredPaintPreviewNode } from './zui-paint-properties';
import { resolveZuiTextStyle } from './zui-text-style';
import { projectSlider } from './zui-slider-projection';
import { projectSegmentedControl } from './zui-segmented-projection';
import {
  projectWorkbenchTable,
  workbenchTableStatePreviewNode,
} from './zui-table-projection';
import { assertPainterMapped } from './zui-render-capabilities';
import { nativePainterSourceParts } from './zui-native-painter-projection';
import { nativePainterComponent } from './zui-native-painter-role';
import { projectDivider } from './zui-divider-projection';
import { projectPropertyRow } from './zui-property-row-projection';
import { projectSlotPadding } from './zui-slot-padding';
import { projectEditorField } from './zui-field-projection';
import {
  inputStatePreviewNode,
  isSelectionControl,
  projectInputControl,
} from './zui-input-projection';
import { parseProjectionText } from '../penpot-rich-text';
import { containerAxisGapKeys } from './penpot-container-gaps';

const MUI_DEFAULT_SPACING_UNIT = 8;

export { reconcileZuiDocument } from './penpot-reconcile';
export type {
  PenpotAssetSnapshot,
  PenpotShapeSnapshot,
  ProjectedZuiNode,
  ProjectionContainer,
  ProjectionEditableState,
  ProjectionGeometry,
  ProjectionPadding,
  ProjectionPaint,
  ProjectionSourcePart,
  ProjectionText,
  ProjectionTextRun,
  ReconciledZuiDocument,
  ZuiAssetProjection,
} from './penpot-projection-model';

export function projectZuiDocument(document: ZuiDocument): ZuiAssetProjection {
  assertValidDocument(document);

  const diagnostics: ZuiDiagnostic[] = [];
  const nodes = zuiNodes(document);
  const rootIds = zuiRootNodeIds(document);
  const reachable = collectReachableNodes(document, rootIds);
  const parentByNode = collectParentMap(document);
  const nodeCount = Object.keys(nodes).length;
  const detachedRootIds = Object.keys(nodes).filter(
    (nodeId) => !reachable.has(nodeId) && !parentByNode.has(nodeId),
  );
  const shapes: ProjectedZuiNode[] = [];

  const projectNode = (
    nodeId: string,
    ancestorHidden = false,
    mount?: ZuiChildMount,
    parentContainer?: ProjectionContainer,
  ): ProjectedZuiNode => {
    const sourceParentId = parentByNode.get(nodeId);
    const ancestors: ZuiNode[] = [];
    let parentId = sourceParentId;
    while (parentId && ancestors.length < nodeCount) {
      ancestors.push(nodes[parentId]);
      parentId = parentByNode.get(parentId);
    }
    const node = inputStatePreviewNode(
      workbenchTableStatePreviewNode(
        document,
        hostButtonPreviewNode(
          document,
          authoredPaintPreviewNode(
            document,
            styledPreviewNode(document, nodes[nodeId], ancestors),
            nodeId,
          ),
        ),
      ),
    );
    const isRoot = rootIds.includes(nodeId);
    const isViewportRoot = isRoot && document.asset.kind === 'view';
    const prefabRole = prefabRoleForNode(node, nodeId, isViewportRoot);
    const previewHidden =
      ancestorHidden || isPreviewHidden(document, node, isRoot);
    const sourceNode = nodes[nodeId];
    const visualDetached =
      sourceParentId !== undefined &&
      isNativeDetachedWindow(nodes[sourceParentId], nodes[nodeId]);
    assertPainterMapped(node, nodeId, previewHidden);
    const projection: ProjectedZuiNode = {
      nodeId,
      component: node.component,
      name: `${nodeId} · ${node.component}`,
      ...(typeof sourceNode?.['penpot_review_source_path'] === 'string'
        ? { sourcePath: sourceNode['penpot_review_source_path'] }
        : {}),
      ...(typeof sourceNode?.['penpot_review_source_node_id'] === 'string'
        ? { sourceNodeId: sourceNode['penpot_review_source_node_id'] }
        : {}),
      controlId:
        typeof sourceNode?.control_id === 'string'
          ? sourceNode.control_id
          : null,
      ...(typeof sourceNode?.['penpot_review_instance_path'] === 'string'
        ? { instancePath: sourceNode['penpot_review_instance_path'] }
        : {}),
      sourceParts: nativePainterSourceParts(node, nodeId),
      ...(visualDetached
        ? {
            visualDetached: true,
            visualDetachedParentId: sourceParentId,
          }
        : {}),
      prefabRole,
      previewHidden,
      inputControl: projectInputControl(document, node),
      geometry: projectGeometry(document, node, isViewportRoot, prefabRole),
      paint: projectPaint(
        document,
        node,
        diagnostics,
        nodeId,
        prefabRole,
        isRoot && document.asset.kind === 'component',
      ),
      text: projectTreeRowText(
        document,
        node,
        projectText(
          document,
          node,
          diagnostics,
          nodeId,
          prefabRole,
          previewHidden,
        ),
      ),
      container: projectContainer(
        document,
        node,
        diagnostics,
        nodeId,
        prefabRole,
      ),
      children: [],
    };
    const slotPadding = projectSlotPadding(
      document,
      nodeId,
      mount,
      parentContainer?.kind,
      sourceNode,
    );
    if (slotPadding) projection.slotPadding = slotPadding;
    if (projection.inputControl?.kind === 'dropdown') {
      projection.paint.fillColor = projection.inputControl.fill;
      projection.paint.strokeColor = projection.inputControl.border;
    }
    const slider = projectSlider(document, node);
    const segmented = projectSegmentedControl(document, node);
    const table = projectWorkbenchTable(document, node);
    const divider = projectDivider(document, node);
    const propertyRow = projectPropertyRow(document, node);
    const field = projectEditorField(document, node);
    if (field) Object.assign(projection, field);
    if (divider) {
      projection.divider = divider;
      projection.text = null;
      projection.paint.fillColor = null;
      projection.paint.strokeColor = null;
      projection.paint.strokeWidth = 0;
    }
    if (table) Object.assign(projection, table);
    if (propertyRow) Object.assign(projection, propertyRow);
    if (segmented) Object.assign(projection, segmented);
    if (slider) {
      Object.assign(projection, slider);
    }
    if (slider || segmented) {
      // Native controls retain authored owner surfaces, without a prefab surface.
      if (node.props?.['background_color'] === undefined)
        projection.paint.fillColor = null;
      if (node.props?.['border_color'] === undefined) {
        projection.paint.strokeColor = null;
        projection.paint.strokeWidth = 0;
      }
      projection.paint.borderRadius =
        resolveDesignNumber(document, node.props?.['corner_radius']) ?? 0;
    }
    applyRichTextPresentation(node, projection);
    shapes.push(projection);
    projection.children = (node.children ?? []).map((child) =>
      projectNode(child.node, previewHidden, child, projection.container),
    );
    return projection;
  };

  const rootNodes = rootIds.map((nodeId) => projectNode(nodeId));
  const detachedNodes = detachedRootIds.map((nodeId) => projectNode(nodeId));
  const projectedIds = new Set(shapes.map(({ nodeId }) => nodeId));
  for (const nodeId of Object.keys(nodes)) {
    if (!projectedIds.has(nodeId)) {
      detachedNodes.push(projectNode(nodeId));
    }
  }

  const preservedCount = countMetadataOnlyFields(document);
  if (preservedCount > 0) {
    diagnostics.push({
      severity: 'info',
      code: 'metadata-preserved',
      message: `${preservedCount} runtime-only or nonvisual fields will be preserved as ZUI metadata.`,
    });
  }

  return {
    assetId: document.asset.id,
    displayName: document.asset.display_name?.trim() || document.asset.id,
    rootNodes,
    detachedNodes,
    shapes,
    diagnostics,
  };
}

/** Re-apply inline presentation after specialized projections replace text. */
function applyRichTextPresentation(
  node: ZuiNode,
  projection: ProjectedZuiNode,
): void {
  const format = node.props?.['rich_text_format'];
  if (format === undefined) return;
  const apply = (text: ProjectionText | null): ProjectionText | null => {
    if (!text || !text.property) return text;
    const raw = node.props?.[text.property];
    if (typeof raw !== 'string') return text;
    const parsed = parseProjectionText(raw, format);
    if (!parsed.runs.length) return text;
    return {
      ...text,
      characters: parsed.characters,
      richTextRuns: parsed.runs,
    };
  };
  projection.text = apply(projection.text);
  if (projection.textFragments) {
    projection.textFragments = Object.fromEntries(
      Object.entries(projection.textFragments).map(([key, text]) => [
        key,
        apply(text)!,
      ]),
    );
  }
}

function isNativeDetachedWindow(
  parent: ZuiNode | undefined,
  node: ZuiNode | undefined,
): boolean {
  return (
    parent?.component === 'DockHost' && node?.component === 'FloatingWindow'
  );
}

export function cloneProjectionSnapshot(
  projection: ZuiAssetProjection,
): PenpotAssetSnapshot {
  const parentByNode = new Map<string, string>();
  for (const shape of projection.shapes) {
    for (const child of shape.children) {
      parentByNode.set(child.nodeId, shape.nodeId);
    }
  }
  return {
    assetId: projection.assetId,
    rootNodeIds: projection.rootNodes.map(({ nodeId }) => nodeId),
    detachedNodeIds: projection.detachedNodes.map(({ nodeId }) => nodeId),
    shapes: projection.shapes.map((shape) => {
      const editable = editableState(shape);
      return {
        nodeId: shape.nodeId,
        component: shape.component,
        ...(shape.sourcePath ? { sourcePath: shape.sourcePath } : {}),
        ...(shape.sourceNodeId ? { sourceNodeId: shape.sourceNodeId } : {}),
        controlId: shape.controlId ?? null,
        ...(shape.instancePath ? { instancePath: shape.instancePath } : {}),
        parentNodeId: parentByNode.get(shape.nodeId) ?? null,
        childNodeIds: shape.children.map(({ nodeId }) => nodeId),
        ...(shape.sourceParts?.length
          ? { sourceParts: shape.sourceParts.map((part) => ({ ...part })) }
          : {}),
        baseline: cloneEditableState(editable),
        current: cloneEditableState(editable),
      };
    }),
  };
}

function projectGeometry(
  document: ZuiDocument,
  node: ZuiNode,
  isRoot: boolean,
  prefabRole: ZuiPrefabRole,
): ProjectionGeometry {
  const layout = node.layout;
  const defaults = defaultSize(node, isRoot, prefabRole);
  return {
    x: designNumberAt(document, layout, 'position', 'x') ?? 0,
    y: designNumberAt(document, layout, 'position', 'y') ?? 0,
    width:
      designNumberAt(document, layout, 'width', 'preferred') ??
      designNumberAt(document, layout, 'width', 'min') ??
      defaults.width,
    height:
      designNumberAt(document, layout, 'height', 'preferred') ??
      designNumberAt(document, layout, 'height', 'min') ??
      defaults.height,
  };
}

function projectPaint(
  document: ZuiDocument,
  node: ZuiNode,
  diagnostics: ZuiDiagnostic[],
  nodeId: string,
  prefabRole: ZuiPrefabRole,
  componentRoot: boolean,
): ProjectionPaint {
  const props = node.props;
  const defaults = prefabVisualDefaults(prefabRole, node, document);
  // NotificationCenter exposes panel paint under explicit painter properties;
  // project those onto the owner board so a component asset is never a blank
  // canvas while keeping the authored property in the source metadata.
  const backgroundProperty =
    props?.['background_color'] !== undefined
      ? 'background_color'
      : node.component === 'NotificationCenter'
        ? 'panel_surface_color'
        : undefined;
  const borderProperty =
    props?.['border_color'] !== undefined
      ? 'border_color'
      : node.component === 'NotificationCenter'
        ? 'panel_border_color'
        : undefined;
  const fill = projectedColor(
    document,
    backgroundProperty ? props?.[backgroundProperty] : undefined,
    diagnostics,
    nodeId,
    backgroundProperty ?? 'background_color',
  );
  const stroke = projectedColor(
    document,
    borderProperty ? props?.[borderProperty] : undefined,
    diagnostics,
    nodeId,
    borderProperty ?? 'border_color',
  );
  return {
    fillColor:
      fill.color ??
      defaults.fillColor ??
      (componentRoot ? PENPOT_PALETTE.canvas : null),
    fillOpacity: fill.opacity,
    strokeColor: stroke.color ?? defaults.strokeColor,
    strokeOpacity: stroke.opacity,
    strokeWidth:
      resolveDesignNumber(document, props?.['border_width']) ??
      (stroke.color ? 1 : defaults.strokeWidth),
    borderRadius:
      resolveDesignNumber(document, props?.['corner_radius']) ??
      defaults.borderRadius,
    opacity: clamp(finiteNumber(props?.['opacity']) ?? 1, 0, 1),
  };
}

function projectText(
  document: ZuiDocument,
  node: ZuiNode,
  diagnostics: ZuiDiagnostic[],
  nodeId: string,
  prefabRole: ZuiPrefabRole,
  previewHidden: boolean,
): ProjectionText | null {
  // Retained host treats these labels as semantic metadata, not painted text.
  if (['Icon', 'IconButton', 'Image', 'SvgIcon'].includes(node.component))
    return null;
  const props = node.props;
  if (prefabRole === 'icon-button' || props?.['icon_placement'] === 'icon_only')
    return null;
  const property =
    isSelectionControl(node) &&
    typeof props?.['label'] === 'string' &&
    props['label'].trim()
      ? 'label'
      : textProperty(props);
  if (!property) return null;
  // Native resolve_control_fallback_text does not paint a layout group's
  // state value. Only authored text/labels or an explicit widget value bind.
  if (
    [
      'HorizontalGroup',
      'VerticalGroup',
      'Container',
      'HorizontalBox',
      'VerticalBox',
      'WrapBox',
      'Overlay',
      'ScrollableBox',
    ].includes(node.component) &&
    property !== 'text' &&
    property !== 'label' &&
    (!isTable(node['widget']) || node['widget']['value_property'] === undefined)
  )
    return null;
  // Tabs.value is the selection key. Runtime resolve_text only displays its text.
  if (node.component === 'Tabs' && property !== 'text') return null;
  const semanticText = property ? props?.[property] : undefined;
  const rawCharacters = typeof semanticText === 'string' ? semanticText : '';
  const parsedText = parseProjectionText(
    rawCharacters,
    props?.['rich_text_format'],
  );
  const characters = parsedText.characters;
  if (previewHidden && characters === '') return null;
  const defaults = prefabVisualDefaults(prefabRole, node, document);
  if (isSelectionControl(node) || node.component === 'Dropdown') {
    defaults.fontWeight = '400';
    defaults.textAlign = 'left';
  }
  const align = props?.['text_align'];
  const textStyle = resolveZuiTextStyle(document, node, {
    size: defaults.fontSize,
    weight: defaults.fontWeight === 'regular' ? '400' : defaults.fontWeight,
  });
  const color = projectedColor(
    document,
    props?.['foreground_color'],
    diagnostics,
    nodeId,
    'foreground_color',
  );
  return {
    characters,
    property,
    color: color.color ?? defaults.textColor,
    colorOpacity: color.opacity,
    fontFamily: textStyle.family,
    lineHeight: textStyle.lineHeight,
    fontSize:
      resolveDesignNumber(document, props?.['font_size']) ?? defaults.fontSize,
    fontWeight:
      projectedFontWeight(
        document,
        props?.['font_weight'],
        diagnostics,
        nodeId,
      ) ?? defaults.fontWeight,
    align:
      align === 'left' ||
      align === 'center' ||
      align === 'right' ||
      align === 'justify'
        ? align
        : defaults.textAlign,
    ...(parsedText.runs.length ? { richTextRuns: parsedText.runs } : {}),
  };
}

function projectContainer(
  document: ZuiDocument,
  node: ZuiNode,
  diagnostics: ZuiDiagnostic[],
  nodeId: string,
  _prefabRole: ZuiPrefabRole,
): ProjectionContainer {
  const layout = node.layout;
  const sourceKind = valueAt(layout, 'container', 'kind') ?? node.component;
  const normalizedKind =
    typeof sourceKind === 'string' ? sourceKind.toLowerCase() : '';
  let kind: ProjectionContainer['kind'] = 'free';
  let direction: ProjectionContainer['direction'] = 'column';
  let wrap = false;
  if (
    normalizedKind === 'horizontalbox' ||
    normalizedKind === 'horizontalgroup'
  ) {
    kind = 'flex';
    direction = 'row';
  } else if (
    normalizedKind === 'verticalbox' ||
    normalizedKind === 'verticalgroup'
  ) {
    kind = 'flex';
  } else if (
    normalizedKind === 'scrollablebox' ||
    normalizedKind === 'scrollbox'
  ) {
    kind = 'flex';
    // ScrollableBox preserves its authored axis in the retained host.  The
    // Penpot projection must use the same main axis so horizontal state
    // samples can overflow into an explicit scroll context instead of being
    // reported as an accidental flex overflow.
    const axis = String(
      valueAt(layout, 'container', 'axis') ?? '',
    ).toLowerCase();
    direction = axis === 'horizontal' ? 'row' : 'column';
  } else if (
    normalizedKind === 'flowbox' ||
    normalizedKind === 'flexbox' ||
    normalizedKind === 'wrapbox'
  ) {
    kind = 'flex';
    direction = 'row';
    wrap = true;
  } else if (
    normalizedKind === 'gridbox' ||
    normalizedKind === 'gridgroup' ||
    normalizedKind === 'masonry' ||
    normalizedKind === 'masonrybox'
  ) {
    kind = 'grid';
  } else if (normalizedKind === 'stack') {
    kind = 'flex';
    direction = responsiveDirection(node.props?.['direction']) ?? 'row';
    // MUI Stack is a linear layout and does not wrap by default.  Treating
    // every Stack as a wrapping flex container makes a responsive column
    // (for example an auth flow at 640px) wrap its fixed-height cards back
    // into a row.  Wrapping remains available as an explicit source
    // contract, while FlowBox/WrapBox keep their intrinsic wrap semantics.
    const authoredWrap =
      valueAt(layout, 'container', 'wrap') ?? node.props?.['wrap'];
    wrap = authoredWrap === true || authoredWrap === 'wrap';
  } else if (
    normalizedKind === 'container' ||
    normalizedKind === 'panel' ||
    normalizedKind === 'dialog' ||
    normalizedKind === 'drawer' ||
    normalizedKind === 'popup' ||
    normalizedKind === 'menu'
  ) {
    kind = childrenUseAbsolutePosition(document, node) ? 'free' : 'flex';
  } else if (
    sourceKind !== undefined &&
    normalizedKind !== 'overlay' &&
    normalizedKind !== 'free' &&
    (node.children?.length ?? 0) > 0
  ) {
    kind = childrenUseAbsolutePosition(document, node) ? 'free' : 'flex';
    diagnostics.push({
      severity: 'info',
      code: 'container-kind-inferred',
      message: `Node ${nodeId} container kind ${String(sourceKind)} is preserved as metadata and projected with a ${kind} Penpot layout.`,
      path: `nodes.${nodeId}.layout.container.kind`,
    });
  }

  const gapValue =
    valueAt(layout, 'container', 'gap') ??
    (normalizedKind.includes('masonry')
      ? valueAt(layout, 'container', 'spacing')
      : undefined);
  const gap = resolveDesignNumber(document, gapValue);
  const stackSpacing =
    normalizedKind === 'stack'
      ? resolveMuiStackSpacing(document, node.props?.['spacing'])
      : null;
  if (gapValue !== undefined && gap === null) {
    diagnostics.push({
      severity: 'info',
      code: 'token-value-preserved',
      message: `Node ${nodeId} uses a nonnumeric layout gap; Penpot shows zero and preserves the source value.`,
      path: `nodes.${nodeId}.layout.container.gap`,
    });
  }
  const defaultPadding = 0;
  // MUI Stack spacing is authored in theme units when it lives in props.
  // Prefer an explicit retained layout gap, then mirror the v2 adapter's
  // numeric spacing conversion before falling back to the Penpot preview gap.
  const nativeContainer = [
    'horizontalbox',
    'horizontalgroup',
    'verticalbox',
    'verticalgroup',
    'scrollablebox',
    'flowbox',
    'flexbox',
    'wrapbox',
    'gridbox',
    'gridgroup',
    'masonry',
    'masonrybox',
  ].includes(normalizedKind);
  const resolvedGap =
    gap ?? stackSpacing ?? (kind === 'free' || nativeContainer ? 0 : 8);
  const gapKeys = containerAxisGapKeys({ kind, wrap }, sourceKind);
  const axisFallback = normalizedKind === 'wrapbox' ? 0 : resolvedGap;
  const rowGap =
    (gapKeys
      ? designNumberAt(document, layout, 'container', gapKeys.row)
      : null) ?? axisFallback;
  const columnGap =
    (gapKeys
      ? designNumberAt(document, layout, 'container', gapKeys.column)
      : null) ?? axisFallback;
  const columns = Math.max(
    1,
    Math.round(
      designNumberAt(document, layout, 'container', 'columns') ??
        (normalizedKind.includes('masonry') ? 3 : 1),
    ),
  );
  const rows = Math.max(
    1,
    Math.round(
      designNumberAt(document, layout, 'container', 'rows') ??
        Math.ceil((node.children?.length ?? 1) / columns),
    ),
  );
  return {
    kind,
    direction,
    wrap,
    gap: resolvedGap,
    rowGap,
    columnGap,
    columns,
    rows,
    padding: {
      top: designNumberAt(document, layout, 'padding', 'top') ?? defaultPadding,
      right:
        designNumberAt(document, layout, 'padding', 'right') ?? defaultPadding,
      bottom:
        designNumberAt(document, layout, 'padding', 'bottom') ?? defaultPadding,
      left:
        designNumberAt(document, layout, 'padding', 'left') ?? defaultPadding,
    },
    alignItems: alignment(
      valueAt(layout, 'container', 'align_items'),
      kind,
      direction,
    ),
    justifyContent: justification(
      valueAt(layout, 'container', 'justify_content'),
    ),
    clip:
      normalizedKind.includes('scroll') ||
      valueAt(layout, 'clip') === true ||
      node.props?.['clip_content'] === true,
  };
}

function resolveMuiStackSpacing(
  document: ZuiDocument,
  value: unknown,
): number | null {
  const candidate = responsivePreviewValue(value);
  if (typeof candidate === 'number' && Number.isFinite(candidate)) {
    return Math.max(0, candidate * MUI_DEFAULT_SPACING_UNIT);
  }
  if (typeof candidate === 'string') {
    const normalized = candidate.trim();
    const parsed = Number(normalized);
    if (Number.isFinite(parsed)) return Math.max(0, parsed);
    // Named design tokens already represent physical preview units. Keep
    // those values unscaled, matching the bridge's existing token resolver.
    if (normalized.startsWith('$')) {
      const resolved = resolveDesignNumber(document, normalized);
      return resolved === null ? null : Math.max(0, resolved);
    }
  }
  return null;
}

function editableState(node: ProjectedZuiNode): ProjectionEditableState {
  return {
    geometry: node.geometry,
    paint: node.paint,
    text: editableText(node.text),
    ...(node.textFragments
      ? {
          textFragments: Object.fromEntries(
            Object.entries(node.textFragments).map(([key, text]) => [
              key,
              editableText(text),
            ]),
          ),
        }
      : {}),
    ...(node.slotPadding ? { slotPadding: node.slotPadding } : {}),
    container: node.container,
  };
}

function cloneEditableState(
  state: ProjectionEditableState,
): ProjectionEditableState {
  return {
    geometry: { ...state.geometry },
    paint: { ...state.paint },
    ...(state.slotPadding ? { slotPadding: { ...state.slotPadding } } : {}),
    text: editableText(state.text),
    ...(state.textFragments
      ? {
          textFragments: Object.fromEntries(
            Object.entries(state.textFragments).map(([key, text]) => [
              key,
              editableText(text),
            ]),
          ),
        }
      : {}),
    container: {
      ...state.container,
      padding: { ...state.container.padding },
    },
  };
}

/** Rich ranges are a rendering instruction, not a second editable text field. */
function editableText(text: ProjectionText): ProjectionText;
function editableText(text: ProjectionText | null): ProjectionText | null;
function editableText(text: ProjectionText | null): ProjectionText | null {
  if (!text) return null;
  const editable = { ...text };
  delete editable.richTextRuns;
  return editable;
}

function collectReachableNodes(
  document: ZuiDocument,
  roots: string[],
): Set<string> {
  const nodes = zuiNodes(document);
  const reachable = new Set<string>();
  const visit = (nodeId: string): void => {
    if (reachable.has(nodeId)) {
      return;
    }
    reachable.add(nodeId);
    for (const child of nodes[nodeId]?.children ?? []) {
      visit(child.node);
    }
  };
  roots.forEach(visit);
  return reachable;
}

function collectParentMap(document: ZuiDocument): Map<string, string> {
  const nodes = zuiNodes(document);
  const parents = new Map<string, string>();
  for (const [parentId, node] of Object.entries(nodes)) {
    for (const child of node.children ?? []) {
      parents.set(child.node, parentId);
    }
  }
  return parents;
}

function countMetadataOnlyFields(document: ZuiDocument): number {
  let count = 0;
  if (document.imports && Object.keys(document.imports).length > 0) count += 1;
  if (document.tokens && Object.keys(document.tokens).length > 0) count += 1;
  if (document.components && Object.keys(document.components).length > 0)
    count += 1;
  if (document.stylesheets && document.stylesheets.length > 0) count += 1;
  const metadataNodeKeys = [
    'params',
    'state',
    'repeat',
    'style',
    'slots',
    'events',
  ];
  for (const node of Object.values(zuiNodes(document))) {
    count += metadataNodeKeys.filter((key) => node[key] !== undefined).length;
    count += Object.keys(node).filter(
      (key) =>
        ![
          'component',
          'control_id',
          'classes',
          'props',
          'layout',
          'children',
          ...metadataNodeKeys,
        ].includes(key),
    ).length;
  }
  return count;
}

function defaultSize(
  node: ZuiNode,
  isRoot: boolean,
  prefabRole: ZuiPrefabRole,
): { width: number; height: number } {
  if (isRoot) return { width: 960, height: 640 };
  switch (prefabRole) {
    case 'title':
      return { width: 220, height: 28 };
    case 'caption':
      return { width: 180, height: 20 };
    case 'label':
      return { width: 180, height: 24 };
    case 'button':
      return { width: 120, height: 32 };
    case 'icon-button':
      return { width: 28, height: 28 };
    case 'field':
      return { width: 220, height: 32 };
    case 'toggle':
      return { width: 96, height: 30 };
    case 'tab':
      return { width: 88, height: 30 };
    case 'row':
      return { width: 240, height: 28 };
    case 'chip':
      return { width: 80, height: 24 };
    case 'badge':
      return { width: 56, height: 20 };
    case 'divider':
      return { width: 160, height: 1 };
    case 'progress':
      return { width: 160, height: 12 };
    case 'canvas':
      return { width: 320, height: 180 };
    case 'panel':
    case 'layout':
      return (node.children?.length ?? 0) > 0
        ? { width: 320, height: 240 }
        : { width: 160, height: 40 };
    case 'space':
      return { width: 24, height: 24 };
    case 'root':
      return { width: 960, height: 640 };
  }
}

interface ProjectedColor {
  color: string | null;
  opacity: number;
}

function projectedColor(
  document: ZuiDocument,
  value: unknown,
  diagnostics: ZuiDiagnostic[],
  nodeId: string,
  property: string,
): ProjectedColor {
  if (typeof value !== 'string' || value.trim() === '') {
    return { color: null, opacity: 1 };
  }
  const normalized = resolveDesignColor(document, value) ?? value.trim();
  if (normalized.toLowerCase() === 'transparent') {
    return { color: '#000000', opacity: 0 };
  }
  const hex = normalized.match(/^#([0-9a-fA-F]{3,4}|[0-9a-fA-F]{6,8})$/);
  if (hex) {
    const digits =
      hex[1].length <= 4
        ? [...hex[1]].map((digit) => `${digit}${digit}`).join('')
        : hex[1];
    const color = `#${digits.slice(0, 6)}`;
    const opacity =
      digits.length === 8 ? Number.parseInt(digits.slice(6), 16) / 255 : 1;
    return { color, opacity };
  }
  const isUnsupportedLiteral = /^(?:rgb|hsl)a?\(/i.test(normalized);
  diagnostics.push({
    severity: isUnsupportedLiteral ? 'warning' : 'info',
    code: isUnsupportedLiteral
      ? 'paint-syntax-preserved'
      : 'token-value-preserved',
    message: isUnsupportedLiteral
      ? `Node ${nodeId} ${property} uses a color syntax that Penpot cannot edit safely; it remains metadata-backed.`
      : `Node ${nodeId} ${property} is not a literal color; it remains metadata-backed.`,
    path: `nodes.${nodeId}.props.${property}`,
  });
  return { color: null, opacity: 1 };
}

function textProperty(props: ZuiTable | undefined): ProjectionText['property'] {
  if (!props) return null;
  for (const property of ['text', 'value_text', 'value'] as const) {
    if (typeof props[property] === 'string' && props[property].length > 0)
      return property;
  }
  if (
    typeof props['placeholder'] === 'string' &&
    props['placeholder'].length > 0 &&
    !props['text'] &&
    !props['value_text'] &&
    !props['value']
  )
    return 'placeholder';
  for (const property of [
    'text',
    'value_text',
    'value',
    'placeholder',
    'title',
    'message',
    'label',
  ] as const) {
    if (typeof props[property] === 'string') {
      return property;
    }
  }
  return null;
}

function valueAt(table: ZuiTable | undefined, ...path: string[]): unknown {
  let value: unknown = table;
  for (const key of path) {
    if (!isTable(value)) return undefined;
    value = value[key];
  }
  return value;
}

function designNumberAt(
  document: ZuiDocument,
  table: ZuiTable | undefined,
  ...path: string[]
): number | null {
  return resolveDesignNumber(document, valueAt(table, ...path));
}

function childrenUseAbsolutePosition(
  document: ZuiDocument,
  node: ZuiNode,
): boolean {
  const childIds = (node.children ?? []).map(({ node: childId }) => childId);
  if (childIds.length === 0) return false;
  const nodes = zuiNodes(document);
  return childIds.every((childId) => {
    const position = nodes[childId]?.layout?.['position'];
    return (
      isTable(position) &&
      (position['x'] !== undefined || position['y'] !== undefined)
    );
  });
}

function isPreviewHidden(
  document: ZuiDocument,
  node: ZuiNode,
  isRoot: boolean,
): boolean {
  // Native painter hosts are retained in the product tree so their runtime
  // state can be opened without rebuilding the shell.  A design projection
  // must still honour that state, even when the painter is the review root.
  // Transient host state takes precedence over the authored default.
  const painter = nativePainterComponent(node);
  const hasOpenContract =
    Object.hasOwn(node.props ?? {}, 'open') ||
    Object.hasOwn(node.props ?? {}, 'popup_open') ||
    Object.hasOwn(node.state ?? {}, 'open') ||
    Object.hasOwn(node.state ?? {}, 'popup_open');
  const open =
    node.state?.['open'] ??
    node.state?.['popup_open'] ??
    node.props?.['open'] ??
    node.props?.['popup_open'];
  if (painter && hasOpenContract && open !== true) return true;
  const component = node.component.toLowerCase();
  if (
    ['modal', 'dialog', 'confirmdialog', 'popover', 'popup'].includes(
      component,
    ) &&
    hasOpenContract &&
    open === false
  )
    return true;
  if (
    [
      'dropdownpopup',
      'contextmenu',
      'contextactionmenu',
      'modal',
      'popover',
      'popup',
    ].includes(component) &&
    hasOpenContract &&
    open !== true
  )
    return true;
  if (isRoot) return false;
  if (
    node.children?.length &&
    node.children.every((child) => {
      const props = document.nodes?.[child.node]?.props;
      return props?.['aria_live'] !== undefined && !props?.['text'];
    })
  )
    return true;
  const responsiveVisibility = responsivePreviewValue(
    node.props?.['visibility'],
  );
  const visibility =
    typeof responsiveVisibility === 'string'
      ? responsiveVisibility.toLowerCase()
      : '';
  if (
    visibility === 'collapsed' ||
    visibility === 'hidden' ||
    visibility === 'none' ||
    node.props?.['visible'] === false
  ) {
    return true;
  }
  const width =
    designNumberAt(document, node.layout, 'width', 'preferred') ??
    designNumberAt(document, node.layout, 'width', 'max');
  const height =
    designNumberAt(document, node.layout, 'height', 'preferred') ??
    designNumberAt(document, node.layout, 'height', 'max');
  const clipsChildren =
    node.layout?.['clip'] === true ||
    ['scrollablebox', 'scrollbox'].includes(
      String(valueAt(node.layout, 'container', 'kind') ?? '').toLowerCase(),
    );
  // A clipped zero-sized host has no visible projection surface. Keep its
  // authored node and descendants in the semantic snapshot, but omit their
  // canvas shapes from the visual preview until the runtime opens the host.
  if (
    clipsChildren &&
    node.children?.length &&
    width !== null &&
    height !== null &&
    width <= 0 &&
    height <= 0
  )
    return true;
  if (
    width !== null &&
    height !== null &&
    width <= 4 &&
    height <= 4 &&
    (designNumberAt(document, node.layout, 'position', 'x') ?? 0) < 0 &&
    node.children?.every(
      (child) => document.nodes?.[child.node]?.props?.['text'] === '',
    )
  )
    return true;
  const text = textProperty(node.props);
  return (
    width !== null &&
    height !== null &&
    width <= 2 &&
    height <= 2 &&
    text !== null &&
    node.props?.[text] === ''
  );
}

function finiteNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function projectedFontWeight(
  document: ZuiDocument,
  value: unknown,
  diagnostics: ZuiDiagnostic[],
  nodeId: string,
): string | null {
  const aliases: Record<string, string> = {
    thin: '100',
    extralight: '200',
    'extra-light': '200',
    light: '300',
    normal: 'regular',
    regular: 'regular',
    medium: '500',
    semibold: '600',
    'semi-bold': '600',
    bold: '700',
    extrabold: '800',
    'extra-bold': '800',
    black: '900',
  };
  const normalized =
    typeof value === 'string' ? value.trim().toLowerCase() : '';
  const resolved = resolveDesignNumber(document, value);
  const numeric = resolved ?? Number.NaN;
  if (
    Number.isInteger(numeric) &&
    numeric >= 100 &&
    numeric <= 900 &&
    numeric % 100 === 0
  ) {
    return numeric === 400 ? 'regular' : String(numeric);
  }
  if (aliases[normalized]) {
    return aliases[normalized];
  }
  if (value !== undefined) {
    diagnostics.push({
      severity: 'info',
      code: 'token-value-preserved',
      message: `Node ${nodeId} font_weight cannot map to a Work Sans variant; it remains metadata-backed.`,
      path: `nodes.${nodeId}.props.font_weight`,
    });
  }
  return null;
}

function alignment(
  value: unknown,
  kind: ProjectionContainer['kind'],
  direction: ProjectionContainer['direction'],
): ProjectionContainer['alignItems'] {
  const normalized = typeof value === 'string' ? value.toLowerCase() : '';
  if (
    normalized === 'start' ||
    normalized === 'center' ||
    normalized === 'end' ||
    normalized === 'stretch'
  ) {
    return normalized;
  }
  if (kind === 'grid' || (kind === 'flex' && direction === 'column')) {
    return 'stretch';
  }
  return kind === 'flex' ? 'center' : 'start';
}

function justification(value: unknown): ProjectionContainer['justifyContent'] {
  const normalized =
    typeof value === 'string' ? value.toLowerCase().replaceAll('_', '-') : '';
  if (
    normalized === 'center' ||
    normalized === 'end' ||
    normalized === 'space-between' ||
    normalized === 'space-around' ||
    normalized === 'space-evenly'
  ) {
    return normalized;
  }
  return 'start';
}

function assertValidDocument(document: ZuiDocument): void {
  const diagnostics = validateZuiDocument(document);
  const errors = diagnostics.filter(({ severity }) => severity === 'error');
  if (errors.length > 0) {
    throw new ZuiDocumentError(
      `Invalid .zui document: ${errors.map(({ message }) => message).join('; ')}`,
      diagnostics,
    );
  }
}

function isTable(value: unknown): value is ZuiTable {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
