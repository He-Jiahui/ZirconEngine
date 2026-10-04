import type { LayoutReviewCaseMessage } from '../model';
import {
  applyComponentReviewData,
  nativePainterReviewStates,
  validateComponentReviewData,
} from './zui-component-review-data';
import {
  cloneZuiDocument,
  zuiRootNodeIds,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
} from './zui-document';
import { nativePainterComponent } from './zui-native-painter-role';
import {
  applyCollectionReviewData,
  isCollectionReviewData,
} from './zui-collection-review';

const REVIEW_STATES: Record<string, string[]> = {
  default: [],
  hover: ['hovered'],
  pressed: ['pressed'],
  focused: ['focused', 'focus_visible'],
  disabled: ['disabled'],
  selected: ['selected'],
  open: [],
  closed: [],
  'focus-return': [],
  dragging: [],
  'drop-allowed': [],
  'drop-blocked': [],
  empty: [],
  running: [],
  complete: [],
  blocked: [],
  success: [],
  failure: [],
  pending: [],
  approved: [],
  denied: [],
  normal: [],
  warning: [],
  exceeded: [],
  error: [],
  'long-en': [],
  'long-zh': [],
  'scroll-before': [],
  'scroll-after': [],
};
const STATE_KEYS = [
  'hover',
  'hovered',
  'pressed',
  'enter_pressed',
  'button_interaction_state',
  'active',
  'focused',
  'focus',
  'focus_visible',
  'focusVisible',
  'focus-visible',
  'selected',
  'disabled',
  'enabled',
  'checked',
  'loading',
  'open',
  'popup_open',
  'dragging',
  'drop_hovered',
  'active_drag_target',
  'drop_allowed',
];
const NATIVE_ONLY_REVIEW_STATES = new Set([
  'open',
  'closed',
  'focus-return',
  'dragging',
  'drop-allowed',
  'drop-blocked',
  'empty',
  'running',
  'complete',
  'blocked',
  'success',
  'failure',
  'pending',
  'approved',
  'denied',
  'normal',
  'warning',
  'exceeded',
  'error',
  'long-en',
  'long-zh',
]);
const REVIEW_CASE_FIELDS = new Set([
  'id',
  'sourcePath',
  'host',
  'viewport',
  'dpi',
  'locale',
  'state',
  'scrollPosition',
  'data',
  'themeSourcePath',
  'reviewHost',
]);
const REVIEW_HOSTS = new Set([
  'editor',
  'plugin',
  'woc',
  'component',
  'toolbar',
  'theme',
  'fixture',
]);

// Keep the review host on the same logical-width breakpoints as the retained
// Workbench host. DPI changes the capture size, while the tier is selected
// from the logical viewport width.
const REVIEW_BREAKPOINTS = {
  ultraMaxWidth: 480,
  narrowMaxWidth: 640,
  wideMinWidth: 1260,
} as const;

// Keep the Penpot review projection on the same breakpoint contract as the
// retained MUI/native host.  The source document remains responsive; review
// cases materialize only the value for their logical viewport on a cloned
// document so the board geometry is measured at the requested breakpoint.
const MUI_REVIEW_BREAKPOINTS = [
  { key: 'xs', minWidth: 0 },
  { key: 'sm', minWidth: 600 },
  { key: 'md', minWidth: 900 },
  { key: 'lg', minWidth: 1200 },
  { key: 'xl', minWidth: 1536 },
] as const;

const RESPONSIVE_LAYOUT_KEYS = new Set([
  'columnSpacing',
  'column_spacing',
  'columns',
  'direction',
  'display',
  'gap',
  'item_min_width',
  'rowSpacing',
  'row_spacing',
  'rows',
  'spacing',
  'visibility',
]);

const WORKBENCH_RIGHT_DRAWER_CONTROL_ID = 'RightDrawerShellRoot';
const RIGHT_DRAWER_DEFAULT_WIDTH_TOKEN = '$editor.density.right_drawer_width';
const RIGHT_DRAWER_COMPACT_MAX_WIDTH_TOKEN =
  '$editor.density.compact_right_drawer_max_width';

type ReviewLayoutTier = 'ultra' | 'narrow' | 'regular' | 'wide';

function reviewLayoutTier(width: number): ReviewLayoutTier {
  if (width <= REVIEW_BREAKPOINTS.ultraMaxWidth) return 'ultra';
  if (width <= REVIEW_BREAKPOINTS.narrowMaxWidth) return 'narrow';
  if (width >= REVIEW_BREAKPOINTS.wideMinWidth) return 'wide';
  return 'regular';
}

function parseReviewLayoutTier(value: unknown): ReviewLayoutTier | null {
  if (typeof value !== 'string') return null;
  const normalized = value.trim().toLowerCase();
  return normalized === 'ultra' ||
    normalized === 'narrow' ||
    normalized === 'regular' ||
    normalized === 'wide'
    ? normalized
    : null;
}

function reviewLayoutTierRank(tier: ReviewLayoutTier): number {
  return tier === 'ultra'
    ? 0
    : tier === 'narrow'
      ? 1
      : tier === 'regular'
        ? 2
        : 3;
}

function responsiveNodeVisible(node: ZuiNode, tier: ReviewLayoutTier): boolean {
  const props = node.props;
  if (!props) return true;
  const minimum = parseReviewLayoutTier(props['responsive_min_tier']);
  const maximum = parseReviewLayoutTier(props['responsive_max_tier']);
  if (!minimum && !maximum) return true;
  const rank = reviewLayoutTierRank(tier);
  return (
    (minimum === null || rank >= reviewLayoutTierRank(minimum)) &&
    (maximum === null || rank <= reviewLayoutTierRank(maximum))
  );
}

function needsResponsiveReview(source: ZuiDocument): boolean {
  return (
    Object.values(source.nodes ?? {}).some(needsResponsiveNode) ||
    Object.values(source.nodes ?? {}).some((node) =>
      hasResponsiveLayoutValue(node),
    )
  );
}

/** A cloned Penpot board cannot change its flex/grid topology at a breakpoint. */
export function reviewBoardNeedsResponsiveRebuild(
  source: ZuiDocument,
  previousViewportWidth: number | undefined,
  viewportWidth: number,
): boolean {
  if (previousViewportWidth === viewportWidth) return false;
  return Object.values(source.nodes ?? {}).some(
    (node) =>
      hasResponsiveLayoutValue(node.props) ||
      hasResponsiveLayoutValue(node.layout),
  );
}

function isTable(value: unknown): value is ZuiTable {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isBreakpointTable(value: unknown): value is ZuiTable {
  return (
    isTable(value) &&
    MUI_REVIEW_BREAKPOINTS.some(({ key }) => Object.hasOwn(value, key))
  );
}

function hasResponsiveLayoutValue(value: unknown): boolean {
  if (!isTable(value)) return false;
  for (const [key, nested] of Object.entries(value)) {
    if (RESPONSIVE_LAYOUT_KEYS.has(key) && isBreakpointTable(nested))
      return true;
    if (hasResponsiveLayoutValue(nested)) return true;
  }
  return false;
}

function responsiveValueAtWidth(
  value: ZuiTable,
  viewportWidth: number,
): unknown {
  const available = MUI_REVIEW_BREAKPOINTS.filter(({ key }) =>
    Object.hasOwn(value, key),
  );
  if (!available.length) return value;
  let selected = available[0];
  for (const breakpoint of available) {
    if (breakpoint.minWidth <= viewportWidth) selected = breakpoint;
  }
  return value[selected.key];
}

function materializeResponsiveLayoutValues(
  value: unknown,
  viewportWidth: number,
): void {
  if (!isTable(value)) return;
  for (const [key, nested] of Object.entries(value)) {
    if (RESPONSIVE_LAYOUT_KEYS.has(key) && isBreakpointTable(nested)) {
      value[key] = responsiveValueAtWidth(nested, viewportWidth) as never;
      continue;
    }
    materializeResponsiveLayoutValues(nested, viewportWidth);
  }
}

function applyResponsiveLayoutValues(
  document: ZuiDocument,
  viewportWidth: number,
): void {
  for (const node of Object.values(document.nodes ?? {})) {
    materializeResponsiveLayoutValues(node.props, viewportWidth);
    materializeResponsiveLayoutValues(node.layout, viewportWidth);
  }
}

function applyResponsiveReviewLayout(
  document: ZuiDocument,
  viewportWidth: number,
): void {
  applyResponsiveLayoutValues(document, viewportWidth);
  const tier = reviewLayoutTier(viewportWidth);
  const rootIds = new Set(zuiRootNodeIds(document));
  for (const [nodeId, node] of Object.entries(document.nodes ?? {})) {
    // A view/component root is the board being reviewed. Its authored tier
    // metadata must not hide the board itself; descendants still collapse.
    if (rootIds.has(nodeId) || !needsResponsiveNode(node)) continue;
    if (responsiveNodeVisible(node, tier)) continue;
    node.props ??= {};
    node.props['visibility'] = 'collapsed';
  }
}

/**
 * Mirror the retained Workbench shell allocator for the Penpot projection.
 *
 * The authored shell keeps the normal right-drawer token as its preferred
 * width.  Penpot's fixed flex children do not apply a max constraint while
 * resolving the initial board, so the review projection must make the
 * compact Regular-tier width explicit.  This mutates only the cloned review
 * document; the product source and its export metadata remain untouched.
 */
function applyWorkbenchResponsiveGeometry(
  document: ZuiDocument,
  viewportWidth: number,
): void {
  const tier = reviewLayoutTier(viewportWidth);
  if (tier !== 'regular' && tier !== 'wide') return;
  for (const [nodeId, node] of Object.entries(document.nodes ?? {})) {
    if (
      node.control_id !== WORKBENCH_RIGHT_DRAWER_CONTROL_ID &&
      nodeId !== 'right_drawer_shell' &&
      !nodeId.endsWith('__right_drawer_shell')
    )
      continue;
    const width = node.layout?.['width'];
    if (!isRecord(width)) continue;
    const token =
      tier === 'regular'
        ? RIGHT_DRAWER_COMPACT_MAX_WIDTH_TOKEN
        : RIGHT_DRAWER_DEFAULT_WIDTH_TOKEN;
    width['preferred'] = token;
    width['max'] = token;
  }
}

function needsResponsiveNode(node: ZuiNode): boolean {
  const props = node.props;
  return (
    parseReviewLayoutTier(props?.['responsive_min_tier']) !== null ||
    parseReviewLayoutTier(props?.['responsive_max_tier']) !== null
  );
}

function isScrollableNode(node: ZuiNode): boolean {
  const component = node.component.toLowerCase();
  if (component === 'scrollablebox' || component === 'scrollbox') return true;
  const container = node.layout?.['container'];
  if (!isRecord(container)) return false;
  const kind = container['kind'];
  return (
    typeof kind === 'string' &&
    (kind.toLowerCase() === 'scrollablebox' ||
      kind.toLowerCase() === 'scrollbox')
  );
}

function isScrollableReviewDocument(document: ZuiDocument): boolean {
  return Object.values(document.nodes ?? {}).some(isScrollableNode);
}

function applyReviewScrollPosition(
  document: ZuiDocument,
  position: 'start' | 'end',
  target?: ZuiNode,
): void {
  const nodes = target ? [target] : Object.values(document.nodes ?? {});
  if (target && !isScrollableNode(target))
    throw new Error('Workbench scroll selector must target a ScrollableBox');
  for (const node of nodes) {
    if (!isScrollableNode(node)) continue;
    node.props ??= {};
    // This is a design-only host input. Export still restores the original
    // product source stored with the imported asset.
    node.props['__zircon_review_scroll_position'] = position;
  }
}

function nativePainterNodes(
  document: ZuiDocument,
  state: string,
  targets?: readonly ZuiNode[],
): ZuiNode[] {
  return (targets ?? Object.values(document.nodes ?? {})).filter((node) =>
    nativePainterReviewStates(node)?.includes(state),
  );
}

const AUTHORED_POPUP_COMPONENTS = new Set([
  'Popup',
  'Popover',
  'Modal',
  'ContextMenu',
  'ContextActionMenu',
  'Dropdown',
  'DropdownPopup',
  'Select',
]);

function authoredPopupNodes(document: ZuiDocument): ZuiNode[] {
  return Object.values(document.nodes ?? {}).filter(isAuthoredPopupNode);
}

function isAuthoredPopupNode(node: ZuiNode): boolean {
  if (!AUTHORED_POPUP_COMPONENTS.has(node.component)) return false;
  const props = node.props ?? {};
  return ['open', 'popup_open'].some((key) => typeof props[key] === 'boolean');
}

/** Apply an authored popup contract without inventing a replacement menu. */
function applyAuthoredPopupReviewState(
  document: ZuiDocument,
  state: string,
  targets?: readonly ZuiNode[],
): boolean {
  if (!['open', 'closed', 'focus-return'].includes(state)) return false;
  const nodes = targets
    ? targets.filter(isAuthoredPopupNode)
    : authoredPopupNodes(document);
  if (!nodes.length) return false;
  for (const node of nodes) {
    if (state === 'focus-return')
      focusInvokingPopupAnchor(document, node, targets !== undefined);
    clearTransientState(node);
    node.state ??= {};
    const open = state === 'open';
    node.state['open'] = open;
    node.state['popup_open'] = open;
  }
  return true;
}

function focusInvokingPopupAnchor(
  document: ZuiDocument,
  popup: ZuiNode,
  requireSourceIdentity = false,
): void {
  const widget = popup['widget'];
  const anchor = isRecord(widget) ? widget['popup_anchor'] : undefined;
  if (!isRecord(anchor))
    throw new Error('Workbench focus return requires an authored popup anchor');

  let controlId: string | undefined;
  if (anchor['kind'] === 'control') {
    controlId =
      typeof anchor['control_id'] === 'string' ? anchor['control_id'] : undefined;
  } else if (anchor['kind'] === 'pointer') {
    const ownerProperty = anchor['owner_property'];
    if (typeof ownerProperty !== 'string' || !ownerProperty)
      throw new Error('Workbench pointer popup anchor has no owner property');
    const owner = popup.props?.[ownerProperty] ?? popup.state?.[ownerProperty];
    controlId = typeof owner === 'string' ? owner : undefined;
  }
  if (!controlId)
    throw new Error('Workbench focus return could not resolve its invoking control');

  const sourcePath = popup['penpot_review_source_path'];
  const instancePath = popup['penpot_review_instance_path'];
  const hasSourceIdentity =
    typeof sourcePath === 'string' &&
    sourcePath.length > 0 &&
    typeof instancePath === 'string' &&
    instancePath.length > 0;
  if (
    (requireSourceIdentity && !hasSourceIdentity) ||
    ((sourcePath !== undefined || instancePath !== undefined) &&
      !hasSourceIdentity)
  )
    throw new Error('Workbench focus return popup has no authored source identity');

  const matches = Object.values(document.nodes ?? {}).filter(
    (node) =>
      node.control_id === controlId &&
      (!hasSourceIdentity ||
        (node['penpot_review_source_path'] === sourcePath &&
          node['penpot_review_instance_path'] === instancePath)),
  );
  if (matches.length !== 1)
    throw new Error(
      `Workbench focus return anchor ${controlId} resolved ${matches.length} nodes`,
    );
  const invokingControl = matches[0]!;
  clearTransientState(invokingControl);
  invokingControl.state = {
    ...(invokingControl.state ?? {}),
    focused: true,
    focus_visible: true,
  };
}

function clearTransientState(node: ZuiNode): void {
  for (const key of STATE_KEYS) {
    if (node.props) delete node.props[key];
    if (node.state) delete node.state[key];
  }
}

/** A single-select Tabs host has one interaction target, not three hovered tabs. */
function applySingleSelectTabReviewState(
  document: ZuiDocument,
  state: string,
): boolean {
  if (!['hover', 'pressed', 'focused', 'disabled', 'selected'].includes(state))
    return false;
  const nodes = document.nodes ?? {};
  const group = nodes[document.root?.node ?? ''];
  if (
    group?.component !== 'Tabs' ||
    group.props?.['selection_state'] !== 'single'
  )
    return false;
  const tabs = (group.children ?? []).map(({ node }) => nodes[node]);
  if (tabs.length < 2 || tabs.some((tab) => tab?.component !== 'Tab'))
    return false;
  const current = group.props?.['selected_index'];
  if (
    typeof current !== 'number' ||
    !Number.isInteger(current) ||
    current < 0 ||
    current >= tabs.length
  )
    throw new Error('Single-select Tabs review needs a valid selected_index');
  const target = (current + 1) % tabs.length;
  for (const node of Object.values(nodes)) clearTransientState(node);
  for (const [index, tab] of tabs.entries()) {
    tab.state ??= {};
    tab.state['selected'] = index === (state === 'selected' ? target : current);
  }
  const selected = tabs[target];
  if (state === 'selected') {
    const value = selected.props?.['text'];
    if (typeof value !== 'string' || !value)
      throw new Error('Single-select Tabs review needs a named target tab');
    group.props = { ...group.props, selected_index: target, value };
  } else {
    for (const key of REVIEW_STATES[state]) tabs[target].state![key] = true;
  }
  return true;
}

function applyNativePainterReviewState(
  document: ZuiDocument,
  state: string,
  targets?: readonly ZuiNode[],
): boolean {
  const nodes = nativePainterNodes(document, state, targets);
  if (!nodes.length) return false;
  for (const node of nodes) {
    const hasPopupOpenContract =
      Object.hasOwn(node.props ?? {}, 'open') ||
      Object.hasOwn(node.props ?? {}, 'popup_open');
    if (state === 'focus-return')
      focusInvokingPopupAnchor(document, node, targets !== undefined);
    clearTransientState(node);
    node.state ??= {};
    switch (state) {
      case 'focused':
        node.state['focused'] = true;
        node.state['focus_visible'] = true;
        break;
      case 'disabled':
        node.state['disabled'] = true;
        break;
      case 'open':
        if (nativePainterComponent(node) === 'TreeView') {
          node.state['expanded'] = true;
        } else {
          node.state['open'] = true;
          node.state['popup_open'] = true;
        }
        break;
      case 'closed':
        node.state['open'] = false;
        node.state['popup_open'] = false;
        break;
      case 'focus-return':
        node.state['open'] = false;
        node.state['popup_open'] = false;
        break;
      case 'selected':
        if (nativePainterComponent(node) === 'NotificationCenter') {
          const first = node.props?.['notifications'];
          const record =
            Array.isArray(first) && typeof first[0] === 'string'
              ? first[0].split('|', 1)[0]?.trim()
              : undefined;
          if (record) node.state['selected_notification_id'] = record;
        }
        node.state['selected'] = true;
        break;
      case 'dragging':
        node.state['dragging'] = true;
        break;
      case 'drop-allowed':
      case 'drop-blocked':
        node.state['dragging'] = true;
        node.state['drop_hovered'] = true;
        node.state['active_drag_target'] = true;
        node.state['drop_allowed'] = state === 'drop-allowed';
        break;
      case 'empty':
        if (nativePainterComponent(node) === 'CommandPalette') {
          node.state['commands'] = [];
          node.state['filtered_commands'] = [];
          node.state['recent_commands'] = [];
          node.state['selected_command_id'] = '';
        } else if (nativePainterComponent(node) === 'NotificationCenter') {
          node.state['notifications'] = [];
          node.state['unread_count'] = 0;
          node.state['selected_notification_id'] = '';
        } else if (
          nativePainterComponent(node) === 'DataGrid' ||
          nativePainterComponent(node) === 'TreeView'
        ) {
          node.state['collection_items'] = [];
        } else if (nativePainterComponent(node) === 'AgentChat') {
          node.state['messages'] = [];
          node.state['composer_text'] = '';
          node.state['streaming'] = false;
          node.state['error'] = false;
        } else if (nativePainterComponent(node) === 'ChatComposer') {
          node.state['composer_text'] = '';
          node.state['streaming'] = false;
        }
        break;
      case 'running':
        if (nativePainterComponent(node) === 'AgentPlan') {
          node.state['component_variant'] = 'running';
        } else if (
          nativePainterComponent(node) === 'AgentChat' ||
          nativePainterComponent(node) === 'ChatComposer'
        ) {
          node.state['streaming'] = true;
          if (nativePainterComponent(node) === 'AgentChat')
            node.state['error'] = false;
        }
        break;
      case 'error':
        if (
          nativePainterComponent(node) === 'AgentChat' ||
          nativePainterComponent(node) === 'ChatComposer'
        ) {
          node.state['streaming'] = false;
          node.state['error'] = true;
        }
        break;
      case 'long-en':
        if (nativePainterComponent(node) === 'AgentChat') {
          node.state['messages'] = [
            'user|Please keep the full context visible while checking the responsive shell at the narrow breakpoint.',
            'agent|The retained host keeps the conversation and cited tool result aligned without hiding overflow.',
          ];
          node.state['streaming'] = false;
        } else if (nativePainterComponent(node) === 'ChatComposer') {
          node.state['composer_text'] =
            'Continue the parity review across the full shell, compact rail, and mobile action bar.';
        }
        break;
      case 'long-zh':
        if (nativePainterComponent(node) === 'AgentChat') {
          node.state['messages'] = [
            'user|请在窄屏断点检查响应式外壳，并保持完整上下文可见。',
            'agent|保留的宿主会让对话、工具结果和引用文件保持对齐，不隐藏溢出内容。',
          ];
          node.state['streaming'] = false;
        } else if (nativePainterComponent(node) === 'ChatComposer') {
          node.state['composer_text'] =
            '请继续检查完整外壳、紧凑图标栏和移动端底部操作栏的一致性。';
        }
        break;
      case 'complete':
        if (nativePainterComponent(node) === 'AgentPlan') {
          node.state['component_variant'] = 'complete';
          node.state['value'] = 1;
          node.state['collection_items'] = sourceStringArray(
            node,
            'collection_items',
          ).map((value) => replaceDelimitedStatus(value, 'done'));
        }
        break;
      case 'blocked':
        if (nativePainterComponent(node) === 'AgentPlan') {
          node.state['component_variant'] = 'blocked';
          node.state['validation_level'] = 'error';
        }
        break;
      case 'success':
        if (nativePainterComponent(node) === 'ToolCalls') {
          node.state['component_variant'] = 'success';
          node.state['collection_items'] = sourceStringArray(
            node,
            'collection_items',
          ).map((value) => replaceDelimitedStatus(value, 'success'));
        }
        break;
      case 'failure':
        if (nativePainterComponent(node) === 'ToolCalls') {
          node.state['component_variant'] = 'failure';
          node.state['collection_items'] = sourceStringArray(
            node,
            'collection_items',
          ).map((value) => replaceDelimitedStatus(value, 'failure'));
        }
        break;
      case 'pending':
      case 'approved':
      case 'denied':
        if (nativePainterComponent(node) === 'AgentApproval') {
          node.state['approval_state'] = state;
          node.state['component_variant'] = state;
          if (state === 'approved') node.state['destructive'] = false;
        }
        break;
      case 'normal':
      case 'warning':
      case 'exceeded':
        if (nativePainterComponent(node) === 'AIUsage') {
          node.state['component_variant'] = state;
          node.state['validation_level'] =
            state === 'exceeded' ? 'error' : state;
          node.state['value'] =
            state === 'normal' ? 0.39 : state === 'warning' ? 0.75 : 1;
        }
        break;
    }
    // Specimens of an invoked popup must remain visible for focus, selection
    // and empty-content reviews; the authored default itself stays closed.
    if (
      state !== 'closed' &&
      state !== 'focus-return' &&
      [
        'Dialog',
        'ConfirmDialog',
        'CommandPalette',
        'NotificationCenter',
      ].includes(nativePainterComponent(node) ?? '') &&
      hasPopupOpenContract
    ) {
      node.state['open'] = true;
      node.state['popup_open'] = true;
    }
  }
  return true;
}

function sourceStringArray(node: ZuiNode, property: string): string[] {
  const value = node.props?.[property];
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : [];
}

function replaceDelimitedStatus(value: string, status: string): string {
  const separator = value.indexOf('|');
  return separator < 0 ? value : `${status}${value.slice(separator)}`;
}

interface WorkbenchStateSelector {
  sourcePath: string;
  controlId: string;
  sourceNodeId?: string;
  instancePath?: string;
  textOverrides?: WorkbenchTextOverride[];
  scrollTarget?: WorkbenchTargetSelector;
}

interface WorkbenchTargetSelector {
  sourcePath: string;
  controlId: string;
  sourceNodeId?: string;
  instancePath?: string;
}

interface WorkbenchTextOverride {
  sourcePath: string;
  sourceNodeId: string;
  controlId: string;
  instancePath?: string;
  property: 'text';
  value: string;
}

function workbenchStateSelector(
  reviewCase: LayoutReviewCaseMessage,
): WorkbenchStateSelector | undefined {
  const value = reviewCase.data['workbenchState'];
  if (value === undefined) {
    if (reviewCase.host === 'editor' && reviewCase.locale === 'zh-CN')
      throw new Error('zh-CN workbench cases require at least one text override');
    return undefined;
  }
  if (!isRecord(value))
    throw new Error('Workbench state selector must be an object');
  rejectUnknownFields(
    value,
    new Set([
      'sourcePath',
      'controlId',
      'sourceNodeId',
      'instancePath',
      'textOverrides',
      'scrollTarget',
    ]),
    'workbenchState',
  );
  requireNonemptyString(value['sourcePath'], 'Workbench state sourcePath');
  requireNonemptyString(value['controlId'], 'Workbench state controlId');
  if (value['sourceNodeId'] !== undefined)
    requireNonemptyString(value['sourceNodeId'], 'Workbench state sourceNodeId');
  if (value['instancePath'] !== undefined)
    parseWorkbenchInstancePath(value['instancePath'], 'Workbench state instancePath');
  const textOverrides = parseWorkbenchTextOverrides(value['textOverrides']);
  const scrollTarget = parseWorkbenchTargetSelector(
    value['scrollTarget'],
    'Workbench scrollTarget',
  );
  if (reviewCase.locale === 'zh-CN' && textOverrides.length === 0)
    throw new Error('zh-CN workbench cases require at least one text override');
  if (reviewCase.host !== 'editor')
    throw new Error('Workbench state selectors require the editor host');
  return {
    sourcePath: value['sourcePath'] as string,
    controlId: value['controlId'] as string,
    ...(value['sourceNodeId'] === undefined
      ? {}
      : { sourceNodeId: value['sourceNodeId'] as string }),
    ...(value['instancePath'] === undefined
      ? {}
      : { instancePath: value['instancePath'] as string }),
    ...(value['textOverrides'] === undefined ? {} : { textOverrides }),
    ...(scrollTarget === undefined ? {} : { scrollTarget }),
  };
}

function parseWorkbenchTargetSelector(
  value: unknown,
  context: string,
): WorkbenchTargetSelector | undefined {
  if (value === undefined) return undefined;
  if (!isRecord(value)) throw new Error(`${context} must be an object`);
  rejectUnknownFields(
    value,
    new Set(['sourcePath', 'controlId', 'sourceNodeId', 'instancePath']),
    context,
  );
  requireNonemptyString(value['sourcePath'], `${context} sourcePath`);
  requireNonemptyString(value['controlId'], `${context} controlId`);
  if (value['sourceNodeId'] !== undefined)
    requireNonemptyString(value['sourceNodeId'], `${context} sourceNodeId`);
  if (value['instancePath'] !== undefined)
    parseWorkbenchInstancePath(value['instancePath'], `${context} instancePath`);
  return {
    sourcePath: value['sourcePath'] as string,
    controlId: value['controlId'] as string,
    ...(value['sourceNodeId'] === undefined
      ? {}
      : { sourceNodeId: value['sourceNodeId'] as string }),
    ...(value['instancePath'] === undefined
      ? {}
      : { instancePath: value['instancePath'] as string }),
  };
}

function parseWorkbenchTextOverrides(value: unknown): WorkbenchTextOverride[] {
  if (value === undefined) return [];
  if (!Array.isArray(value) || value.length === 0)
    throw new Error('Workbench state textOverrides must be a nonempty array');
  const seen = new Set<string>();
  return value.map((entry, index) => {
    if (!isRecord(entry))
      throw new Error(`Workbench text override ${index} must be an object`);
    rejectUnknownFields(
      entry,
      new Set([
        'sourcePath',
        'sourceNodeId',
        'controlId',
        'instancePath',
        'property',
        'value',
      ]),
      `workbenchState.textOverrides[${index}]`,
    );
    requireNonemptyString(
      entry['sourcePath'],
      `Workbench text override ${index} sourcePath`,
    );
    requireNonemptyString(
      entry['sourceNodeId'],
      `Workbench text override ${index} sourceNodeId`,
    );
    requireNonemptyString(
      entry['controlId'],
      `Workbench text override ${index} controlId`,
    );
    if (entry['instancePath'] !== undefined)
      parseWorkbenchInstancePath(
        entry['instancePath'],
        `Workbench text override ${index} instancePath`,
      );
    if (entry['property'] !== 'text')
      throw new Error(`Workbench text override ${index} property must be text`);
    requireNonemptyString(
      entry['value'],
      `Workbench text override ${index} value`,
    );
    const text = entry['value'] as string;
    if (new TextEncoder().encode(text).byteLength > 4096)
      throw new Error(`Workbench text override ${index} exceeds 4096 UTF-8 bytes`);
    if (/\p{Cc}/u.test(text))
      throw new Error(`Workbench text override ${index} contains a control character`);
    const key = JSON.stringify([
      entry['sourcePath'],
      entry['sourceNodeId'],
      entry['controlId'],
      entry['instancePath'] ?? null,
      entry['property'],
    ]);
    if (seen.has(key))
      throw new Error(`Duplicate workbench text override ${entry['sourcePath']}#${entry['sourceNodeId']}`);
    seen.add(key);
    return {
      sourcePath: entry['sourcePath'] as string,
      sourceNodeId: entry['sourceNodeId'] as string,
      controlId: entry['controlId'] as string,
      ...(entry['instancePath'] === undefined
        ? {}
        : { instancePath: entry['instancePath'] as string }),
      property: 'text',
      value: text,
    };
  });
}

function parseWorkbenchInstancePath(value: unknown, name: string): string {
  requireNonemptyString(value, name);
  let steps: unknown;
  try {
    steps = JSON.parse(value as string) as unknown;
  } catch {
    throw new Error(`${name} must be canonical JSON invocation steps`);
  }
  if (!Array.isArray(steps))
    throw new Error(`${name} must be an array of invocation steps`);
  const normalized = steps.map((step, index) => {
    if (!isRecord(step))
      throw new Error(`${name}[${index}] must be an object`);
    rejectUnknownFields(step, new Set(['sourcePath', 'sourceNodeId']), `${name}[${index}]`);
    requireNonemptyString(step['sourcePath'], `${name}[${index}].sourcePath`);
    requireNonemptyString(step['sourceNodeId'], `${name}[${index}].sourceNodeId`);
    return {
      sourcePath: step['sourcePath'] as string,
      sourceNodeId: step['sourceNodeId'] as string,
    };
  });
  const canonical = JSON.stringify(normalized);
  if (canonical !== value)
    throw new Error(`${name} must use canonical compact camelCase JSON`);
  return canonical;
}

function reviewDataWithoutWorkbenchState(
  data: Record<string, unknown>,
): Record<string, unknown> {
  if (
    !Object.hasOwn(data, 'workbenchState') &&
    !Object.hasOwn(data, 'workbenchPresentation')
  )
    return data;
  const remaining = { ...data };
  delete remaining['workbenchState'];
  delete remaining['workbenchPresentation'];
  return remaining;
}

function resolveWorkbenchStateTarget(
  document: ZuiDocument,
  selector: WorkbenchTargetSelector,
): ZuiNode {
  const matches = Object.values(document.nodes ?? {}).filter(
    (node) =>
      node.control_id === selector.controlId &&
      node['penpot_review_source_path'] === selector.sourcePath &&
      (selector.sourceNodeId === undefined ||
        node['penpot_review_source_node_id'] === selector.sourceNodeId) &&
      (selector.instancePath === undefined ||
        node['penpot_review_instance_path'] === selector.instancePath),
  );
  if (matches.length !== 1)
    throw new Error(
      `Workbench state selector resolved ${matches.length} nodes for ${selector.sourcePath}#${selector.sourceNodeId ?? selector.controlId}`,
    );
  return matches[0]!;
}

function applyWorkbenchTextOverrides(
  document: ZuiDocument,
  overrides: readonly WorkbenchTextOverride[],
): void {
  for (const override of overrides) {
    const matches = Object.values(document.nodes ?? {}).filter(
      (node) =>
        node.control_id === override.controlId &&
        node['penpot_review_source_path'] === override.sourcePath &&
        node['penpot_review_source_node_id'] === override.sourceNodeId &&
        (override.instancePath === undefined ||
          node['penpot_review_instance_path'] === override.instancePath),
    );
    if (matches.length !== 1)
      throw new Error(
        `Workbench text override resolved ${matches.length} nodes for ${override.sourcePath}#${override.sourceNodeId}`,
      );
    const node = matches[0]!;
    if (typeof node.props?.['text'] !== 'string')
      throw new Error(
        `Workbench text override ${override.sourcePath}#${override.sourceNodeId} does not target an authored text property`,
      );
    node.props = { ...node.props, text: override.value };
  }
}

function validateReviewCaseEnvelope(reviewCase: LayoutReviewCaseMessage): void {
  if (!isRecord(reviewCase)) throw new Error('Review case must be an object');
  rejectUnknownFields(reviewCase, REVIEW_CASE_FIELDS, 'review case');
  requireNonemptyString(reviewCase.id, 'Review case id');
  requireNonemptyString(reviewCase.sourcePath, 'Review case sourcePath');
  if (!REVIEW_HOSTS.has(reviewCase.host))
    throw new Error(
      `Review case host is unsupported: ${String(reviewCase.host)}`,
    );
  if (!isRecord(reviewCase.viewport))
    throw new Error('Review case viewport must be an object');
  rejectUnknownFields(
    reviewCase.viewport,
    new Set(['width', 'height']),
    'review viewport',
  );
  if (
    ![
      reviewCase.viewport.width,
      reviewCase.viewport.height,
      reviewCase.dpi,
    ].every((value) => Number.isFinite(value) && value > 0)
  )
    throw new Error('Review viewport and DPI must be positive finite numbers');
  if (!isRecord(reviewCase.data))
    throw new Error('Review case data must be an object');
  const workbenchSelector = workbenchStateSelector(reviewCase);
  const componentData = reviewDataWithoutWorkbenchState(reviewCase.data);
  requireNonemptyString(reviewCase.locale, 'Review case locale');
  requireNonemptyString(reviewCase.state, 'Review case state');
  if (
    reviewCase.scrollPosition !== undefined &&
    !['start', 'end'].includes(reviewCase.scrollPosition)
  )
    throw new Error('Review case scrollPosition must be start or end');
  if (
    (reviewCase.state === 'scroll-before' &&
      reviewCase.scrollPosition === 'end') ||
    (reviewCase.state === 'scroll-after' &&
      reviewCase.scrollPosition === 'start')
  )
    throw new Error('Review case scrollPosition contradicts its state');
  if (reviewCase.themeSourcePath !== undefined)
    requireNonemptyString(
      reviewCase.themeSourcePath,
      'Review case themeSourcePath',
    );
  if (reviewCase.reviewHost !== undefined) {
    if (!isRecord(reviewCase.reviewHost))
      throw new Error('Review case reviewHost must be an object');
    rejectUnknownFields(
      reviewCase.reviewHost,
      new Set(['path', 'sha256']),
      'reviewHost',
    );
    if (
      typeof reviewCase.reviewHost.path !== 'string' ||
      reviewCase.reviewHost.path.trim() === '' ||
      typeof reviewCase.reviewHost.sha256 !== 'string' ||
      !/^[0-9a-f]{64}$/i.test(reviewCase.reviewHost.sha256)
    )
      throw new Error(
        'Review case reviewHost requires a nonempty path and SHA-256',
      );
  }
  if (!Object.hasOwn(REVIEW_STATES, reviewCase.state))
    throw new Error(`Penpot state adapter unavailable for ${reviewCase.state}`);
  if (
    !(reviewCase.host === 'fixture' && isCollectionReviewData(componentData))
  )
    validateComponentReviewData(componentData);
  if (
    Object.hasOwn(componentData, 'componentInput') &&
    reviewCase.host !== 'component'
  )
    throw new Error('Component input data requires the component host');
  if (
    reviewCase.locale !== 'en-US' &&
    !(
      reviewCase.locale === 'zh-CN' &&
      (componentData['componentInput'] ||
        (reviewCase.host === 'fixture' &&
          isCollectionReviewData(componentData)) ||
        reviewCase.state === 'long-zh' ||
        (reviewCase.host === 'editor' &&
          (workbenchSelector?.textOverrides?.length ?? 0) > 0))
    )
  )
    throw new Error(
      `Penpot locale adapter unavailable for ${reviewCase.locale}`,
    );
}

function rejectUnknownFields(
  value: Record<string, unknown>,
  supported: ReadonlySet<string>,
  context: string,
): void {
  const field = Object.keys(value).find((key) => !supported.has(key));
  if (field) throw new Error(`Unknown ${context} field: ${field}`);
}

function requireNonemptyString(value: unknown, name: string): void {
  if (typeof value !== 'string' || value.trim() === '')
    throw new Error(`${name} must be a nonempty string`);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** A review state is a temporary host input, never an edit to the source asset. */
export function reviewDocumentForCase(
  source: ZuiDocument,
  reviewCase: LayoutReviewCaseMessage,
): ZuiDocument {
  validateReviewCaseEnvelope(reviewCase);
  const selector = workbenchStateSelector(reviewCase);
  const responsiveReview = needsResponsiveReview(source);
  if (
    reviewCase.state === 'default' &&
    !responsiveReview &&
    reviewCase.scrollPosition === undefined &&
    !Object.keys(reviewCase.data).length
  )
    return source;
  if (!Object.keys(source.nodes ?? {}).length)
    throw new Error('Theme state review requires a component consumer host');

  const document = cloneZuiDocument(source);
  const componentData = reviewDataWithoutWorkbenchState(reviewCase.data);
  const workbenchTarget = selector
    ? resolveWorkbenchStateTarget(document, selector)
    : undefined;
  applyWorkbenchTextOverrides(document, selector?.textOverrides ?? []);
  if (!(
    reviewCase.host === 'fixture' && isCollectionReviewData(componentData)
  ))
    applyComponentReviewData(document, componentData);
  applyCollectionReviewData(document, componentData);
  if (responsiveReview) {
    applyResponsiveReviewLayout(document, reviewCase.viewport.width);
    applyWorkbenchResponsiveGeometry(document, reviewCase.viewport.width);
  }
  const finish = (): ZuiDocument => {
    const position =
      reviewCase.scrollPosition ??
      (reviewCase.state === 'scroll-before'
        ? 'start'
        : reviewCase.state === 'scroll-after'
          ? 'end'
          : undefined);
    if (position) {
      const scrollTarget = selector?.scrollTarget
        ? resolveWorkbenchStateTarget(document, selector.scrollTarget)
        : workbenchTarget;
      if (
        scrollTarget
          ? !isScrollableNode(scrollTarget)
          : !isScrollableReviewDocument(document)
      )
        throw new Error(
          'Review scroll position requires an authored ScrollableBox',
        );
      applyReviewScrollPosition(document, position, scrollTarget);
    }
    return document;
  };
  if (reviewCase.state === 'default') return finish();
  if (
    reviewCase.state === 'scroll-before' ||
    reviewCase.state === 'scroll-after'
  )
    return finish();
  if (
    reviewCase.host === 'editor' &&
    isRecord(reviewCase.data['workbenchPresentation']) &&
    (reviewCase.state === 'empty' || reviewCase.state === 'selected')
  )
    return finish();
  const stateTargets = workbenchTarget ? [workbenchTarget] : undefined;
  if (applyNativePainterReviewState(document, reviewCase.state, stateTargets))
    return finish();
  if (
    applyAuthoredPopupReviewState(document, reviewCase.state, stateTargets)
  )
    return finish();
  if (NATIVE_ONLY_REVIEW_STATES.has(reviewCase.state))
    throw new Error(
      `Review state ${reviewCase.state} requires a native painter component`,
    );
  if (
    !workbenchTarget &&
    applySingleSelectTabReviewState(document, reviewCase.state)
  )
    return finish();
  for (const node of workbenchTarget
    ? [workbenchTarget]
    : Object.values(document.nodes ?? {})) {
    clearTransientState(node);
    node.state ??= {};
    for (const key of REVIEW_STATES[reviewCase.state]) node.state[key] = true;
  }
  return finish();
}
