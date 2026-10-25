import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { parse as parseToml } from 'smol-toml';
import {
  cloneZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
} from '../src/bridge/zui-document';
import type { LayoutDependencies } from './zui-layout-dependencies';
import { resolveDesignNumber } from '../src/bridge/zui-prefab-system';
import {
  prepareUnrealDockHeaderViews,
  UNREAL_DOCK_HEADER_SOURCE,
} from './zui-layout-unreal-dock-headers';
import {
  applyWorkbenchPresentationProjection,
  prepareWorkbenchPresentationStructure,
} from './zui-layout-workbench-projection';

const TAB_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/tab.zui';
const PANEL_TITLE_SOURCE =
  'zircon_editor/assets/ui/editor/workbench/primitives/chrome/section_title.zui';
const PANEL_ACTION_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui';
const NUMBER_FIELD_SOURCE =
  'zircon_editor/assets/ui/editor/workbench/primitives/inputs/number_field.zui';
const STATUS_ITEM_SOURCE =
  'zircon_editor/assets/ui/editor/workbench/primitives/feedback/status_item.zui';
const ACTIVITY_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/workbench/shell/activity_drawer_window.zui';
const UI_LAYOUT_EDITOR_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.zui';
const ASSET_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/windows/asset_window.zui';
const EDITOR_MAIN_FRAME_SOURCE =
  'zircon_editor/assets/ui/editor/host/editor_main_frame.zui';
const WORKBENCH_SHELL_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
const WORKBENCH_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const UNREAL_COMPONENT_SOURCES = [
  [
    'zircon_editor/assets/ui/editor/unreal/components/toolbar.zui',
    'UnrealLevelEditorToolbar',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/menus.zui',
    'UnrealLevelEditorMenus',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/menu_popups.zui',
    'UnrealLevelEditorMenuPopups',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/viewport.zui',
    'UnrealLevelViewport',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/outliner.zui',
    'UnrealOutlinerPanel',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/details.zui',
    'UnrealDetailsPanel',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/status_bar.zui',
    'UnrealEditorStatusBar',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/content_drawer.zui',
    'UnrealContentDrawer',
  ],
  [
    'zircon_editor/assets/ui/editor/unreal/components/bottom_activity_dock.zui',
    'UnrealBottomActivityDock',
  ],
] as const;

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

/** Attach the actual editor locale to both authored Unreal inputs and consumer hosts. */
export async function prepareUnrealReviewTranslations(
  repoRoot: string,
  sourcePath: string,
  document: ZuiDocument,
  consumerProjection?: ZuiDocument,
  workbenchPresentation?: Record<string, unknown>,
): Promise<boolean> {
  const source = resolve(repoRoot, sourcePath);
  const unrealSources = [
    EDITOR_MAIN_FRAME_SOURCE,
    WORKBENCH_SHELL_SOURCE,
    WORKBENCH_WINDOW_SOURCE,
    ...UNREAL_COMPONENT_SOURCES.map(([path]) => path),
  ];
  if (!unrealSources.some((path) => resolve(repoRoot, path) === source))
    return false;

  const locale = workbenchPresentation?.['activeLocale'] ?? 'en';
  if (locale !== 'en' && locale !== 'zh-CN')
    throw new Error('Unreal review locale must be en or zh-CN');
  const localePath = 'zircon_editor/assets/i18n/' + locale + '.toml';
  const localeSource = parseToml(
    await readFile(resolve(repoRoot, localePath), 'utf8'),
  );
  const translations = localeSource['translations'];
  if (
    localeSource['locale'] !== locale ||
    !translations ||
    typeof translations !== 'object' ||
    Array.isArray(translations) ||
    !Object.values(translations).every((value) => typeof value === 'string')
  )
    throw new Error('Unreal review translation source does not match its locale');

  for (const projection of new Set([document, consumerProjection])) {
    if (!projection) continue;
    projection['penpot_i18n_tables'] = {
      ...((projection['penpot_i18n_tables'] as ZuiTable | undefined) ?? {}),
      editor: { locale, translations: structuredClone(translations) },
    };
    projection['penpot_dependency_sources'] = uniqueStrings([
      ...strings(projection['penpot_dependency_sources']),
      localePath,
    ]);
  }
  return true;
}

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
    const definitions = Object.entries(component.components ?? {});
    if (component.asset.kind !== 'component' || definitions.length !== 1)
      return null;
    const [name, exported] = definitions[0];
    if (!component.nodes?.[exported.root])
      throw new Error(`Component review source has no root: ${component.asset.id}#${name}`);
    // A component has no view [root]. Invoke its original export through the
    // same dependency expansion used by the specialized hosts. Do not copy
    // source nodes, fill missing slots, or promote defaults to observed state.
    const host: ZuiDocument = {
      asset: {
        kind: 'view',
        version: 2,
        id: `${component.asset.id}#review-host`,
        display_name: component.asset.display_name,
      },
      imports: {
        widgets: uniqueStrings([
          `${component.asset.id}#${name}`,
          ...strings(component.imports?.['widgets']),
        ]),
        styles: strings(component.imports?.['styles']),
      },
      tokens: component.tokens,
      stylesheets: component.stylesheets,
      root: { node: 'review_component' },
      nodes: {
        review_component: { component: `${component.asset.id}#${name}` },
      },
    };
    const source = serializeZuiDocument(host);
    const projection = cloneZuiDocument(host);
    dependencies.embed(projection, sourcePath, themeSourcePath);
    // The sole invocation belongs to this generated review host, not to the
    // canonical component. Keep real nested callsites, but do not advertise
    // review_component as an authored node in the original source owner.
    for (const node of Object.values(projection.nodes ?? {})) {
      const rawPath = node['penpot_review_instance_path'];
      if (typeof rawPath !== 'string') continue;
      const steps = JSON.parse(rawPath) as Array<{
        sourcePath: string; sourceNodeId: string;
      }>;
      if (steps[0]?.sourcePath === sourcePath &&
          steps[0]?.sourceNodeId === 'review_component')
        node['penpot_review_instance_path'] = JSON.stringify(steps.slice(1));
    }
    return {
      source,
      projection,
      consumers: [{ sourcePath, component: name }],
    };
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
 * Compose the authored Unreal editor shell and level-editor window for Penpot
 * review.  The shell owns the retained host/document/pane contract; the level
 * window is mounted as the real pane content.  No runtime-only task bar,
 * document tab strip, legacy drawer shell, or snapshot-owned source view is
 * invented here.
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
  const windowInput = sourcePath.endsWith(WORKBENCH_WINDOW_SOURCE);
  const frameInput = sourcePath.endsWith(EDITOR_MAIN_FRAME_SOURCE);
  if (!windowInput && !frameInput) return null;

  const [shell, authoredWindow, frame] = await Promise.all([
    readFile(resolve(repoRoot, WORKBENCH_SHELL_SOURCE), 'utf8').then((text) =>
      parseZuiDocument(text).document,
    ),
    windowInput
      ? Promise.resolve(sourceDocument)
      : readFile(resolve(repoRoot, WORKBENCH_WINDOW_SOURCE), 'utf8').then(
          (text) => parseZuiDocument(text).document,
        ),
    frameInput
      ? Promise.resolve(sourceDocument)
      : readFile(resolve(repoRoot, EDITOR_MAIN_FRAME_SOURCE), 'utf8').then(
          (text) => parseZuiDocument(text).document,
        ),
  ]);

  if (
    shell.asset.kind !== 'view' ||
    !shell.root?.node ||
    !shell.nodes?.[shell.root.node] ||
    shell.nodes[shell.root.node].component !== 'UiHostWindow'
  )
    throw new Error('Unreal editor shell review host is not a rooted UiHostWindow');
  if (
    authoredWindow.asset.kind !== 'view' ||
    !authoredWindow.root?.node ||
    !authoredWindow.nodes?.[authoredWindow.root.node] ||
    authoredWindow.nodes[authoredWindow.root.node].component !== 'Overlay'
  )
    throw new Error('Unreal level editor review host is not a rooted Overlay view');

  const host = frameInput ? cloneZuiDocument(frame) : cloneZuiDocument(shell);
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
    ]),
    styles: uniqueStrings([
      ...strings(host.imports?.['styles']),
      ...strings(shell.imports?.['styles']),
      ...strings(authoredWindow.imports?.['styles']),
    ]),
  };
  host.tokens = {
    ...(host.tokens ?? {}),
    ...(shell.tokens ?? {}),
    ...(authoredWindow.tokens ?? {}),
  };
  host.stylesheets = mergeStylesheets(
    mergeStylesheets(host.stylesheets, shell.stylesheets),
    authoredWindow.stylesheets,
  );

  const hostNodes = host.nodes ?? (host.nodes = {});
  const shellPrefix = frameInput ? 'review_editor_shell' : '';
  const windowPrefix = 'review_level_editor_window';
  const shellMountParentPath = frameInput
    ? instancePathForCallsite(EDITOR_MAIN_FRAME_SOURCE, 'active_window_host_slot')
    : '[]';
  let shellHost: ZuiNode | undefined;
  if (frameInput) {
    const activeWindowHost = hostNodes['active_window_host_slot'];
    if (
      !activeWindowHost ||
      activeWindowHost.component !== 'Slot' ||
      activeWindowHost.props?.['name'] !== 'active_window_host'
    )
      throw new Error(
        'EditorMainFrame review host does not match its active_window_host Slot contract',
      );
    activeWindowHost.component = 'Container';
    activeWindowHost.children = [];
    mountViewIntoContainer(host, activeWindowHost, shell, shellPrefix);
    shellHost = hostNodes[`${shellPrefix}__${shell.root.node}`];
  } else {
    shellHost = hostNodes[shell.root.node];
  }
  if (!shellHost)
    throw new Error('Unreal editor shell review host lost its root node');

  const paneSurfaceId = shellPrefix
    ? `${shellPrefix}__pane_surface`
    : 'pane_surface';
  const paneSurface = hostNodes[paneSurfaceId];
  if (!paneSurface)
    throw new Error('Unreal editor shell review host is missing pane_surface');
  mountViewIntoContainer(host, paneSurface, authoredWindow, windowPrefix);

  const source = serializeZuiDocument(host);
  const projection = cloneZuiDocument(host);
  if (frameInput)
    annotateMountedSourceIdentity(
      projection,
      '',
      EDITOR_MAIN_FRAME_SOURCE,
      Object.keys(frame.nodes ?? {}),
      '[]',
    );
  annotateMountedSourceIdentity(
    projection,
    shellPrefix,
    WORKBENCH_SHELL_SOURCE,
    Object.keys(shell.nodes ?? {}),
    shellMountParentPath,
  );
  annotateMountedSourceIdentity(
    projection,
    windowPrefix,
    WORKBENCH_WINDOW_SOURCE,
    Object.keys(authoredWindow.nodes ?? {}),
    instancePathForCallsite(
      WORKBENCH_SHELL_SOURCE,
      'pane_surface',
      shellMountParentPath,
    ),
  );
  if (workbenchPresentation)
    prepareWorkbenchPresentationStructure(projection, workbenchPresentation);

  // Embed only the authored roots participating in this host.  Their imports
  // remain source-addressable for Penpot export and reconciliation.
  const dependencySources = new Set<string>();
  const embedRoot = (sourcePath: string): void => {
    dependencies.embed(projection, sourcePath, themeSourcePath, {
      preserveUnselectedBranchDefaults: true,
    });
    for (const path of strings(projection['penpot_dependency_sources']))
      dependencySources.add(path);
    projection['penpot_dependency_sources'] = [...dependencySources];
  };
  if (frameInput) embedRoot(EDITOR_MAIN_FRAME_SOURCE);
  embedRoot(WORKBENCH_SHELL_SOURCE);
  embedRoot(WORKBENCH_WINDOW_SOURCE);
  const dockHeaders = await prepareUnrealDockHeaderViews(repoRoot, workbenchPresentation);
  const mountedKinds = new Set<string>();
  const windowInstancePath = instancePathForCallsite(
    WORKBENCH_SHELL_SOURCE, 'pane_surface', shellMountParentPath,
  );
  for (const [index, prepared] of dockHeaders.entries()) {
    const observation = prepared.observation;
    // These are the two actual authored mount slots in the level-editor view.
    // Additional dock/floating surfaces require their own actual view hosts.
    if (!['document', 'bottom'].includes(observation.kind) || mountedKinds.has(observation.kind))
      throw new Error('Actual Header surface has no unique authored level-editor mount: ' +
        observation.kind + ':' + observation.surfaceKey);
    mountedKinds.add(observation.kind);
    if (observation.kind === 'document') {
      // Native owns_authored_level_viewport bypasses the old document-dock
      // painter. Its cached Header Source is still published, but is not an
      // extra painted child of this exact authored viewport invocation.
      assertAuthoredLevelViewport(projection, windowInstancePath);
      continue;
    }
    const bottomSource = 'zircon_editor/assets/ui/editor/unreal/components/bottom_activity_dock.zui';
    const bottomInstance = instancePathForCallsite(WORKBENCH_WINDOW_SOURCE, 'bottom_activity_dock', windowInstancePath);
    const root = requiredMountedSource(projection, WORKBENCH_WINDOW_SOURCE, 'root', windowInstancePath,
      'WorkbenchWindowRoot', 'Overlay');
    const content = requiredMountedSource(projection, WORKBENCH_WINDOW_SOURCE, 'editor_content', windowInstancePath,
      'UnrealLevelEditorContent', 'VerticalGroup');
    const shellEntry = requiredMountedSource(projection, bottomSource, 'shell', bottomInstance,
      'BottomDrawerShellRoot', 'VerticalGroup');
    const headerEntry = requiredMountedSource(projection, bottomSource, 'header', bottomInstance,
      'BottomDrawerHeaderRoot', 'HorizontalGroup');
    const separatorEntry = requiredMountedSource(projection, bottomSource, 'separator', bottomInstance,
      'BottomDrawerSeparator', 'Container');
    const bodyEntry = requiredMountedSource(projection, bottomSource, 'content', bottomInstance,
      'BottomDrawerContentRoot', 'VerticalGroup');
    for (const [parent, child] of [[root, content], [content, shellEntry], [shellEntry, headerEntry],
      [shellEntry, separatorEntry], [shellEntry, bodyEntry]]) {
      const parents = Object.values(projection.nodes ?? {}).filter((node) =>
        node.children?.some((item) => item.node === child[0]));
      if (parents.length !== 1 || parents[0] !== parent[1] ||
          parent[1].children?.filter((item) => item.node === child[0]).length !== 1)
        throw new Error('Actual bottom Header is outside its authored drawer shell owner chain');
    }
    const shell = shellEntry[1], target = headerEntry[1];
    const panelHeight = resolveDesignNumber(projection, '$editor.chrome.panel_header.height');
    const authoredHeight = target.layout?.['height'];
    if (panelHeight === null || panelHeight <= 0 || !authoredHeight || typeof authoredHeight !== 'object' ||
        Array.isArray(authoredHeight) || authoredHeight instanceof Date || authoredHeight['stretch'] !== 'Fixed' ||
        ['min', 'preferred', 'max'].some((axis) => resolveDesignNumber(projection, authoredHeight[axis]) !== panelHeight))
      throw new Error('Actual bottom Header lost its loaded Source panel-header height');
    const mounts = shell.children ?? [];
    const container = shell.layout?.['container'] as ZuiTable | undefined;
    const separatorHeight = separatorEntry[1].layout?.['height'] as ZuiTable | undefined;
    const separatorExtent = resolveDesignNumber(projection, separatorHeight?.['preferred']);
    const bodyHeight = bodyEntry[1].layout?.['height'] as ZuiTable | undefined;
    if (mounts.length !== 3 || [headerEntry, separatorEntry, bodyEntry].some((entry, i) => mounts[i]?.node !== entry[0]) ||
        shell.layout?.['clip'] !== true || container?.['kind'] !== 'VerticalBox' ||
        resolveDesignNumber(projection, container['gap']) !== 0 || separatorExtent === null || separatorExtent <= 0 ||
        separatorHeight?.['stretch'] !== 'Fixed' ||
        ['min', 'max'].some((axis) => resolveDesignNumber(projection, separatorHeight?.[axis]) !== separatorExtent) ||
        bodyHeight?.['stretch'] !== 'Stretch' || resolveDesignNumber(projection, bodyHeight['min']) !== 0 ||
        bodyHeight['preferred'] !== undefined || bodyHeight['max'] !== undefined ||
        mounts.some((mount, i) => {
          const layout = mount.slot?.['layout'] as ZuiTable | undefined;
          const linear = layout?.['linear_size'] as ZuiTable | undefined;
          return !layout || Object.keys(layout).some((key) => key !== 'linear_size') ||
            (i < 2 ? linear?.['rule'] !== 'Auto' || linear['shrink_value'] !== 0 :
              linear?.['rule'] !== 'Stretch' || linear['value'] !== 1 || linear['shrink_value'] !== undefined);
        }))
      throw new Error('Actual bottom Header lost its loaded Source clipped-body layout');
    const drawers = workbenchPresentation?.['drawers'];
    const bottom = Array.isArray(drawers)
      ? drawers.find((drawer) => drawer && drawer.slot === 'bottom') : undefined;
    if (!bottom || bottom.visible !== true || !Array.isArray(bottom.tabs) || !bottom.tabs.length ||
        !['collapsed', 'pinned', 'autoHide'].includes(bottom.mode) || typeof bottom.extent !== 'number' ||
        !Number.isFinite(bottom.extent) || bottom.extent < 0)
      throw new Error('Consumed bottom Header has no visible logical business drawer extent');
    // Match drawer_region_input: the stored expanded extent does not allocate
    // space while collapsed. No observed rectangle becomes a layout input.
    const extent = bottom.mode === 'collapsed' ? panelHeight : Math.max(bottom.extent, panelHeight);
    // Native keeps these nodes self-visible but clips their arranged frames.
    // Penpot cannot allocate a zero-height board. Project only the effective
    // visibility from the proven Source stack; retain every authored dimension.
    for (const [entry, start] of [[separatorEntry, panelHeight], [bodyEntry, panelHeight + separatorExtent]] as const) {
      const visible = extent > start, node = entry[1];
      const visibility = visible ? 'visible' : 'collapsed';
      node.props = { ...(node.props ?? {}), visible, visibility };
      node.state = { ...(node.state ?? {}), visible, visibility };
      node['penpot_review_derived_drawer_visibility'] = {
        authority: 'loaded-source-and-business-drawer', slot: 'bottom', mode: bottom.mode,
        extent, panelHeight, separatorExtent, visible,
      };
    }
    shell.props = { ...(shell.props ?? {}), visibility: 'visible', visible: true };
    shell.layout = { ...(shell.layout ?? {}),
      height: { min: extent, preferred: extent, max: extent, stretch: 'Fixed' } };
    target.component = 'Overlay';
    target.layout = { ...(target.layout ?? {}), container: { kind: 'Overlay' } };
    const view = cloneZuiDocument(prepared.document);
    dependencies.embed(view, UNREAL_DOCK_HEADER_SOURCE, themeSourcePath, {
      preserveUnselectedBranchDefaults: true,
    });
    projection.imports = {
      widgets: uniqueStrings([...strings(projection.imports?.['widgets']), ...strings(view.imports?.['widgets'])]),
      styles: uniqueStrings([...strings(projection.imports?.['styles']), ...strings(view.imports?.['styles'])]),
    };
    projection.tokens = { ...(projection.tokens ?? {}), ...(view.tokens ?? {}) };
    projection.stylesheets = mergeStylesheets(projection.stylesheets, view.stylesheets);
    for (const path of strings(view['penpot_dependency_sources'])) dependencySources.add(path);
    projection['penpot_dependency_sources'] = [...dependencySources];
    // Only actual Source instance inputs determine layout. Observed row frames
    // and mount rectangles remain evidence and are never copied into this view.
    const headerPrefix = 'review_consumed_dock_header_' + index;
    mountViewIntoContainer(projection, target, view, headerPrefix);
  }
  await prepareUnrealReviewTranslations(
    repoRoot,
    sourcePath,
    projection,
    undefined,
    workbenchPresentation,
  );
  if (workbenchPresentation)
    applyWorkbenchPresentationProjection(projection, workbenchPresentation);

  const consumers = new Map<string, string>();
  const addConsumer = (path: string, component: string): void => {
    if (!consumers.has(`${path}#${component}`)) consumers.set(`${path}#${component}`, component);
  };
  if (frameInput) addConsumer(EDITOR_MAIN_FRAME_SOURCE, 'view');
  addConsumer(sourcePath, 'view');
  addConsumer(WORKBENCH_SHELL_SOURCE, 'view');
  addConsumer(WORKBENCH_WINDOW_SOURCE, 'view');
  for (const [path, component] of UNREAL_COMPONENT_SOURCES)
    addConsumer(path, component);
  if (dockHeaders.length) addConsumer(UNREAL_DOCK_HEADER_SOURCE, 'view');
  return {
    source,
    projection,
    consumers: [...consumers].map(([key, component]) => ({
      sourcePath: key.slice(0, key.lastIndexOf('#')),
      component,
    })),
  };
}

function requiredMountedSource(document: ZuiDocument, sourcePath: string, sourceNodeId: string,
  instancePath: string, controlId: string, component: string): [string, ZuiNode] {
  const matches = Object.entries(document.nodes ?? {}).filter(([, node]) =>
    node['penpot_review_source_path'] === sourcePath && node['penpot_review_source_node_id'] === sourceNodeId &&
    node['penpot_review_instance_path'] === instancePath);
  if (matches.length !== 1 || matches[0][1].control_id !== controlId || matches[0][1].component !== component)
    throw new Error('Actual Header authored mount is missing or ambiguous: ' + sourcePath + '#' + sourceNodeId);
  return matches[0];
}

/** Prove the same authored media ownership that bypasses Native's old dock painter. */
function assertAuthoredLevelViewport(document: ZuiDocument, ownerInstance: string): void {
  const root = requiredMountedSource(document, WORKBENCH_WINDOW_SOURCE, 'root', ownerInstance,
    'WorkbenchWindowRoot', 'Overlay');
  const content = requiredMountedSource(document, WORKBENCH_WINDOW_SOURCE, 'editor_content', ownerInstance,
    'UnrealLevelEditorContent', 'VerticalGroup');
  const main = requiredMountedSource(document, WORKBENCH_WINDOW_SOURCE, 'main_band', ownerInstance,
    'WorkbenchWindowMainBandRegion', 'HorizontalGroup');
  const source = 'zircon_editor/assets/ui/editor/unreal/components/viewport.zui';
  const instance = instancePathForCallsite(WORKBENCH_WINDOW_SOURCE, 'viewport', ownerInstance);
  const viewport = requiredMountedSource(document, source, 'root', instance,
    'WorkbenchMainBandViewportPanel', 'VerticalGroup');
  const overlay = requiredMountedSource(document, source, 'viewport', instance,
    'UnrealSceneViewViewport', 'Overlay');
  const media = requiredMountedSource(document, source, 'scene_media', instance,
    'WorkbenchViewportSurface', 'SceneViewViewport');
  const toolbar = requiredMountedSource(document, source, 'toolbar_layer', instance,
    'UnrealViewportToolbarLayer', 'Overlay');
  for (const [parent, child] of [[root, content], [content, main], [main, viewport],
    [viewport, overlay], [overlay, media], [overlay, toolbar]]) {
    const parents = Object.values(document.nodes ?? {}).filter((node) =>
      node.children?.some((item) => item.node === child[0]));
    if (parents.length !== 1 || parents[0] !== parent[1] ||
        parent[1].children?.filter((item) => item.node === child[0]).length !== 1)
      throw new Error('Actual document Header lost its complete authored viewport owner chain');
  }
  if (viewport[1].children?.length !== 1 || overlay[1].children?.length !== 2 ||
      overlay[1].children?.[0].node !== media[0] || overlay[1].children?.[1].node !== toolbar[0])
    throw new Error('Actual document Header lost its original authored viewport subtree');
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
      viewPath,
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
  sourceOwner?: string,
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
      ...(sourceOwner ? {
        penpot_review_source_path: sourceOwner,
        penpot_review_source_node_id: id,
        penpot_review_instance_path: '[]',
      } : {}),
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

function uniqueStrings(values: string[]): string[] {
  return [...new Set(values)];
}
