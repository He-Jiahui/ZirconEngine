import { readFile } from 'node:fs/promises';
import { readFileSync } from 'node:fs';
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
  type ZuiNode,
} from './zui-document';
import { reconcileReviewSource, reviewHost } from './zui-review-host';
import { reviewDocumentForCase } from './zui-review-case';
import { segmentedGeometry } from './zui-segmented-geometry';

const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
const directory =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs';
const strip =
  'zircon_editor/assets/ui/editor/workbench/primitives/inputs/tab_strip.zui';
const tab = `${directory}/tab.zui`;
const diagnostic =
  'zircon_editor/assets/ui/editor/workbench/composites/feedback/diagnostic_row.zui';
const statusItem =
  'zircon_editor/assets/ui/editor/workbench/primitives/feedback/status_item.zui';
const panelHeader =
  'zircon_editor/assets/ui/editor/workbench/composites/chrome/panel_header.zui';
const sectionTitle =
  'zircon_editor/assets/ui/editor/workbench/primitives/chrome/section_title.zui';
const button = `${directory}/button.zui`;
const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const activityWindow =
  'zircon_editor/assets/ui/editor/workbench/shell/activity_drawer_window.zui';
const editorMainFrame =
  'zircon_editor/assets/ui/editor/host/editor_main_frame.zui';
const editorShell =
  'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
const levelEditorWindow =
  'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const notificationCenter =
  'zircon_editor/assets/ui/editor/workbench/feedback/notification_center.zui';
const tableRow =
  'zircon_editor/assets/ui/editor/workbench/primitives/data/table_row.zui';

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
    expect(table.nodes?.['root']?.props?.['layout_min_width']).toBe(360);
    const result = (await prepareComponentReviewHost(
      repo,
      tableRow,
      table,
      tableDependencies,
      theme,
    ))!;
    const host = parseZuiDocument(result.source).document;
    expect(host.nodes?.['review_scroll']?.component).toBe('ScrollableBox');
    expect(host.nodes?.['review_scroll']?.layout?.['container']).toEqual({
      kind: 'ScrollableBox',
      axis: 'Horizontal',
      gap: 0,
      scrollbar_visibility: 'Auto',
    });
    expect(host.nodes?.['review_scroll']?.children).toEqual([
      { node: 'review_component' },
    ]);
    expect(host.nodes?.['review_component']?.component).toBe(
      `${table.asset.id}#WorkbenchTableRow`,
    );
    expect(host.nodes?.['review_component']?.layout?.['width']).toEqual({
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
    expect(tabs[0].text?.fontSize).toBe(13.333333);
    expect(tabs[0].segmented?.colors.background).toBe('#242424');
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
    expect(focusTab.segmented?.colors.background).toBeNull();
    expect(segmentedGeometry(focusTab.segmented!, 120, 32).parts['focus-ring']).toMatchObject({
      fill: 'transparent',
      stroke: '#0070E0',
      strokeWidth: 1,
    });
    expect(segmentedGeometry(tabs[0].segmented!, 120, 32).parts['focus-ring']).toBeUndefined();
    expect(focusTab.text?.color).toBe('#C0C0C0');
    expect(
      segmentedGeometry(focusTab.segmented!, 120, 32).texts['primary'].x,
    ).toBe(8);
  });

  it.each([
    [{ focused: true, focus_visible: false }],
    [{ focused: true, focus_visible: true, disabled: true }],
  ])('keeps the authored Tab focus ring quiet for %j', (state) => {
    const document = parseZuiDocument(readFileSync(`${repo}/${tab}`, 'utf8')).document;
    dependencies.embed(document, tab, theme);
    document.nodes!['root'].state = state;
    const before = cloneZuiDocument(document);
    const projection = projectZuiDocument(document);
    const shape = projection.rootNodes[0];
    expect(segmentedGeometry(shape.segmented!, 120, 32).parts['focus-ring']).toBeUndefined();
    expect(reconcileReviewSource(document, cloneProjectionSnapshot(projection)).changes).toEqual([]);
    expect(document).toEqual(before);
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
    expect(host.nodes!['review_action'].layout?.['height']).toMatchObject({
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
    expect(host.nodes?.['root']?.component).toBe('ActivityDrawerWindow');
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


  it('mounts the authored Unreal shell, Level Editor and owned dock components', async () => {
    const allSources = await trackedZuiPaths();
    const mainFrameDependencies = new LayoutDependencies();
    await mainFrameDependencies.load(repo, allSources);
    const mainFrameSource = parseZuiDocument(
      await readFile(repo + '/' + editorMainFrame, 'utf8'),
    ).document;
    const before = normalizeZuiDocument(mainFrameSource);
    const result = await prepareComponentReviewHost(
      repo, editorMainFrame, mainFrameSource, mainFrameDependencies, theme,
    );
    expect(result).not.toBeNull();
    const host = parseZuiDocument(result!.source).document;
    const windowSource = parseZuiDocument(
      await readFile(repo + '/' + levelEditorWindow, 'utf8'),
    ).document;
    expect(host.asset.id).toBe(mainFrameSource.asset.id + '#review-host');
    expect(host.nodes?.['task_bar_slot']).toBeUndefined();
    expect(host.nodes?.['window_tab_strip_slot']).toBeUndefined();
    expect(host.nodes?.['active_window_host_slot']?.children).toHaveLength(1);
    expect(host.nodes?.['review_editor_shell__host']?.component).toBe('UiHostWindow');
    expect(host.nodes?.['review_level_editor_window__root']?.component).toBe('Overlay');
    expect(host.nodes?.['review_level_editor_window__root']?.children).toHaveLength(
      windowSource.nodes?.['root']?.children?.length ?? 0,
    );
    expect(result!.consumers.map(({ sourcePath }) => sourcePath)).toEqual(
      expect.arrayContaining([
        editorMainFrame, editorShell, levelEditorWindow,
        'zircon_editor/assets/ui/editor/unreal/components/menus.zui',
        'zircon_editor/assets/ui/editor/unreal/components/toolbar.zui',
        'zircon_editor/assets/ui/editor/unreal/components/viewport.zui',
        'zircon_editor/assets/ui/editor/unreal/components/outliner.zui',
        'zircon_editor/assets/ui/editor/unreal/components/details.zui',
        'zircon_editor/assets/ui/editor/unreal/components/status_bar.zui',
        'zircon_editor/assets/ui/editor/unreal/components/content_drawer.zui',
        'zircon_editor/assets/ui/editor/unreal/components/menu_popups.zui',
      ]),
    );
    const projected = Object.values(result!.projection.nodes ?? {});
    const detailsTitle = projected.find(
      (node) => node.control_id === 'UnrealDetailsDockTitle',
    )!;
    expect(detailsTitle).toMatchObject({
      penpot_review_source_path: 'zircon_editor/assets/ui/editor/unreal/components/details.zui',
      penpot_review_source_node_id: 'title',
    });
    expect(projected.some((node) => node.control_id === 'UnrealRightDock')).toBe(true);
    expect(projected.some((node) => node.component === 'Slot')).toBe(false);
    expect(projected.some((node) => node.control_id === 'WorkbenchNotificationCenter')).toBe(false);
    expect(normalizeZuiDocument(mainFrameSource)).toEqual(before);
    const original = { ...mainFrameSource, penpot_review_host: result!.projection };
    expect(normalizeZuiDocument(reconcileReviewSource(
      original, cloneProjectionSnapshot(projectZuiDocument(reviewHost(original)!)),
    ).document)).toEqual(normalizeZuiDocument(original));
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

describe('actual consumed Unreal Header source projection', () => {
  it('mounts one consumed Header above the original viewport, preserves Source ownership and used slots, and rejects source drift', async () => {
    const { createHash } = await import('node:crypto');
    const { prepareUnrealDockHeaderViews, UNREAL_DOCK_HEADER_SOURCE } =
      await import('../../tools/zui-layout-unreal-dock-headers');
    const { canonicalSha256, validateWorkbenchPresentation } =
      await import('../../tools/zui-layout-workbench-presentation');
    const header = parseZuiDocument(await readFile(repo + '/' + UNREAL_DOCK_HEADER_SOURCE, 'utf8')).document;
    const paths = [UNREAL_DOCK_HEADER_SOURCE, 'zircon_editor/assets/ui/editor/unreal/starship.zui',
      'zircon_editor/assets/ui/editor/theme/editor_tokens.zui'];
    const sourceFingerprints = await Promise.all(paths.map(async (sourcePath) => ({
      sourcePath, sha256: createHash('sha256').update(await readFile(repo + '/' + sourcePath)).digest('hex'),
    })));
    // Unit observations are fixture inputs, never normal-frame acceptance evidence.
    const observation = (count: number) => {
      const document = cloneZuiDocument(header);
      document.nodes!['dock_tab_strip'].children = document.nodes!['dock_tab_strip'].children!.slice(0, count);
      const erase = (id: string): void => {
        for (const child of document.nodes![id].children ?? []) erase(child.node);
        delete document.nodes![id];
      };
      for (let index = count; index < 3; index++) erase('dock_tab_group_' + index);
      for (let index = 0; index < count; index++) {
        for (const id of ['dock_tab_group_' + index, 'dock_tab_' + index])
          document.nodes![id].props = { ...document.nodes![id].props, selected: index === 0,
            surface_variant: index === 0 ? 'inset' : 'transparent', text_tone: index === 0 ? 'default' : 'subtle' };
        document.nodes!['dock_tab_' + index].props!['text'] = 'Actual long caption ' + index;
      }
      const frame = { x: 0, y: 0, width: 900, height: 31 };
      return { kind: 'document', surfaceKey: 'fixture-document', assetId: header.asset.id,
        sourcePath: UNREAL_DOCK_HEADER_SOURCE, sourceDocument: document, viewport: { width: 900, height: 31 },
        mount: { region: frame, header: frame }, sourceFingerprints,
        rows: Object.entries(document.nodes!).map(([id, node]) => ({
          controlId: node.control_id, sourcePath: UNREAL_DOCK_HEADER_SOURCE, sourceNodeId: id, instancePath: [],
          role: node.component, text: node.props?.['text'] ?? '', selected: node.props?.['selected'] === true,
          hovered: false, pressed: false, disabled: false, frame, renderRef: null,
        })),
      };
    };
    const one = observation(1);
    const first = (await prepareUnrealDockHeaderViews(repo, { dockHeaders: [one] }))[0].document;
    expect(first.nodes!['dock_tab_strip'].children).toHaveLength(1);
    expect(first.nodes!['dock_tab_group_1']).toBeUndefined();
    expect(first.nodes!['dock_tab_0'].props?.['text']).toBe('Actual long caption 0');
    expect(first.nodes!['dock_tab_0'].props?.['selected']).toBe(true);
    expect(first.nodes!['dock_tab_0']['penpot_review_source_path']).toBe(UNREAL_DOCK_HEADER_SOURCE);
    expect(first.nodes!['dock_tab_0']['penpot_review_source_node_id']).toBe('dock_tab_0');
    expect(first.nodes!['dock_tab_0']['penpot_review_instance_path']).toBe('[]');
    const restored = (await prepareUnrealDockHeaderViews(repo, { dockHeaders: [observation(3)] }))[0].document;
    expect(restored.nodes!['dock_tab_strip'].children).toHaveLength(3);
    expect(restored.nodes!['dock_tab_2'].props?.['text']).toBe('Actual long caption 2');
    const viewportSource = 'zircon_editor/assets/ui/editor/unreal/components/viewport.zui';
    const dependencies = new LayoutDependencies();
    await dependencies.load(repo, await trackedZuiPaths());
    const mainFrameSource = parseZuiDocument(await readFile(repo + '/' + editorMainFrame, 'utf8')).document;
    const before = normalizeZuiDocument(mainFrameSource);
    // Match the complete empty-workspace DTO in zui-layout-strict-parity.spec.ts.
    const presentation = {
      schema: 'dev.zircon.editor.workbench-presentation', version: 1, activeLocale: 'en',
      sourceFingerprint: {
        sourcePath: levelEditorWindow,
        sha256: createHash('sha256').update(await readFile(repo + '/' + levelEditorWindow)).digest('hex'),
      },
      stateFingerprint: '',
      viewportSnap: { translationEnabled: false, rotationEnabled: true, scaleEnabled: false },
      layout: {
        active_main_page: 'main',
        main_pages: [{ WorkbenchPage: { id: 'main', title: 'Workbench', activity_window: 'main-window' } }],
        activity_windows: {
          'main-window': {
            window_id: 'main-window', descriptor_id: 'editor.main', host_mode: 'EmbeddedMainFrame',
            activity_drawers: {},
            content_workspace: { Tabs: { node_id: 'document-tabs', tabs: [], active_tab: null } },
            menu_overflow_mode: 'Auto', region_overrides: {}, view_overrides: {},
          },
        },
        floating_windows: [],
      },
      window: { id: 'main-window', title: 'Zircon Editor' },
      pages: { activeId: 'main', items: [{ id: 'main', title: 'Workbench', activityWindowId: 'main-window' }] },
      documents: { activeId: null, items: [] }, drawers: [],
      hierarchy: { filterQuery: '', expandedIds: [], selectedIds: [], rows: [] },
      inspector: {
        entityId: null, name: '', parent: '', translation: ['0', '0', '0'], scale: ['1', '1', '1'],
        renderLayerMask: 0, components: [],
      },
      status: { primary: '', secondary: null, viewportLabel: '', projectPath: '' },
      dockHeaders: [],
    };
    const { stateFingerprint: plainFingerprint, ...plainState } = presentation;
    void plainFingerprint;
    presentation.stateFingerprint = canonicalSha256(plainState);
    validateWorkbenchPresentation(presentation);
    const withHeader = { ...presentation, dockHeaders: [one] };
    const { stateFingerprint: mountedFingerprint, ...mountedState } = withHeader;
    void mountedFingerprint;
    withHeader.stateFingerprint = canonicalSha256(mountedState);
    validateWorkbenchPresentation(withHeader);
    const plain = (await prepareComponentReviewHost(
      repo, editorMainFrame, mainFrameSource, dependencies, theme, presentation,
    ))!;
    const originals = new Map<string, { node: ZuiNode; baseline: ZuiNode; children: NonNullable<ZuiNode['children']> }>();
    const embed = dependencies.embed.bind(dependencies);
    dependencies.embed = (...args) => {
      const result = embed(...args);
      if (args[1] === levelEditorWindow)for (const [id, node] of Object.entries(args[0].nodes ?? {})) {
        if (node['penpot_review_source_path'] === viewportSource)
          originals.set(id, { node, baseline: structuredClone(plain.projection.nodes![id]), children: [...(node.children ?? [])] });
      }
      return result;
    };
    const mounted = (await prepareComponentReviewHost(
      repo, editorMainFrame, mainFrameSource, dependencies, theme, withHeader,
    ))!;
    expect(mounted.source).toBe(plain.source);
    expect(normalizeZuiDocument(mainFrameSource)).toEqual(before);
    const nodes = mounted.projection.nodes!;
    const viewportEntries = Object.entries(nodes).filter(([, node]) =>
      node['penpot_prefab_source'] === viewportSource + '#UnrealLevelViewport');
    expect(viewportEntries).toHaveLength(1);
    const [viewportId, viewport] = viewportEntries[0];
    expect(viewport.component).toBe('VerticalGroup');
    expect(viewport.control_id).toBe('WorkbenchMainBandViewportPanel');
    expect(viewport.layout?.['container']).toEqual({ kind: 'VerticalBox', gap: 0 });
    expect(viewport.layout?.['height']).toEqual({ stretch: 'Stretch' });
    expect(viewport.children).toHaveLength(3);
    const [headerMount, separatorMount, innerMount] = viewport.children!;
    expect(nodes[headerMount.node]).toMatchObject({
      control_id: 'DockHeaderRoot', penpot_review_source_path: UNREAL_DOCK_HEADER_SOURCE,
      penpot_review_source_node_id: header.root!.node, penpot_review_instance_path: '[]',
    });
    expect(nodes[headerMount.node].layout?.['height']).toEqual({ min: 31, preferred: 31, max: 31, stretch: 'Fixed' });
    expect(nodes[separatorMount.node].component).toBe('Separator');
    expect(nodes[separatorMount.node].props?.['penpot_generated_host_structure']).toBe(true);
    expect(nodes[headerMount.node].props?.['penpot_generated_host_structure']).toBeUndefined();
    expect(viewport.props?.['penpot_generated_host_structure']).toBeUndefined();
    expect(nodes[separatorMount.node].layout?.['height']).toEqual({ min: 1, preferred: 1, max: 1, stretch: 'Fixed' });
    expect(nodes[separatorMount.node]['penpot_review_source_path']).toBeUndefined();
    expect(nodes[separatorMount.node]['penpot_review_source_node_id']).toBeUndefined();
    expect(innerMount).toBe(originals.get(viewportId)!.children[0]);
    expect(nodes[innerMount.node]).toMatchObject({ component: 'Overlay', control_id: 'UnrealSceneViewViewport' });
    expect(nodes[innerMount.node].layout?.['height']).toEqual({ stretch: 'Stretch' });
    for (const [id, original] of originals) {
      expect(nodes[id]).toBe(original.node);
      if (id === viewportId) {
        expect(nodes[id].events).toEqual(original.baseline.events);
        continue;
      }
      expect(nodes[id]).toEqual(original.baseline);
      expect(nodes[id].children ?? []).toHaveLength(original.children.length);
      original.children.forEach((child, index) => expect(nodes[id].children![index]).toBe(child));
    }
    const projected = Object.values(nodes);
    expect(projected.filter((node) => node.control_id === 'DockHeaderRoot')).toHaveLength(1);
    expect(projected.filter((node) => node.control_id === 'DockHeaderBar')).toHaveLength(1);
    expect(projected.filter((node) => node.component === 'SceneViewViewport')).toHaveLength(1);
    expect(projected.filter((node) => node.control_id === 'WorkbenchViewportSurface')).toHaveLength(1);
    const context = projected.find((node) => node.control_id === 'UnrealProjectDocumentContext')!;
    expect(context.component).toBe('HorizontalGroup');
    expect(context.children).toHaveLength(1);
    expect(nodes[context.children![0].node].control_id).toBe('UnrealProjectDocumentContextText');
    expect(Object.keys(parseZuiDocument(mounted.source).document.nodes!))
      .not.toContain(headerMount.node);
    expect(Object.keys(parseZuiDocument(mounted.source).document.nodes!))
      .not.toContain(separatorMount.node);
    one.sourceDocument.nodes!['dock_tab_group_0'].layout!['width'] = { min: 1, max: 2 };
    await expect(prepareUnrealDockHeaderViews(repo, { dockHeaders: [one] }))
      .rejects.toThrow('differs from the current authored');
  }, 120_000);
});
