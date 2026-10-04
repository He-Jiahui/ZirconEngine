import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { prepareComponentReviewHost } from '../../tools/zui-layout-component-hosts';
import { trackedZuiPaths } from '../../tools/zui-layout-catalog';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
} from './penpot-projection';
import {
  cloneZuiDocument,
  normalizeZuiDocument,
  parseZuiDocument,
  type ZuiDocument,
} from './zui-document';
import { reconcileReviewSource, reviewHost } from './zui-review-host';
import { reviewDocumentForCase } from './zui-review-case';
import { segmentedGeometry } from './zui-segmented-geometry';

const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
const directory =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs';
const strip = `${directory}/workbench_tab_strip.zui`;
const tab = `${directory}/workbench_tab.zui`;
const diagnostic =
  'zircon_editor/assets/ui/editor/components/workbench/composites/feedback/workbench_diagnostic_row.zui';
const statusItem =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui';
const panelHeader =
  'zircon_editor/assets/ui/editor/components/workbench/composites/chrome/workbench_panel_header.zui';
const sectionTitle =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui';
const button = `${directory}/workbench_button.zui`;
const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const activityWindow =
  'zircon_editor/assets/ui/editor/components/workbench/shell/activity_drawer_window.zui';
const editorMainFrame =
  'zircon_editor/assets/ui/editor/host/editor_main_frame.zui';
const workbenchShell =
  'zircon_editor/assets/ui/editor/host/workbench_shell.zui';
const workbenchWindow =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
const notificationCenter =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_notification_center.zui';
const tableRow =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_table_row.zui';

describe('component slot review host', () => {
  const dependencies = new LayoutDependencies();
  let source: ZuiDocument;
  beforeAll(async () => {
    await dependencies.load(repo, [strip, tab, theme]);
    source = parseZuiDocument(
      await readFile(`${repo}/${strip}`, 'utf8'),
    ).document;
  });

  it('mounts declared tab instances in the real default slot without changing the source', async () => {
    const before = normalizeZuiDocument(source);
    const result = (await prepareComponentReviewHost(
      repo,
      strip,
      source,
      dependencies,
      theme,
    ))!;
    const host = parseZuiDocument(result.source).document;
    expect(host.nodes!['review_component'].component).toBe(
      `${source.asset.id}#WorkbenchTabStrip`,
    );
    expect(host.nodes!['review_component'].children).toHaveLength(3);
    expect(host.nodes!['review_tab_0'].props).toEqual({
      text: 'overview',
      selected: true,
    });
    expect(host.nodes!['review_tab_1'].props).toEqual({
      text: 'details',
      selected: false,
    });
    expect(host.nodes!['review_tab_2'].props).toEqual({
      text: 'stats',
      selected: false,
    });
    expect(
      Object.values(result.projection.nodes!).some(
        (node) => node.component === 'Slot',
      ),
    ).toBe(false);
    expect(
      result.projection.nodes!['review_tab_0']['penpot_prefab_source'],
    ).toBe(`${tab}#WorkbenchTab`);
    const projection = projectZuiDocument(result.projection);
    expect(projection.rootNodes[0].container).toMatchObject({
      kind: 'flex',
      direction: 'row',
      gap: 0,
    });
    expect(projection.rootNodes[0].children).toHaveLength(3);
    expect(projection.rootNodes[0].text).toBeNull();
    expect(
      projection.rootNodes[0].children.map((node) => node.text?.characters),
    ).toEqual(['overview', 'details', 'stats']);
    expect(normalizeZuiDocument(source)).toEqual(before);
  });

  it('wraps the shared table row in an authored horizontal review host', async () => {
    const tableDependencies = new LayoutDependencies();
    await tableDependencies.load(repo, [tableRow, theme]);
    const table = parseZuiDocument(
      await readFile(`${repo}/${tableRow}`, 'utf8'),
    ).document;
    expect(table.nodes?.root?.props?.['layout_min_width']).toBe(360);
    const result = (await prepareComponentReviewHost(
      repo,
      tableRow,
      table,
      tableDependencies,
      theme,
    ))!;
    const host = parseZuiDocument(result.source).document;
    expect(host.nodes?.review_scroll?.component).toBe('ScrollableBox');
    expect(host.nodes?.review_scroll?.layout?.['container']).toEqual({
      kind: 'ScrollableBox',
      axis: 'Horizontal',
      gap: 0,
      scrollbar_visibility: 'Auto',
    });
    expect(host.nodes?.review_scroll?.children).toEqual([
      { node: 'review_component' },
    ]);
    expect(host.nodes?.review_component?.component).toBe(
      `${table.asset.id}#WorkbenchTableRow`,
    );
    expect(host.nodes?.review_component?.layout?.['width']).toEqual({
      min: 360,
      preferred: 360,
      max: 360,
      stretch: 'Fixed',
    });
  });

  it('keeps the source contract exact on export and rejects unmapped specimen edits', async () => {
    const result = (await prepareComponentReviewHost(
      repo,
      strip,
      source,
      dependencies,
      theme,
    ))!;
    const original = { ...source, penpot_review_host: result.projection };
    const snapshot = cloneProjectionSnapshot(
      projectZuiDocument(reviewHost(original)!),
    );
    expect(
      normalizeZuiDocument(reconcileReviewSource(original, snapshot).document),
    ).toEqual(normalizeZuiDocument(original));
    snapshot.shapes.find(
      (item) => item.nodeId === 'review_tab_1',
    )!.current.text!.characters = 'Changed';
    expect(() => reconcileReviewSource(original, snapshot)).toThrow(
      'explicit source mapping',
    );
    const invalid = cloneZuiDocument(source);
    invalid.components!['WorkbenchTabStrip']['slots'] = {
      default: { multiple: false },
    };
    await expect(
      prepareComponentReviewHost(repo, strip, invalid, dependencies, theme),
    ).rejects.toThrow('slot contract');
  });

  it('uses flat native tabs and visibly distinguishes keyboard focus through the source stylesheet', async () => {
    const result = (await prepareComponentReviewHost(
      repo,
      strip,
      source,
      dependencies,
      theme,
    ))!;
    const normal = projectZuiDocument(result.projection);
    const tabs = normal.rootNodes[0].children;
    expect(
      tabs.every(
        (node) =>
          node.component === 'Tab' &&
          node.paint.borderRadius === 0 &&
          node.paint.strokeWidth === 0,
      ),
    ).toBe(true);
    expect(tabs[0].segmented?.active).toBe(true);
    expect(tabs[0].text?.fontSize).toBe(14);
    expect(tabs[0].segmented?.colors.background).toBe('#243f5a');
    const focused = projectZuiDocument(
      reviewDocumentForCase(result.projection, {
        id: 'focused-360x520-dpi1',
        sourcePath: strip,
        host: 'component',
        viewport: { width: 360, height: 520 },
        dpi: 1,
        locale: 'en-US',
        state: 'focused',
        data: {},
      }),
    );
    const focusTab = focused.rootNodes[0].children[1];
    expect(focusTab.segmented?.colors.background).toBe('#253f59');
    expect(focusTab.text?.color).toBe('#e8e8e8');
    expect(
      segmentedGeometry(focusTab.segmented!, 120, 32).texts['primary'].x,
    ).toBe(8);
  });

  it('mounts explicit status-item instances for the diagnostic row slot contract', async () => {
    const diagnosticSource = parseZuiDocument(
      await readFile(`${repo}/${diagnostic}`, 'utf8'),
    ).document;
    const diagnosticDependencies = new LayoutDependencies();
    await diagnosticDependencies.load(repo, [diagnostic, statusItem, theme]);
    const result = (await prepareComponentReviewHost(
      repo,
      diagnostic,
      diagnosticSource,
      diagnosticDependencies,
      theme,
    ))!;
    const host = parseZuiDocument(result.source).document;
    expect(host.nodes!['review_component'].children).toEqual([
      { node: 'review_severity', slot: { name: 'severity' } },
      { node: 'review_message', slot: { name: 'message' } },
    ]);
    expect(host.nodes!['review_severity'].props?.['text']).toBe('[Warning]');
    expect(host.nodes!['review_message'].props?.['text']).toMatch(
      /validation requires attention/,
    );
    const projection = projectZuiDocument(result.projection);
    expect(projection.rootNodes[0].geometry.height).toBe(48);
    expect(
      projection.rootNodes[0].children.map((node) => node.text?.characters),
    ).toEqual(['[Warning]', 'Asset validation requires attention.']);
  });

  it('renders the required panel title and an action without changing the product slot contract', async () => {
    const panelSource = parseZuiDocument(
      await readFile(`${repo}/${panelHeader}`, 'utf8'),
    ).document;
    const before = normalizeZuiDocument(panelSource);
    const panelDependencies = new LayoutDependencies();
    await panelDependencies.load(repo, [
      panelHeader,
      sectionTitle,
      button,
      theme,
    ]);
    const result = (await prepareComponentReviewHost(
      repo,
      panelHeader,
      panelSource,
      panelDependencies,
      theme,
    ))!;
    const host = parseZuiDocument(result.source).document;
    expect(host.nodes!['review_component'].children).toEqual([
      { node: 'review_title', slot: { name: 'title' } },
      { node: 'review_action', slot: { name: 'actions' } },
    ]);
    expect(host.nodes!['review_title'].component).toContain(
      '#WorkbenchSectionTitle',
    );
    expect(host.nodes!['review_action'].component).toContain(
      '#WorkbenchButton',
    );
    expect(host.nodes!['review_action'].layout?.height).toMatchObject({
      preferred: '$editor.control.height.dense',
      stretch: 'Fixed',
    });
    expect(result.consumers).toHaveLength(3);
    expect(
      Object.values(result.projection.nodes!).some(
        (node) => node.component === 'Slot',
      ),
    ).toBe(false);
    const projected = projectZuiDocument(result.projection);
    expect(projected.rootNodes[0].container.padding).toEqual({
      top: 0,
      right: 8,
      bottom: 0,
      left: 8,
    });
    expect(
      projected.rootNodes[0].children.map((node) => node.text?.characters),
    ).toEqual(['Inspector', 'More']);
    expect(normalizeZuiDocument(panelSource)).toEqual(before);
    const original = { ...panelSource, penpot_review_host: result.projection };
    const snapshot = cloneProjectionSnapshot(
      projectZuiDocument(reviewHost(original)!),
    );
    expect(
      normalizeZuiDocument(reconcileReviewSource(original, snapshot).document),
    ).toEqual(normalizeZuiDocument(original));
    snapshot.shapes.find(
      (item) => item.nodeId === 'review_title',
    )!.current.text!.characters = 'Changed';
    expect(() => reconcileReviewSource(original, snapshot)).toThrow(
      'explicit source mapping',
    );

    const invalid = cloneZuiDocument(panelSource);
    invalid.components!['WorkbenchPanelHeader']['slots'] = {
      title: { required: false },
    };
    await expect(
      prepareComponentReviewHost(
        repo,
        panelHeader,
        invalid,
        panelDependencies,
        theme,
      ),
    ).rejects.toThrow('slot contract');
  });

  it('rejects an unmapped required component slot instead of exporting a blank specimen', async () => {
    const unsupported = cloneZuiDocument(source);
    unsupported.components = {
      UnmappedReviewComponent: {
        root: 'root',
        slots: { title: { required: true, multiple: false } },
      },
    };
    await expect(
      prepareComponentReviewHost(repo, strip, unsupported, dependencies, theme),
    ).rejects.toThrow('requires an explicit review host');
  });

  it('mounts real editor panes into the ActivityDrawerWindow slots', async () => {
    const activitySource = parseZuiDocument(
      await readFile(`${repo}/${activityWindow}`, 'utf8'),
    ).document;
    // Host construction is independent from the Penpot API.  A no-op embed
    // keeps this contract test focused on source views and slot ownership.
    const noOpDependencies = {
      embed: () => [],
    } as unknown as LayoutDependencies;
    const result = await prepareComponentReviewHost(
      repo,
      activityWindow,
      activitySource,
      noOpDependencies,
      theme,
    );
    expect(result).not.toBeNull();
    const host = parseZuiDocument(result!.source).document;
    expect(host.asset.kind).toBe('view');
    expect(host.nodes?.root?.component).toBe('ActivityDrawerWindow');
    const mountedText = Object.values(host.nodes ?? {}).flatMap((node) => {
      const text = node.props?.['text'];
      return typeof text === 'string' ? [text] : [];
    });
    expect(mountedText).toEqual(
      expect.arrayContaining([
        'Assets',
        'Asset Browser',
        'Runtime diagnostics',
      ]),
    );
    for (const id of [
      'palette',
      'hierarchy',
      'inspector',
      'style',
      'diagnostics',
      'debug',
      'content',
    ]) {
      expect(host.nodes?.[id]?.component).toBe('Container');
      expect(host.nodes?.[id]?.children?.length).toBe(1);
    }
    expect(result!.consumers.map(({ sourcePath }) => sourcePath)).toEqual(
      expect.arrayContaining([
        activityWindow,
        'zircon_editor/assets/ui/editor/assets_activity.zui',
        'zircon_editor/assets/ui/editor/asset_browser.zui',
        'zircon_editor/assets/ui/editor/ui_asset_editor.zui',
      ]),
    );
  });

  it('mounts the authored WorkbenchShell and Window without inventing frame state', async () => {
    const allSources = await trackedZuiPaths();
    const mainFrameDependencies = new LayoutDependencies();
    await mainFrameDependencies.load(repo, allSources);
    const mainFrameSource = parseZuiDocument(
      await readFile(`${repo}/${editorMainFrame}`, 'utf8'),
    ).document;
    const before = normalizeZuiDocument(mainFrameSource);

    const result = await prepareComponentReviewHost(
      repo,
      editorMainFrame,
      mainFrameSource,
      mainFrameDependencies,
      theme,
    );

    expect(result).not.toBeNull();
    const host = parseZuiDocument(result!.source).document;
    const windowSource = parseZuiDocument(
      await readFile(`${repo}/${workbenchWindow}`, 'utf8'),
    ).document;
    expect(host.asset.id).toBe(`${mainFrameSource.asset.id}#review-host`);
    expect(host.nodes?.['task_bar_slot']?.component).toBe('Container');
    expect(host.nodes?.['task_bar_slot']?.children).toHaveLength(0);
    expect(host.nodes?.['window_tab_strip_slot']?.component).toBe(
      'HorizontalGroup',
    );
    expect(host.nodes?.['window_tab_strip_slot']?.children).toHaveLength(0);
    expect(host.nodes?.['active_window_host_slot']?.component).toBe(
      'Container',
    );
    expect(host.nodes?.['active_window_host_slot']?.children).toHaveLength(1);
    expect(host.nodes?.['review_workbench_shell__host']?.component).toBe(
      'UiHostWindow',
    );
    expect(host.nodes?.['review_workbench_window__root']?.component).toBe(
      'Overlay',
    );
    expect(host.nodes?.['review_workbench_window__root']?.children).toHaveLength(
      windowSource.nodes?.['root']?.children?.length ?? 0,
    );
    expect(
      host.nodes?.['review_workbench_window__notification_center'],
    ).toBeDefined();
    expect(host.nodes?.['review_workbench_window__settings_window']).toBeDefined();
    expect(host.nodes?.['review_workbench_window__command_palette']).toBeDefined();
    expect(result!.source).not.toContain('penpot_review_source_path');
    expect(result!.consumers.map(({ sourcePath }) => sourcePath)).toEqual(
      expect.arrayContaining([
        editorMainFrame,
        workbenchShell,
        workbenchWindow,
        tab,
      ]),
    );
    expect(
      Object.values(result!.projection.nodes ?? {}).some(
        (node) => node.props?.['text'] === 'Scene',
      ),
    ).toBe(true);
    expect(
      result!.projection.nodes?.[
        'review_workbench_window__notification_center'
      ],
    ).toMatchObject({
      penpot_review_source_path: notificationCenter,
      penpot_review_source_node_id: 'root',
      control_id: 'WorkbenchNotificationCenter',
    });
    const settingsWindowProjection =
      result!.projection.nodes?.['review_workbench_window__settings_window'];
    expect(settingsWindowProjection).toMatchObject({
      penpot_review_source_path:
        'zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui',
      penpot_review_source_node_id: 'preferences',
    });
    expect(
      JSON.parse(
        settingsWindowProjection?.['penpot_review_instance_path'] as string,
      ),
    ).toEqual([
      { sourcePath: editorMainFrame, sourceNodeId: 'active_window_host_slot' },
      { sourcePath: workbenchShell, sourceNodeId: 'body' },
      { sourcePath: workbenchWindow, sourceNodeId: 'settings_window' },
    ]);
    expect(
      Object.values(result!.projection.nodes ?? {}).some(
        (node) => node.component === 'Slot',
      ),
    ).toBe(false);
    expect(normalizeZuiDocument(mainFrameSource)).toEqual(before);
    const original = {
      ...mainFrameSource,
      penpot_review_host: result!.projection,
    };
    expect(
      normalizeZuiDocument(
        reconcileReviewSource(
          original,
          cloneProjectionSnapshot(projectZuiDocument(reviewHost(original)!)),
        ).document,
      ),
    ).toEqual(normalizeZuiDocument(original));
    // Loading and parsing every source dependency is deliberately real-host
    // work.  Keep the test bounded while allowing Windows I/O contention.
  }, 120_000);

  it('allows a view consumer to carry an explicit design review host', () => {
    const source: ZuiDocument = {
      asset: { kind: 'view', id: 'res://ui/window.zui', version: 2 },
      root: { node: 'root' },
      nodes: { root: { component: 'Panel' } },
      penpot_review_host: {
        asset: {
          kind: 'view',
          id: 'res://ui/window.zui#review-host',
          version: 2,
        },
        root: { node: 'host' },
        nodes: { host: { component: 'Panel' } },
      },
    };

    expect(reviewHost(source)?.asset.kind).toBe('view');
  });
});
