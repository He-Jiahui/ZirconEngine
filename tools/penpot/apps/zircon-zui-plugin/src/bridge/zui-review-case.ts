import { readViewportToolbarPublication } from './zui-viewport-toolbar-publication';
import { deriveViewportToolbarReview } from './zui-viewport-toolbar';
import { deriveTreeSelectionOwnerReview, hasTreeSelectionRows } from './zui-tree-selection-owner';
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

/** UTF-8 byte length, including replacement of lone UTF-16 surrogates, without sandbox globals. */
export function reviewUtf8ByteLength(text: string): number {
  let length = 0;
  for (let index = 0; index < text.length; index++) {
    const unit = text.charCodeAt(index);
    if (unit <= 0x7f) length++;
    else if (unit <= 0x7ff) length += 2;
    else if (unit >= 0xd800 && unit <= 0xdbff &&
      index + 1 < text.length && text.charCodeAt(index + 1) >= 0xdc00 && text.charCodeAt(index + 1) <= 0xdfff) {
      length += 4;
      index++;
    } else length += 3;
  }
  return length;
}

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
  'PopupMenu',
]);

function authoredPopupNodes(document: ZuiDocument): ZuiNode[] {
  return Object.values(document.nodes ?? {}).filter(isAuthoredPopupNode);
}

function isAuthoredPopupNode(node: ZuiNode): boolean {
  if (!AUTHORED_POPUP_COMPONENTS.has(node.component)) return false;
  if (node.component === 'PopupMenu') return true;
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
  let nodes = targets
    ? targets.filter(isAuthoredPopupNode)
    : authoredPopupNodes(document);
  if (!targets) {
    const popupMenus = nodes.filter((node) => node.component === 'PopupMenu');
    if (popupMenus.length > 1) return false;
    if (popupMenus.length === 1) nodes = popupMenus;
  }
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

  // UiPopupAnchor resolves a unique control in the whole mounted surface.
  // The invoking control can belong to a sibling component of the popup.
  const matches = Object.values(document.nodes ?? {}).filter(
    (node) => node.control_id === controlId,
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
    if (reviewUtf8ByteLength(text) > 4096)
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

function sameWorkbenchReviewValue(left: unknown, right: unknown): boolean {
  if (left === right) return true;
  if (Array.isArray(left) || Array.isArray(right))
    return Array.isArray(left) && Array.isArray(right) && left.length === right.length &&
      left.every((value, index) => sameWorkbenchReviewValue(value, right[index]));
  if (!isRecord(left) || !isRecord(right)) return false;
  const keys = Object.keys(left);
  return keys.length === Object.keys(right).length && keys.every((key) =>
    Object.hasOwn(right, key) && sameWorkbenchReviewValue(left[key], right[key]));
}

function requireManagedReview(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error('Managed scene fingerprint review: ' + message);
}

/** Browser binding check; the producer separately verifies the physical files. */
function validateManagedWorkbenchReviewMetadata(reviewCase: LayoutReviewCaseMessage): void {
  const data = reviewCase.data;
  requireManagedReview(reviewCase.host === 'editor', 'requires the editor host');
  const fingerprint = data['managedSceneFingerprint'];
  requireManagedReview(isRecord(fingerprint) && Object.keys(fingerprint).length === 2 &&
    Object.hasOwn(fingerprint, 'projectRoot') && Object.hasOwn(fingerprint, 'sources'),
    'requires exact projectRoot and sources fields');
  const projectRoot = fingerprint['projectRoot'];
  requireManagedReview(typeof projectRoot === 'string' &&
    /^(?:[A-Za-z]:[\\/]|[\\/])/u.test(projectRoot) && !/\p{Cc}/u.test(projectRoot),
    'projectRoot must be an absolute physical project path');
  const paths = ['zircon-project.toml', 'assets/scenes/main.scene.toml',
    'assets/scenes/workbench-review-empty.scene.toml'];
  const sources = fingerprint['sources'];
  requireManagedReview(Array.isArray(sources) && sources.length === paths.length,
    'requires the ordered project manifest and both managed scenes');
  for (const [index, source] of sources.entries())
    requireManagedReview(isRecord(source) && Object.keys(source).length === 2 &&
      Object.hasOwn(source, 'sourcePath') && Object.hasOwn(source, 'sha256') &&
      source['sourcePath'] === paths[index] && typeof source['sha256'] === 'string' &&
      /^[0-9a-f]{64}$/.test(source['sha256']), 'has an invalid ordered source/hash');

  const evidence = data['normal_window_evidence'];
  const presentation = data['workbenchPresentation'];
  requireManagedReview(isRecord(evidence) && isRecord(presentation),
    'requires its actual normal evidence and Workbench presentation');
  const normalFields = ['workbenchStatePublication', 'presentationCursor', 'presentedFrameCount',
    'viewportToolbarPublication', 'semanticNodes', 'focusPublication', 'designTokenInput', 'arrangedInventoryPublication', 'runtimeSceneMedia', 'commands'];
  requireManagedReview(normalFields.slice(0, 4).every((key) => Object.hasOwn(evidence, key)) &&
    Object.keys(evidence).every((key) => normalFields.includes(key)),
    'requires the complete canonical normal publication without unknown fields');
  if (Object.hasOwn(evidence, 'commands')) {
    const commands = evidence['commands'];
    requireManagedReview(Array.isArray(commands) && commands.every((command) => isRecord(command) &&
      typeof command['commandIndex'] === 'number' && Number.isSafeInteger(command['commandIndex']) && command['commandIndex'] >= 0) &&
      new Set(commands.map((command) => command['commandIndex'])).size === commands.length, 'has incomplete or duplicate actual command indices');
  }
  const publication = evidence['workbenchStatePublication'];
  const publicationFields = ['authority', 'observed', 'presentationCursor', 'presentedFrameCount',
    'workbenchPresentation', 'managedSceneFingerprint'];
  requireManagedReview(isRecord(publication) && Object.keys(publication).length === publicationFields.length &&
    publicationFields.every((key) => Object.hasOwn(publication, key)) &&
    publication['authority'] === 'normal-retained-workbench-projection' && publication['observed'] === true,
    'requires an actual observed Workbench business publication');
  requireManagedReview(typeof evidence['presentedFrameCount'] === 'number' &&
    Number.isSafeInteger(evidence['presentedFrameCount']) && evidence['presentedFrameCount'] > 0 &&
    publication['presentedFrameCount'] === evidence['presentedFrameCount'],
    'business publication belongs to a different successful frame');
  const cursor = evidence['presentationCursor'];
  const businessCursor = publication['presentationCursor'];
  const counters = ['structure', 'geometry', 'interaction', 'viewport', 'hitTest', 'theme', 'diagnostics'];
  requireManagedReview(isRecord(cursor) && isRecord(businessCursor) &&
    Object.keys(cursor).length === counters.length && Object.keys(businessCursor).length === counters.length &&
    counters.every((key) => Object.hasOwn(cursor, key) && Object.hasOwn(businessCursor, key) &&
      typeof cursor[key] === 'number' && Number.isSafeInteger(cursor[key]) && cursor[key] >= 0 &&
      cursor[key] === businessCursor[key]), 'business publication has a missing or different actual cursor');
  const source = presentation['sourceFingerprint'];
  requireManagedReview(presentation['schema'] === 'dev.zircon.editor.workbench-presentation' &&
    presentation['version'] === 1 && isRecord(source) && Object.keys(source).length === 2 &&
    Object.hasOwn(source, 'sourcePath') && Object.hasOwn(source, 'sha256') &&
    source['sourcePath'] === reviewCase.sourcePath && typeof source['sha256'] === 'string' &&
    /^[0-9a-f]{64}$/.test(source['sha256']), 'presentation has a different or invalid Source owner');
  requireManagedReview(sameWorkbenchReviewValue(presentation, publication['workbenchPresentation']),
    'presentation differs from its actual business publication');
  requireManagedReview(sameWorkbenchReviewValue(fingerprint, publication['managedSceneFingerprint']),
    'differs from its actual business publication');
  const status = presentation['status'];
  const comparablePath = (value: string): string => {
    const physical = value.replaceAll('/', '\\');
    return (physical.startsWith('\\\\?\\') ? physical.slice(4) : physical).replace(/\\+$/u, '').toLowerCase();
  };
  requireManagedReview(isRecord(status) && typeof status['projectPath'] === 'string' &&
    comparablePath(status['projectPath']) === comparablePath(projectRoot),
    'presentation differs from its bound project root');
  runtimeSceneMediaForCase(reviewCase);
}

function reviewDataWithoutWorkbenchState(
  reviewCase: LayoutReviewCaseMessage,
): Record<string, unknown> {
  const data = reviewCase.data;
  const managed = Object.hasOwn(data, 'managedSceneFingerprint');
  if (managed) validateManagedWorkbenchReviewMetadata(reviewCase);
  if (
    !Object.hasOwn(data, 'workbenchState') &&
    !Object.hasOwn(data, 'workbenchPresentation') &&
    !Object.hasOwn(data, 'normal_window_evidence') && !managed
  )
    return data;
  const remaining = { ...data };
  delete remaining['workbenchState'];
  delete remaining['workbenchPresentation'];
  delete remaining['normal_window_evidence'];
  if (managed) delete remaining['managedSceneFingerprint'];
  return remaining;
}

export interface RuntimeSceneOwner {
  sourcePath: string;
  sourceNodeId: string;
  instancePath: string;
}
export interface RuntimeSceneImage extends Record<string, unknown> {
  owner: RuntimeSceneOwner;
  viewportRoot: RuntimeSceneOwner;
  physicalSize: [number, number];
  pngSha256: string;
  pngByteLength: number;
}
export interface RuntimeSceneMedia {
  schema: 'dev.zircon.editor.runtime-scene-media';
  version: 1;
  bootstrapCase: { path: string; caseJsonSha256: string; caseSha256: string };
  bootstrapPixelAssociation: { path: string; sha256: string; observed: Record<string, unknown> };
  image: { mime: 'image/png'; byteLength: number; pngSha256: string; base64: string };
}
export interface RuntimeSceneEmbeddedImage {
  uri: string;
  name: string;
  mime: 'image/png';
  base64: string;
  runtimeScene: { pngSha256: string; byteLength: number; physicalSize: [number, number] };
}
const runtimeSceneNeed = (condition: unknown, message: string): void => {
  if (!condition) throw new Error('Runtime scene media: ' + message);
};
function sceneAssert(condition: unknown, message: string): asserts condition { runtimeSceneNeed(condition, message); }
const sceneHash = (value: unknown): value is string => typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);
const sceneString = (value: unknown): value is string => typeof value === 'string' && value.trim().length > 0;
const sceneUint = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
function scenePathAlias(value: string): string {
  return value.startsWith('res://ui/editor/') ? 'zircon_editor/assets/' + value.slice(6) : value;
}
function scenePath(value: unknown): string {
  sceneAssert(sceneString(value), 'missing source path');
  const path = scenePathAlias(value);
  sceneAssert(!path.includes('\\') && !path.split('/').some((part) => part === '.' || part === '..'), 'invalid source alias');
  return path;
}
interface SceneStep { sourcePath: string; sourceNodeId: string }
function sceneSteps(value: unknown): SceneStep[] {
  let parsed: unknown = value;
  if (typeof value === 'string') parsed = JSON.parse(value);
  sceneAssert(Array.isArray(parsed) && parsed.length <= 64, 'invalid full invocation');
  return parsed.map((step) => {
    sceneAssert(isRecord(step) && Object.keys(step).length === 2 && sceneString(step['sourceNodeId']), 'invalid invocation step');
    return { sourcePath: scenePath(step['sourcePath']), sourceNodeId: step['sourceNodeId'] };
  });
}
function scenePrefix(steps: SceneStep[], prefix: SceneStep[]): boolean {
  return prefix.length <= steps.length && prefix.every((step, index) =>
    step.sourcePath === steps[index]!.sourcePath && step.sourceNodeId === steps[index]!.sourceNodeId);
}
function sceneTuple(value: Record<string, unknown>, prefix: SceneStep[] = [], replace = prefix): string {
  sceneAssert(sceneString(value['sourceNodeId']), 'missing source node');
  const steps = sceneSteps(value['instancePath']);
  sceneAssert(scenePrefix(steps, prefix), 'node is outside its proved owner prefix');
  return JSON.stringify([scenePath(value['sourcePath']), value['sourceNodeId'], [...replace, ...steps.slice(prefix.length)]]);
}
function sceneNodeTuple(node: ZuiNode): Record<string, unknown> {
  return { sourcePath: node['penpot_review_source_path'], sourceNodeId: node['penpot_review_source_node_id'],
    instancePath: node['penpot_review_instance_path'] };
}

function sceneSubmittedCommand(normal: Record<string, unknown>, scene: Record<string, unknown>): Record<string, unknown> {
  const commands = normal['commands'], owner = scene['owner'];
  sceneAssert(Array.isArray(commands) && commands.every(isRecord) && isRecord(owner), 'missing actual same-case scene commands');
  const viewport = { ...owner, sourceNodeId: 'viewport' };
  const indices = new Set<number>();
  for (const command of commands) {
    sceneAssert(sceneUint(command['commandIndex']) && !indices.has(command['commandIndex']), 'partial or duplicate actual command index');
    indices.add(command['commandIndex']);
  }
  const ownerPath = scenePath(owner['sourcePath']);
  // Source-less normal brushes are unrelated; a relevant partial tuple must fail.
  const matches = commands.filter((command) => {
    const source = command['source'];
    return isRecord(source) && source['sourceNodeId'] === owner['sourceNodeId'] && typeof source['sourcePath'] === 'string' &&
      scenePathAlias(source['sourcePath']) === ownerPath && sceneTuple(source) === sceneTuple(owner);
  });
  sceneAssert(matches.length === 1, 'scene has no unique actual submitted source command');
  const command = matches[0]!, source = command['source'], paint = command['paint'];
  sceneAssert(isRecord(source) && isRecord(paint) && paint['kind'] === 'image' &&
    source['controlId'] === 'WorkbenchViewportSurface' && sceneString(source['parentNodeId']) &&
    command['commandIndex'] === scene['commandIndex'] && sameWorkbenchReviewValue(source['sourceRef'], scene['sourceRef']) &&
    sameWorkbenchReviewValue(command['bounds'], scene['commandBounds']) && sameWorkbenchReviewValue(command['clip'], scene['commandClip']) &&
    paint['resourceKey'] === scene['resourceKey'] && paint['generation'] === scene['resourceGeneration'] &&
    paint['residentAfterSuccessfulPresent'] === true && paint['hasUploadBytes'] === false &&
    sameWorkbenchReviewValue([paint['width'],paint['height']], scene['physicalSize']) &&
    sceneTuple({ sourcePath: source['parentSourcePath'], sourceNodeId: source['parentSourceNodeId'],
      instancePath: source['parentInstancePath'] }) === sceneTuple(viewport), 'scene receipt differs from its actual submitted command');
  return command;
}

/** Structural browser checks complement the producer/recorder's physical receipt verification. */
export function runtimeSceneMediaForCase(reviewCase: LayoutReviewCaseMessage): RuntimeSceneMedia | undefined {
  const normal = reviewCase.data['normal_window_evidence'];
  if (!isRecord(normal) || !Object.hasOwn(normal, 'runtimeSceneMedia')) return;
  sceneAssert(reviewCase.host === 'editor' && isRecord(reviewCase.data['managedSceneFingerprint']) &&
    isRecord(reviewCase.data['workbenchPresentation']), 'runtime PNG requires an actual managed Editor case');
  const packet = normal['runtimeSceneMedia'];
  sceneAssert(isRecord(packet) && packet['schema'] === 'dev.zircon.editor.runtime-scene-media' && packet['version'] === 1 &&
    Object.keys(packet).length === 5, 'unsupported packet');
  const bootstrap = packet['bootstrapCase'], association = packet['bootstrapPixelAssociation'], image = packet['image'];
  sceneAssert(isRecord(bootstrap) && Object.keys(bootstrap).length === 3 && sceneString(bootstrap['path']) &&
    sceneHash(bootstrap['caseJsonSha256']) && sceneHash(bootstrap['caseSha256']) &&
    isRecord(association) && Object.keys(association).length === 3 && sceneString(association['path']) &&
    sceneHash(association['sha256']) && isRecord(association['observed']), 'missing immutable bootstrap inputs');
  const observed = association['observed'], readback = observed['normalReadbackReceipt'];
  sceneAssert(observed['schema'] === 'dev.zircon.editor.normal-frame-pixel-association' && observed['version'] === 1 &&
    observed['pixelFrameVerified'] === true && observed['wholeUiAccepted'] === false && sceneString(observed['nonce']) &&
    observed['caseJsonSha256'] === bootstrap['caseJsonSha256'] && observed['caseSha256'] === bootstrap['caseSha256'] &&
    sceneHash(observed['normalReadbackReceiptSha256']) && sceneHash(observed['normalSourceProfileSha256']) &&
    isRecord(readback) && readback['nonce'] === observed['nonce'] && readback['caseSha256'] === observed['caseSha256'] &&
    readback['gpuReadbackCompleted'] === true && readback['failure'] === null &&
    readback['schema'] === 'dev.zircon.editor.normal-frame-readback' && readback['version'] === 1 &&
    readback['authority'] === 'normal-actual-gpu-packet-and-readback-receipt', 'bootstrap is not an observed actual normal readback');
  const scene = readback['sceneImage'];
  sceneAssert(isRecord(scene) && scene['schema'] === 'dev.zircon.editor.normal-scene-image-readback' && scene['version'] === 1 &&
    scene['authority'] === 'normal-actual-submitted-runtime-image-readback' && scene['readbackCompleted'] === true &&
    scene['failure'] === null && scene['paneKind'] === 'Scene' && sceneString(scene['surfaceKey']) && scene['atlasUv'] === null &&
    typeof scene['overlayPresent'] === 'boolean' && scene['alphaMode'] === 'opaque' && scene['opaquePixels'] === true &&
    scene['pngColorEncoding'] === 'srgb' && Array.isArray(scene['physicalSize']) && scene['physicalSize'].length === 2 &&
    scene['physicalSize'].every((value) => sceneUint(value) && value > 0 && value <= 8192) &&
    Number(scene['physicalSize'][0]) * Number(scene['physicalSize'][1]) * 4 <= 24 * 1024 * 1024 &&
    sceneHash(scene['rawSha256']) && sceneHash(scene['rgba8Sha256']) && sceneHash(scene['pngSha256']) &&
    sceneUint(scene['resourceGeneration']) && sceneString(scene['resourceKey']), 'scene image is unavailable or unsupported');
  sceneAssert(isRecord(scene['owner']) && isRecord(scene['viewportRoot']) &&
    Object.keys(scene['owner']).length === 3 && Object.keys(scene['viewportRoot']).length === 3 &&
    typeof scene['owner']['instancePath'] === 'string' && typeof scene['viewportRoot']['instancePath'] === 'string' &&
    scenePath(scene['owner']['sourcePath']) === 'zircon_editor/assets/ui/editor/unreal/components/viewport.zui' && scene['owner']['sourceNodeId'] === 'scene_media' &&
    scenePath(scene['viewportRoot']['sourcePath']) === scenePath(scene['owner']['sourcePath']) && scene['viewportRoot']['sourceNodeId'] === 'root' &&
    JSON.stringify(sceneSteps(scene['owner']['instancePath'])) === JSON.stringify(sceneSteps(scene['viewportRoot']['instancePath'])),
    'image has a different authored Scene owner');
  const receipt = scene['receipt'], conversion = scene['conversion'], reference = scene['sourceRef'];
  const size = scene['physicalSize'] as number[];
  const formats: Record<string, [string, string, string]> = {
    Rgba8UnormSrgb: ['rgba8', 'srgb', 'identity-rgba8-srgb'],
    Bgra8UnormSrgb: ['bgra8', 'srgb', 'bgra8-srgb-to-rgba8-srgb'],
    Rgba8Unorm: ['rgba8', 'linear', 'linear8-to-rgba8-srgb-iec61966-2-1'],
    Bgra8Unorm: ['bgra8', 'linear', 'linear8-to-rgba8-srgb-iec61966-2-1'],
  };
  const format = typeof scene['sourceFormat'] === 'string' && Object.hasOwn(formats, scene['sourceFormat']) ? formats[scene['sourceFormat']] : undefined;
  sceneAssert(format && scene['sourceByteLayout'] === format[0] && scene['colorEncoding'] === format[1] &&
    isRecord(conversion) && conversion['algorithm'] === format[2], 'GPU view format and conversion disagree');
  sceneAssert(isRecord(receipt) && receipt['kind'] === 'Texture' && receipt['terminal'] === 'Succeeded' && sceneUint(receipt['request']) &&
    isRecord(receipt['frame_key']) && sameWorkbenchReviewValue(receipt['frame_key']['submission'], readback['submission']) &&
    isRecord(readback['receipt']) && readback['receipt']['request'] !== receipt['request'] &&
    sceneUint(scene['rawByteLength']) && scene['rawByteLength'] === size[0]! * size[1]! * 4 && scene['rawRowBytes'] === size[0]! * 4 &&
    isRecord(conversion) && Object.keys(conversion).length === 3 && conversion['inputSha256'] === scene['rawSha256'] &&
    conversion['outputSha256'] === scene['rgba8Sha256'] &&
    isRecord(reference) && Object.keys(reference).length === 7 && reference['authority'] === 'surface-node-owner' &&
    reference['primitiveIdentity'] === false && sceneString(reference['treeId']) &&
    ['generation','nodeId','nodeCommandIndex','fragmentIndex'].every((key) => sceneUint(reference[key])) &&
    sceneUint(scene['commandIndex']) && [scene['commandBounds'],scene['commandClip']].every((bounds) => isRecord(bounds) &&
      Object.keys(bounds).length === 4 && ['x','y','width','height'].every((key) => typeof bounds[key] === 'number' && Number.isFinite(bounds[key])) &&
      Number(bounds['width']) > 0 && Number(bounds['height']) > 0), 'partial actual scene receipt or conversion');
  sceneAssert(isRecord(image) && Object.keys(image).length === 4 && image['mime'] === 'image/png' &&
    sceneUint(image['byteLength']) && image['byteLength'] > 0 && image['byteLength'] <= 8 * 1024 * 1024 &&
    image['byteLength'] === scene['pngByteLength'] && image['pngSha256'] === scene['pngSha256'] &&
    typeof image['base64'] === 'string' && image['base64'].length === 4 * Math.ceil(image['byteLength'] / 3) &&
    /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(image['base64']), 'inline PNG extent or identity differs');
  sceneAssert(reviewUtf8ByteLength(JSON.stringify(reviewCase)) <= 8 * 1024 * 1024, 'complete case exceeds 8 MiB');
  sceneSubmittedCommand(normal, scene);
  return packet as unknown as RuntimeSceneMedia;
}

/** Locate one actual invocation, never a suffix/control-id replacement or source geometry override. */
export function runtimeSceneMediaReviewTarget(document: ZuiDocument, reviewCase: LayoutReviewCaseMessage): {
  nodeId: string; image: RuntimeSceneEmbeddedImage;
} | undefined {
  const packet = runtimeSceneMediaForCase(reviewCase);
  if (!packet) return;
  const normal = reviewCase.data['normal_window_evidence'] as Record<string, unknown>;
  const scene = (packet.bootstrapPixelAssociation.observed['normalReadbackReceipt'] as Record<string, unknown>)['sceneImage'] as RuntimeSceneImage;
  const roots = Object.entries(document.nodes ?? {}).filter(([, node]) => node.control_id === 'WorkbenchWindowRoot');
  sceneAssert(roots.length === 1 && roots[0]![1].component === 'Overlay', 'missing or duplicate review Workbench root');
  const [rootId, root] = roots[0]!;
  sceneAssert(scenePath(root['penpot_review_source_path']) === 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui' &&
    root['penpot_review_source_node_id'] === 'root', 'review Workbench owner differs');
  const reviewPrefix = sceneSteps(root['penpot_review_instance_path']);
  const inventory = normal['arrangedInventoryPublication'];
  sceneAssert(isRecord(inventory) && inventory['schema'] === 'dev.zircon.editor.normal-arranged-inventory' && inventory['version'] === 1 &&
    inventory['authority'] === 'normal-retained-runtime-surfaces' && inventory['observed'] === true && typeof inventory['complete'] === 'boolean' &&
    sameWorkbenchReviewValue(inventory['presentationCursor'], normal['presentationCursor']) && inventory['presentedFrameCount'] === normal['presentedFrameCount'] &&
    Array.isArray(inventory['surfaces']) && inventory['surfaces'].every(isRecord), 'missing same-frame actual owner inventory');
  const surfaces = inventory['surfaces'].filter((surface) => surface['scope'] === 'workbench-window');
  sceneAssert(surfaces.length === 1 && surfaces[0]!['schema'] === 'dev.zircon.editor.normal-surface-arranged-inventory' && surfaces[0]!['version'] === 1 &&
    surfaces[0]!['coordinateSpace'] === 'surface-logical' && surfaces[0]!['frameAuthority'] === 'retained-host-projection' &&
    sceneUint(surfaces[0]!['sourceFrameGeneration']) && surfaces[0]!['documentId'] === 'res://ui/editor/unreal/level_editor_window.zui', 'actual Workbench surface differs');
  const surface = surfaces[0]!, rows = surface['nodes'];
  sceneAssert(Array.isArray(rows) && rows.every(isRecord), 'actual owner rows are missing');
  const nativeRoots = rows.filter((row) => row['controlId'] === 'WorkbenchWindowRoot');
  sceneAssert(nativeRoots.length === 1, 'actual Workbench root is missing or duplicated');
  const nativeRoot = nativeRoots[0]!, nativePrefix = sceneSteps(nativeRoot['instancePath']);
  sceneAssert(nativeRoot['component'] === 'Overlay' && nativeRoot['parent'] === null && nativeRoot['sourceNodeId'] === 'root' &&
    scenePath(nativeRoot['sourcePath']) === scenePath(root['penpot_review_source_path']) && sceneUint(nativeRoot['surfaceNodeId']) &&
    nativeRoot['surfaceNodeId'] > 0 && sameWorkbenchReviewValue(surface['rootIds'], [nativeRoot['surfaceNodeId']]), 'actual root identity differs');
  const ownerKey = sceneTuple({ ...scene.owner }, nativePrefix, reviewPrefix);
  const authoredRootKey = sceneTuple({ ...scene.viewportRoot }, nativePrefix, reviewPrefix);
  const viewportKey = sceneTuple({ ...scene.owner, sourceNodeId: 'viewport' }, nativePrefix, reviewPrefix);
  sceneAssert(sameWorkbenchReviewValue(sceneSteps(scene.owner.instancePath), [...nativePrefix, {
    sourcePath: scenePath(nativeRoot['sourcePath']), sourceNodeId: 'viewport',
  }]), 'receipt invocation is not the actual Workbench viewport mount');
  const targets = Object.entries(document.nodes ?? {}).filter(([, node]) => {
    const raw = sceneNodeTuple(node);
    return raw['sourceNodeId'] === scene.owner.sourceNodeId && typeof raw['sourcePath'] === 'string' &&
      scenePath(raw['sourcePath']) === scenePath(scene.owner.sourcePath) && sceneTuple(raw, reviewPrefix) === ownerKey;
  });
  const sourceNodes = (sourceNodeId: string, key: string) => Object.entries(document.nodes ?? {}).filter(([, node]) => {
    const raw = sceneNodeTuple(node);
    return raw['sourceNodeId'] === sourceNodeId && typeof raw['sourcePath'] === 'string' &&
      scenePath(raw['sourcePath']) === scenePath(scene.owner.sourcePath) && sceneTuple(raw, reviewPrefix) === key;
  });
  const authoredRoots = sourceNodes('root', authoredRootKey), viewports = sourceNodes('viewport', viewportKey);
  sceneAssert(targets.length === 1 && targets[0]![1].control_id === 'WorkbenchViewportSurface' &&
    targets[0]![1].component === 'SceneViewViewport' &&
    authoredRoots.length === 1 && authoredRoots[0]![1].component === 'VerticalGroup' && authoredRoots[0]![1].control_id === 'WorkbenchMainBandViewportPanel' &&
    viewports.length === 1 && viewports[0]![1].component === 'Overlay' && viewports[0]![1].control_id === 'UnrealSceneViewViewport',
    'source Scene owner is missing or ambiguous');
  const [nodeId] = targets[0]!, [viewportId] = viewports[0]!, [authoredRootId] = authoredRoots[0]!;
  const parents = new Map<string, string>();
  for (const [id, node] of Object.entries(document.nodes ?? {})) for (const child of node.children ?? []) {
    sceneAssert(!parents.has(child.node), 'review node has multiple parents'); parents.set(child.node, id);
  }
  sceneAssert(parents.get(nodeId) === viewportId && parents.get(viewportId) === authoredRootId,
    'review Scene root, viewport and media ancestry differs');
  const seen = new Set<string>();
  let current: string | undefined = nodeId;
  while (current !== rootId) {
    sceneAssert(current && !seen.has(current), 'review Scene is outside the actual main host'); seen.add(current);
    sceneAssert(scenePrefix(sceneSteps(document.nodes![current]!['penpot_review_instance_path']), reviewPrefix), 'review ancestry has another invocation');
    current = parents.get(current);
  }
  const nativeMatches = rows.filter((row) => row['sourceNodeId'] === scene.owner.sourceNodeId &&
    typeof row['sourcePath'] === 'string' && scenePath(row['sourcePath']) === scenePath(scene.owner.sourcePath) &&
    sceneTuple(row, nativePrefix, reviewPrefix) === ownerKey);
  const nativeSourceRows = (sourceNodeId: string, key: string) => rows.filter((row) => row['sourceNodeId'] === sourceNodeId &&
    typeof row['sourcePath'] === 'string' && scenePath(row['sourcePath']) === scenePath(scene.owner.sourcePath) &&
    sceneTuple(row, nativePrefix, reviewPrefix) === key);
  const nativeAuthoredRoots = nativeSourceRows('root', authoredRootKey), nativeViewports = nativeSourceRows('viewport', viewportKey);
  const command = sceneSubmittedCommand(normal, scene), commandSource = command['source'];
  sceneAssert(nativeMatches.length === 1 && nativeMatches[0]!['component'] === 'SceneViewViewport' && nativeMatches[0]!['controlId'] === 'WorkbenchViewportSurface' &&
    nativeAuthoredRoots.length === 1 && nativeAuthoredRoots[0]!['component'] === 'VerticalGroup' && nativeAuthoredRoots[0]!['controlId'] === 'WorkbenchMainBandViewportPanel' &&
    nativeViewports.length === 1 && nativeViewports[0]!['component'] === 'Overlay' && nativeViewports[0]!['controlId'] === 'UnrealSceneViewViewport' &&
    sceneUint(nativeMatches[0]!['surfaceNodeId']) && nativeMatches[0]!['surfaceNodeId'] > 0 &&
    isRecord(scene['sourceRef']) && scene['sourceRef']['nodeId'] === nativeMatches[0]!['surfaceNodeId'] &&
    scene['sourceRef']['treeId'] === `template.v2.${surface['documentId']}` &&
    sceneUint(scene['sourceRef']['generation']) && scene['sourceRef']['generation'] > 0 &&
    scene['sourceRef']['generation'] <= Number(surface['sourceFrameGeneration']) &&
    isRecord(commandSource), 'receipt has no unique actual submitted source node');
  const nativeViewportParent = nativeMatches[0]!['parent'];
  sceneAssert(isRecord(nativeViewportParent) &&
    commandSource['parentNodeId'] === nativeViewportParent['nodeId'] &&
    sceneTuple(nativeViewportParent, nativePrefix, reviewPrefix) === viewportKey, 'actual Scene is outside its receipt viewport root');
  const byId = new Map<string, Record<string, unknown>>();
  const surfaceIds = new Set<number>();
  for (const row of rows) {
    sceneAssert(sceneString(row['nodeId']) && !byId.has(row['nodeId']), 'duplicate actual node id');
    const actualId = row['surfaceNodeId'];
    if (actualId !== undefined && actualId !== null) {
      sceneAssert(sceneUint(actualId) && actualId > 0 && !surfaceIds.has(actualId), 'invalid or duplicate actual source id'); surfaceIds.add(actualId);
    }
    byId.set(row['nodeId'], row);
  }
  const nativeAuthoredParent = nativeViewports[0]!['parent'];
  sceneAssert(isRecord(nativeViewportParent) && sceneString(nativeViewportParent['nodeId']) && byId.get(nativeViewportParent['nodeId']) === nativeViewports[0] &&
    isRecord(nativeAuthoredParent) && sceneString(nativeAuthoredParent['nodeId']) && byId.get(nativeAuthoredParent['nodeId']) === nativeAuthoredRoots[0] &&
    sceneTuple(nativeAuthoredParent, nativePrefix, reviewPrefix) === authoredRootKey,
    'actual Scene root, viewport and media ancestry differs');
  let row = nativeMatches[0]!;
  const nativeSeen = new Set<string>();
  while (true) {
    sceneAssert(sceneString(row['nodeId']) && !nativeSeen.has(row['nodeId']) && sceneUint(row['surfaceNodeId']) && row['surfaceNodeId'] > 0, 'actual Scene owner ancestry is incomplete');
    nativeSeen.add(row['nodeId']);
    const projected = row['projected'];
    sceneAssert(isRecord(projected) && projected['surfaceNodeId'] === row['surfaceNodeId'] && projected['nodeId'] === row['nodeId'] &&
      projected['component'] === row['component'] && projected['controlId'] === row['controlId'] &&
      sceneTuple(projected, nativePrefix) === sceneTuple(row, nativePrefix), 'actual source projection differs');
    if (row === nativeRoot) { sceneAssert(projected['parentId'] === null, 'actual root is rebound'); break; }
    const parent = row['parent'];
    sceneAssert(isRecord(parent) && sceneString(parent['nodeId']) && projected['parentId'] === parent['nodeId'], 'actual Scene parent is incomplete');
    const actualParent = byId.get(parent['nodeId']);
    sceneAssert(actualParent && sceneTuple(actualParent, nativePrefix) === sceneTuple(parent, nativePrefix), 'actual Scene parent owner differs');
    row = actualParent;
  }
  return { nodeId, image: { uri: 'runtime-scene:' + packet.image.pngSha256, name: 'normal-scene-' + packet.image.pngSha256.slice(0, 16) + '.png',
    mime: 'image/png', base64: packet.image.base64,
    runtimeScene: { pngSha256: packet.image.pngSha256, byteLength: packet.image.byteLength, physicalSize: scene.physicalSize } } };
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
  const componentData = reviewDataWithoutWorkbenchState(reviewCase);
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
    !Object.keys(reviewCase.data).length &&
    !hasTreeSelectionRows(source)
  )
    return source;
  if (!Object.keys(source.nodes ?? {}).length)
    throw new Error('Theme state review requires a component consumer host');

  const document = cloneZuiDocument(source);
  const componentData = reviewDataWithoutWorkbenchState(reviewCase);
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
    const normalEvidence = reviewCase.data['normal_window_evidence'];
    deriveTreeSelectionOwnerReview(document, normalEvidence);
    deriveViewportToolbarReview(document, normalEvidence, reviewCase.dpi);
    const scene = runtimeSceneMediaReviewTarget(document, reviewCase);
    if (scene) document.nodes![scene.nodeId]!['penpot_image'] = scene.image;
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

/** The complete case is already present in the typed review host metadata. */
export function importedActualNormalReviewCase(
  document: ZuiDocument,
): LayoutReviewCaseMessage | undefined {
  const value = document['penpot_actual_normal_review_case'];
  if (value === undefined) return undefined;
  return requireActualNormalReviewCase(value);
}

function requireActualNormalReviewCase(value: unknown): LayoutReviewCaseMessage {
  if (!isRecord(value) || value['host'] !== 'editor' || !isRecord(value['data']) ||
    !isRecord(value['data']['normal_window_evidence']))
    throw new Error('Imported actual normal review requires its complete Editor case');
  const reviewCase = value as unknown as LayoutReviewCaseMessage;
  validateReviewCaseEnvelope(reviewCase);
  if (!readViewportToolbarPublication(reviewCase.data['normal_window_evidence']))
    throw new Error('Imported actual normal review lacks its toolbar publication');
  return reviewCase;
}

/** Read saved Board metadata anew; no previous document or WeakMap is required. */
export function reviewCaseFromSavedMetadata(
  document: ZuiDocument,
  encoded: string,
): LayoutReviewCaseMessage | null {
  const imported = importedActualNormalReviewCase(document);
  if (!encoded) {
    if (imported) throw new Error('Imported actual normal review lacks saved case metadata');
    return null;
  }
  const value: unknown = JSON.parse(encoded);
  if (imported) {
    const saved = requireActualNormalReviewCase(value);
    if (saved.sourcePath !== imported.sourcePath)
      throw new Error('Saved actual normal review belongs to a different Source owner');
    if (Object.hasOwn(imported.data, 'managedSceneFingerprint') &&
      ['managedSceneFingerprint', 'workbenchPresentation', 'normal_window_evidence'].some((key) =>
        !sameWorkbenchReviewValue(saved.data[key], imported.data[key])))
      throw new Error('Saved actual normal review differs from its imported managed evidence');
    return saved;
  }
  const saved = value as LayoutReviewCaseMessage;
  validateReviewCaseEnvelope(saved);
  return saved;
}
