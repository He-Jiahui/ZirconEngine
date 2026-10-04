import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import {
  cloneZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
} from '../src/bridge/zui-document';
import type { LayoutDependencies } from './zui-layout-dependencies';
import { prepareDynamicReviewState } from './zui-layout-dynamic-hosts';
import {
  applyWorkbenchPresentationProjection,
  prepareWorkbenchPresentationStructure,
} from './zui-layout-workbench-projection';

const TAB_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab.zui';
const PANEL_TITLE_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui';
const PANEL_ACTION_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui';
const NUMBER_FIELD_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_number_field.zui';
const STATUS_ITEM_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui';
const ACTIVITY_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/activity_drawer_window.zui';
const UI_LAYOUT_EDITOR_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.zui';
const ASSET_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/windows/asset_window.zui';
const EDITOR_MAIN_FRAME_SOURCE =
  'zircon_editor/assets/ui/editor/host/editor_main_frame.zui';
const WORKBENCH_SHELL_SOURCE =
  'zircon_editor/assets/ui/editor/host/workbench_shell.zui';
const WORKBENCH_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
const WORKBENCH_MAIN_BAND_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui';
const WORKBENCH_COMPONENT_DRAWER_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui';

const UI_LAYOUT_EDITOR_MOUNTS = [
  [
    'palette',
    'zircon_editor/assets/ui/editor/assets_activity.zui',
    ['toolbar_title_row', 'toolbar_search_row', 'toolbar_filter_row'],
  ],
  [
    'hierarchy',
    'zircon_editor/assets/ui/editor/host/hierarchy_body.zui',
    ['header'],
  ],
  [
    'inspector',
    'zircon_editor/assets/ui/editor/host/asset_surface_controls.zui',
    ['filter_row', 'display_row', 'navigation_row'],
  ],
  [
    'style',
    'zircon_editor/assets/ui/editor/ui_asset_editor.zui',
    ['header_asset_row', 'header_status_row', 'header_action_row'],
  ],
  [
    'diagnostics',
    'zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui',
    ['header'],
  ],
  [
    'debug',
    'zircon_editor/assets/ui/editor/host/console_body.zui',
    ['body', 'footer'],
  ],
  [
    'content',
    'zircon_editor/assets/ui/editor/asset_browser.zui',
    [
      'toolbar_title_row',
      'toolbar_search_row',
      'content_header_row',
      'content_asset_table',
    ],
  ],
] as const;

const ASSET_WINDOW_MOUNTS = [
  [
    'tree',
    'zircon_editor/assets/ui/editor/host/hierarchy_body.zui',
    ['header'],
  ],
  [
    'collections',
    'zircon_editor/assets/ui/editor/assets_activity.zui',
    ['toolbar_title_row', 'toolbar_search_row', 'toolbar_filter_row'],
  ],
  [
    'details',
    'zircon_editor/assets/ui/editor/host/asset_surface_controls.zui',
    ['filter_row', 'display_row', 'navigation_row'],
  ],
  [
    'preview',
    'zircon_editor/assets/ui/editor/ui_asset_editor.zui',
    ['header_asset_row', 'header_status_row', 'header_action_row'],
  ],
  [
    'import_log',
    'zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui',
    ['header'],
  ],
  [
    'dependency',
    'zircon_editor/assets/ui/editor/host/console_body.zui',
    ['body', 'footer'],
  ],
  [
    'browser',
    'zircon_editor/assets/ui/editor/asset_browser.zui',
    [
      'toolbar_title_row',
      'toolbar_search_row',
      'content_header_row',
      'content_asset_table',
    ],
  ],
] as const;

export function requiredComponentSlots(document: ZuiDocument): string[] {
  if (document.asset.kind !== 'component') return [];
  const required: string[] = [];
  for (const [component, definition] of Object.entries(
    document.components ?? {},
  )) {
    const slots = definition['slots'];
    if (!slots || typeof slots !== 'object' || Array.isArray(slots)) continue;
    for (const [name, contract] of Object.entries(slots)) {
      if (
        contract &&
        typeof contract === 'object' &&
        !Array.isArray(contract) &&
        contract['required'] === true
      )
        required.push(`${component}.${name}`);
    }
  }
  return required;
}

export async function prepareComponentReviewHost(
  repoRoot: string,
  sourcePath: string,
  component: ZuiDocument,
  dependencies: LayoutDependencies,
  themeSourcePath?: string,
  workbenchPresentation?: Record<string, unknown>,
): Promise<{
  source: string;
  projection: ZuiDocument;
  consumers: Array<{ sourcePath: string; component: string }>;
} | null> {
  const editorMainFrameHost = await prepareEditorMainFrameReviewHost(
    repoRoot,
    sourcePath,
    component,
    dependencies,
    themeSourcePath,
    workbenchPresentation,
  );
  if (editorMainFrameHost) return editorMainFrameHost;

  const activityHost = await prepareActivityDrawerReviewHost(
    repoRoot,
    sourcePath,
    component,
    dependencies,
    themeSourcePath,
  );
  if (activityHost) return activityHost;

  const tableRowDefinition = component.components?.['WorkbenchTableRow'];
  const tableRowRoot = tableRowDefinition
    ? component.nodes?.[tableRowDefinition.root]
    : undefined;
  if (tableRowDefinition && tableRowRoot?.component === 'Table') {
    const host: ZuiDocument = {
      asset: {
        kind: 'view',
        version: 2,
        id: `${component.asset.id}#review-host`,
        display_name: component.asset.display_name,
      },
      imports: { widgets: [component.asset.id], styles: [] },
      root: { node: 'review_scroll' },
      nodes: {
        review_scroll: {
          component: 'ScrollableBox',
          control_id: 'WorkbenchTableRowReviewScroll',
          props: { input_hoverable: true },
          layout: {
            clip: true,
            container: {
              kind: 'ScrollableBox',
              axis: 'Horizontal',
              gap: 0,
              scrollbar_visibility: 'Auto',
            },
            input_policy: 'Receive',
            width: { stretch: 'Stretch' },
            height: { stretch: 'Stretch' },
          },
          children: [{ node: 'review_component' }],
        },
        review_component: {
          component: `${component.asset.id}#WorkbenchTableRow`,
          layout: {
            width: { min: 360, preferred: 360, max: 360, stretch: 'Fixed' },
            height: { stretch: 'Stretch' },
          },
        },
      },
    };
    const source = serializeZuiDocument(host);
    const projection = cloneZuiDocument(host);
    dependencies.embed(projection, sourcePath, themeSourcePath);
    return {
      source,
      projection,
      consumers: [{ sourcePath, component: 'WorkbenchTableRow' }],
    };
  }

  const panelHeader = component.components?.['WorkbenchPanelHeader'];
  if (panelHeader) {
    const root = component.nodes?.[panelHeader.root];
    const slots = panelHeader['slots'] as ZuiTable | undefined;
    const title = slots?.['title'] as ZuiTable | undefined;
    const actions = slots?.['actions'] as ZuiTable | undefined;
    const titleSlot = component.nodes?.['title_slot'];
    const actionsSlot = component.nodes?.['actions_slot'];
    if (
      component.asset.kind !== 'component' ||
      root?.component !== 'HorizontalGroup' ||
      title?.['required'] !== true ||
      title['multiple'] === true ||
      !Array.isArray(title['accepts']) ||
      !title['accepts'].includes('WorkbenchSectionTitle') ||
      actions?.['multiple'] !== true ||
      !Array.isArray(actions['accepts']) ||
      !actions['accepts'].includes('WorkbenchButton') ||
      titleSlot?.component !== 'Slot' ||
      titleSlot.props?.['name'] !== 'title' ||
      actionsSlot?.component !== 'Slot' ||
      actionsSlot.props?.['name'] !== 'actions'
    )
      throw new Error(
        'WorkbenchPanelHeader review host does not match its slot contract',
      );
    const [titleComponent, actionComponent] = await Promise.all(
      [PANEL_TITLE_SOURCE, PANEL_ACTION_SOURCE].map(
        async (path) =>
          parseZuiDocument(await readFile(resolve(repoRoot, path), 'utf8'))
            .document,
      ),
    );
    if (
      !titleComponent.components?.['WorkbenchSectionTitle'] ||
      !actionComponent.components?.['WorkbenchButton']
    )
      throw new Error(
        'WorkbenchPanelHeader review specimen is missing a component export',
      );
    const denseHeight = '$editor.control.height.dense';
    const actionWidth = '$editor.density.toolbar_action_width';
    const host: ZuiDocument = {
      asset: {
        kind: 'view',
        version: 2,
        id: `${component.asset.id}#review-host`,
        display_name: component.asset.display_name,
      },
      imports: {
        widgets: [
          component.asset.id,
          titleComponent.asset.id,
          actionComponent.asset.id,
        ],
        styles: [],
      },
      root: { node: 'review_component' },
      nodes: {
        review_component: {
          component: `${component.asset.id}#WorkbenchPanelHeader`,
          children: [
            { node: 'review_title', slot: { name: 'title' } },
            { node: 'review_action', slot: { name: 'actions' } },
          ],
        },
        review_title: {
          component: `${titleComponent.asset.id}#WorkbenchSectionTitle`,
          props: { text: 'Inspector' },
        },
        review_action: {
          component: `${actionComponent.asset.id}#WorkbenchButton`,
          props: { text: 'More' },
          layout: {
            width: {
              min: actionWidth,
              preferred: actionWidth,
              max: actionWidth,
              stretch: 'Fixed',
            },
            height: {
              min: denseHeight,
              preferred: denseHeight,
              max: denseHeight,
              stretch: 'Fixed',
            },
          },
        },
      },
    };
    const source = serializeZuiDocument(host);
    const projection = cloneZuiDocument(host);
    dependencies.embed(projection, sourcePath, themeSourcePath);
    return {
      source,
      projection,
      consumers: [
        { sourcePath, component: 'WorkbenchPanelHeader' },
        { sourcePath: PANEL_TITLE_SOURCE, component: 'WorkbenchSectionTitle' },
        { sourcePath: PANEL_ACTION_SOURCE, component: 'WorkbenchButton' },
      ],
    };
  }

  const diagnosticDefinition = component.components?.['WorkbenchDiagnosticRow'];
  if (diagnosticDefinition) {
    const root = component.nodes?.[diagnosticDefinition.root];
    const slots = diagnosticDefinition['slots'] as ZuiTable | undefined;
    const severity = slots?.['severity'] as ZuiTable | undefined;
    const message = slots?.['message'] as ZuiTable | undefined;
    if (
      component.asset.kind !== 'component' ||
      root?.component !== 'HorizontalGroup' ||
      !severity ||
      !message ||
      severity['required'] !== true ||
      message['required'] !== true ||
      severity['multiple'] === true ||
      message['multiple'] === true ||
      !Array.isArray(severity['accepts']) ||
      !Array.isArray(message['accepts']) ||
      !severity['accepts'].includes('WorkbenchStatusItem') ||
      !message['accepts'].includes('WorkbenchStatusItem')
    )
      throw new Error(
        'WorkbenchDiagnosticRow review host does not match its slot contract',
      );
    const statusItem = parseZuiDocument(
      await readFile(resolve(repoRoot, STATUS_ITEM_SOURCE), 'utf8'),
    ).document;
    if (!statusItem.components?.['WorkbenchStatusItem'])
      throw new Error(
        'WorkbenchStatusItem review specimen is missing its component export',
      );
    const host: ZuiDocument = {
      asset: {
        kind: 'view',
        version: 2,
        id: `${component.asset.id}#review-host`,
        display_name: component.asset.display_name,
      },
      imports: {
        widgets: [component.asset.id, statusItem.asset.id],
        styles: [],
      },
      root: { node: 'review_component' },
      nodes: {
        review_component: {
          component: `${component.asset.id}#WorkbenchDiagnosticRow`,
          children: [
            { node: 'review_severity', slot: { name: 'severity' } },
            { node: 'review_message', slot: { name: 'message' } },
          ],
        },
        review_severity: {
          component: `${statusItem.asset.id}#WorkbenchStatusItem`,
          props: {
            text: '[Warning]',
            component_variant: 'diagnostic_signal',
            validation_level: 'warning',
            text_tone: 'warning',
          },
        },
        review_message: {
          component: `${statusItem.asset.id}#WorkbenchStatusItem`,
          props: {
            text: 'Asset validation requires attention.',
            text_tone: 'subtle',
          },
        },
      },
    };
    const source = serializeZuiDocument(host);
    const projection = cloneZuiDocument(host);
    dependencies.embed(projection, sourcePath, themeSourcePath);
    return {
      source,
      projection,
      consumers: [
        { sourcePath, component: 'WorkbenchDiagnosticRow' },
        { sourcePath: STATUS_ITEM_SOURCE, component: 'WorkbenchStatusItem' },
      ],
    };
  }
  const propertyEditorDefinition =
    component.components?.['WorkbenchPropertyEditorRow'];
  if (propertyEditorDefinition) {
    const root = component.nodes?.[propertyEditorDefinition.root];
    const slot = (propertyEditorDefinition['slots'] as ZuiTable | undefined)?.[
      'value'
    ] as ZuiTable | undefined;
    if (
      component.asset.kind !== 'component' ||
      root?.component !== 'PropertyRow' ||
      slot?.['required'] !== true ||
      slot?.['multiple'] === true ||
      !Array.isArray(slot?.['accepts']) ||
      !slot['accepts'].includes('WorkbenchNumberField')
    )
      throw new Error(
        'WorkbenchPropertyEditorRow review host does not match its slot contract',
      );
    const numberField = parseZuiDocument(
      await readFile(resolve(repoRoot, NUMBER_FIELD_SOURCE), 'utf8'),
    ).document;
    if (!numberField.components?.['WorkbenchNumberField'])
      throw new Error(
        'WorkbenchNumberField review specimen is missing its component export',
      );
    const host: ZuiDocument = {
      asset: {
        kind: 'view',
        version: 2,
        id: `${component.asset.id}#review-host`,
        display_name: component.asset.display_name,
      },
      imports: {
        widgets: [component.asset.id, numberField.asset.id],
        styles: [],
      },
      root: { node: 'review_component' },
      nodes: {
        review_component: {
          component: `${component.asset.id}#WorkbenchPropertyEditorRow`,
          children: [{ node: 'review_value', slot: { name: 'value' } }],
        },
        review_value: {
          component: `${numberField.asset.id}#WorkbenchNumberField`,
        },
      },
    };
    const source = serializeZuiDocument(host);
    const projection = cloneZuiDocument(host);
    dependencies.embed(projection, sourcePath, themeSourcePath);
    return {
      source,
      projection,
      consumers: [
        { sourcePath, component: 'WorkbenchPropertyEditorRow' },
        { sourcePath: NUMBER_FIELD_SOURCE, component: 'WorkbenchNumberField' },
      ],
    };
  }
  const definition = component.components?.['WorkbenchTabStrip'];
  if (!definition) {
    const required = requiredComponentSlots(component);
    if (required.length)
      throw new Error(
        `${component.asset.id} requires an explicit review host for ${required.join(', ')}`,
      );
    return null;
  }
  const root = component.nodes?.[definition.root];
  const slot = (definition['slots'] as ZuiTable | undefined)?.['default'] as
    ZuiTable | undefined;
  const options = root?.props?.['options'];
  if (
    component.asset.kind !== 'component' ||
    root?.component !== 'Tabs' ||
    slot?.['multiple'] !== true ||
    !Array.isArray(slot['accepts']) ||
    !slot['accepts'].includes('WorkbenchTab') ||
    !Array.isArray(options) ||
    !options.length ||
    !options.every((item) => typeof item === 'string')
  )
    throw new Error(
      'WorkbenchTabStrip review host does not match its slot contract',
    );
  const tab = parseZuiDocument(
    await readFile(resolve(repoRoot, TAB_SOURCE), 'utf8'),
  ).document;
  if (!tab.components?.['WorkbenchTab'])
    throw new Error(
      'WorkbenchTab review specimen is missing its component export',
    );
  const selected = root.props?.['selected_index'] ?? 0;
  const nodes: Record<string, ZuiNode> = {
    review_component: {
      component: `${component.asset.id}#WorkbenchTabStrip`,
      children: options.map((_, index) => ({
        node: `review_tab_${index}`,
        slot: { name: 'default' },
      })),
    },
  };
  for (const [index, text] of options.entries()) {
    nodes[`review_tab_${index}`] = {
      component: `${tab.asset.id}#WorkbenchTab`,
      props: { text, selected: index === selected },
    };
  }
  const host: ZuiDocument = {
    asset: {
      kind: 'view',
      version: 2,
      id: `${component.asset.id}#review-host`,
      display_name: component.asset.display_name,
    },
    imports: { widgets: [component.asset.id, tab.asset.id], styles: [] },
    root: { node: 'review_component' },
    nodes,
  };
  const source = serializeZuiDocument(host);
  const projection = cloneZuiDocument(host);
  dependencies.embed(projection, sourcePath, themeSourcePath);
  return {
    source,
    projection,
    consumers: [
      { sourcePath, component: 'WorkbenchTabStrip' },
      { sourcePath: TAB_SOURCE, component: 'WorkbenchTab' },
    ],
  };
}

/**
 * EditorMainFrame is intentionally a minimal retained-runtime Slot host.  Its
 * real task bar, window tabs, and active window arrive from Rust state, which
 * Penpot cannot instantiate.  Build a design-only, reversible consumer from
 * the same authored Workbench Shell and Workbench Window assets instead of
 * putting example business content in the runtime source.
 */
async function prepareEditorMainFrameReviewHost(
  repoRoot: string,
  sourcePath: string,
  sourceDocument: ZuiDocument,
  dependencies: LayoutDependencies,
  themeSourcePath?: string,
  workbenchPresentation?: Record<string, unknown>,
): Promise<{
  source: string;
  projection: ZuiDocument;
  consumers: Array<{ sourcePath: string; component: string }>;
} | null> {
  const workbenchWindowInput = sourcePath.endsWith(WORKBENCH_WINDOW_SOURCE);
  if (!workbenchWindowInput && !sourcePath.endsWith(EDITOR_MAIN_FRAME_SOURCE))
    return null;

  const frameSourcePath = EDITOR_MAIN_FRAME_SOURCE;
  const frameDocument = workbenchWindowInput
    ? parseZuiDocument(
        await readFile(resolve(repoRoot, EDITOR_MAIN_FRAME_SOURCE), 'utf8'),
      ).document
    : sourceDocument;

  const sourceNodes = frameDocument.nodes ?? {};
  const frameRootId = frameDocument.root?.node;
  const root = frameRootId ? sourceNodes[frameRootId] : undefined;
  const taskBar = sourceNodes['task_bar_slot'];
  const windowTabs = sourceNodes['window_tab_strip_slot'];
  const activeWindow = sourceNodes['active_window_host_slot'];
  if (
    frameDocument.asset.kind !== 'view' ||
    root?.component !== 'VerticalGroup' ||
    taskBar?.component !== 'Slot' ||
    taskBar.props?.['name'] !== 'task_bar' ||
    windowTabs?.component !== 'Slot' ||
    windowTabs.props?.['name'] !== 'window_tab_strip' ||
    activeWindow?.component !== 'Slot' ||
    activeWindow.props?.['name'] !== 'active_window_host'
  )
    throw new Error(
      'EditorMainFrame review host does not match its Slot contract',
    );

  const [shell, authoredWindow, tab] = await Promise.all(
    [WORKBENCH_SHELL_SOURCE, WORKBENCH_WINDOW_SOURCE, TAB_SOURCE].map(
      async (path) =>
        parseZuiDocument(await readFile(resolve(repoRoot, path), 'utf8'))
          .document,
    ),
  );
  if (
    shell.asset.kind !== 'view' ||
    !shell.root?.node ||
    !shell.nodes?.[shell.root.node] ||
    authoredWindow.asset.kind !== 'view' ||
    !authoredWindow.nodes?.['window_content'] ||
    !tab.components?.['WorkbenchTab']
  )
    throw new Error(
      'EditorMainFrame review host is missing an authored Workbench consumer',
    );

  const host = cloneZuiDocument(frameDocument);
  host.asset = {
    ...host.asset,
    id: `${sourceDocument.asset.id}#review-host`,
    display_name: sourceDocument.asset.display_name,
  };
  host.imports = {
    widgets: uniqueStrings([
      ...strings(host.imports?.['widgets']),
      ...strings(shell.imports?.['widgets']),
      ...strings(authoredWindow.imports?.['widgets']),
      `${tab.asset.id}#WorkbenchTab`,
    ]),
    styles: uniqueStrings([
      ...strings(host.imports?.['styles']),
      ...strings(shell.imports?.['styles']),
      ...strings(authoredWindow.imports?.['styles']),
      ...strings(tab.imports?.['styles']),
    ]),
  };
  host.tokens = {
    ...(host.tokens ?? {}),
    ...(shell.tokens ?? {}),
    ...(authoredWindow.tokens ?? {}),
    ...(tab.tokens ?? {}),
  };
  host.stylesheets = mergeStylesheets(
    mergeStylesheets(host.stylesheets, shell.stylesheets),
    mergeStylesheets(authoredWindow.stylesheets, tab.stylesheets),
  );

  const hostNodes = host.nodes ?? {};
  const taskBarHost = hostNodes['task_bar_slot'];
  const windowTabHost = hostNodes['window_tab_strip_slot'];
  const activeWindowHost = hostNodes['active_window_host_slot'];
  if (!taskBarHost || !windowTabHost || !activeWindowHost)
    throw new Error('EditorMainFrame review host lost a declared Slot');

  // EditorMainFrame's taskbar is host-owned state. The workbench menu belongs
  // to the complete WorkbenchShell mounted below, so leave this empty when the
  // product snapshot does not provide a taskbar consumer.
  taskBarHost.component = 'Container';
  taskBarHost.children = [];

  // The frame tab is the active host window from the product snapshot. Page
  // and document tabs are projected inside the Shell from their own DTO rows.
  windowTabHost.component = 'HorizontalGroup';
  windowTabHost.layout = {
    ...(windowTabHost.layout ?? {}),
    container: { kind: 'HorizontalBox', gap: '$editor.density.gap.xsmall' },
  };
  const presentationWindow = workbenchPresentation
    ? record(workbenchPresentation['window'], 'workbenchPresentation.window')
    : undefined;
  windowTabHost.children = presentationWindow
    ? [{ node: 'review_main_frame_tab_active_window' }]
    : [];
  if (presentationWindow)
    hostNodes['review_main_frame_tab_active_window'] = {
      component: `${tab.asset.id}#WorkbenchTab`,
      props: {
        text: stringValue(presentationWindow['title'], 'window.title'),
        selected: true,
        checked: true,
        window_id: stringValue(presentationWindow['id'], 'window.id'),
      },
    };

  // Mount the complete WorkbenchShell. The Window is then mounted at the
  // Shell's WorkbenchBody frame; the Shell keeps its rails, document host,
  // page/status chrome, and root drawer overlays beneath that layer.
  activeWindowHost.component = 'Container';
  activeWindowHost.children = [];
  const mountedWindow = workbenchWindowInput ? sourceDocument : authoredWindow;
  if (
    mountedWindow.asset.kind !== 'view' ||
    !mountedWindow.root?.node ||
    !mountedWindow.nodes?.[mountedWindow.root.node]
  )
    throw new Error('WorkbenchWindow review host input is not a rooted view');
  mountViewIntoContainer(host, activeWindowHost, shell, 'review_workbench_shell');
  const shellBody = hostNodes['review_workbench_shell__body'];
  if (!shellBody || shellBody.component !== 'HorizontalGroup')
    throw new Error('WorkbenchShell review host is missing its WorkbenchBody');
  const shellBodyLayout = { ...(shellBody.layout ?? {}) };
  const shellBodyChildren = [...(shellBody.children ?? [])];
  const bodyContentId = 'review_workbench_shell_body_content';
  if (hostNodes[bodyContentId])
    throw new Error(`EditorMainFrame review host node id collision: ${bodyContentId}`);
  hostNodes[bodyContentId] = {
    ...shellBody,
    control_id: undefined,
    children: shellBodyChildren,
    layout: shellBodyLayout,
    penpot_review_generated_host_layer: 'workbench-shell-body-content',
  };
  shellBody.component = 'Overlay';
  shellBody.layout = {
    ...shellBodyLayout,
    container: { kind: 'Overlay', gap: 0.0 },
  };
  shellBody.children = [{ node: bodyContentId }];
  mountViewIntoContainer(
    host,
    shellBody,
    mountedWindow,
    'review_workbench_window',
  );
  const snapshotPaneMounts = workbenchPresentation
    ? await mountWorkbenchPresentationPaneViews(
        repoRoot,
        host,
        workbenchPresentation,
      )
    : [];

  const source = serializeZuiDocument(host);
  const projection = cloneZuiDocument(host);
  annotateMountedSourceIdentity(
    projection,
    '',
    frameSourcePath,
    Object.keys(frameDocument.nodes ?? {}),
    '[]',
  );
  annotateMountedSourceIdentity(
    projection,
    'review_workbench_shell',
    WORKBENCH_SHELL_SOURCE,
    Object.keys(shell.nodes ?? {}),
    instancePathForCallsite(frameSourcePath, 'active_window_host_slot'),
  );
  annotateMountedSourceIdentity(
    projection,
    'review_workbench_window',
    WORKBENCH_WINDOW_SOURCE,
    Object.keys(mountedWindow.nodes ?? {}),
    instancePathForCallsite(
      WORKBENCH_SHELL_SOURCE,
      'body',
      instancePathForCallsite(frameSourcePath, 'active_window_host_slot'),
    ),
  );
  for (const mount of snapshotPaneMounts)
    annotateMountedSourceIdentity(
      projection,
      mount.prefix,
      mount.sourcePath,
      mount.sourceNodeIds,
      '[]',
    );
  if (workbenchPresentation)
    prepareWorkbenchPresentationStructure(projection, workbenchPresentation);
  dependencies.embed(projection, frameSourcePath, themeSourcePath, {
    preserveUnselectedBranchDefaults: true,
  });
  // The mounted active window contains the same retained dynamic branches as
  // its standalone source. Keep the default Scene branch, but leave the
  // component drawer's inactive authored branches untouched when the product
  // snapshot does not identify an active drawer view for them.
  const dynamicReviewContext = {
    preserveUnselectedBranchDefaults: true,
  };
  prepareDynamicReviewState(
    projection,
    WORKBENCH_MAIN_BAND_SOURCE,
    dynamicReviewContext,
  );
  prepareDynamicReviewState(
    projection,
    WORKBENCH_COMPONENT_DRAWER_SOURCE,
    dynamicReviewContext,
  );
  if (workbenchPresentation)
    applyWorkbenchPresentationProjection(projection, workbenchPresentation);
  return {
    source,
    projection,
    consumers: [
      { sourcePath: frameSourcePath, component: 'view' },
      { sourcePath, component: 'view' },
      { sourcePath: WORKBENCH_SHELL_SOURCE, component: 'view' },
      { sourcePath: WORKBENCH_WINDOW_SOURCE, component: 'view' },
      { sourcePath: TAB_SOURCE, component: 'WorkbenchTab' },
      {
        sourcePath: WORKBENCH_MAIN_BAND_SOURCE,
        component: 'WorkbenchMainBand',
      },
      {
        sourcePath: WORKBENCH_COMPONENT_DRAWER_SOURCE,
        component: 'WorkbenchComponentDrawer',
      },
      ...snapshotPaneMounts.map(({ sourcePath }) => ({
        sourcePath,
        component: 'view',
      })),
    ],
  };
}

interface MountedWorkbenchSnapshotPane {
  sourcePath: string;
  prefix: string;
  sourceNodeIds: string[];
}

/** Mount only the active source-authored pane bodies named by product state. */
async function mountWorkbenchPresentationPaneViews(
  repoRoot: string,
  host: ZuiDocument,
  presentation: Record<string, unknown>,
): Promise<MountedWorkbenchSnapshotPane[]> {
  const mounts: MountedWorkbenchSnapshotPane[] = [];
  const mountedSourcePaths = new Set<string>();
  const mount = async (
    targetNodeId: string,
    sourcePath: string | undefined,
    mountPrefix: string,
    context: string,
  ): Promise<void> => {
    if (!sourcePath) return;
    if (!sourcePath.startsWith('zircon_editor/assets/'))
      throw new Error(`${context} sourcePath is outside zircon_editor/assets`);
    if (mountedSourcePaths.has(sourcePath))
      throw new Error(
        `Workbench snapshot repeats source view ${sourcePath}; its authored pane identity is ambiguous`,
      );
    mountedSourcePaths.add(sourcePath);
    const view = parseZuiDocument(
      await readFile(resolve(repoRoot, sourcePath), 'utf8'),
    ).document;
    if (view.asset.kind !== 'view' || !view.root?.node)
      throw new Error(`${context} source is not a rooted ZUI view: ${sourcePath}`);
    const target = host.nodes?.[targetNodeId];
    if (!target)
      throw new Error(`${context} target node is missing: ${targetNodeId}`);
    mountViewIntoContainer(host, target, view, mountPrefix);
    const nodeIds = Object.keys(view.nodes ?? {});
    mounts.push({ sourcePath, prefix: mountPrefix, sourceNodeIds: nodeIds });
    host.imports = {
      widgets: uniqueStrings([
        ...strings(host.imports?.['widgets']),
        ...strings(view.imports?.['widgets']),
      ]),
      styles: uniqueStrings([
        ...strings(host.imports?.['styles']),
        ...strings(view.imports?.['styles']),
      ]),
    };
    host.tokens = { ...(host.tokens ?? {}), ...(view.tokens ?? {}) };
    host.stylesheets = mergeStylesheets(host.stylesheets, view.stylesheets);
  };

  const documents = record(
    presentation['documents'],
    'workbenchPresentation.documents',
  );
  const activeDocumentId = nullableString(
    documents['activeId'],
    'workbenchPresentation.documents.activeId',
  );
  const documentItems = unknownArray(
    documents['items'],
    'workbenchPresentation.documents.items',
  ).map((item) => record(item, 'workbench document'));
  if (activeDocumentId !== null) {
    const activeDocument = documentItems.find(
      (item) => item['id'] === activeDocumentId,
    );
    if (!activeDocument)
      throw new Error(
        `Workbench snapshot active document ${activeDocumentId} has no tab`,
      );
    await mount(
      'review_workbench_shell__pane_surface',
      nullablePath(activeDocument['sourcePath'], 'active document sourcePath'),
      'review_workbench_document_content',
      `Workbench document ${activeDocumentId}`,
    );
  }

  const drawerEntries = unknownArray(
    presentation['drawers'],
    'workbenchPresentation.drawers',
  ).map((item) => record(item, 'workbench drawer'));
  const drawerSides: ReadonlyArray<readonly [string, readonly string[]]> = [
    ['left', ['leftTop', 'leftBottom']],
    ['right', ['rightTop', 'rightBottom']],
    ['bottom', ['bottom']],
  ];
  for (const [side, slots] of drawerSides) {
    const slotSet = new Set(slots);
    const entries = drawerEntries.filter((entry) =>
      slotSet.has(String(entry['slot'])),
    );
    const activeEntries = entries.filter(
      (entry) => entry['visible'] === true && entry['mode'] !== 'collapsed',
    );
    if (activeEntries.length > 1)
      throw new Error(
        `Workbench snapshot has multiple visible ${side} drawers; their pane geometry is not representable by the authored Shell`,
      );
    const drawer = activeEntries[0];
    if (!drawer) continue;
    const activeTabId = nullableString(
      drawer['activeTabId'],
      `workbench ${side} drawer activeTabId`,
    );
    if (activeTabId === null) continue;
    const tabs = unknownArray(drawer['tabs'], `workbench ${side} drawer tabs`)
      .map((item) => record(item, `workbench ${side} drawer tab`));
    const activeTab = tabs.find((item) => item['id'] === activeTabId);
    if (!activeTab)
      throw new Error(
        `Workbench snapshot active ${side} drawer tab ${activeTabId} has no tab projection`,
      );
    const slot = stringValue(drawer['slot'], `workbench ${side} drawer slot`);
    await mount(
      `review_workbench_shell__${side}_drawer_content`,
      nullablePath(activeTab['sourcePath'], `${slot} active sourcePath`),
      `review_workbench_${slot}_content`,
      `Workbench drawer ${slot}/${activeTabId}`,
    );
  }
  return mounts;
}

/** Keep authored node identity on the design-only host projection for case selectors. */
function annotateMountedSourceIdentity(
  projection: ZuiDocument,
  prefix: string,
  sourcePath: string,
  sourceNodeIds: readonly string[],
  instancePath: string,
): void {
  for (const sourceNodeId of sourceNodeIds) {
    const mountedNodeId = prefix
      ? `${prefix}__${sourceNodeId}`
      : sourceNodeId;
    const node = projection.nodes?.[mountedNodeId];
    if (!node) continue;
    const localSourcePath =
      typeof node['penpot_review_source_path'] === 'string' &&
      node['penpot_review_source_path']
        ? node['penpot_review_source_path']
        : sourcePath;
    const localSourceNodeId =
      typeof node['penpot_review_source_node_id'] === 'string' &&
      node['penpot_review_source_node_id']
        ? node['penpot_review_source_node_id']
        : sourceNodeId;
    const localInstancePath =
      typeof node['penpot_review_instance_path'] === 'string'
        ? node['penpot_review_instance_path']
        : '[]';
    node['penpot_review_source_path'] = localSourcePath;
    node['penpot_review_source_node_id'] = localSourceNodeId;
    node['penpot_review_instance_path'] = prefixInstancePath(
      instancePath,
      localInstancePath,
    );
  }
}

function instancePathForCallsite(
  sourcePath: string,
  sourceNodeId: string,
  parentInstancePath = '[]',
): string {
  const parentSteps: unknown = JSON.parse(parentInstancePath);
  if (!Array.isArray(parentSteps))
    throw new Error('Review host parent instance path must be an array');
  return JSON.stringify([...parentSteps, { sourcePath, sourceNodeId }]);
}

function prefixInstancePath(prefix: string, local: string): string {
  if (!prefix || !local) return '';
  try {
    const prefixSteps: unknown = JSON.parse(prefix);
    const localSteps: unknown = JSON.parse(local);
    if (!Array.isArray(prefixSteps) || !Array.isArray(localSteps)) return '';
    return JSON.stringify([...prefixSteps, ...localSteps]);
  } catch {
    return '';
  }
}

/**
 * Build a review host for the retained ActivityDrawerWindow contract.
 *
 * The source component intentionally exposes neutral Container slots because
 * the retained runtime fills them from the active window.  Penpot has no
 * runtime router, so the review projection mounts the same authored product
 * views used by the two shipped window consumers.  The source host remains a
 * normal editable .zui view and can be exported/reconciled independently.
 */
async function prepareActivityDrawerReviewHost(
  repoRoot: string,
  sourcePath: string,
  sourceDocument: ZuiDocument,
  dependencies: LayoutDependencies,
  themeSourcePath?: string,
): Promise<{
  source: string;
  projection: ZuiDocument;
  consumers: Array<{ sourcePath: string; component: string }>;
} | null> {
  const isActivityComponent = sourcePath.endsWith(ACTIVITY_WINDOW_SOURCE);
  const isUiLayoutEditor = sourcePath.endsWith(UI_LAYOUT_EDITOR_WINDOW_SOURCE);
  const isAssetWindow = sourcePath.endsWith(ASSET_WINDOW_SOURCE);
  if (!isActivityComponent && !isUiLayoutEditor && !isAssetWindow) return null;

  const basePath = isActivityComponent
    ? UI_LAYOUT_EDITOR_WINDOW_SOURCE
    : sourcePath;
  const base = parseZuiDocument(
    await readFile(resolve(repoRoot, basePath), 'utf8'),
  ).document;
  if (base.asset.kind !== 'view' || !base.root?.node)
    throw new Error(`Activity review consumer is not a view: ${basePath}`);
  const baseRoot = base.nodes?.[base.root.node];
  if (!baseRoot || baseRoot.component !== 'ActivityDrawerWindow')
    throw new Error(
      `Activity review consumer is missing ActivityDrawerWindow: ${basePath}`,
    );

  const host = cloneZuiDocument(base);
  host.asset = {
    ...host.asset,
    id: `${sourceDocument.asset.id}#review-host`,
    display_name: sourceDocument.asset.display_name ?? base.asset.display_name,
  };
  host.imports = {
    widgets: uniqueStrings([...strings(host.imports?.['widgets'])]),
    styles: uniqueStrings([...strings(host.imports?.['styles'])]),
  };
  const consumers: Array<{ sourcePath: string; component: string }> = [
    { sourcePath, component: 'ActivityDrawerWindow' },
  ];
  if (basePath !== sourcePath)
    consumers.push({ sourcePath: basePath, component: 'ActivityDrawerWindow' });

  const mounts = isAssetWindow ? ASSET_WINDOW_MOUNTS : UI_LAYOUT_EDITOR_MOUNTS;
  for (const [slotNodeId, viewPath, selectedChildren] of mounts) {
    const view = parseZuiDocument(
      await readFile(resolve(repoRoot, viewPath), 'utf8'),
    ).document;
    const target = host.nodes?.[slotNodeId];
    if (!target || target.component !== 'Container')
      throw new Error(`Activity review slot ${slotNodeId} is not a Container`);
    mountViewIntoContainer(
      host,
      target,
      view,
      `review_${slotNodeId}`,
      selectedChildren,
    );
    host.imports!['widgets'] = uniqueStrings([
      ...strings(host.imports?.['widgets']),
      ...strings(view.imports?.['widgets']),
    ]);
    host.imports!['styles'] = uniqueStrings([
      ...strings(host.imports?.['styles']),
      ...strings(view.imports?.['styles']),
    ]);
    host.tokens = { ...(host.tokens ?? {}), ...(view.tokens ?? {}) };
    host.stylesheets = mergeStylesheets(host.stylesheets, view.stylesheets);
    consumers.push({ sourcePath: viewPath, component: 'view' });
  }

  const source = serializeZuiDocument(host);
  const projection = cloneZuiDocument(host);
  dependencies.embed(projection, sourcePath, themeSourcePath);
  return { source, projection, consumers };
}

function mountViewIntoContainer(
  host: ZuiDocument,
  target: ZuiNode,
  view: ZuiDocument,
  prefix: string,
  selectedNodes?: readonly string[],
): void {
  const rootId = view.root?.node;
  if (!rootId || !view.nodes?.[rootId])
    throw new Error(`Activity review view has no root: ${view.asset.id}`);
  const sourceNodes = cloneZuiDocument(view).nodes ?? {};
  if (selectedNodes?.length) sliceViewNodes(sourceNodes, rootId, selectedNodes);
  const hostNodes = (host.nodes ??= {});
  const mapping = new Map<string, string>();
  for (const id of Object.keys(sourceNodes)) {
    const mapped = `${prefix}__${id}`;
    if (hostNodes[mapped])
      throw new Error(`Activity review node id collision: ${mapped}`);
    mapping.set(id, mapped);
  }
  for (const [id, sourceNode] of Object.entries(sourceNodes)) {
    const node: ZuiNode = {
      ...sourceNode,
      ...(sourceNode.children
        ? {
            children: sourceNode.children.map((child) => ({
              ...child,
              node: mapping.get(child.node) ?? child.node,
            })),
          }
        : {}),
    };
    hostNodes[mapping.get(id)!] = node;
  }
  target.children = [
    ...(target.children ?? []),
    { node: mapping.get(rootId)! },
  ];
}

/** Keep selected authored subtrees together with their real ancestor layout. */
function sliceViewNodes(
  nodes: Record<string, ZuiNode>,
  rootId: string,
  selectedNodes: readonly string[],
): void {
  const parentByNode = new Map<string, string>();
  for (const [parentId, node] of Object.entries(nodes))
    for (const child of node.children ?? [])
      parentByNode.set(child.node, parentId);
  const allowed = new Set<string>([rootId]);
  const targets = selectedNodes.filter((id) => nodes[id]);
  if (targets.length !== selectedNodes.length)
    throw new Error(
      `Activity review view is missing selected nodes: ${selectedNodes
        .filter((id) => !nodes[id])
        .join(', ')}`,
    );
  const includeDescendants = (id: string): void => {
    if (allowed.has(id)) {
      // A node may already be an ancestor of another target; still walk its
      // children when this call is for a selected target.
    }
    allowed.add(id);
    for (const child of nodes[id]?.children ?? []) {
      allowed.add(child.node);
      includeDescendants(child.node);
    }
  };
  for (const target of targets) {
    let current: string | undefined = target;
    while (current) {
      allowed.add(current);
      if (current === rootId) break;
      current = parentByNode.get(current);
    }
    includeDescendants(target);
  }
  for (const [id, node] of Object.entries(nodes)) {
    if (!allowed.has(id)) {
      delete nodes[id];
      continue;
    }
    if (node.children) {
      node.children = node.children
        .filter((child) => allowed.has(child.node))
        .map((child) => normalizeFlexibleMount(child, nodes[child.node]));
    }
    if (
      node.control_id === 'AssetBrowserContentPanel' ||
      node.control_id === 'AssetsActivityContentPanel'
    ) {
      // The full product view reserves a 320px content minimum.  Activity
      // drawer slots are intentionally fluid and can be narrower at the
      // required 900px/640px review widths; retain the real subtree while
      // letting this projection fit its host.
      const width = node.layout?.['width'];
      if (width && typeof width === 'object' && !Array.isArray(width)) {
        node.layout = {
          ...(node.layout ?? {}),
          width: {
            ...(width as Record<string, unknown>),
            min: 0,
            preferred: 0,
            stretch: 'Stretch',
          },
        };
      }
    }
  }
}

function normalizeFlexibleMount(
  mount: { node: string; [key: string]: unknown },
  child: ZuiNode | undefined,
): { node: string; [key: string]: unknown } {
  const slot = mount['slot'];
  if (!slot || !child) return mount;
  if (typeof slot !== 'object' || Array.isArray(slot)) return mount;
  const slotTable = slot as Record<string, unknown>;
  const width = slotTable['layout'];
  if (
    !width ||
    typeof width !== 'object' ||
    Array.isArray(width) ||
    ![
      'ScrollableBox',
      'Container',
      'Space',
      'VerticalBox',
      'HorizontalBox',
    ].includes(child.component)
  )
    return mount;
  const widthTable = (width as Record<string, unknown>)['width'];
  if (
    !widthTable ||
    typeof widthTable !== 'object' ||
    Array.isArray(widthTable)
  )
    return mount;
  return {
    ...mount,
    slot: {
      ...slotTable,
      layout: {
        ...(width as Record<string, unknown>),
        width: {
          ...(widthTable as Record<string, unknown>),
          min: 0,
          preferred: 0,
          stretch: 'Stretch',
        },
      },
    },
  };
}

function mergeStylesheets(
  current: ZuiTable[] | undefined,
  incoming: ZuiTable[] | undefined,
): ZuiTable[] | undefined {
  const result = [...(current ?? [])];
  for (const sheet of incoming ?? []) {
    const id = sheet['id'];
    const index =
      typeof id === 'string'
        ? result.findIndex((candidate) => candidate['id'] === id)
        : -1;
    if (index >= 0) result[index] = sheet;
    else result.push(sheet);
  }
  return result.length ? result : undefined;
}

function strings(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : [];
}

function record(value: unknown, context: string): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value))
    throw new Error(`${context} must be an object`);
  return value as Record<string, unknown>;
}

function stringValue(value: unknown, context: string): string {
  if (typeof value !== 'string' || !value.trim())
    throw new Error(`${context} must be a nonempty string`);
  return value;
}

function nullableString(value: unknown, context: string): string | null {
  if (value === null) return null;
  return stringValue(value, context);
}

function nullablePath(value: unknown, context: string): string | undefined {
  const path = nullableString(value, context);
  if (path === null) return undefined;
  if (!path.startsWith('zircon_editor/assets/'))
    throw new Error(`${context} is outside zircon_editor/assets`);
  return path;
}

function unknownArray(value: unknown, context: string): unknown[] {
  if (!Array.isArray(value)) throw new Error(`${context} must be an array`);
  return value;
}

function uniqueStrings(values: string[]): string[] {
  return [...new Set(values)];
}
