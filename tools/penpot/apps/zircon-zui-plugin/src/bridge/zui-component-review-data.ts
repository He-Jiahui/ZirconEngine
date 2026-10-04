import {
  zuiRootNodeIds,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
} from './zui-document';
import { nativePainterComponent } from './zui-native-painter-role';
import { isWorkbenchTable } from './zui-table-cells';

const NATIVE_PAINTER_REVIEW_STATES: Readonly<Record<string, string[]>> = {
  Dialog: ['default', 'open', 'closed', 'focused', 'focus-return'],
  ConfirmDialog: ['default', 'open', 'closed', 'focused', 'focus-return'],
  CommandPalette: [
    'default',
    'open',
    'closed',
    'focused',
    'empty',
    'focus-return',
  ],
  NotificationCenter: [
    'default',
    'open',
    'closed',
    'selected',
    'empty',
    'focus-return',
  ],
  DragOverlay: ['default', 'dragging', 'drop-allowed', 'drop-blocked'],
  WorkbenchToast: ['default', 'open'],
  AgentPlan: ['default', 'running', 'complete', 'blocked'],
  ToolCalls: ['default', 'running', 'success', 'failure'],
  AgentApproval: ['default', 'pending', 'approved', 'denied'],
  AIUsage: ['default', 'normal', 'warning', 'exceeded'],
  DataGrid: ['default', 'selected', 'empty'],
  TreeView: ['default', 'open', 'selected', 'empty'],
  AgentChat: [
    'default',
    'focused',
    'disabled',
    'empty',
    'running',
    'error',
    'long-en',
    'long-zh',
  ],
  ChatComposer: [
    'default',
    'focused',
    'disabled',
    'empty',
    'running',
    'error',
    'long-en',
    'long-zh',
  ],
};

export function componentReviewRoot(
  document: ZuiDocument,
): ZuiNode | undefined {
  if (
    document.asset.kind !== 'component' ||
    Object.keys(document.components ?? {}).length !== 1
  )
    return undefined;
  const roots = zuiRootNodeIds(document);
  return roots.length === 1 ? document.nodes?.[roots[0]] : undefined;
}

export function componentInputKey(
  node?: ZuiNode,
): 'value' | 'query' | undefined {
  if (node?.component === 'InputField') return 'value';
  if (node?.component === 'SearchField') return 'query';
  return undefined;
}

/** States that are implemented by the Runtime painter rather than DOM-like controls. */
export function nativePainterReviewStates(
  node?: ZuiNode,
): string[] | undefined {
  const component = nativePainterComponent(node);
  const states = component && NATIVE_PAINTER_REVIEW_STATES[component];
  return states ? [...states] : undefined;
}

export function validateComponentReviewData(
  data: Record<string, unknown>,
): void {
  if (!Object.keys(data).length) return;
  const input = data['componentInput'];
  if (
    Object.keys(data).length !== 1 ||
    !input ||
    typeof input !== 'object' ||
    Array.isArray(input)
  )
    throw new Error(
      'Nonempty review data requires an explicit business host adapter',
    );
  for (const [key, value] of Object.entries(input)) {
    if (
      !['value', 'query', 'validation_level'].includes(key) ||
      typeof value !== 'string'
    )
      throw new Error(`Unsupported component input property: ${key}`);
    if (key === 'validation_level' && !['normal', 'error'].includes(value))
      throw new Error('Unsupported component validation level');
  }
}

export function applyComponentReviewData(
  document: ZuiDocument,
  data: Record<string, unknown>,
): void {
  validateComponentReviewData(data);
  const input = data['componentInput'] as ZuiTable | undefined;
  if (!input) return;
  const root = componentReviewRoot(document);
  const valueKey = componentInputKey(root);
  if (
    !root ||
    !valueKey ||
    !Object.hasOwn(input, valueKey) ||
    Object.keys(input).some(
      (key) => key !== valueKey && key !== 'validation_level',
    )
  )
    throw new Error(
      'Component input requires the declared text field value property',
    );
  root.props = { ...root.props, ...input };
}

export function componentReviewStates(document?: ZuiDocument): string[] {
  const root = document && componentReviewRoot(document);
  if (!root) return ['default'];
  const nativePainterStates = nativePainterReviewStates(root);
  if (nativePainterStates) return nativePainterStates;
  const props = root.props ?? {};
  const states = ['default'];
  if (props['input_hoverable'] === true) states.push('hover');
  if (props['input_clickable'] === true) states.push('pressed');
  if (props['input_focusable'] === true) states.push('focused');
  if (props['input_interactive'] === true) states.push('disabled');
  if (
    [
      'Button',
      'IconButton',
      'Checkbox',
      'Radio',
      'Toggle',
      'Slider',
      'RangeSlider',
      'SegmentedControl',
      'Tab',
      'Tabs',
    ].includes(root.component) ||
    (isWorkbenchTable(root) && typeof props['selected'] === 'boolean')
  )
    states.push('selected');
  // A composite can expose its interaction through an accepted action slot
  // even when its root is a passive layout group. The review host mounts a
  // real button instance, so its states belong in the component case matrix.
  const acceptsButtonAction = Object.values(document?.components ?? {}).some(
    (definition) => {
      const slots = definition['slots'];
      if (!slots || typeof slots !== 'object' || Array.isArray(slots))
        return false;
      const actions = (slots as Record<string, unknown>)['actions'];
      if (
        actions === null ||
        typeof actions !== 'object' ||
        Array.isArray(actions)
      )
        return false;
      const accepts = (actions as Record<string, unknown>)['accepts'];
      return Array.isArray(accepts) && accepts.includes('WorkbenchButton');
    },
  );
  if (acceptsButtonAction)
    for (const state of ['hover', 'pressed', 'focused', 'disabled', 'selected'])
      if (!states.includes(state)) states.push(state);
  return states;
}
