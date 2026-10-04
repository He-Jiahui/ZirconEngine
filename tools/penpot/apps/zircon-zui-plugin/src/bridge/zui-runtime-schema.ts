import type { ZuiDiagnostic } from './zui-document';

const EVENT_KINDS = [
  'Click',
  'DoubleClick',
  'Hover',
  'Press',
  'Release',
  'Change',
  'Submit',
  'Toggle',
  'Focus',
  'Blur',
  'Scroll',
  'Resize',
  'DragBegin',
  'DragUpdate',
  'DragEnd',
  'Drop',
] as const;

const COMPONENT_EVENT_KINDS = [
  'ValueChanged',
  'Commit',
  'KeyboardAction',
  'KeyboardText',
  'TypeaheadExpired',
  'Focus',
  'Hover',
  'Press',
  'BeginDrag',
  'DragDelta',
  'LargeDragDelta',
  'EndDrag',
  'DropHover',
  'ActiveDragTarget',
  'OpenPopup',
  'OpenPopupAt',
  'ClosePopup',
  'SelectOption',
  'ToggleExpanded',
  'AddElement',
  'SetElement',
  'RemoveElement',
  'MoveElement',
  'AddMapEntry',
  'SetMapEntry',
  'RenameMapKey',
  'RemoveMapEntry',
  'DropReference',
  'ClearReference',
  'LocateReference',
  'OpenReference',
  'SetVisibleRange',
  'SetPage',
  'SetWorldTransform',
  'SetWorldSurface',
] as const;

const SLOT_KINDS = [
  'free',
  'container',
  'overlay',
  'linear',
  'grid',
  'flow',
  'canvas',
  'scrollable',
  'splitter',
  'scale',
] as const;

export function validateZuiRuntimeSchema(document: unknown): ZuiDiagnostic[] {
  const diagnostics: ZuiDiagnostic[] = [];
  const root = table(document);
  if (!root) return diagnostics;

  validateAsset(tableField(root, 'asset', '', diagnostics, true), diagnostics);
  validateImports(tableField(root, 'imports', '', diagnostics), diagnostics);
  tableField(root, 'tokens', '', diagnostics);
  validateRoot(tableField(root, 'root', '', diagnostics), diagnostics);
  validateNodes(tableField(root, 'nodes', '', diagnostics), diagnostics);
  validateComponents(
    tableField(root, 'components', '', diagnostics),
    diagnostics,
  );
  validateStylesheets(root['stylesheets'], diagnostics);
  return diagnostics;
}

function validateAsset(
  asset: Record<string, unknown> | undefined,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!asset) return;
  enumField(
    asset,
    'kind',
    ['view', 'component', 'style', 'theme_tokens'],
    'asset',
    diagnostics,
    true,
  );
  stringField(asset, 'id', 'asset', diagnostics, true);
  unsignedIntegerField(asset, 'version', 'asset', diagnostics, 0xffff_ffff);
  stringField(asset, 'display_name', 'asset', diagnostics);
}

function validateImports(
  imports: Record<string, unknown> | undefined,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!imports) return;
  stringArrayField(imports, 'widgets', 'imports', diagnostics);
  stringArrayField(imports, 'styles', 'imports', diagnostics);
  const resources = arrayField(imports, 'resources', 'imports', diagnostics);
  resources?.forEach((resource, index) => {
    const path = `imports.resources.${index}`;
    const item = expectTable(resource, path, diagnostics);
    if (!item) return;
    enumField(
      item,
      'kind',
      ['font', 'image', 'media', 'generic_asset'],
      path,
      diagnostics,
      true,
    );
    stringField(item, 'uri', path, diagnostics, true);
    const fallback = tableField(item, 'fallback', path, diagnostics);
    if (fallback) {
      enumField(
        fallback,
        'mode',
        ['none', 'placeholder', 'optional'],
        `${path}.fallback`,
        diagnostics,
      );
      stringField(fallback, 'uri', `${path}.fallback`, diagnostics);
    }
  });
}

function validateRoot(
  root: Record<string, unknown> | undefined,
  diagnostics: ZuiDiagnostic[],
): void {
  if (root) stringField(root, 'node', 'root', diagnostics, true);
}

function validateNodes(
  nodes: Record<string, unknown> | undefined,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!nodes) return;
  for (const [nodeId, value] of Object.entries(nodes)) {
    const path = `nodes.${nodeId}`;
    const node = expectTable(value, path, diagnostics);
    if (!node) continue;
    stringField(node, 'component', path, diagnostics, true);
    stringField(node, 'control_id', path, diagnostics);
    enumField(
      node,
      'pixel_snapping',
      ['inherit', 'disabled', 'snap_to_pixel'],
      path,
      diagnostics,
    );
    tableField(node, 'params', path, diagnostics);
    stringArrayField(node, 'classes', path, diagnostics);
    tableField(node, 'props', path, diagnostics);
    tableField(node, 'state', path, diagnostics);
    tableField(node, 'layout', path, diagnostics);
    validateRepeat(
      tableField(node, 'repeat', path, diagnostics),
      path,
      diagnostics,
    );
    validateStyleBlock(
      tableField(node, 'style', path, diagnostics),
      `${path}.style`,
      diagnostics,
    );
    tableField(node, 'slots', path, diagnostics);
    validateBindings(
      arrayField(node, 'events', path, diagnostics),
      `${path}.events`,
      diagnostics,
    );
    validateChildren(
      arrayField(node, 'children', path, diagnostics),
      `${path}.children`,
      diagnostics,
    );
  }
}

function validateRepeat(
  repeat: Record<string, unknown> | undefined,
  nodePath: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!repeat) return;
  const path = `${nodePath}.repeat`;
  enumField(repeat, 'kind', ['virtual_rows'], path, diagnostics, true);
  nonEmptyStringField(repeat, 'prototype', path, diagnostics);
  nonEmptyStringField(repeat, 'virtual_control_prefix', path, diagnostics);
  positiveIntegerField(
    repeat,
    'authored_count',
    path,
    diagnostics,
    9_223_372_036_854_775_807n,
  );
  stringField(repeat, 'node_path_namespace', path, diagnostics);
}

function validateChildren(
  children: unknown[] | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  children?.forEach((value, index) => {
    const childPath = `${path}.${index}`;
    const child = expectTable(value, childPath, diagnostics);
    if (!child) return;
    stringField(child, 'node', childPath, diagnostics, true);
    tableField(child, 'slot', childPath, diagnostics);
  });
}

function validateBindings(
  bindings: unknown[] | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  bindings?.forEach((value, index) => {
    const bindingPath = `${path}.${index}`;
    const binding = expectTable(value, bindingPath, diagnostics);
    if (!binding) return;
    stringField(binding, 'id', bindingPath, diagnostics, true);
    enumField(binding, 'event', EVENT_KINDS, bindingPath, diagnostics, true);
    enumField(
      binding,
      'mode',
      ['OneTime', 'OneWay', 'TwoWay', 'Event', 'Command'],
      bindingPath,
      diagnostics,
    );
    enumField(
      binding,
      'component_event',
      COMPONENT_EVENT_KINDS,
      bindingPath,
      diagnostics,
    );
    stringField(binding, 'route', bindingPath, diagnostics);
    validateAction(
      tableField(binding, 'action', bindingPath, diagnostics),
      `${bindingPath}.action`,
      diagnostics,
    );
    const targets = arrayField(binding, 'targets', bindingPath, diagnostics);
    targets?.forEach((targetValue, targetIndex) => {
      const targetPath = `${bindingPath}.targets.${targetIndex}`;
      const assignment = expectTable(targetValue, targetPath, diagnostics);
      if (!assignment) return;
      const target = tableField(
        assignment,
        'target',
        targetPath,
        diagnostics,
        true,
      );
      if (target) {
        enumField(
          target,
          'kind',
          ['prop', 'class', 'visibility', 'enabled', 'action_payload'],
          `${targetPath}.target`,
          diagnostics,
          true,
        );
        stringField(target, 'name', `${targetPath}.target`, diagnostics);
        validateMissingPolicy(
          tableField(
            target,
            'missing_policy',
            `${targetPath}.target`,
            diagnostics,
          ),
          `${targetPath}.target.missing_policy`,
          diagnostics,
        );
      }
      stringField(assignment, 'expression', targetPath, diagnostics, true);
    });
  });
}

function validateAction(
  action: Record<string, unknown> | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!action) return;
  stringField(action, 'route', path, diagnostics);
  stringField(action, 'action', path, diagnostics);
  tableField(action, 'payload', path, diagnostics);
  validateMissingPolicy(
    tableField(action, 'payload_missing_policy', path, diagnostics),
    `${path}.payload_missing_policy`,
    diagnostics,
  );
}

function validateMissingPolicy(
  policy: Record<string, unknown> | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!policy) return;
  enumField(
    policy,
    'policy',
    ['required', 'optional', 'default', 'fallback', 'error'],
    path,
    diagnostics,
    true,
  );
  const policyName = policy['policy'];
  if (policyName === 'default' || policyName === 'fallback') {
    if (!Object.hasOwn(policy, 'value')) {
      mismatch(`${path}.value`, 'must be present', diagnostics);
    } else {
      validateUiValue(policy['value'], `${path}.value`, diagnostics);
    }
  }
}

function validateUiValue(
  value: unknown,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (value === 'Null') return;
  const tagged = table(value);
  if (!tagged || Object.keys(tagged).length !== 1) {
    mismatch(path, 'must be a tagged Runtime UI value', diagnostics);
    return;
  }
  const [kind, payload] = Object.entries(tagged)[0];
  if (kind === 'Bool' && typeof payload === 'boolean') return;
  if (kind === 'Int' && isSignedTomlInteger(payload)) return;
  if (kind === 'Float' && typeof payload === 'number') return;
  if (
    ['String', 'Color', 'AssetRef', 'InstanceRef', 'Enum'].includes(kind) &&
    typeof payload === 'string'
  ) {
    return;
  }
  if (['Vec2', 'Vec3', 'Vec4'].includes(kind)) {
    const expected = Number(kind.slice(-1));
    if (
      Array.isArray(payload) &&
      payload.length === expected &&
      payload.every((item) => typeof item === 'number')
    ) {
      return;
    }
  }
  if (kind === 'Flags' && isStringArray(payload)) return;
  if (kind === 'Array' && Array.isArray(payload)) {
    payload.forEach((item, index) =>
      validateUiValue(item, `${path}.Array.${index}`, diagnostics),
    );
    return;
  }
  const payloadTable = table(payload);
  if (kind === 'Map' && payloadTable) {
    for (const [key, item] of Object.entries(payloadTable)) {
      validateUiValue(item, `${path}.Map.${key}`, diagnostics);
    }
    return;
  }
  mismatch(path, `contains invalid ${kind} value`, diagnostics);
}

function validateComponents(
  components: Record<string, unknown> | undefined,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!components) return;
  for (const [componentId, value] of Object.entries(components)) {
    const path = `components.${componentId}`;
    const component = expectTable(value, path, diagnostics);
    if (!component) continue;
    stringField(component, 'root', path, diagnostics, true);
    enumField(component, 'style_scope', ['open', 'closed'], path, diagnostics);
    validateComponentContract(
      tableField(component, 'contract', path, diagnostics),
      `${path}.contract`,
      diagnostics,
    );
    validateParamSchemas(
      tableField(component, 'params', path, diagnostics),
      `${path}.params`,
      diagnostics,
    );
    validateSlotSchemas(
      tableField(component, 'slots', path, diagnostics),
      `${path}.slots`,
      diagnostics,
    );
    stringArrayField(component, 'default_classes', path, diagnostics);
  }
}

function validateComponentContract(
  contract: Record<string, unknown> | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!contract) return;
  const apiVersion = optionalValue(contract, 'api_version');
  if (
    apiVersion !== undefined &&
    (typeof apiVersion !== 'string' || !validApiVersion(apiVersion))
  ) {
    mismatch(
      `${path}.api_version`,
      'must be a major.minor.patch string',
      diagnostics,
    );
  }
  const publicParts = tableField(contract, 'public_parts', path, diagnostics);
  if (publicParts) {
    for (const [partId, value] of Object.entries(publicParts)) {
      const partPath = `${path}.public_parts.${partId}`;
      const part = expectTable(value, partPath, diagnostics);
      if (!part) continue;
      stringField(part, 'node_id', partPath, diagnostics);
      stringField(part, 'control_id', partPath, diagnostics);
    }
  }
  enumField(
    contract,
    'root_class_policy',
    ['append_only', 'closed'],
    path,
    diagnostics,
  );
  const focus = tableField(contract, 'focus', path, diagnostics);
  if (focus) {
    booleanField(focus, 'root_focusable', `${path}.focus`, diagnostics);
    stringField(focus, 'initial_focus', `${path}.focus`, diagnostics);
    const targets = tableField(
      focus,
      'public_targets',
      `${path}.focus`,
      diagnostics,
    );
    if (targets)
      stringMap(targets, `${path}.focus.public_targets`, diagnostics);
  }
  const bindings = tableField(contract, 'bindings', path, diagnostics);
  const actions = bindings
    ? tableField(bindings, 'public_actions', `${path}.bindings`, diagnostics)
    : undefined;
  if (actions) {
    for (const [actionId, value] of Object.entries(actions)) {
      const actionPath = `${path}.bindings.public_actions.${actionId}`;
      const action = expectTable(value, actionPath, diagnostics);
      if (!action) continue;
      stringField(action, 'target', actionPath, diagnostics);
      stringField(action, 'payload_kind', actionPath, diagnostics);
    }
  }
}

function validateParamSchemas(
  params: Record<string, unknown> | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!params) return;
  for (const [paramId, value] of Object.entries(params)) {
    const schemaPath = `${path}.${paramId}`;
    const schema = expectTable(value, schemaPath, diagnostics);
    if (schema) stringField(schema, 'type', schemaPath, diagnostics);
  }
}

function validateSlotSchemas(
  slots: Record<string, unknown> | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!slots) return;
  for (const [slotId, value] of Object.entries(slots)) {
    const schemaPath = `${path}.${slotId}`;
    const schema = expectTable(value, schemaPath, diagnostics);
    if (!schema) continue;
    booleanField(schema, 'required', schemaPath, diagnostics);
    booleanField(schema, 'multiple', schemaPath, diagnostics);
    enumField(schema, 'kind', SLOT_KINDS, schemaPath, diagnostics);
    stringArrayField(schema, 'accepts', schemaPath, diagnostics);
  }
}

function validateStylesheets(
  value: unknown,
  diagnostics: ZuiDiagnostic[],
): void {
  if (value === undefined) return;
  if (!Array.isArray(value)) {
    mismatch('stylesheets', 'must be an array', diagnostics);
    return;
  }
  value.forEach((sheetValue, sheetIndex) => {
    const path = `stylesheets.${sheetIndex}`;
    const sheet = expectTable(sheetValue, path, diagnostics);
    if (!sheet) return;
    stringField(sheet, 'id', path, diagnostics);
    const rules = arrayField(sheet, 'rules', path, diagnostics);
    rules?.forEach((ruleValue, ruleIndex) => {
      const rulePath = `${path}.rules.${ruleIndex}`;
      const rule = expectTable(ruleValue, rulePath, diagnostics);
      if (!rule) return;
      stringField(rule, 'id', rulePath, diagnostics);
      stringField(rule, 'selector', rulePath, diagnostics, true);
      validateStyleBlock(
        tableField(rule, 'set', rulePath, diagnostics),
        `${rulePath}.set`,
        diagnostics,
      );
    });
  });
}

function validateStyleBlock(
  block: Record<string, unknown> | undefined,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  if (!block) return;
  tableField(block, 'self', path, diagnostics);
  tableField(block, 'slot', path, diagnostics);
}

function tableField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
  required = false,
): Record<string, unknown> | undefined {
  const value = optionalValue(owner, key);
  const valuePath = path ? `${path}.${key}` : key;
  if (value === undefined) {
    if (required) mismatch(valuePath, 'must be a table', diagnostics);
    return undefined;
  }
  return expectTable(value, valuePath, diagnostics);
}

function arrayField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
): unknown[] | undefined {
  const value = optionalValue(owner, key);
  if (value === undefined) return undefined;
  if (!Array.isArray(value)) {
    mismatch(`${path}.${key}`, 'must be an array', diagnostics);
    return undefined;
  }
  return value;
}

function stringField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
  required = false,
): void {
  const value = optionalValue(owner, key);
  if (value === undefined && !required) return;
  if (typeof value !== 'string') {
    mismatch(`${path}.${key}`, 'must be a string', diagnostics);
  }
}

function nonEmptyStringField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  const value = optionalValue(owner, key);
  if (typeof value !== 'string' || value.trim() === '') {
    mismatch(`${path}.${key}`, 'must be a non-empty string', diagnostics);
  }
}

function booleanField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  const value = optionalValue(owner, key);
  if (value !== undefined && typeof value !== 'boolean') {
    mismatch(`${path}.${key}`, 'must be a boolean', diagnostics);
  }
}

function stringArrayField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  const value = optionalValue(owner, key);
  if (value !== undefined && !isStringArray(value)) {
    mismatch(`${path}.${key}`, 'must be an array of strings', diagnostics);
  }
}

function enumField<T>(
  owner: Record<string, unknown>,
  key: string,
  allowed: readonly T[],
  path: string,
  diagnostics: ZuiDiagnostic[],
  required = false,
): void {
  const value = optionalValue(owner, key);
  if (value === undefined && !required) return;
  if (!allowed.some((candidate) => Object.is(candidate, value))) {
    mismatch(
      `${path}.${key}`,
      `must be one of ${allowed.map(String).join(', ')}`,
      diagnostics,
    );
  }
}

function unsignedIntegerField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
  maximum: number | bigint,
): void {
  const value = optionalValue(owner, key);
  if (value === undefined) return;
  const integer = integerAsBigInt(value);
  if (integer === null || integer < 0 || integer > BigInt(maximum)) {
    mismatch(
      `${path}.${key}`,
      'must be a non-negative runtime integer',
      diagnostics,
    );
  }
}

function positiveIntegerField(
  owner: Record<string, unknown>,
  key: string,
  path: string,
  diagnostics: ZuiDiagnostic[],
  maximum: number | bigint,
): void {
  const value = optionalValue(owner, key);
  const integer = integerAsBigInt(value);
  if (integer === null || integer <= 0 || integer > BigInt(maximum)) {
    mismatch(
      `${path}.${key}`,
      'must be a positive runtime integer',
      diagnostics,
    );
  }
}

function expectTable(
  value: unknown,
  path: string,
  diagnostics: ZuiDiagnostic[],
): Record<string, unknown> | undefined {
  const result = table(value);
  if (!result) mismatch(path, 'must be a table', diagnostics);
  return result;
}

function stringMap(
  value: Record<string, unknown>,
  path: string,
  diagnostics: ZuiDiagnostic[],
): void {
  for (const [key, item] of Object.entries(value)) {
    if (typeof item !== 'string')
      mismatch(`${path}.${key}`, 'must be a string', diagnostics);
  }
}

function mismatch(
  path: string,
  expectation: string,
  diagnostics: ZuiDiagnostic[],
): void {
  diagnostics.push({
    severity: 'error',
    code: 'runtime-schema-type-mismatch',
    message: `${path} ${expectation} for the Zircon Runtime v2 loader.`,
    path,
  });
}

function optionalValue(owner: Record<string, unknown>, key: string): unknown {
  return Object.hasOwn(owner, key) ? owner[key] : undefined;
}

function table(value: unknown): Record<string, unknown> | undefined {
  return typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    !(value instanceof Date)
    ? (value as Record<string, unknown>)
    : undefined;
}

function isStringArray(value: unknown): value is string[] {
  return (
    Array.isArray(value) && value.every((item) => typeof item === 'string')
  );
}

function integerAsBigInt(value: unknown): bigint | null {
  if (typeof value === 'bigint') return value;
  if (typeof value === 'number' && Number.isSafeInteger(value)) {
    return BigInt(value);
  }
  return null;
}

function isSignedTomlInteger(value: unknown): boolean {
  const integer = integerAsBigInt(value);
  return (
    integer !== null &&
    integer >= -9_223_372_036_854_775_808n &&
    integer <= 9_223_372_036_854_775_807n
  );
}

function validApiVersion(value: string): boolean {
  const parts = value.split('.');
  return (
    parts.length === 3 &&
    parts.every((part) => {
      if (!/^\d+$/.test(part)) return false;
      const number = BigInt(part);
      return number <= 0xffff_ffffn;
    })
  );
}
