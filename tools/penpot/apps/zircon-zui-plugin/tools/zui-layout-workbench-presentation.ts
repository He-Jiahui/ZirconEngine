import { createHash } from 'node:crypto';

const CONTENT_KINDS = new Set([
  'welcome',
  'project',
  'hierarchy',
  'inspector',
  'scene',
  'game',
  'assets',
  'console',
  'prefab_editor',
  'asset_browser',
  'ui_asset_editor',
  'ui_component_showcase',
  'animation_sequence_editor',
  'animation_graph_editor',
  'runtime_diagnostics',
  'performance_timeline',
  'module_plugins',
  'build_export',
  'generated_bottom',
  'placeholder',
]);
const DRAWER_SLOTS = new Set([
  'leftTop',
  'leftBottom',
  'rightTop',
  'rightBottom',
  'bottom',
]);
const DRAWER_MODES = new Set(['pinned', 'autoHide', 'collapsed']);
const LAYOUT_DRAWER_SLOTS = new Set([
  'LeftTop',
  'LeftBottom',
  'RightTop',
  'RightBottom',
  'Bottom',
]);
const MAX_PRESENTATION_BYTES = 256 * 1024;
const MAX_IDENTITY_BYTES = 1024;
const MAX_USER_TEXT_BYTES = 4096;
const DECIMAL_U64 = /^(?:0|[1-9]\d{0,19})$/;

type JsonObject = Record<string, unknown>;

export interface WorkbenchPresentationSnapshot extends Record<string, unknown> {
  schema: 'dev.zircon.editor.workbench-presentation';
  version: 1;
  activeLocale: 'en' | 'zh-CN';
  sourceFingerprint: { sourcePath: string; sha256: string };
  stateFingerprint: string;
}

function isRecord(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function exactFields(
  value: JsonObject,
  keys: readonly string[],
  context: string,
): void {
  const supported = new Set(keys);
  for (const key of Object.keys(value))
    if (!supported.has(key))
      throw new Error(`Unknown ${context} field: ${key}`);
  for (const key of keys)
    if (!Object.hasOwn(value, key))
      throw new Error(`Missing ${context} field: ${key}`);
}

function allowedFields(
  value: JsonObject,
  keys: readonly string[],
  context: string,
): void {
  const supported = new Set(keys);
  for (const key of Object.keys(value))
    if (!supported.has(key))
      throw new Error(`Unknown ${context} field: ${key}`);
}

function string(
  value: unknown,
  name: string,
  maxBytes = MAX_IDENTITY_BYTES,
): asserts value is string {
  if (
    typeof value !== 'string' ||
    value.trim() === '' ||
    Buffer.byteLength(value, 'utf8') > maxBytes
  )
    throw new Error(
      `${name} must be a nonempty string of at most ${maxBytes} UTF-8 bytes`,
    );
}

function boundedText(
  value: unknown,
  name: string,
  maxBytes = MAX_USER_TEXT_BYTES,
): asserts value is string {
  if (typeof value !== 'string' || Buffer.byteLength(value, 'utf8') > maxBytes)
    throw new Error(`${name} must be text of at most ${maxBytes} UTF-8 bytes`);
  if (/\p{Cc}/u.test(value))
    throw new Error(`${name} contains a control character`);
}

function nullableString(
  value: unknown,
  name: string,
  maxBytes = MAX_IDENTITY_BYTES,
): asserts value is string | null {
  if (value !== null) string(value, name, maxBytes);
}

function pathString(value: unknown, name: string): asserts value is string {
  string(value, name, MAX_IDENTITY_BYTES);
  if (
    value.startsWith('/') ||
    value.includes('\\') ||
    value.split('/').some((part) => !part || part === '.' || part === '..')
  )
    throw new Error(`${name} must be a contained repo-relative POSIX path`);
}

function idList(value: unknown, name: string, max: number): string[] {
  if (!Array.isArray(value) || value.length > max)
    throw new Error(`${name} must be an array with at most ${max} items`);
  const ids = value.map((item, index) => {
    string(item, `${name}[${index}]`);
    return item;
  });
  if (new Set(ids).size !== ids.length)
    throw new Error(`${name} contains duplicate IDs`);
  return ids;
}

function contentKind(value: unknown, name: string): asserts value is string {
  string(value, name);
  if (!CONTENT_KINDS.has(value)) throw new Error(`${name} is unsupported`);
}

export function canonicalSha256(value: unknown): string {
  const canonical = (item: unknown): unknown =>
    Array.isArray(item)
      ? item.map(canonical)
      : item !== null && typeof item === 'object'
        ? Object.fromEntries(
            Object.entries(item)
              .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
              .map(([key, child]) => [key, canonical(child)]),
          )
        : item;
  return createHash('sha256')
    .update(JSON.stringify(canonical(value)))
    .digest('hex');
}

function validateBoundedJson(value: unknown, context: string): void {
  let nodes = 0;
  const visit = (item: unknown, depth: number): void => {
    nodes += 1;
    if (nodes > 8192 || depth > 32)
      throw new Error(`${context} exceeds its structural bound`);
    if (
      item === null ||
      typeof item === 'boolean' ||
      (typeof item === 'number' && Number.isFinite(item))
    )
      return;
    if (typeof item === 'string') {
      if (Buffer.byteLength(item, 'utf8') > MAX_USER_TEXT_BYTES)
        throw new Error(`${context} contains text above 4096 UTF-8 bytes`);
      return;
    }
    if (Array.isArray(item)) {
      if (item.length > 512)
        throw new Error(`${context} contains an oversized array`);
      for (const child of item) visit(child, depth + 1);
      return;
    }
    if (!isRecord(item) || Object.keys(item).length > 256)
      throw new Error(`${context} contains a non-JSON or oversized object`);
    for (const [key, child] of Object.entries(item)) {
      if (Buffer.byteLength(key, 'utf8') > MAX_IDENTITY_BYTES)
        throw new Error(`${context} contains an oversized key`);
      visit(child, depth + 1);
    }
  };
  visit(value, 0);
}

function validateLayout(value: unknown): void {
  if (!isRecord(value))
    throw new Error('workbenchPresentation.layout must be an object');
  exactFields(
    value,
    ['active_main_page', 'main_pages', 'activity_windows', 'floating_windows'],
    'WorkbenchLayout',
  );
  string(value['active_main_page'], 'WorkbenchLayout.active_main_page');
  if (!Array.isArray(value['main_pages']) || value['main_pages'].length > 32)
    throw new Error('WorkbenchLayout.main_pages exceeds 32 entries');
  if (
    !isRecord(value['activity_windows']) ||
    Object.keys(value['activity_windows']).length > 32
  )
    throw new Error('WorkbenchLayout.activity_windows exceeds 32 entries');
  if (
    !Array.isArray(value['floating_windows']) ||
    value['floating_windows'].length > 32
  )
    throw new Error('WorkbenchLayout.floating_windows exceeds 32 entries');
  const pageIds = new Set<string>();
  for (const [index, page] of value['main_pages'].entries()) {
    if (!isRecord(page) || Object.keys(page).length !== 1)
      throw new Error(
        `WorkbenchLayout.main_pages[${index}] must have one page variant`,
      );
    const variant = Object.keys(page)[0];
    if (variant === 'WorkbenchPage') {
      const body = page[variant];
      if (!isRecord(body))
        throw new Error(`main_pages[${index}].WorkbenchPage must be an object`);
      exactFields(
        body,
        ['id', 'title', 'activity_window'],
        `main_pages[${index}].WorkbenchPage`,
      );
      string(body['id'], `main_pages[${index}].WorkbenchPage.id`);
      string(
        body['title'],
        `main_pages[${index}].WorkbenchPage.title`,
        MAX_USER_TEXT_BYTES,
      );
      string(
        body['activity_window'],
        `main_pages[${index}].WorkbenchPage.activity_window`,
      );
      if (pageIds.has(body['id']))
        throw new Error('WorkbenchLayout.main_pages has duplicate IDs');
      pageIds.add(body['id']);
    } else if (variant === 'ExclusiveActivityWindowPage') {
      const body = page[variant];
      if (!isRecord(body))
        throw new Error(
          `main_pages[${index}].ExclusiveActivityWindowPage must be an object`,
        );
      exactFields(
        body,
        ['id', 'title', 'window_instance'],
        `main_pages[${index}].ExclusiveActivityWindowPage`,
      );
      string(body['id'], `main_pages[${index}].ExclusiveActivityWindowPage.id`);
      string(
        body['title'],
        `main_pages[${index}].ExclusiveActivityWindowPage.title`,
        MAX_USER_TEXT_BYTES,
      );
      string(
        body['window_instance'],
        `main_pages[${index}].ExclusiveActivityWindowPage.window_instance`,
      );
      if (pageIds.has(body['id']))
        throw new Error('WorkbenchLayout.main_pages has duplicate IDs');
      pageIds.add(body['id']);
    } else
      throw new Error(
        `WorkbenchLayout.main_pages[${index}] has an unsupported variant`,
      );
  }
  if (!pageIds.has(value['active_main_page']))
    throw new Error(
      'WorkbenchLayout.active_main_page does not identify a page',
    );

  const treeCounts = { nodes: 0 };
  const treeIds = new Set<string>();
  const visitDocumentNode = (node: unknown, depth: number): void => {
    if (depth > 16)
      throw new Error('WorkbenchLayout document tree exceeds depth 16');
    if (!isRecord(node) || Object.keys(node).length !== 1)
      throw new Error('WorkbenchLayout document node must have one variant');
    const variant = Object.keys(node)[0];
    const body = node[variant];
    if (!isRecord(body))
      throw new Error(`WorkbenchLayout ${variant} must be an object`);
    treeCounts.nodes += 1;
    if (treeCounts.nodes > 256)
      throw new Error('WorkbenchLayout document tree exceeds 256 nodes');
    if (variant === 'SplitNode') {
      exactFields(
        body,
        ['node_id', 'axis', 'ratio', 'first', 'second'],
        'DocumentNode.SplitNode',
      );
      string(body['node_id'], 'DocumentNode.SplitNode.node_id');
      if (treeIds.has(body['node_id']))
        throw new Error('WorkbenchLayout document tree has duplicate node IDs');
      treeIds.add(body['node_id']);
      if (!['Horizontal', 'Vertical'].includes(body['axis'] as string))
        throw new Error('DocumentNode.SplitNode.axis is unsupported');
      if (typeof body['ratio'] !== 'number' || !Number.isFinite(body['ratio']))
        throw new Error('DocumentNode.SplitNode.ratio must be finite');
      visitDocumentNode(body['first'], depth + 1);
      visitDocumentNode(body['second'], depth + 1);
    } else if (variant === 'Tabs') {
      exactFields(body, ['node_id', 'tabs', 'active_tab'], 'DocumentNode.Tabs');
      string(body['node_id'], 'DocumentNode.Tabs.node_id');
      if (treeIds.has(body['node_id']))
        throw new Error('WorkbenchLayout document tree has duplicate node IDs');
      treeIds.add(body['node_id']);
      const tabs = idList(body['tabs'], 'DocumentNode.Tabs.tabs', 64);
      nullableString(body['active_tab'], 'DocumentNode.Tabs.active_tab');
      if (
        body['active_tab'] !== null &&
        !tabs.includes(body['active_tab'] as string)
      )
        throw new Error('DocumentNode.Tabs.active_tab does not identify a tab');
    } else
      throw new Error(
        `Unsupported WorkbenchLayout DocumentNode variant: ${variant}`,
      );
  };
  const validateAxisOverride = (axis: unknown, context: string): void => {
    if (!isRecord(axis)) throw new Error(`${context} must be an object`);
    allowedFields(
      axis,
      ['min', 'max', 'preferred', 'priority', 'weight', 'stretch_mode'],
      context,
    );
    for (const key of ['min', 'max', 'preferred', 'weight'] as const) {
      const item = axis[key];
      if (
        item !== undefined &&
        item !== null &&
        (typeof item !== 'number' || !Number.isFinite(item))
      )
        throw new Error(`${context}.${key} must be finite or null`);
    }
    const priority = axis['priority'];
    if (
      priority !== undefined &&
      priority !== null &&
      (!Number.isInteger(priority) ||
        (priority as number) < -2147483648 ||
        (priority as number) > 2147483647)
    )
      throw new Error(`${context}.priority must be i32 or null`);
    const stretchMode = axis['stretch_mode'];
    if (stretchMode !== undefined && stretchMode !== null)
      string(stretchMode, `${context}.stretch_mode`);
  };
  const validatePaneOverride = (pane: unknown, context: string): void => {
    if (!isRecord(pane)) throw new Error(`${context} must be an object`);
    exactFields(pane, ['width', 'height'], context);
    validateAxisOverride(pane['width'], `${context}.width`);
    validateAxisOverride(pane['height'], `${context}.height`);
  };
  const windowIds = new Set(Object.keys(value['activity_windows']));
  for (const [windowId, activityWindow] of Object.entries(
    value['activity_windows'],
  )) {
    if (!isRecord(activityWindow))
      throw new Error(`activity_windows.${windowId} must be an object`);
    exactFields(
      activityWindow,
      [
        'window_id',
        'descriptor_id',
        'host_mode',
        'activity_drawers',
        'content_workspace',
        'menu_overflow_mode',
        'region_overrides',
        'view_overrides',
      ],
      `activity_windows.${windowId}`,
    );
    string(
      activityWindow['window_id'],
      `activity_windows.${windowId}.window_id`,
    );
    string(
      activityWindow['descriptor_id'],
      `activity_windows.${windowId}.descriptor_id`,
    );
    if (
      !['EmbeddedMainFrame', 'NativeWindowHandle'].includes(
        activityWindow['host_mode'] as string,
      )
    )
      throw new Error(`activity_windows.${windowId}.host_mode is unsupported`);
    if (
      !isRecord(activityWindow['activity_drawers']) ||
      Object.keys(activityWindow['activity_drawers']).length > 5
    )
      throw new Error(
        `activity_windows.${windowId}.activity_drawers is invalid`,
      );
    for (const [slot, drawer] of Object.entries(
      activityWindow['activity_drawers'],
    )) {
      if (!LAYOUT_DRAWER_SLOTS.has(slot))
        throw new Error(
          `activity_windows.${windowId} has an unsupported drawer slot`,
        );
      if (!isRecord(drawer))
        throw new Error(
          `activity_windows.${windowId}.activity_drawers.${slot} must be an object`,
        );
      exactFields(
        drawer,
        ['slot', 'tab_stack', 'active_view', 'mode', 'extent', 'visible'],
        `activity_drawers.${slot}`,
      );
      if (drawer['slot'] !== slot)
        throw new Error(`activity_drawers.${slot}.slot differs from its key`);
      if (!isRecord(drawer['tab_stack']))
        throw new Error(`activity_drawers.${slot}.tab_stack must be an object`);
      exactFields(
        drawer['tab_stack'],
        ['tabs', 'active_tab'],
        `activity_drawers.${slot}.tab_stack`,
      );
      const drawerTabs = idList(
        drawer['tab_stack']['tabs'],
        `activity_drawers.${slot}.tab_stack.tabs`,
        64,
      );
      nullableString(
        drawer['tab_stack']['active_tab'],
        `activity_drawers.${slot}.tab_stack.active_tab`,
      );
      if (
        drawer['tab_stack']['active_tab'] !== null &&
        !drawerTabs.includes(drawer['tab_stack']['active_tab'] as string)
      )
        throw new Error(
          `activity_drawers.${slot}.active_tab does not identify a tab`,
        );
      nullableString(
        drawer['active_view'],
        `activity_drawers.${slot}.active_view`,
      );
      if (
        !['Pinned', 'AutoHide', 'Collapsed'].includes(drawer['mode'] as string)
      )
        throw new Error(`activity_drawers.${slot}.mode is unsupported`);
      if (
        typeof drawer['extent'] !== 'number' ||
        !Number.isFinite(drawer['extent']) ||
        drawer['extent'] < 0
      )
        throw new Error(
          `activity_drawers.${slot}.extent must be finite and nonnegative`,
        );
      if (typeof drawer['visible'] !== 'boolean')
        throw new Error(`activity_drawers.${slot}.visible must be boolean`);
    }
    visitDocumentNode(activityWindow['content_workspace'], 0);
    if (
      !['Auto', 'Scroll', 'MultiColumn'].includes(
        activityWindow['menu_overflow_mode'] as string,
      )
    )
      throw new Error(
        `activity_windows.${windowId}.menu_overflow_mode is unsupported`,
      );
    if (!isRecord(activityWindow['region_overrides']))
      throw new Error(
        `activity_windows.${windowId}.region_overrides must be an object`,
      );
    allowedFields(
      activityWindow['region_overrides'],
      ['Left', 'Document', 'Right', 'Bottom'],
      `activity_windows.${windowId}.region_overrides`,
    );
    for (const [region, pane] of Object.entries(
      activityWindow['region_overrides'],
    ))
      validatePaneOverride(
        pane,
        `activity_windows.${windowId}.region_overrides.${region}`,
      );
    if (!isRecord(activityWindow['view_overrides']))
      throw new Error(
        `activity_windows.${windowId}.view_overrides must be an object`,
      );
    for (const [viewId, pane] of Object.entries(
      activityWindow['view_overrides'],
    )) {
      string(viewId, `activity_windows.${windowId}.view_overrides key`);
      validatePaneOverride(
        pane,
        `activity_windows.${windowId}.view_overrides.${viewId}`,
      );
    }
  }
  for (const [index, window] of value['floating_windows'].entries()) {
    if (!isRecord(window))
      throw new Error(`floating_windows[${index}] must be an object`);
    exactFields(
      window,
      ['window_id', 'title', 'workspace', 'focused_view', 'frame'],
      `floating_windows[${index}]`,
    );
    string(window['window_id'], `floating_windows[${index}].window_id`);
    string(
      window['title'],
      `floating_windows[${index}].title`,
      MAX_USER_TEXT_BYTES,
    );
    visitDocumentNode(window['workspace'], 0);
    nullableString(
      window['focused_view'],
      `floating_windows[${index}].focused_view`,
    );
    if (!isRecord(window['frame']))
      throw new Error(`floating_windows[${index}].frame must be an object`);
    exactFields(
      window['frame'],
      ['x', 'y', 'width', 'height'],
      `floating_windows[${index}].frame`,
    );
    for (const axis of ['x', 'y', 'width', 'height'] as const)
      if (
        typeof window['frame'][axis] !== 'number' ||
        !Number.isFinite(window['frame'][axis])
      )
        throw new Error(
          `floating_windows[${index}].frame.${axis} must be finite`,
        );
  }
  for (const page of value['main_pages']) {
    const body = page[Object.keys(page)[0]] as JsonObject;
    if (
      Object.hasOwn(body, 'activity_window') &&
      !windowIds.has(body['activity_window'] as string)
    )
      throw new Error('WorkbenchPage references an unknown activity window');
  }
  validateBoundedJson(value, 'WorkbenchLayout');
}

/** Validate the actual bounded product snapshot carried by workbench cases. */
export function validateWorkbenchPresentation(value: unknown): void {
  if (!isRecord(value))
    throw new Error(
      'LayoutReviewCase.data.workbenchPresentation must be an object',
    );
  exactFields(
    value,
    [
      'schema',
      'version',
      'activeLocale',
      'sourceFingerprint',
      'stateFingerprint',
      'layout',
      'window',
      'pages',
      'documents',
      'drawers',
      'hierarchy',
      'inspector',
      'status',
    ],
    'workbenchPresentation',
  );
  if (value['schema'] !== 'dev.zircon.editor.workbench-presentation')
    throw new Error('workbenchPresentation.schema is unsupported');
  if (value['version'] !== 1)
    throw new Error('workbenchPresentation.version must be 1');
  if (value['activeLocale'] !== 'en' && value['activeLocale'] !== 'zh-CN')
    throw new Error('workbenchPresentation.activeLocale must be en or zh-CN');
  if (!isRecord(value['sourceFingerprint']))
    throw new Error(
      'workbenchPresentation.sourceFingerprint must be an object',
    );
  exactFields(
    value['sourceFingerprint'],
    ['sourcePath', 'sha256'],
    'sourceFingerprint',
  );
  pathString(
    value['sourceFingerprint']['sourcePath'],
    'sourceFingerprint.sourcePath',
  );
  if (
    typeof value['sourceFingerprint']['sha256'] !== 'string' ||
    !/^[0-9a-f]{64}$/.test(value['sourceFingerprint']['sha256'])
  )
    throw new Error('sourceFingerprint.sha256 must be lowercase SHA-256');
  if (
    typeof value['stateFingerprint'] !== 'string' ||
    !/^[0-9a-f]{64}$/.test(value['stateFingerprint'])
  )
    throw new Error(
      'workbenchPresentation.stateFingerprint must be lowercase SHA-256',
    );
  const { stateFingerprint, ...state } = value;
  if (canonicalSha256(state) !== stateFingerprint)
    throw new Error('workbenchPresentation.stateFingerprint is stale');
  validateLayout(value['layout']);

  if (!isRecord(value['window']))
    throw new Error('workbenchPresentation.window must be an object');
  exactFields(value['window'], ['id', 'title'], 'workbenchPresentation.window');
  string(value['window']['id'], 'workbenchPresentation.window.id');
  boundedText(value['window']['title'], 'workbenchPresentation.window.title');

  if (!isRecord(value['pages']))
    throw new Error('workbenchPresentation.pages must be an object');
  exactFields(
    value['pages'],
    ['activeId', 'items'],
    'workbenchPresentation.pages',
  );
  string(value['pages']['activeId'], 'workbenchPresentation.pages.activeId');
  if (
    !Array.isArray(value['pages']['items']) ||
    value['pages']['items'].length > 32
  )
    throw new Error('workbenchPresentation.pages.items exceeds 32 entries');
  const pageIds = new Set<string>();
  for (const [index, item] of value['pages']['items'].entries()) {
    if (!isRecord(item))
      throw new Error(`pages.items[${index}] must be an object`);
    exactFields(
      item,
      ['id', 'title', 'activityWindowId'],
      `pages.items[${index}]`,
    );
    string(item['id'], `pages.items[${index}].id`);
    boundedText(item['title'], `pages.items[${index}].title`);
    nullableString(
      item['activityWindowId'],
      `pages.items[${index}].activityWindowId`,
    );
    if (pageIds.has(item['id']))
      throw new Error('pages.items contains duplicate IDs');
    pageIds.add(item['id']);
  }
  if (!pageIds.has(value['pages']['activeId']))
    throw new Error('pages.activeId does not identify a page');

  if (!isRecord(value['documents']))
    throw new Error('workbenchPresentation.documents must be an object');
  exactFields(
    value['documents'],
    ['activeId', 'items'],
    'workbenchPresentation.documents',
  );
  nullableString(
    value['documents']['activeId'],
    'workbenchPresentation.documents.activeId',
  );
  if (
    !Array.isArray(value['documents']['items']) ||
    value['documents']['items'].length > 64
  )
    throw new Error('workbenchPresentation.documents.items exceeds 64 entries');
  const documentIds = new Set<string>();
  for (const [index, item] of value['documents']['items'].entries()) {
    if (!isRecord(item))
      throw new Error(`documents.items[${index}] must be an object`);
    exactFields(
      item,
      ['id', 'title', 'contentKind', 'sourcePath'],
      `documents.items[${index}]`,
    );
    string(item['id'], `documents.items[${index}].id`);
    boundedText(item['title'], `documents.items[${index}].title`);
    contentKind(item['contentKind'], `documents.items[${index}].contentKind`);
    nullablePathString(
      item['sourcePath'],
      `documents.items[${index}].sourcePath`,
    );
    if (documentIds.has(item['id']))
      throw new Error('documents.items contains duplicate IDs');
    documentIds.add(item['id']);
  }
  if (
    value['documents']['activeId'] !== null &&
    !documentIds.has(value['documents']['activeId'] as string)
  )
    throw new Error('documents.activeId does not identify a document');

  if (!Array.isArray(value['drawers']) || value['drawers'].length > 5)
    throw new Error('workbenchPresentation.drawers exceeds five entries');
  const drawerSlots = new Set<string>();
  for (const [index, drawer] of value['drawers'].entries()) {
    if (!isRecord(drawer))
      throw new Error(`drawers[${index}] must be an object`);
    exactFields(
      drawer,
      [
        'slot',
        'mode',
        'extent',
        'visible',
        'activeTabId',
        'activeViewId',
        'tabs',
      ],
      `drawers[${index}]`,
    );
    if (typeof drawer['slot'] !== 'string' || !DRAWER_SLOTS.has(drawer['slot']))
      throw new Error(`drawers[${index}].slot is unsupported`);
    if (drawerSlots.has(drawer['slot']))
      throw new Error('drawers contains duplicate slots');
    drawerSlots.add(drawer['slot']);
    if (typeof drawer['mode'] !== 'string' || !DRAWER_MODES.has(drawer['mode']))
      throw new Error(`drawers[${index}].mode is unsupported`);
    if (
      typeof drawer['extent'] !== 'number' ||
      !Number.isFinite(drawer['extent']) ||
      drawer['extent'] < 0
    )
      throw new Error(
        `drawers[${index}].extent must be finite and nonnegative`,
      );
    if (typeof drawer['visible'] !== 'boolean')
      throw new Error(`drawers[${index}].visible must be boolean`);
    nullableString(drawer['activeTabId'], `drawers[${index}].activeTabId`);
    nullableString(drawer['activeViewId'], `drawers[${index}].activeViewId`);
    if (!Array.isArray(drawer['tabs']) || drawer['tabs'].length > 64)
      throw new Error(`drawers[${index}].tabs exceeds 64 entries`);
    const tabIds = new Set<string>();
    for (const [tabIndex, tab] of drawer['tabs'].entries()) {
      if (!isRecord(tab))
        throw new Error(
          `drawers[${index}].tabs[${tabIndex}] must be an object`,
        );
      exactFields(
        tab,
        ['id', 'descriptorId', 'title', 'iconKey', 'contentKind', 'sourcePath'],
        `drawers[${index}].tabs[${tabIndex}]`,
      );
      for (const key of ['id', 'descriptorId', 'iconKey'] as const)
        string(tab[key], `drawers[${index}].tabs[${tabIndex}].${key}`);
      boundedText(tab['title'], `drawers[${index}].tabs[${tabIndex}].title`);
      contentKind(
        tab['contentKind'],
        `drawers[${index}].tabs[${tabIndex}].contentKind`,
      );
      nullablePathString(
        tab['sourcePath'],
        `drawers[${index}].tabs[${tabIndex}].sourcePath`,
      );
      if (tabIds.has(tab['id'] as string))
        throw new Error('drawer tabs contain duplicate IDs');
      tabIds.add(tab['id'] as string);
    }
    if (
      drawer['activeTabId'] !== null &&
      !tabIds.has(drawer['activeTabId'] as string)
    )
      throw new Error(`drawers[${index}].activeTabId does not identify a tab`);
  }

  if (!isRecord(value['hierarchy']))
    throw new Error('workbenchPresentation.hierarchy must be an object');
  const hierarchy = value['hierarchy'];
  exactFields(
    hierarchy,
    ['filterQuery', 'expandedIds', 'selectedIds', 'rows'],
    'workbenchPresentation.hierarchy',
  );
  boundedText(hierarchy['filterQuery'], 'hierarchy.filterQuery');
  const expandedIds = idList(
    hierarchy['expandedIds'],
    'hierarchy.expandedIds',
    256,
  );
  if (expandedIds.some((id) => !DECIMAL_U64.test(id)))
    throw new Error(
      'hierarchy.expandedIds must contain canonical u64 decimal IDs',
    );
  const selectedIds = idList(
    hierarchy['selectedIds'],
    'hierarchy.selectedIds',
    16,
  );
  if (!Array.isArray(hierarchy['rows']) || hierarchy['rows'].length > 256)
    throw new Error('hierarchy.rows exceeds 256 entries');
  const hierarchyRows = hierarchy['rows'];
  const rowIds = new Set<string>();
  const rowInfo = new Map<
    string,
    { depth: number; hasChildren: boolean; childCount: number }
  >();
  for (const [index, row] of hierarchyRows.entries()) {
    if (!isRecord(row))
      throw new Error(`hierarchy.rows[${index}] must be an object`);
    exactFields(
      row,
      [
        'id',
        'parentId',
        'name',
        'kind',
        'depth',
        'active',
        'hasChildren',
        'generation',
        'subtreeHash',
      ],
      `hierarchy.rows[${index}]`,
    );
    for (const key of ['id', 'generation', 'subtreeHash'] as const)
      if (typeof row[key] !== 'string' || !DECIMAL_U64.test(row[key]))
        throw new Error(
          `hierarchy.rows[${index}].${key} must be a canonical u64 decimal string`,
        );
    if (
      row['parentId'] !== null &&
      (typeof row['parentId'] !== 'string' ||
        !DECIMAL_U64.test(row['parentId']))
    )
      throw new Error(
        `hierarchy.rows[${index}].parentId must be null or a canonical u64 string`,
      );
    boundedText(row['name'], `hierarchy.rows[${index}].name`);
    string(row['kind'], `hierarchy.rows[${index}].kind`, 128);
    if (
      !Number.isInteger(row['depth']) ||
      (row['depth'] as number) < 0 ||
      (row['depth'] as number) > 16
    )
      throw new Error(`hierarchy.rows[${index}].depth must be in 0..=16`);
    if (
      typeof row['active'] !== 'boolean' ||
      typeof row['hasChildren'] !== 'boolean'
    )
      throw new Error(
        `hierarchy.rows[${index}] active/hasChildren must be boolean`,
      );
    const id = row['id'] as string;
    const depth = row['depth'] as number;
    const parentId = row['parentId'] as string | null;
    if (rowIds.has(id))
      throw new Error('hierarchy.rows contains duplicate IDs');
    if (parentId === null) {
      if (depth !== 0)
        throw new Error(`hierarchy.rows[${index}] root depth must be zero`);
    } else {
      const parent = rowInfo.get(parentId);
      if (!parent || depth !== parent.depth + 1)
        throw new Error(
          `hierarchy.rows[${index}] must follow its parent at the preceding depth`,
        );
      parent.childCount += 1;
      if (!parent.hasChildren)
        throw new Error(
          `hierarchy.rows[${index}] parent must report hasChildren`,
        );
    }
    rowIds.add(id);
    rowInfo.set(id, {
      depth,
      hasChildren: row['hasChildren'] as boolean,
      childCount: 0,
    });
  }
  if (
    [...rowInfo.values()].some((row) => row.hasChildren !== row.childCount > 0)
  )
    throw new Error('hierarchy.rows has inconsistent parent/hasChildren facts');
  if (selectedIds.some((id) => !rowIds.has(id)))
    throw new Error('hierarchy.selectedIds contains an unknown row');
  if (expandedIds.some((id) => !rowIds.has(id)))
    throw new Error('hierarchy.expandedIds contains an unknown row');
  if (
    expandedIds.some(
      (id) =>
        !hierarchyRows.some(
          (row) =>
            isRecord(row) && row['id'] === id && row['hasChildren'] === true,
        ),
    )
  )
    throw new Error('hierarchy.expandedIds must identify rows with children');

  if (value['inspector'] === null) {
    // The product controller has no inspector snapshot when the selection is empty.
  } else if (!isRecord(value['inspector'])) {
    throw new Error(
      'workbenchPresentation.inspector must be an object or null',
    );
  } else {
    validateInspectorSnapshot(value['inspector']);
  }

  if (!isRecord(value['status']))
    throw new Error('workbenchPresentation.status must be an object');
  exactFields(
    value['status'],
    ['primary', 'secondary', 'viewportLabel', 'projectPath'],
    'workbenchPresentation.status',
  );
  boundedText(value['status']['primary'], 'status.primary');
  if (value['status']['secondary'] !== null)
    boundedText(value['status']['secondary'], 'status.secondary');
  boundedText(value['status']['viewportLabel'], 'status.viewportLabel');
  boundedText(value['status']['projectPath'], 'status.projectPath');

  validateBoundedJson(value, 'workbenchPresentation');
  if (Buffer.byteLength(JSON.stringify(value), 'utf8') > MAX_PRESENTATION_BYTES)
    throw new Error('workbenchPresentation exceeds 256 KiB');
}

function validateInspectorSnapshot(inspector: JsonObject): void {
  exactFields(
    Object.fromEntries(
      Object.entries(inspector).filter(([key]) => key !== 'rotationDegrees'),
    ),
    [
      'entityId',
      'name',
      'parent',
      'translation',
      'scale',
      'renderLayerMask',
      'components',
    ],
    'workbenchPresentation.inspector',
  );
  if (
    Object.hasOwn(inspector, 'rotationDegrees') &&
    inspector['rotationDegrees'] !== null
  ) {
    const axes = inspector['rotationDegrees'];
    if (!Array.isArray(axes) || axes.length !== 3)
      throw new Error(
        'inspector.rotationDegrees must be null or contain three axes',
      );
    axes.forEach((axis, index) =>
      string(axis, `inspector.rotationDegrees[${index}]`),
    );
  }
  nullableString(inspector['entityId'], 'inspector.entityId');
  if (
    typeof inspector['entityId'] === 'string' &&
    !DECIMAL_U64.test(inspector['entityId'])
  )
    throw new Error(
      'inspector.entityId must be a canonical u64 decimal string',
    );
  boundedText(inspector['name'], 'inspector.name');
  boundedText(inspector['parent'], 'inspector.parent');
  for (const key of ['translation', 'scale'] as const) {
    const axes = inspector[key];
    if (!Array.isArray(axes) || axes.length !== 3)
      throw new Error(`inspector.${key} must contain three axes`);
    axes.forEach((axis, index) => string(axis, `inspector.${key}[${index}]`));
  }
  if (
    !Number.isInteger(inspector['renderLayerMask']) ||
    (inspector['renderLayerMask'] as number) < 0 ||
    (inspector['renderLayerMask'] as number) > 0xffffffff
  )
    throw new Error('inspector.renderLayerMask must be u32');
  if (
    !Array.isArray(inspector['components']) ||
    inspector['components'].length > 32
  )
    throw new Error('inspector.components exceeds 32 entries');
  let propertyCount = 0;
  for (const [index, component] of inspector['components'].entries()) {
    if (!isRecord(component))
      throw new Error(`inspector.components[${index}] must be an object`);
    exactFields(
      component,
      ['id', 'title', 'properties'],
      `inspector.components[${index}]`,
    );
    string(component['id'], `inspector.components[${index}].id`);
    boundedText(component['title'], `inspector.components[${index}].title`);
    if (
      !Array.isArray(component['properties']) ||
      component['properties'].length > 64
    )
      throw new Error(
        `inspector.components[${index}].properties exceeds 64 entries`,
      );
    const propertyIds = new Set<string>();
    for (const [propertyIndex, property] of component['properties'].entries()) {
      if (!isRecord(property))
        throw new Error(
          `inspector.components[${index}].properties[${propertyIndex}] must be an object`,
        );
      exactFields(
        property,
        ['id', 'label', 'value', 'kind', 'editable'],
        `inspector.components[${index}].properties[${propertyIndex}]`,
      );
      string(property['id'], 'inspector property id');
      boundedText(property['label'], 'inspector property label');
      string(property['kind'], 'inspector property kind', 128);
      if (typeof property['editable'] !== 'boolean')
        throw new Error('inspector property editable must be boolean');
      validateBoundedJson(property['value'], 'inspector property value');
      if (propertyIds.has(property['id'] as string))
        throw new Error('inspector component has duplicate property IDs');
      propertyIds.add(property['id'] as string);
      propertyCount += 1;
    }
  }
  if (propertyCount > 512) throw new Error('inspector exceeds 512 properties');
}

function nullablePathString(
  value: unknown,
  name: string,
): asserts value is string | null {
  if (value !== null) pathString(value, name);
}
