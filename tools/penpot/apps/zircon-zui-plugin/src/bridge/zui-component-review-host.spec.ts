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
import {
  prepareRealOwnerInput, prepareRealOwnerGapEdit, prepareRealWorkbenchInput,
  validateRealOwnerExportSources, validateRealOwnerGapExport, validateRealWorkbenchExportSources,
} from '../../tools/zui-layout-real-workbench-input';
import { parseSourceExportManifest, type ZuiSourceExportManifest } from './zui-export-apply-contract';
import { applyZuiExportToSourceFiles } from './zui-export-apply-sources';
import { sourceOwnerSpacingNode } from '../../tools/penpot-workbench-canvas-edit';
import { sha256 } from '../../tools/zui-layout-real-session-contract.mjs';
import { applyObservedWorkbenchMenuAndRunState } from '../../tools/zui-layout-workbench-projection';
import { sourceNodeByControl } from '../../tools/zui-layout-source-control';
import type { ToolbarSourceNode } from './zui-viewport-toolbar-publication';

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

  it('prepares a component owner without borrowing a Level Editor case or source identity', async () => {
    const original = await readFile(`${repo}/${button}`, 'utf8');
    const prepared = await prepareRealOwnerInput(repo, `${repo}/${button}`,
      original, parseZuiDocument(original).document);
    expect(prepared.input['penpot_original_source_path']).toBe(button);
    expect(prepared.input['penpot_original_source']).toBe(original);
    expect(prepared.sourceFingerprints).toEqual(expect.arrayContaining([
      {sourcePath: button, sha256: sha256(original)},
    ]));
    expect(prepared.sourceFingerprints.some((item) => item.sourcePath === levelEditorWindow)).toBe(false);
    expect(prepared.managedPresentationPrepared).toBe(false);
    const projection = prepared.input['penpot_review_host'] as ZuiDocument;
    expect(projection.root?.node).toBeTruthy();
    expect(projection['penpot_actual_normal_review_case']).toBeUndefined();
  });

  it('rejects stale owner bytes before preparing an authored family closure', async () => {
    const original = await readFile(`${repo}/${button}`, 'utf8');
    await expect(prepareRealOwnerInput(repo, `${repo}/${button}`,
      original + '\n', parseZuiDocument(original).document)).rejects.toThrow('source owner changed');
  });

  it.each([
    'zircon_editor/assets/ui/editor/asset_browser.zui',
    'zircon_editor/assets/ui/editor/windows/asset_window.zui',
    'zircon_editor/assets/ui/editor/unreal/components/content_drawer.zui',
    'zircon_editor/assets/ui/editor/modules/data/extension_data_table_workspace.zui',
  ])('preserves the resource owner contract without borrowing a Level Editor case: %s', async (sourcePath) => {
    const source = await readFile(`${repo}/${sourcePath}`, 'utf8');
    const original = parseZuiDocument(source).document;
    const before = normalizeZuiDocument(original);
    const prepared = await prepareRealOwnerInput(repo, `${repo}/${sourcePath}`, source, original);
    const reparsed = parseZuiDocument(prepared.source).document;
    expect(normalizeZuiDocument(original)).toEqual(before);
    expect(reparsed.asset).toEqual(original.asset);
    expect(reparsed.root).toEqual(original.root);
    expect(reparsed.components).toEqual(original.components);
    expect(reparsed.nodes).toEqual(original.nodes);
    expect(reparsed.imports).toEqual(original.imports);
    expect(reparsed['penpot_original_source']).toBe(source);
    expect(reparsed['penpot_original_source_path']).toBe(sourcePath);
    expect(prepared.sourceFingerprints).toContainEqual({ sourcePath, sha256: sha256(source) });
    expect(prepared.sourceFingerprints.some((entry) => entry.sourcePath === levelEditorWindow)).toBe(false);
    expect(prepared.managedPresentationPrepared).toBe(false);
    expect(prepared.reviewCase).toBeUndefined();
    const host = reviewHost(reparsed)!;
    expect(host.asset.kind).toBe('view');
    expect(host.nodes?.[host.root!.node]).toBeDefined();
    expect(host['penpot_actual_normal_review_case']).toBeUndefined();
    const manifest: ZuiSourceExportManifest = {
      schema: 'dev.zircon.zui.penpot-source-exports', version: 1,
      rootSourcePath: sourcePath, baselineProjection: '', exportedProjection: '',
      identityMap: [], edits: [], sources: prepared.sourceFingerprints,
    };
    expect(validateRealOwnerExportSources(manifest, sourcePath,
      prepared.sourceFingerprints, sha256(source))).toBe(prepared.sourceFingerprints.length);
    expect(() => validateRealWorkbenchExportSources(manifest,
      prepared.sourceFingerprints, sha256(source))).toThrow('original root owner');
  });

  it('keeps Drawer authored defaults and virtual-row events in its original component invocation', async () => {
    const sourcePath = 'zircon_editor/assets/ui/editor/unreal/components/content_drawer.zui';
    const source = await readFile(`${repo}/${sourcePath}`, 'utf8');
    const original = parseZuiDocument(source).document;
    const prepared = await prepareRealOwnerInput(repo, `${repo}/${sourcePath}`, source, original);
    const host = reviewHost(prepared.input)!;
    const root = host.nodes?.[host.root!.node];
    expect(root?.['penpot_review_source_path']).toBe(sourcePath);
    expect(root?.['penpot_review_source_node_id']).toBe('drawer');
    expect(root?.control_id).toBe('UnrealContentDrawer');
    expect(root?.props?.['visible']).toBe(false);
    expect(root?.props?.['visibility']).toBe('collapsed');
    expect(root?.['penpot_prefab_source']).toBe(`${sourcePath}#UnrealContentDrawer`);
    const body = Object.values(host.nodes ?? {}).find((node) =>
      node['penpot_review_source_path'] === sourcePath && node['penpot_review_source_node_id'] === 'content');
    expect(body?.repeat).toEqual(original.nodes?.['content'].repeat);
    const events = Object.values(host.nodes ?? {}).filter((node) =>
      node['penpot_review_source_path'] === sourcePath).flatMap((node) => node.events ?? []);
    expect(events).toEqual(expect.arrayContaining(
      Object.values(original.nodes ?? {}).flatMap((node) => node.events ?? []),
    ));
    expect(Object.values(original.nodes ?? {}).flatMap((node) => node.events ?? [])).toHaveLength(5);
  });

  it.each([
    ['zircon_editor/assets/ui/editor/workbench/primitives/data/timeline_strip.zui',
      'WorkbenchTimelineStripRoot'],
    ['zircon_editor/assets/ui/editor/workbench/animation/transport_controls.zui',
      'WorkbenchTransportControlsRoot'],
  ])('keeps the original animation owner and events when preparing %s', async (sourcePath, rootControlId) => {
    const source = await readFile(`${repo}/${sourcePath}`, 'utf8');
    const document = parseZuiDocument(source).document;
    const prepared = await prepareRealOwnerInput(repo, `${repo}/${sourcePath}`, source,
      document, { themeSourcePath: theme });
    expect(prepared.input.asset).toEqual(document.asset);
    expect(prepared.input['penpot_original_source']).toBe(source);
    expect(prepared.input['penpot_original_source_path']).toBe(sourcePath);
    expect(prepared.themeSourcePath).toBe(theme);
    expect(prepared.sourceFingerprints).toEqual(expect.arrayContaining([
      {sourcePath, sha256: sha256(source)},
      {sourcePath: theme, sha256: sha256(await readFile(`${repo}/${theme}`))},
    ]));
    expect(prepared.sourceFingerprints.some((item) => item.sourcePath === levelEditorWindow)).toBe(false);
    expect(prepared.reviewCase).toBeUndefined();
    expect(prepared.managedPresentationPrepared).toBe(false);
    const projection = prepared.input['penpot_review_host'] as ZuiDocument;
    expect(Object.values(projection.nodes ?? {}).some((node) => node.control_id === rootControlId)).toBe(true);
    const exportedEvents = Object.values(projection.nodes ?? {}).flatMap((node) => node.events ?? []);
    for (const event of Object.values(document.nodes ?? {}).flatMap((node) => node.events ?? []))
      expect(exportedEvents).toContainEqual(event);
    if (rootControlId === 'WorkbenchTimelineStripRoot') {
      expect(Object.values(document.nodes ?? {})).toHaveLength(1);
      expect(document.nodes?.['root']?.component).toBe('Canvas');
      expect(document.nodes?.['root']?.props?.['input_interactive']).toBe(false);
    } else {
      expect(Object.values(document.nodes ?? {}).map((node) => node.control_id))
        .toContain('WorkbenchTransportFill');
      expect(Object.values(document.nodes ?? {}).flatMap((node) => node.events ?? [])).toHaveLength(6);
    }
  });

  it('rejects a changed loaded dependency instead of fingerprinting a newer file as the prepared source', async () => {
    const source = await readFile(`${repo}/${button}`, 'utf8');
    const originalFingerprint = LayoutDependencies.prototype.sourceFingerprint;
    const spy = vi.spyOn(LayoutDependencies.prototype, 'sourceFingerprint').mockImplementation(
      function (this: LayoutDependencies, sourcePath: string) {
        const loaded = originalFingerprint.call(this, sourcePath);
        return sourcePath === theme && loaded ? {...loaded, sha256: '0'.repeat(64)} : loaded;
      });
    try {
      await expect(prepareRealOwnerInput(repo, `${repo}/${button}`, source,
        parseZuiDocument(source).document)).rejects.toThrow('source changed while preparing');
    } finally { spy.mockRestore(); }
  });

  it('rejects a theme outside the admitted source owner scope', async () => {
    const source = await readFile(`${repo}/${button}`, 'utf8');
    await expect(prepareRealOwnerInput(repo, `${repo}/${button}`, source,
      parseZuiDocument(source).document, {themeSourcePath: '../other/theme.zui'})).rejects.toThrow();
  });

  it('validates the prepared family owner and refuses root substitution and changed closure', () => {
    const fingerprint = {sourcePath: button, sha256: 'a'.repeat(64)};
    const envelope: ZuiSourceExportManifest = {
      schema: 'dev.zircon.zui.penpot-source-exports', version: 1,
      rootSourcePath: button, baselineProjection: '', exportedProjection: '',
      identityMap: [], edits: [], sources: [fingerprint],
    };
    expect(validateRealOwnerExportSources(envelope, button, [fingerprint], fingerprint.sha256)).toBe(1);
    expect(() => validateRealOwnerExportSources({...envelope, rootSourcePath: levelEditorWindow},
      button, [fingerprint], fingerprint.sha256)).toThrow('original root owner');
    expect(() => validateRealOwnerExportSources({...envelope,
      sources: [{...fingerprint, sha256: 'b'.repeat(64)}]}, button,
      [fingerprint], fingerprint.sha256)).toThrow();
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
    const browserSource = 'zircon_editor/assets/ui/editor/asset_browser.zui';
    const browserNodes = Object.values(host.nodes ?? {}).filter((node) =>
      node['penpot_review_source_path'] === browserSource);
    expect(browserNodes.length).toBeGreaterThan(0);
    for (const node of browserNodes) {
      expect(node['penpot_review_source_node_id']).not.toMatch(/^review_/);
      expect(node['penpot_review_instance_path']).toBe('[]');
    }
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
  it('actual host state projection retains the owned viewport and panel Header heights without changing verified Source; actual missing semantic icons retain their business identity; authored disabled Inspector filter remains Source owned', async () => {
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
    const starship = parseZuiDocument(await readFile(repo + '/' + paths[1], 'utf8')).document;
    const editorTokens = parseZuiDocument(await readFile(repo + '/' + paths[2], 'utf8')).document;
    // The actual native file cache retains the Header stylesheet first and
    // appends its breadth-first loaded imports. Theme-token expansion belongs
    // to later rendering; it is not part of the captured Source document.
    const loadedHeader = cloneZuiDocument(header);
    loadedHeader.tokens = { ...header.tokens, ...starship.tokens, ...editorTokens.tokens };
    loadedHeader.stylesheets = [
      ...(header.stylesheets ?? []), ...(starship.stylesheets ?? []), ...(editorTokens.stylesheets ?? []),
    ];
    // Unit observations are fixture inputs, never normal-frame acceptance evidence.
    const observation = (count: number) => {
      const document = cloneZuiDocument(loadedHeader);
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
    const capturedSource = cloneZuiDocument(one.sourceDocument);
    const first = (await prepareUnrealDockHeaderViews(repo, { dockHeaders: [one] }))[0].document;
    expect(one.sourceDocument).toEqual(capturedSource);
    expect(first.tokens).toEqual(loadedHeader.tokens);
    expect(first.stylesheets).toEqual(loadedHeader.stylesheets);
    expect(first.stylesheets!.map((sheet) => sheet['id']))
      .toEqual(['unreal_dock_header_states', 'unreal_starship_editor']);
    expect(first.nodes!['dock_tab_strip'].children).toHaveLength(1);
    expect(first.nodes!['dock_tab_group_1']).toBeUndefined();
    expect(first.nodes!['dock_tab_0'].props?.['text']).toBe('Actual long caption 0');
    expect(first.nodes!['dock_tab_0'].props?.['selected']).toBe(true);
    expect(first.nodes!['dock_tab_0']['penpot_review_source_path']).toBe(UNREAL_DOCK_HEADER_SOURCE);
    expect(first.nodes!['dock_tab_0']['penpot_review_source_node_id']).toBe('dock_tab_0');
    expect(first.nodes!['dock_tab_0']['penpot_review_instance_path']).toBe('[]');
    const changedToken = observation(1);
    changedToken.sourceDocument.tokens!['unreal_primary'] = '#ff00ff';
    await expect(prepareUnrealDockHeaderViews(repo, { dockHeaders: [changedToken] }))
      .rejects.toThrow('differs from the current authored');
    const changedStyle = observation(1);
    const rules = changedStyle.sourceDocument.stylesheets![1]['rules'] as
      Array<{ set: { self: { background_color: string } } }>;
    rules[0].set.self['background_color'] = '#123456';
    await expect(prepareUnrealDockHeaderViews(repo, { dockHeaders: [changedStyle] }))
      .rejects.toThrow('differs from the current authored');
    const changedFingerprint = structuredClone(observation(1));
    changedFingerprint.sourceFingerprints.find(({ sourcePath }) => sourcePath === paths[1])!.sha256 = '0'.repeat(64);
    await expect(prepareUnrealDockHeaderViews(repo, { dockHeaders: [changedFingerprint] }))
      .rejects.toThrow('Actual Header loaded source changed: ' + paths[1]);
    const missingTransitiveSource = observation(1);
    missingTransitiveSource.sourceFingerprints = sourceFingerprints.filter(({ sourcePath }) => sourcePath !== paths[2]);
    await expect(prepareUnrealDockHeaderViews(repo, { dockHeaders: [missingTransitiveSource] }))
      .rejects.toThrow('loaded import closure is missing: ' + editorTokens.asset.id);
    const missingLoadedStyles = observation(1);
    missingLoadedStyles.sourceDocument.stylesheets = cloneZuiDocument(header).stylesheets;
    await expect(prepareUnrealDockHeaderViews(repo, { dockHeaders: [missingLoadedStyles] }))
      .rejects.toThrow('differs from the current authored');
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
    const inspectorSource = 'zircon_editor/assets/ui/editor/unreal/components/details.zui';
    const filterNode = (document: ZuiDocument) => {
      // Embedding preserves the caller control and full invocation, while the
      // authored triple belongs to the actual WorkbenchSearchInput prefab root.
      const node = sourceNodeByControl(document, inspectorSource, 'WorkbenchInspectorFilter');
      expect(node.component).toBe('SearchField');
      expect(node['penpot_review_source_path']).toBe('zircon_editor/assets/ui/editor/workbench/primitives/inputs/search_input.zui');
      expect(node['penpot_review_source_node_id']).toBe('root');
      expect(JSON.parse(node['penpot_review_instance_path'] as string).slice(-2)).toEqual([
        { sourcePath: levelEditorWindow, sourceNodeId: 'details' },
        { sourcePath: inspectorSource, sourceNodeId: 'search' },
      ]);
      return node;
    };
    // The actual Source intentionally disables this search until its business
    // implementation exists. A non-null Inspector snapshot only controls visibility.
    expect(filterNode(plain.projection).props?.['disabled']).toBe(true);
    const originalInspectorEmbed = dependencies.embed;
    const embedInspector = originalInspectorEmbed.bind(dependencies);
    try {
      for (const disabled of [true, false, undefined]) for (const inspector of [presentation.inspector, null]) {
        const state = { ...presentation, inspector };
        const { stateFingerprint: ignoredFingerprint, ...stateInput } = state;
        void ignoredFingerprint;
        state.stateFingerprint = canonicalSha256(stateInput);
        let configured = 0;
        dependencies.embed = (...args) => {
          const result = embedInspector(...args);
          if (args[1] === levelEditorWindow) {
            const filter = filterNode(args[0]);
            if (disabled === undefined) delete filter.props!['disabled'];
            else filter.props!['disabled'] = disabled;
            configured++;
          }
          return result;
        };
        // Each run starts with the actual unconsumed virtual-row prototype;
        // official preparation applies the business projection exactly once.
        const prepared = (await prepareComponentReviewHost(
          repo, editorMainFrame, mainFrameSource, dependencies, theme, state,
        ))!;
        expect(configured).toBe(1);
        const projection = prepared.projection;
        expect(filterNode(projection).props?.['disabled']).toBe(disabled);
        expect(Object.hasOwn(filterNode(projection).props!, 'disabled')).toBe(disabled !== undefined);
        expect(filterNode(projection).props?.['visibility']).toBe(inspector === null ? 'collapsed' : 'visible');
      }
    } finally {
      dependencies.embed = originalInspectorEmbed;
    }
    expect(normalizeZuiDocument(mainFrameSource)).toEqual(before);
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
    expect(viewport.children).toHaveLength(1);
    const [innerMount] = viewport.children!;
    expect(viewport.props?.['penpot_generated_host_structure']).toBeUndefined();
    expect(innerMount).toBe(originals.get(viewportId)!.children[0]);
    expect(nodes[innerMount.node]).toMatchObject({ component: 'Overlay', control_id: 'UnrealSceneViewViewport' });
    expect(nodes[innerMount.node].layout?.['height']).toEqual({ stretch: 'Stretch' });
    for (const [id, original] of originals) {
      expect(nodes[id]).toBe(original.node);
      expect(nodes[id]).toEqual(original.baseline);
      expect(nodes[id].children ?? []).toHaveLength(original.children.length);
      original.children.forEach((child, index) => expect(nodes[id].children![index]).toBe(child));
    }
    const projected = Object.values(nodes);
    expect(projected.filter((node) => node.control_id === 'DockHeaderRoot')).toHaveLength(0);
    expect(projected.filter((node) => node.control_id === 'DockHeaderBar')).toHaveLength(0);
    expect(projected.filter((node) => node.props?.['penpot_generated_host_structure'] === true)).toHaveLength(0);
    expect(projected.filter((node) => node.component === 'SceneViewViewport')).toHaveLength(1);
    expect(projected.filter((node) => node.control_id === 'WorkbenchViewportSurface')).toHaveLength(1);
    const context = projected.find((node) => node.control_id === 'UnrealProjectDocumentContext')!;
    expect(context.component).toBe('HorizontalGroup');
    expect(context.children).toHaveLength(1);
    expect(nodes[context.children![0].node].control_id).toBe('UnrealProjectDocumentContextText');
    expect(mounted.projection['penpot_review_workbench_presentation']).toEqual(withHeader);
    expect(one.sourceDocument).toEqual(capturedSource);

    // A cached document Header is skipped only for the exact authored media
    // invocation and parent chain. An almost matching viewport must fail.
    const originalEmbed = dependencies.embed;
    for (const mutate of [
      (document: ZuiDocument) => { const node = Object.values(document.nodes!).find((item) =>
        item.control_id === 'WorkbenchViewportSurface')!; node['penpot_review_source_node_id'] = 'wrong_media'; },
      (document: ZuiDocument) => { const node = Object.values(document.nodes!).find((item) =>
        item.control_id === 'UnrealSceneViewViewport')!; node.children = [...node.children!, node.children![0]]; },
      (document: ZuiDocument) => { const node = Object.values(document.nodes!).find((item) =>
        item.control_id === 'WorkbenchMainBandViewportPanel')!;
        const steps = JSON.parse(node['penpot_review_instance_path'] as string) as Array<{ sourcePath: string; sourceNodeId: string }>;
        steps[steps.length - 1].sourceNodeId = 'other_viewport'; node['penpot_review_instance_path'] = JSON.stringify(steps); },
    ]) {
      dependencies.embed = (...args) => {
        const result = embed(...args);
        if (args[1] === levelEditorWindow) mutate(args[0]);
        return result;
      };
      await expect(prepareComponentReviewHost(repo, editorMainFrame, mainFrameSource, dependencies, theme, withHeader))
        .rejects.toThrow(/authored mount|authored viewport/);
    }
    dependencies.embed = originalEmbed;

    // The published Header remains a verified Source dependency, even when
    // Native's authored viewport bypasses its painter. These are unit inputs.
    const ownerSource = await readFile(repo + '/' + levelEditorWindow, 'utf8');
    const rootHash = createHash('sha256').update(ownerSource).digest('hex');
    const ownerSets: Set<string>[] = [];
    for (const state of [presentation, withHeader]) {
      const prepared = await prepareRealWorkbenchInput(
        repo, repo + '/' + levelEditorWindow, ownerSource,
        parseZuiDocument(ownerSource).document, false, {
          id: 'unit-header-owner-export', sourcePath: levelEditorWindow, host: 'editor',
          viewport: { width: 900, height: 620 }, dpi: 1, locale: 'en-US', state: 'default',
          data: { workbenchPresentation: state }, themeSourcePath: theme,
        },
      );
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(reviewHost(prepared.input)!));
      const edit = snapshot.shapes.find((shape) => shape.sourcePath === levelEditorWindow &&
        shape.sourceNodeId === 'editor_content')!;
      expect(edit).toBeDefined();
      edit.current.container.rowGap += 1;
      const exported = reconcileReviewSource(prepared.input, snapshot);
      const envelope = parseSourceExportManifest(exported.document['penpot_source_exports']);
      const count = validateRealWorkbenchExportSources(envelope, prepared.sourceFingerprints, rootHash);
      expect(count).toBe(prepared.sourceFingerprints.length);
      expect(envelope.edits.map(({ sourcePath, sourceNodeId, fieldPath }) =>
        ({ sourcePath, sourceNodeId, fieldPath }))).toEqual([{
        sourcePath: levelEditorWindow, sourceNodeId: 'editor_content',
        fieldPath: ['layout', 'container', 'gap'],
      }]);
      expect(validateRealWorkbenchExportSources({ ...envelope, sources: [...envelope.sources].reverse() },
        prepared.sourceFingerprints, rootHash)).toBe(count);
      const owners = new Set(prepared.sourceFingerprints.map(({ sourcePath }) => sourcePath));
      ownerSets.push(owners);
      expect(() => validateRealWorkbenchExportSources({ ...envelope,
        sources: envelope.sources.slice(1) }, prepared.sourceFingerprints, rootHash)).toThrow();
      expect(() => validateRealWorkbenchExportSources({ ...envelope,
        sources: [...envelope.sources, envelope.sources[0]] }, prepared.sourceFingerprints, rootHash)).toThrow();
      expect(() => validateRealWorkbenchExportSources({ ...envelope,
        sources: envelope.sources.map((entry, index) => index === 0 ? { ...entry, sha256: '0'.repeat(64) } : entry) },
      prepared.sourceFingerprints, rootHash)).toThrow();
      expect(() => validateRealWorkbenchExportSources(envelope,
        [...prepared.sourceFingerprints, prepared.sourceFingerprints[0]], rootHash)).toThrow();
      expect(() => validateRealWorkbenchExportSources(envelope,
        prepared.sourceFingerprints, '0'.repeat(64))).toThrow();
    }
    expect(ownerSets[0].has(UNREAL_DOCK_HEADER_SOURCE)).toBe(false);
    expect(ownerSets[1]).toEqual(new Set([...ownerSets[0], UNREAL_DOCK_HEADER_SOURCE]));

    // Native drawer_region_input uses the panel-header token for collapsed
    // mode and max(expanded extent, panel height) otherwise. The actual Header
    // observation still has its distinct document-header viewport height 31.
    const bottomSource = 'zircon_editor/assets/ui/editor/unreal/components/bottom_activity_dock.zui';
    for (const [mode, expandedExtent, expectedExtent] of [
      ['collapsed', 148, 30], ['pinned', 30, 30], ['pinned', 31, 31], ['pinned', 30.5, 30.5],
      ['pinned', 148, 148], ['autoHide', 148, 148], ['pinned', 12, 30],
    ] as const) {
      const bottomTabs = [
        { id: 'editor.console#1', descriptorId: 'editor.console', title: 'Console', iconKey: 'console', contentKind: 'console', sourcePath: null },
        { id: 'editor.runtime_diagnostics#1', descriptorId: 'editor.runtime_diagnostics', title: 'Runtime Diagnostics', iconKey: 'runtime-diagnostics', contentKind: 'runtime_diagnostics', sourcePath: null },
        { id: 'editor.build_export_desktop#1', descriptorId: 'editor.build_export_desktop', title: 'Desktop Export', iconKey: 'build-export', contentKind: 'build_export', sourcePath: null },
      ];
      const bottomHeader = { ...observation(3), kind: 'bottom', surfaceKey: 'bottom' };
      // Unit Source constructor inputs must agree with the business drawer,
      // including collapsed drawers whose tabs are all unselected.
      for (const [index, tab] of bottomTabs.entries()) {
        for (const id of ['dock_tab_group_' + index, 'dock_tab_' + index])
          bottomHeader.sourceDocument.nodes![id].props = { ...bottomHeader.sourceDocument.nodes![id].props,
            selected: false, surface_variant: 'transparent', text_tone: 'subtle' };
        bottomHeader.sourceDocument.nodes!['dock_tab_' + index].props!['text'] = tab.title;
      }
      for (const row of bottomHeader.rows) {
        const node = bottomHeader.sourceDocument.nodes![row.sourceNodeId];
        row.text = node.props?.['text'] ?? '';
        row.selected = node.props?.['selected'] === true;
      }
      // Native can publish multiple render fragments for one real Source node.
      bottomHeader.rows.push(structuredClone(bottomHeader.rows.find((row) => row.controlId === 'DockTab0')!));
      const bottomState = { ...presentation, dockHeaders: [bottomHeader], drawers: [{
        slot: 'bottom', mode, visible: true, extent: expandedExtent, activeTabId: null, activeViewId: null,
        tabs: bottomTabs,
      }] };
      const { stateFingerprint: previousFingerprint, ...bottomStateBody } = bottomState;
      void previousFingerprint;
      bottomState.stateFingerprint = canonicalSha256(bottomStateBody);
      validateWorkbenchPresentation(bottomState);
      const originalState = structuredClone(bottomState);
      const prepared = (await prepareComponentReviewHost(
        repo, editorMainFrame, mainFrameSource, dependencies, theme, bottomState,
      ))!;
      const bottomNodes = Object.values(prepared.projection.nodes!);
      const shell = bottomNodes.find((node) => node.control_id === 'BottomDrawerShellRoot')!;
      const target = bottomNodes.find((node) => node.control_id === 'BottomDrawerHeaderRoot')!;
      const renderedHeader = bottomNodes.find((node) => node.control_id === 'DockHeaderRoot')!;
      expect(shell['penpot_review_source_path']).toBe(bottomSource);
      expect(shell.layout?.['height']).toEqual({ min: expectedExtent, preferred: expectedExtent,
        max: expectedExtent, stretch: 'Fixed' });
      expect(target.layout?.['height']).toEqual(plain.projection.nodes![
        Object.entries(plain.projection.nodes!).find(([, node]) => node.control_id === 'BottomDrawerHeaderRoot')![0]
      ].layout?.['height']);
      expect(target.layout?.['container']).toEqual({ kind: 'Overlay' });
      expect(renderedHeader.layout?.['height']).toEqual(header.nodes![header.root!.node].layout?.['height']);
      expect(renderedHeader['penpot_review_source_path']).toBe(UNREAL_DOCK_HEADER_SOURCE);
      expect(bottomHeader.viewport.height).toBe(31);
      const shapes = projectZuiDocument(prepared.projection).shapes;
      for (const [controlId, start] of [['BottomDrawerSeparator', 30], ['BottomDrawerContentRoot', 31]] as const) {
        const node = bottomNodes.find((candidate) => candidate.control_id === controlId)!;
        const original = Object.values(plain.projection.nodes!).find((candidate) => candidate.control_id === controlId)!;
        const visible = expectedExtent > start;
        expect(node.props).toEqual({ ...original.props, visible, visibility: visible ? 'visible' : 'collapsed' });
        expect(node.state).toEqual({ ...original.state, visible, visibility: visible ? 'visible' : 'collapsed' });
        expect(node['penpot_review_derived_drawer_visibility']).toEqual({
          authority: 'loaded-source-and-business-drawer', slot: 'bottom', mode,
          extent: expectedExtent, panelHeight: 30, separatorExtent: 1, visible,
        });
        expect(shapes.find((shape) => shape.controlId === controlId)!.previewHidden).toBe(!visible);
        for (const key of ['layout', 'children', 'events', 'classes', 'component', 'control_id',
          'penpot_review_source_path', 'penpot_review_source_node_id', 'penpot_review_instance_path'])
          expect(node[key]).toEqual(original[key]);
      }
      expect(bottomState).toEqual(originalState);
      expect(prepared.source).toBe(plain.source);
      expect(normalizeZuiDocument(mainFrameSource)).toEqual(before);
      if (mode === 'collapsed') {
        const { createPrefabIcon, nativeSemanticGlyph, nativeSemanticGlyphGeometry,
          refreshNativeSemanticIcons } = await import('../penpot-prefab-icons');
        const unitCaseHost = cloneZuiDocument(prepared.projection);
        // Honest unit binding: Header Source width 900 physical pixels, case
        // width 720 DIP at 125%. No observed row rectangle is a layout input.
        unitCaseHost['penpot_actual_normal_review_case'] = { host: 'editor', sourcePath: levelEditorWindow,
          viewport: { width: 720, height: 620 }, dpi: 1.25, data: { workbenchPresentation: bottomState } };
        const glyphPaths = ['ionicons/terminal-outline.svg', 'ionicons/grid-outline.svg', 'ionicons/share-outline.svg'];
        for (const [index, assetPath] of glyphPaths.entries()) {
          const entry = Object.values(unitCaseHost.nodes!).find((node) =>
            node['penpot_review_source_path'] === UNREAL_DOCK_HEADER_SOURCE &&
            node['penpot_review_source_node_id'] === 'dock_tab_' + index)!;
          expect(entry['penpot_review_native_tab_icon']).toMatchObject({
            authority: 'actual-workbench-bottom-tab', sourcePath: UNREAL_DOCK_HEADER_SOURCE,
            sourceNodeId: 'dock_tab_' + index, instancePath: '[]', surfaceKey: 'bottom',
            tabId: bottomTabs[index].id, descriptorId: bottomTabs[index].descriptorId,
            iconKey: bottomTabs[index].iconKey, assetPath, sourceUnits: 'physical-pixels',
            sourceViewport: { width: 900, height: 31 },
          });
          expect(entry['penpot_icon_svg']).toBe(await readFile(repo + '/zircon_editor/assets/icons/' + assetPath, 'utf8'));
          expect(entry.props?.['icon']).toBeUndefined();
          expect(nativeSemanticGlyph(unitCaseHost, entry)).toMatchObject({ family: 'bottom-tab', assetPath, sourceScale: 1.25 });
          const geometry = nativeSemanticGlyphGeometry(unitCaseHost, entry, 120, 32, 16)!;
          expect(geometry.icon!.width).toBeCloseTo(15.6);
          expect(geometry.text.x).toBeCloseTo(23.6);
          expect(nativeSemanticGlyphGeometry(unitCaseHost, entry, 120, 32, 16)).toEqual(geometry);
          for (const mutate of [
            (node: ZuiNode) => { node['penpot_review_runtime_surface_key'] = 'other'; },
            (node: ZuiNode) => { node['penpot_review_instance_path'] = JSON.stringify([{ sourcePath: levelEditorWindow, sourceNodeId: 'bottom_activity_dock' }]); },
            (node: ZuiNode) => { node['penpot_review_source_node_id'] = 'dock_tab_' + ((index + 1) % 3); },
            (node: ZuiNode) => { (node['penpot_review_native_tab_icon'] as Record<string, unknown>)['descriptorId'] = 'wrong-owner'; },
            (node: ZuiNode) => { (node['penpot_review_native_tab_icon'] as Record<string, unknown>)['sourceUnits'] = 'unknown'; },
            (node: ZuiNode) => { (node['penpot_review_native_tab_icon'] as Record<string, unknown>)['stateFingerprint'] = '0'.repeat(64); },
          ]) {
            const invalid = structuredClone(entry);
            mutate(invalid);
            expect(nativeSemanticGlyph(unitCaseHost, invalid)).toBeNull();
          }
          const unbound = cloneZuiDocument(unitCaseHost);
          delete unbound['penpot_actual_normal_review_case'];
          expect(nativeSemanticGlyph(unbound, entry)).toBeNull();
          {
            const values: Record<string, string> = {};
            const leaves = Array.from({ length: [3, 4, 5][index]! }, (_, ordinal) => ({
              id: 'tab-svg-' + index + '-' + ordinal, type: 'path', fills: [],
              strokes: [{ strokeColor: '#000000', strokeWidth: 32 }],
            }));
            const viewport = { id: 'tab-svg-viewport-' + index, type: 'rectangle', fills: [], strokes: [] };
            const icon = { id: 'tab-svg-group-' + index, type: 'group', x: 0, y: 0, width: 0, height: 0,
              hidden: false, fills: [], strokes: [], children: [] as unknown[],
              getSharedPluginData: (_namespace: string, key: string) => values[key] ?? '',
              setSharedPluginData: (_namespace: string, key: string, value: string) => { values[key] = value; },
              resize(width: number, height: number) { this.width = width; this.height = height; } };
            const children: unknown[] = [];
            const board = { x: 10, y: 20, width: 120, height: 32, children,
              appendChild: (shape: unknown) => children.push(shape) } as unknown as import('@penpot/plugin-types').Board;
            const create = vi.fn(() => icon);
            vi.stubGlobal('penpot', { createShapeFromSvg: create });
            try {
              createPrefabIcon(board, entry, unitCaseHost);
              expect(create).toHaveBeenCalledWith(entry['penpot_icon_svg']);
              expect(icon.hidden).toBe(true); // Official SVG import has not materialized its leaves yet.
              icon.children = [viewport, ...leaves];
              refreshNativeSemanticIcons(board, entry, unitCaseHost);
              expect(icon.hidden).toBe(false);
              expect(icon.width).toBeCloseTo(15.6);
              expect(leaves.every((leaf) => Math.abs(leaf.strokes[0]!.strokeWidth - 32 * 15.6 / 512) < 1e-6)).toBe(true);
              expect(viewport.strokes).toEqual([]);
              expect(leaves.every((leaf) => leaf.fills.length === 0)).toBe(true);
              const first = [icon.x, icon.y, icon.width, icon.height];
              refreshNativeSemanticIcons(board, entry, unitCaseHost);
              expect([icon.x, icon.y, icon.width, icon.height]).toEqual(first);
              expect(leaves.every((leaf) => Math.abs(leaf.strokes[0]!.strokeWidth - 32 * 15.6 / 512) < 1e-6)).toBe(true);
              refreshNativeSemanticIcons(board, entry, unbound);
              expect(icon.hidden).toBe(true);
            } finally { vi.unstubAllGlobals(); }
          }
        }
        for (const mutate of [
          (state: typeof bottomState) => { state.sourceFingerprint.sourcePath = 'zircon_editor/assets/ui/editor/unreal/other.zui'; },
          (state: typeof bottomState) => { state.sourceFingerprint.sha256 = '0'.repeat(64); },
          (state: typeof bottomState) => { state.drawers[0].tabs.pop(); },
          (state: typeof bottomState) => { state.drawers[0].tabs[1].id = state.drawers[0].tabs[0].id; },
          (state: typeof bottomState) => { state.drawers[0].tabs[0].title = 'Wrong business title'; },
          (state: typeof bottomState) => { state.drawers[0].tabs.reverse(); },
          (state: typeof bottomState) => { state.drawers[0].tabs[0].iconKey = 'unsupported'; },
        ]) {
          const invalid = structuredClone(bottomState);
          mutate(invalid);
          await expect(prepareUnrealDockHeaderViews(repo, invalid)).rejects.toThrow(/Actual bottom tab/);
        }
        for (const mutate of [
          (document: ZuiDocument) => { const node = Object.values(document.nodes!).find((item) =>
            item.control_id === 'BottomDrawerHeaderRoot')!;
            node.layout!['height'] = { min: 31, preferred: 31, max: 31, stretch: 'Fixed' }; },
          (document: ZuiDocument) => { const node = Object.values(document.nodes!).find((item) =>
            item.control_id === 'BottomDrawerShellRoot')!;
            const steps = JSON.parse(node['penpot_review_instance_path'] as string) as Array<{ sourcePath: string; sourceNodeId: string }>;
            steps[steps.length - 1].sourceNodeId = 'other_drawer'; node['penpot_review_instance_path'] = JSON.stringify(steps); },
          (document: ZuiDocument) => {
            const entry = Object.entries(document.nodes!).find(([, item]) => item.control_id === 'BottomDrawerHeaderRoot')!;
            for (const node of Object.values(document.nodes!)) node.children = node.children?.filter((child) => child.node !== entry[0]);
            Object.values(document.nodes!).find((item) => item.control_id === 'WorkbenchWindowRoot')!
              .children!.push({ node: entry[0] });
          },
          (document: ZuiDocument) => { const node = Object.values(document.nodes!).find((item) =>
            item.control_id === 'BottomDrawerSeparator')!; node['penpot_review_source_node_id'] = 'other_separator'; },
          (document: ZuiDocument) => {
            const node = Object.values(document.nodes!).find((item) => item.control_id === 'BottomDrawerContentRoot')!;
            document.nodes!['unit_duplicate_body'] = structuredClone(node);
          },
          (document: ZuiDocument) => {
            const node = Object.values(document.nodes!).find((item) => item.control_id === 'BottomDrawerShellRoot')!;
            node.children = [node.children![0], node.children![2], node.children![1]];
          },
          (document: ZuiDocument) => { const node = Object.values(document.nodes!).find((item) =>
            item.control_id === 'BottomDrawerContentRoot')!;
            node.layout!['height'] = { min: 1, stretch: 'Stretch' }; },
        ]) {
          dependencies.embed = (...args) => {
            const result = embed(...args);
            if (args[1] === levelEditorWindow) mutate(args[0]);
            return result;
          };
          await expect(prepareComponentReviewHost(repo, editorMainFrame, mainFrameSource, dependencies, theme, bottomState))
            .rejects.toThrow(/panel-header height|authored mount|drawer shell owner chain|clipped-body layout/);
        }
        dependencies.embed = originalEmbed;
        const input = await prepareRealWorkbenchInput(
          repo, repo + '/' + levelEditorWindow, ownerSource, parseZuiDocument(ownerSource).document, false, {
            id: 'unit-bottom-effective-visibility-no-edit', sourcePath: levelEditorWindow, host: 'editor',
            viewport: { width: 900, height: 620 }, dpi: 1, locale: 'en-US', state: 'default',
            data: { workbenchPresentation: bottomState }, themeSourcePath: theme,
          },
        );
        const saved = parseZuiDocument(input.source).document;
        const savedHost = reviewHost(saved)!;
        expect(Object.values(savedHost.nodes!).find((node) => node.control_id === 'BottomDrawerContentRoot')!
          ['penpot_review_derived_drawer_visibility']).toMatchObject({ visible: false, extent: 30 });
        const exported = reconcileReviewSource(saved, cloneProjectionSnapshot(projectZuiDocument(savedHost)));
        expect(exported.changes).toEqual([]);
        expect(exported.document.nodes).toEqual(parseZuiDocument(ownerSource).document.nodes);
        expect(exported.source).toBe(ownerSource);
      }
    }

    const interaction = observation(2);
    interaction.rows.find((row) => row.controlId === 'DockTabGroup1')!.hovered = true;
    interaction.rows.find((row) => row.controlId === 'DockTabClose0')!.pressed = true;
    const interacted = (await prepareUnrealDockHeaderViews(repo, { dockHeaders: [interaction] }))[0].document;
    expect(interacted.nodes!['dock_tab_group_1'].state).toMatchObject({ hovered: true, pressed: false, disabled: false });
    expect(interacted.nodes!['dock_tab_close_0'].state).toMatchObject({ hovered: false, pressed: true, disabled: false });
    expect(interacted.nodes!['dock_tab_close_0'].props).toEqual(interaction.sourceDocument.nodes!['dock_tab_close_0'].props);
    const neutral = (await prepareUnrealDockHeaderViews(repo, { dockHeaders: [observation(2)] }))[0].document;
    dependencies.embed(neutral, UNREAL_DOCK_HEADER_SOURCE, theme);
    dependencies.embed(interacted, UNREAL_DOCK_HEADER_SOURCE, theme);
    const neutralPaint = projectZuiDocument(neutral).shapes;
    const interactionPaint = projectZuiDocument(interacted).shapes;
    expect(interactionPaint.find((shape) => shape.nodeId === 'dock_tab_group_1')!.paint.fillColor)
      .not.toBe(neutralPaint.find((shape) => shape.nodeId === 'dock_tab_group_1')!.paint.fillColor);
    interaction.rows.find((row) => row.controlId === 'DockTabClose0')!.disabled = true;
    const disabled = (await prepareUnrealDockHeaderViews(repo, { dockHeaders: [interaction] }))[0].document;
    expect(disabled.nodes!['dock_tab_close_0'].state).toMatchObject({ pressed: true, disabled: true });
    expect(disabled.nodes!['dock_tab_close_0'].props).toEqual(interaction.sourceDocument.nodes!['dock_tab_close_0'].props);
    one.sourceDocument.nodes!['dock_tab_group_0'].layout!['width'] = { min: 1, max: 2 };
    await expect(prepareUnrealDockHeaderViews(repo, { dockHeaders: [one] }))
      .rejects.toThrow('differs from the current authored');
  }, 120_000);
});

describe('same-frame Unreal menu and Run source projection', () => {
  type Step = { sourcePath: string; sourceNodeId: string };
  type Row = {
    surfaceNodeId: number | null; nodeId: string; sourcePath: string; sourceNodeId: string; instancePath: Step[];
    controlId: string | null; component: string; selfVisible: boolean; enabled: boolean; children: number[];
    parent: { nodeId: string; sourcePath: string; sourceNodeId: string; instancePath: Step[] } | null;
    rawMetadata: { attributes: Record<string, unknown>; component: string; control_id: string;
      source_path: string; source_node_id: string; instance_path: Step[] };
    projected: { nodeId: string; surfaceNodeId: number | null; parentId: string | null; sourcePath: string;
      sourceNodeId: string; instancePath: Step[]; controlId: string | null; component: string;
      disabled?: boolean; properties: Record<string, unknown> };
  };
  const normalCursor = { structure: 4, geometry: 4, interaction: 1, viewport: 0, hitTest: 4, theme: 2, diagnostics: 1 };
  const nativeOwner = 'res://ui/editor/unreal/level_editor_window.zui';
  const nativeSource = (name: string) => `res://ui/editor/unreal/components/${name}.zui`;
  let authored: ZuiDocument;
  let authoredSource: string;
  let host: ZuiDocument;
  beforeAll(async () => {
    authoredSource = await readFile(repo + '/' + levelEditorWindow, 'utf8');
    authored = parseZuiDocument(authoredSource).document;
    const dependencies = new LayoutDependencies();
    await dependencies.load(repo, await trackedZuiPaths());
    host = (await prepareComponentReviewHost(repo, levelEditorWindow, authored, dependencies, theme))!.projection;
  }, 120_000);

  // Reduced observations retain the actual normal inventory's fields and
  // source topology. They are unit fixtures, never normal-frame acceptance.
  function observation(playing = false, nativePrefix: Step[] = []) {
    const rows: Row[] = [];
    const add = (source: string, id: string, controlId: string, component: string,
      parent: Row | null, path: Step[], props: Record<string, unknown> = {}, visible = true, enabled = true) => {
      const surfaceNodeId = rows.length + 1, nodeId = source === nativeOwner && id === 'root' ? 'v2/root' : 'v2/' + controlId;
      const identity = { sourcePath: source, sourceNodeId: id, instancePath: structuredClone(path), controlId, component };
      const row: Row = { ...identity, surfaceNodeId, nodeId, selfVisible: visible, enabled, children: [],
        parent: parent ? { nodeId: parent.nodeId, sourcePath: parent.sourcePath,
          sourceNodeId: parent.sourceNodeId, instancePath: structuredClone(parent.instancePath) } : null,
        rawMetadata: { attributes: structuredClone(props), component, control_id: controlId,
          source_path: source, source_node_id: id, instance_path: structuredClone(path) },
        projected: { ...structuredClone(identity), nodeId, surfaceNodeId, parentId: parent?.nodeId ?? null,
          disabled: props['disabled'] === true, properties: structuredClone(props) } };
      parent?.children.push(surfaceNodeId);
      rows.push(row);
      return row;
    };
    const root = add(nativeOwner, 'root', 'WorkbenchWindowRoot', 'Overlay', null, nativePrefix);
    const content = add(nativeOwner, 'editor_content', 'UnrealLevelEditorContent', 'VerticalBox', root, nativePrefix);
    const context = add(nativeOwner, 'project_context', 'UnrealProjectDocumentContext', 'HorizontalGroup',
      content, nativePrefix, { visible: true, visibility: 'visible' });
    add(nativeOwner, 'project_context_text', 'UnrealProjectDocumentContextText', 'Label',
      context, nativePrefix, { text: 'Scene · Unit project' });
    const callsite = (id: string): Step[] => [...nativePrefix, { sourcePath: nativeOwner, sourceNodeId: id }];
    const menus = add(nativeSource('menus'), 'menus', 'WorkbenchMenuBarRoot', 'HorizontalBox', content, callsite('menu_bar'));
    for (const [index, id] of ['file', 'edit', 'window', 'tools', 'build', 'help'].entries())
      add(nativeSource('menus'), id, 'MenuSlot' + index, 'Button', menus, callsite('menu_bar'),
        { text: id === 'help' ? 'Help' : id[0].toUpperCase() + id.slice(1), selected: false, value: false,
          ...(id === 'help' ? { disabled: true } : {}) });
    const toolbar = add(nativeSource('toolbar'), 'toolbar', 'WorkbenchWindowTopToolbarRegion', 'HorizontalBox', content, callsite('toolbar'));
    const commandRow = add(nativeSource('toolbar'), 'command_scroll', 'WorkbenchToolbarCommandRow', 'ScrollableBox', toolbar, callsite('toolbar'));
    for (const [id, controlId, component] of [
      ['save', 'WorkbenchToolbarSave', 'IconButton'], ['find_content', 'WorkbenchToolbarAssets', 'IconButton'],
      ['modes', 'WorkbenchToolSelect', 'Button'], ['add', 'WorkbenchToolbarAddActor', 'Button'],
      ['blueprints', 'UnrealToolbarBlueprints', 'Button'], ['cinematics', 'UnrealToolbarCinematics', 'Button'],
      ['play', 'WorkbenchRunPlay', 'IconButton'], ['stop', 'WorkbenchRunStop', 'IconButton'],
      ['play_options', 'WorkbenchRunMode', 'IconButton'], ['settings', 'UnrealToolbarSettings', 'Button'],
    ]) {
      const disabled = ['blueprints', 'cinematics'].includes(id) || (id === 'stop' && !playing) || (id === 'play' && playing);
      add(nativeSource('toolbar'), id, controlId, component, id === 'settings' ? toolbar : commandRow, callsite('toolbar'),
        { disabled, ...(id === 'modes' ? { text: 'Selection Mode', selected: false, value: false } : {}) },
        id === 'stop' ? playing : id === 'play' ? !playing : true, id !== 'find_content');
    }
    const popups = add(nativeSource('menu_popups'), 'popups', 'UnrealEditorMenuPopups', 'Overlay', root, callsite('menu_popups'));
    for (const [id, controlId] of [
      ['file_popup', 'UnrealFileMenuPopup'], ['edit_popup', 'UnrealEditMenuPopup'],
      ['window_popup', 'UnrealWindowMenuPopup'], ['tools_popup', 'UnrealToolsMenuPopup'],
      ['build_popup', 'UnrealBuildMenuPopup'], ['help_popup', 'UnrealHelpMenuPopup'],
      ['modes_popup', 'UnrealModesMenuPopup'], ['add_popup', 'UnrealAddMenuPopup'],
      ['blueprints_popup', 'UnrealBlueprintsMenuPopup'], ['cinematics_popup', 'UnrealCinematicsMenuPopup'],
      ['settings_popup', 'UnrealSettingsMenuPopup'], ['run_mode_popup', 'WorkbenchRunModeMenu'],
    ]) add(nativeSource('menu_popups'), id, controlId, 'PopupMenu', popups, callsite('menu_popups'),
      { menu_items: id === 'file_popup' ? ['Save Project|action=file.project.save|Ctrl+S'] :
        id === 'modes_popup' ? ['Select|action=viewport.tool.select'] : [], popup_open: false, selected: false }, false);

    const footer = add(nativeSource('status_bar'), 'root', 'WorkbenchWindowStatusBarRegion',
      'HorizontalGroup', content, callsite('status_bar'));
    for (const [id, controlId, text, component] of [
      ['ready', 'WorkbenchStatusReady', 'Project opened: Unit project', 'Label'],
      ['errors', 'WorkbenchStatusErrors', '0 Errors', 'Label'],
      ['warnings', 'WorkbenchStatusWarnings', '0 Warnings', 'Label'],
      ['messages', 'WorkbenchStatusMessages', '1 Message', 'Label'],
      ['grid', 'WorkbenchStatusGrid', 'Grid: 1 m', 'Button'],
      ['snap', 'WorkbenchStatusSnap', 'T:1 m R:15° S:off', 'Button'],
    ]) add(nativeSource('status_bar'), id, controlId, component, footer, callsite('status_bar'), { text });
    add(nativeSource('status_bar'), 'snap_toggle', 'WorkbenchStatusSnapToggle', 'IconButton', footer,
      callsite('status_bar'), { checked: false });

    // The existing toolbar reader validates a full, independent snap graph;
    // none of its geometry is copied into the menu/run Source projection.
    const nodes: ToolbarSourceNode[] = [];
    const toolbarNode = (id: string, control: string, parent: string | null, visible = true) => {
      const node: ToolbarSourceNode = { nodeId: id, parentNodeId: parent, sourcePath: nativeSource('viewport'),
        sourceNodeId: id, instancePath: '[]', parentSourcePath: parent ? nativeSource('viewport') : null,
        parentSourceNodeId: parent, parentInstancePath: parent ? '[]' : null, controlId: control, component: 'HorizontalGroup',
        visible, bounds: { x: 0, y: 0, width: 28, height: 28 }, clip: { x: 0, y: 0, width: 960, height: 640 },
        state: { selected: null, checked: null, popupOpen: null, value: null, text: null, valueText: null, disabled: false }, scroll: null };
      nodes.push(node); return node;
    };
    toolbarNode('root', 'UnrealViewportToolbar', null);
    toolbarNode('row', 'UnrealViewportToolbarRow', 'root');
    const more = toolbarNode('more', 'UnrealViewportToolbarMore', 'row');
    more.component = 'IconButton'; more.state.selected = false; more.state.checked = false;
    const popup = toolbarNode('popup', 'UnrealViewportToolbarOverflow', 'root', false);
    popup.component = 'ScrollableBox'; popup.state.popupOpen = false;
    const groups = ['translation-group', 'rotation-group', 'scale-group'];
    for (const [index, control] of ['ToggleTranslateSnap', 'ToggleRotateSnap', 'ToggleScaleSnap'].entries()) {
      toolbarNode(groups[index], groups[index], index === 0 ? 'row' : 'popup', index === 0);
      const snap = toolbarNode(control, control, groups[index], index === 0);
      snap.component = 'IconButton'; snap.state.selected = index === 1; snap.state.checked = index === 1; snap.state.value = index === 1;
    }
    return { presentationCursor: { ...normalCursor }, presentedFrameCount: 1,
      workbenchStatePublication: { authority: 'normal-retained-workbench-projection', observed: true,
        presentationCursor: { ...normalCursor }, presentedFrameCount: 1,
        workbenchPresentation: { sourceFingerprint: { sourcePath: levelEditorWindow },
          viewportSnap: { translationEnabled: false, rotationEnabled: true, scaleEnabled: false } } },
      viewportToolbarPublication: { schema: 'dev.zircon.editor.viewport-toolbar-publication', version: 1,
        authority: 'normal-prepared-source-surface', observed: true, coordinateSpace: 'physical-pixels',
        surfaceGeneration: 11, surfaceTreeId: 'unit-observed-snap-surface', rootNodeId: 'root', rowNodeId: 'row',
        moreNodeId: 'more', popupNodeId: 'popup', groupNodeIds: groups, nodes,
        presentationCursor: { ...normalCursor }, presentedFrameCount: 1 },
      arrangedInventoryPublication: { schema: 'dev.zircon.editor.normal-arranged-inventory', version: 1,
        authority: 'normal-retained-runtime-surfaces', observed: true, complete: false,
        inventoryAudit: { complete: false, surfaceCount: 1 }, pending: ['mounted-pane-source-style-overlay-closure'],
        presentationCursor: { ...normalCursor }, presentedFrameCount: 1,
        surfaces: [{ schema: 'dev.zircon.editor.normal-surface-arranged-inventory', version: 1,
          scope: 'workbench-window', documentId: nativeOwner, frameAuthority: 'retained-host-projection',
          coordinateSpace: 'surface-logical', sourceFrameGeneration: 9, rootIds: [root.surfaceNodeId],
          inventoryAudit: { complete: false, issues: ['unrelated Details repeat closure pending'] }, nodes: rows }] } };
  }
  const control = (document: ZuiDocument, id: string) => Object.values(document.nodes!).find((node) => node.control_id === id)!;
  const actualRow = (evidence: ReturnType<typeof observation>, id: string) =>
    evidence.arrangedInventoryPublication.surfaces[0].nodes.find((row) => row.controlId === id)!;

  it('projects actual footer text and checked state through full Source parents without importing paint or geometry', () => {
    const nativePrefix = [{ sourcePath: 'res://ui/editor/unreal/editor_shell.zui', sourceNodeId: 'actual_mount' }];
    const evidence = observation(false, nativePrefix), document = cloneZuiDocument(host);
    const before = cloneZuiDocument(document), originalSource = normalizeZuiDocument(authored);
    const messages = control(document, 'WorkbenchStatusMessages'), snap = control(document, 'WorkbenchStatusSnapToggle');
    messages.state = { text: 'stale', retained_marker: 'keep' };
    snap.state = { checked: true, retained_marker: 'keep' };
    const row = actualRow(evidence, 'WorkbenchStatusMessages');
    row.rawMetadata.attributes['background_color'] = '#ff00ff';
    row.rawMetadata.attributes['layout'] = { width: { preferred: 9999 } };
    const inputBefore = structuredClone(evidence);
    applyObservedWorkbenchMenuAndRunState(document, evidence);
    expect(messages.props?.['text']).toBe('1 Message');
    expect(messages.state).toMatchObject({ text: '1 Message', retained_marker: 'keep' });
    expect(snap.props?.['checked']).toBe(false);
    expect(snap.state).toMatchObject({ checked: false, retained_marker: 'keep' });
    expect(control(document, 'WorkbenchStatusGrid').props?.['text']).toBe('Grid: 1 m');
    expect(control(document, 'WorkbenchStatusSnap').props?.['text']).toBe('T:1 m R:15° S:off');
    expect(messages.layout).toEqual(control(before, 'WorkbenchStatusMessages').layout);
    expect(messages.style).toEqual(control(before, 'WorkbenchStatusMessages').style);
    expect(messages.props?.['background_color']).toEqual(control(before, 'WorkbenchStatusMessages').props?.['background_color']);
    expect(snap.events).toEqual(control(before, 'WorkbenchStatusSnapToggle').events);
    expect(document['penpot_review_same_frame_footer']).toMatchObject({ targetCount: 7, inventoryComplete: false,
      presentationCursor: normalCursor, presentedFrameCount: 1 });
    expect(evidence).toEqual(inputBefore);
    expect(normalizeZuiDocument(authored)).toEqual(originalSource);
    for (const values of [row.rawMetadata.attributes, row.projected.properties]) values['text'] = '0 Messages';
    const toggle = actualRow(evidence, 'WorkbenchStatusSnapToggle');
    for (const values of [toggle.rawMetadata.attributes, toggle.projected.properties]) values['checked'] = true;
    evidence.workbenchStatePublication.workbenchPresentation.viewportSnap.translationEnabled = true;
    const prepared = evidence.viewportToolbarPublication.nodes.find((node) => node.controlId === 'ToggleTranslateSnap')!;
    prepared.state.checked = true; prepared.state.selected = true; prepared.state.value = true;
    applyObservedWorkbenchMenuAndRunState(document, evidence);
    expect(messages.state?.['text']).toBe('0 Messages');
    expect(snap.state?.['checked']).toBe(true);
  });

  it('rejects missing, duplicate, conflicting or rebound footer observations before mutating the clone', () => {
    const mutations: Array<(evidence: ReturnType<typeof observation>, document: ZuiDocument) => void> = [
      (evidence) => { const surface = evidence.arrangedInventoryPublication.surfaces[0];
        surface.nodes = surface.nodes.filter((row) => row.controlId !== 'WorkbenchStatusMessages'); },
      (evidence) => { const row = structuredClone(actualRow(evidence, 'WorkbenchStatusMessages'));
        row.nodeId += '-duplicate'; row.projected.nodeId = row.nodeId;
        row.surfaceNodeId = row.surfaceNodeId! + 1000; row.projected.surfaceNodeId = row.surfaceNodeId;
        actualRow(evidence, 'WorkbenchWindowStatusBarRegion').children.push(row.surfaceNodeId);
        evidence.arrangedInventoryPublication.surfaces[0].nodes.push(row); },
      (evidence) => { actualRow(evidence, 'WorkbenchStatusMessages').projected.properties['text'] = '2 Messages'; },
      (evidence) => { delete actualRow(evidence, 'WorkbenchStatusMessages').rawMetadata.attributes['text']; },
      (evidence) => { const row = actualRow(evidence, 'WorkbenchStatusSnapToggle');
        for (const values of [row.rawMetadata.attributes, row.projected.properties]) values['checked'] = 'true'; },
      (evidence) => { actualRow(evidence, 'WorkbenchStatusMessages').rawMetadata.source_node_id = 'errors'; },
      (evidence) => { actualRow(evidence, 'WorkbenchStatusMessages').projected.parentId = 'v2/root'; },
      (evidence) => { const row = actualRow(evidence, 'WorkbenchStatusMessages');
        row.instancePath[0].sourceNodeId = 'other_status_bar';
        row.projected.instancePath = structuredClone(row.instancePath);
        row.rawMetadata.instance_path = structuredClone(row.instancePath); },
      (_evidence, document) => { control(document, 'WorkbenchStatusMessages')['penpot_review_source_node_id'] = 'other'; },
      (_evidence, document) => { control(document, 'WorkbenchStatusMessages').component = 'Button'; },
      (evidence) => { evidence.arrangedInventoryPublication.presentationCursor.interaction += 1; },
    ];
    for (const mutate of mutations) {
      const evidence = observation(), document = cloneZuiDocument(host);
      mutate(evidence, document);
      const before = cloneZuiDocument(document);
      expect(() => applyObservedWorkbenchMenuAndRunState(document, evidence)).toThrow();
      expect(document).toEqual(before);
    }
  });

  it('actual host state projection consumes project context visibility and exact text without arranged geometry', () => {
    const evidence = observation(false, [{ sourcePath: 'res://ui/editor/unreal/editor_shell.zui', sourceNodeId: 'actual_mount' }]);
    const document = cloneZuiDocument(host), baseline = cloneZuiDocument(host);
    const context = actualRow(evidence, 'UnrealProjectDocumentContext');
    const text = actualRow(evidence, 'UnrealProjectDocumentContextText');
    const title = 'Scene · 项目 🧱';
    for (const properties of [text.rawMetadata.attributes, text.projected.properties]) properties['text'] = title;
    for (const row of [context, text]) for (const properties of [row.rawMetadata.attributes, row.projected.properties]) {
      properties['layout'] = { width: { preferred: 9999 }, height: { preferred: 8888 } };
      properties['bounds'] = { x: 7000, y: 8000, width: 9000, height: 10000 };
      properties['background_color'] = '#ff00ff';
    }
    const originalEvidence = structuredClone(evidence);
    applyObservedWorkbenchMenuAndRunState(document, evidence);
    expect(control(document, 'UnrealProjectDocumentContext').props).toMatchObject({ visible: true, visibility: 'visible' });
    expect(control(document, 'UnrealProjectDocumentContext').state).toMatchObject({ visible: true, visibility: 'visible' });
    expect(control(document, 'UnrealProjectDocumentContextText').props?.['text']).toBe(title);
    expect(control(document, 'UnrealProjectDocumentContextText').state?.['text']).toBe(title);
    expect(document['penpot_review_same_frame_project_context']).toMatchObject({
      authority: 'normal-retained-runtime-surfaces', scope: 'workbench-window',
      presentationCursor: normalCursor, presentedFrameCount: 1, sourceFrameGeneration: 9, targetCount: 2,
    });
    for (const id of ['UnrealProjectDocumentContext', 'UnrealProjectDocumentContextText']) {
      const node = control(document, id), original = control(baseline, id);
      for (const key of ['layout', 'children', 'events', 'classes', 'component', 'control_id',
        'penpot_review_source_path', 'penpot_review_source_node_id', 'penpot_review_instance_path'])
        expect(node[key]).toEqual(original[key]);
      expect(node.props?.['background_color']).toEqual(original.props?.['background_color']);
      expect(node.props?.['bounds']).toBeUndefined();
    }
    expect(evidence).toEqual(originalEvidence);
    context.selfVisible = false;
    for (const properties of [context.rawMetadata.attributes, context.projected.properties]) {
      properties['visible'] = false; properties['visibility'] = 'collapsed';
    }
    for (const properties of [text.rawMetadata.attributes, text.projected.properties]) properties['text'] = '';
    applyObservedWorkbenchMenuAndRunState(document, evidence);
    expect(control(document, 'UnrealProjectDocumentContext').props).toMatchObject({ visible: false, visibility: 'collapsed' });
    expect(control(document, 'UnrealProjectDocumentContextText').props?.['text']).toBe('');
    expect(control(document, 'UnrealProjectDocumentContextText').state?.['visible']).toBe(true);
  });

  it('actual host state projection rejects missing, ambiguous, wrong-owner and conflicting context inputs atomically', () => {
    const mutations: Array<(evidence: ReturnType<typeof observation>, document: ZuiDocument) => void> = [
      (evidence) => { const surface = evidence.arrangedInventoryPublication.surfaces[0];
        surface.nodes = surface.nodes.filter((row) => row.controlId !== 'UnrealProjectDocumentContextText'); },
      (evidence) => { const duplicate = structuredClone(actualRow(evidence, 'UnrealProjectDocumentContextText'));
        duplicate.nodeId += '-duplicate'; duplicate.projected.nodeId = duplicate.nodeId;
        duplicate.surfaceNodeId = duplicate.surfaceNodeId! + 1000; duplicate.projected.surfaceNodeId = duplicate.surfaceNodeId;
        actualRow(evidence, 'UnrealProjectDocumentContext').children.push(duplicate.surfaceNodeId);
        evidence.arrangedInventoryPublication.surfaces[0].nodes.push(duplicate); },
      (evidence) => { const row = actualRow(evidence, 'UnrealProjectDocumentContext');
        row.sourcePath = nativeSource('viewport'); row.projected.sourcePath = row.sourcePath; },
      (evidence) => { const row = actualRow(evidence, 'UnrealProjectDocumentContextText');
        row.controlId += '-wrong'; row.projected.controlId = row.controlId; },
      (evidence) => { const row = actualRow(evidence, 'UnrealProjectDocumentContext');
        row.instancePath.push({ sourcePath: nativeOwner, sourceNodeId: 'other_owner' });
        row.projected.instancePath = structuredClone(row.instancePath); },
      (evidence) => { const row = actualRow(evidence, 'UnrealProjectDocumentContext');
        row.surfaceNodeId = null; row.projected.surfaceNodeId = null; },
      (evidence) => { const row = actualRow(evidence, 'UnrealProjectDocumentContextText');
        const oldParent = actualRow(evidence, 'UnrealProjectDocumentContext'), root = actualRow(evidence, 'WorkbenchWindowRoot');
        oldParent.children = oldParent.children.filter((id) => id !== row.surfaceNodeId);
        root.children.push(row.surfaceNodeId!);
        row.parent = { nodeId: root.nodeId, sourcePath: root.sourcePath, sourceNodeId: root.sourceNodeId,
          instancePath: structuredClone(root.instancePath) }; row.projected.parentId = root.nodeId; },
      (_evidence, document) => { control(document, 'UnrealProjectDocumentContextText')['penpot_review_source_node_id'] = 'wrong_text'; },
      (_evidence, document) => { const entry = Object.entries(document.nodes!).find(([, node]) =>
        node.control_id === 'UnrealProjectDocumentContext')!;
        for (const node of Object.values(document.nodes!)) node.children = node.children?.filter((child) => child.node !== entry[0]);
        control(document, 'WorkbenchWindowRoot').children!.push({ node: entry[0] }); },
      (evidence) => { actualRow(evidence, 'UnrealProjectDocumentContext').rawMetadata.attributes['visible'] = false; },
      (evidence) => { actualRow(evidence, 'UnrealProjectDocumentContext').selfVisible = false; },
      (evidence) => { const row = actualRow(evidence, 'UnrealProjectDocumentContext');
        for (const properties of [row.rawMetadata.attributes, row.projected.properties]) properties['visible'] = 'true'; },
      (evidence) => { actualRow(evidence, 'UnrealProjectDocumentContextText').projected.properties['text'] = 'different title'; },
      (evidence) => { const row = actualRow(evidence, 'UnrealProjectDocumentContextText');
        for (const properties of [row.rawMetadata.attributes, row.projected.properties]) properties['text'] = 42; },
    ];
    for (const mutate of mutations) {
      const evidence = observation(), document = cloneZuiDocument(host);
      mutate(evidence, document);
      const before = cloneZuiDocument(document);
      expect(() => applyObservedWorkbenchMenuAndRunState(document, evidence)).toThrow();
      expect(document).toEqual(before);
    }
  });

  it('projects observed Edit and Playing controls, disabled empty entries and menu values without editing Source geometry or events', () => {
    const original = normalizeZuiDocument(authored), baseline = cloneZuiDocument(host);
    const document = cloneZuiDocument(host), edit = observation();
    const stop = control(document, 'WorkbenchRunStop');
    stop.state = { visible: true, visibility: 'visible', disabled: false, selected: false, retained_marker: 'keep' };
    const file = actualRow(edit, 'UnrealFileMenuPopup');
    file.selfVisible = true;
    file.rawMetadata.attributes['popup_open'] = true; file.projected.properties['popup_open'] = true;
    const fileEntry = actualRow(edit, 'MenuSlot0');
    for (const values of [fileEntry.rawMetadata.attributes, fileEntry.projected.properties]) {
      values['selected'] = true; values['value'] = true;
    }
    file.rawMetadata.attributes['background_color'] = '#ff00ff';
    file.rawMetadata.attributes['layout'] = { width: { preferred: 9999 } };
    const inventoryBefore = structuredClone(edit.arrangedInventoryPublication);
    applyObservedWorkbenchMenuAndRunState(document, edit);
    expect(stop.props).toMatchObject({ visible: false, visibility: 'collapsed', disabled: true, enabled: true });
    expect(stop.state).toMatchObject({ visible: false, visibility: 'collapsed', disabled: true, retained_marker: 'keep' });
    expect(stop['penpot_review_same_frame_menu_run']).toMatchObject({ enabled: true, projectedDisabled: true,
      rawDisabled: true, disabledFromEnabled: false, disabledFromProjected: true });
    expect(control(document, 'WorkbenchToolbarAssets')['penpot_review_same_frame_menu_run'])
      .toMatchObject({ disabledFromEnabled: true, disabledFromProjected: false });
    for (const id of ['MenuSlot5', 'UnrealToolbarBlueprints', 'UnrealToolbarCinematics']) {
      expect(control(document, id).props).toMatchObject({ visible: true, disabled: true });
      expect(control(document, id).state).toMatchObject({ visible: true, disabled: true });
    }
    expect(control(document, 'UnrealHelpMenuPopup').props?.['menu_items']).toEqual([]);
    expect(control(document, 'UnrealFileMenuPopup').props).toMatchObject({ popup_open: true, open: true,
      menu_items: ['Save Project|action=file.project.save|Ctrl+S'] });
    expect(control(document, 'MenuSlot0').state).toMatchObject({ selected: true, value: true });
    expect(document['penpot_review_same_frame_menu_run']).toMatchObject({ inventoryComplete: false,
      inventoryAudit: edit.arrangedInventoryPublication.inventoryAudit, pending: edit.arrangedInventoryPublication.pending });
    const shapes = projectZuiDocument(document).shapes;
    expect(shapes.find((shape) => shape.controlId === 'WorkbenchRunStop')!.previewHidden).toBe(true);
    expect(shapes.find((shape) => shape.controlId === 'MenuSlot5')!.previewHidden).toBe(false);
    for (const [id, node] of Object.entries(document.nodes!)) {
      const before = baseline.nodes![id];
      for (const key of ['layout', 'children', 'events', 'classes', 'component', 'control_id',
        'penpot_review_source_path', 'penpot_review_source_node_id', 'penpot_review_instance_path'])
        expect(node[key]).toEqual(before[key]);
      expect(node.props?.['background_color']).toEqual(before.props?.['background_color']);
    }
    expect(normalizeZuiDocument(authored)).toEqual(original);
    expect(edit.arrangedInventoryPublication).toEqual(inventoryBefore);
    applyObservedWorkbenchMenuAndRunState(document, observation(true));
    expect(control(document, 'WorkbenchRunPlay').props).toMatchObject({ visible: false, disabled: true });
    expect(stop.props).toMatchObject({ visible: true, disabled: false, enabled: true });
    expect(stop.state).toMatchObject({ visible: true, disabled: false });
    expect(control(document, 'UnrealFileMenuPopup').state).toMatchObject({ popup_open: false, open: false });
  });

  it('actual host state projection uses complete owner-root prefixes and survives real preparation and no-edit export', async () => {
    const evidence = observation(false, [{ sourcePath: 'res://ui/editor/unreal/editor_shell.zui', sourceNodeId: 'actual_host_mount' }]);
    const prepared = await prepareRealWorkbenchInput(repo, repo + '/' + levelEditorWindow, authoredSource,
      authored, false, { id: 'unit-same-frame-menu-run', sourcePath: levelEditorWindow, host: 'editor',
        viewport: { width: 960, height: 640 }, dpi: 1.25, locale: 'en-US', state: 'default',
        data: { normal_window_evidence: evidence }, themeSourcePath: theme });
    const projection = reviewHost(prepared.input)!;
    expect(control(projection, 'WorkbenchRunStop').state).toMatchObject({ visible: false, disabled: true });
    expect(control(projection, 'WorkbenchWindowRoot')['penpot_review_instance_path'])
      .toBe(control(host, 'WorkbenchWindowRoot')['penpot_review_instance_path']);
    expect(control(projection, 'MenuSlot5').events).toEqual(control(host, 'MenuSlot5').events);
    expect(control(projection, 'UnrealProjectDocumentContext').props).toMatchObject({ visible: true, visibility: 'visible' });
    expect(control(projection, 'UnrealProjectDocumentContextText').props?.['text']).toBe('Scene · Unit project');
    expect(prepared.input.nodes).toEqual(authored.nodes);
    const parsed = reviewHost(parseZuiDocument(prepared.source).document)!;
    expect(parsed['penpot_review_same_frame_menu_run']).toMatchObject({ inventoryComplete: false, targetCount: 28 });
    expect(parsed['penpot_review_same_frame_project_context']).toMatchObject({ presentationCursor: normalCursor,
      presentedFrameCount: 1, targetCount: 2 });
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(parsed));
    const exported = reconcileReviewSource(prepared.input, snapshot);
    expect(exported.changes).toEqual([]);
    expect(exported.document.nodes).toEqual(authored.nodes);
    expect(exported.source).toBe(authoredSource);
  }, 120_000);

  it('rejects missing publications or targets and stale counters or successful frames without partial writes', () => {
    const mutations: Array<(evidence: ReturnType<typeof observation>) => void> = [
      (evidence) => { delete (evidence as unknown as Record<string, unknown>)['arrangedInventoryPublication']; },
      (evidence) => { delete (evidence as unknown as Record<string, unknown>)['viewportToolbarPublication']; },
      (evidence) => { const surface = evidence.arrangedInventoryPublication.surfaces[0]; surface.nodes = surface.nodes.filter((row) => row.controlId !== 'WorkbenchRunStop'); },
      (evidence) => { evidence.arrangedInventoryPublication.presentedFrameCount += 1; },
      (evidence) => { evidence.workbenchStatePublication.presentedFrameCount += 1; },
      ...Object.keys(normalCursor).map((key) => (evidence: ReturnType<typeof observation>) => {
        (evidence.arrangedInventoryPublication.presentationCursor as Record<string, number>)[key] += 1;
      }),
    ];
    for (const mutate of mutations) {
      const evidence = observation(), document = cloneZuiDocument(host), before = cloneZuiDocument(document);
      mutate(evidence);
      expect(() => applyObservedWorkbenchMenuAndRunState(document, evidence)).toThrow();
      expect(document).toEqual(before);
    }
  });

  it('rejects duplicate or wrong identities, owners, scopes and nodes outside either actual owner parent chain', () => {
    const mutations: Array<(evidence: ReturnType<typeof observation>, document: ZuiDocument) => void> = [
      (evidence) => { const duplicate = structuredClone(actualRow(evidence, 'WorkbenchRunStop'));
        duplicate.nodeId += '-duplicate'; duplicate.projected.nodeId = duplicate.nodeId;
        duplicate.surfaceNodeId = duplicate.surfaceNodeId! + 1000; duplicate.projected.surfaceNodeId = duplicate.surfaceNodeId;
        actualRow(evidence, 'WorkbenchToolbarCommandRow').children.push(duplicate.surfaceNodeId);
        evidence.arrangedInventoryPublication.surfaces[0].nodes.push(duplicate); },
      (evidence) => { const root = actualRow(evidence, 'WorkbenchWindowRoot'); root.sourceNodeId = 'wrong_root'; root.projected.sourceNodeId = 'wrong_root'; },
      (_evidence, document) => { control(document, 'WorkbenchWindowRoot')['penpot_review_source_node_id'] = 'wrong_root'; },
      (evidence) => { evidence.arrangedInventoryPublication.surfaces[0].scope = 'host-shell'; },
      (evidence) => { evidence.arrangedInventoryPublication.surfaces[0].documentId = 'res://ui/editor/unreal/editor_shell.zui'; },
      (evidence) => { const row = actualRow(evidence, 'WorkbenchRunStop'); row.controlId = 'WrongControl'; row.projected.controlId = 'WrongControl'; },
      (evidence) => { const row = actualRow(evidence, 'WorkbenchRunStop');
        row.surfaceNodeId = actualRow(evidence, 'WorkbenchToolbarSave').surfaceNodeId;
        row.projected.surfaceNodeId = row.surfaceNodeId; },
      (evidence) => { const row = actualRow(evidence, 'WorkbenchRunStop'); row.parent = null; row.projected.parentId = null; },
      (evidence) => { actualRow(evidence, 'WorkbenchRunStop').projected.parentId = 'different_projected_parent'; },
      (evidence) => { const row = actualRow(evidence, 'WorkbenchRunStop'); row.instancePath[0].sourceNodeId += '_similar'; row.projected.instancePath = structuredClone(row.instancePath); },
      (_evidence, document) => { const row = control(document, 'WorkbenchRunStop');
        const steps = JSON.parse(row['penpot_review_instance_path'] as string) as Step[];
        steps[0].sourceNodeId += '_similar'; row['penpot_review_instance_path'] = JSON.stringify(steps); },
      (_evidence, document) => { const stop = Object.entries(document.nodes!).find(([, node]) => node.control_id === 'WorkbenchRunStop')!;
        for (const node of Object.values(document.nodes!)) node.children = node.children?.filter((child) => child.node !== stop[0]);
        document.nodes![document.root!.node].children!.push({ node: stop[0] }); },
    ];
    for (const mutate of mutations) {
      const evidence = observation(), document = cloneZuiDocument(host);
      mutate(evidence, document);
      const before = cloneZuiDocument(document);
      expect(() => applyObservedWorkbenchMenuAndRunState(document, evidence)).toThrow();
      expect(document).toEqual(before);
    }
    const withNativePrefix = observation(false, [{ sourcePath: 'res://ui/editor/unreal/editor_shell.zui', sourceNodeId: 'actual_mount' }]);
    const misplaced = actualRow(withNativePrefix, 'WorkbenchRunStop');
    misplaced.instancePath[0].sourceNodeId += '_similar';
    misplaced.projected.instancePath = structuredClone(misplaced.instancePath);
    expect(() => applyObservedWorkbenchMenuAndRunState(cloneZuiDocument(host), withNativePrefix))
      .toThrow('complete owner prefix');
  });

  it('rejects nonboolean or conflicting disabled sources and conflicting observed menu values', () => {
    const mutations: Array<(row: Row) => void> = [
      (row) => { Object.assign(row.projected, { disabled: 'true' }); },
      (row) => { row.rawMetadata.attributes['disabled'] = 'true'; },
      (row) => { row.rawMetadata.attributes['disabled'] = false; },
      (row) => { row.rawMetadata.attributes['selected'] = true; row.projected.properties['selected'] = false; },
    ];
    for (const mutate of mutations) {
      const evidence = observation(), document = cloneZuiDocument(host), before = cloneZuiDocument(document);
      mutate(actualRow(evidence, 'WorkbenchRunStop'));
      expect(() => applyObservedWorkbenchMenuAndRunState(document, evidence)).toThrow(/boolean|sources conflict/);
      expect(document).toEqual(before);
    }
  });

  it('retains unrelated projection-only unknown runtime IDs as pending and rejects them on actual targets or ancestors', () => {
    const evidence = observation(), document = cloneZuiDocument(host);
    const prototype = structuredClone(actualRow(evidence, 'WorkbenchRunStop'));
    prototype.nodeId = 'root.0.4.1.1.5.3.1.0'; prototype.surfaceNodeId = null;
    prototype.controlId = 'WorkbenchComponentPropertySlot04Row';
    prototype.sourcePath = 'res://ui/editor/workbench/data/component_property_row.zui';
    prototype.sourceNodeId = 'root'; prototype.component = 'InputField'; prototype.parent = null;
    prototype.instancePath = [{ sourcePath: nativeOwner, sourceNodeId: 'details' },
      { sourcePath: nativeSource('details'), sourceNodeId: 'prototype' }];
    Object.assign(prototype.projected, { nodeId: prototype.nodeId, surfaceNodeId: null, controlId: prototype.controlId,
      sourcePath: prototype.sourcePath, sourceNodeId: prototype.sourceNodeId, component: prototype.component,
      instancePath: structuredClone(prototype.instancePath), parentId: 'v2/WorkbenchInspectorMeshProperties' });
    evidence.arrangedInventoryPublication.surfaces[0].nodes.push(prototype);
    const before = structuredClone(evidence.arrangedInventoryPublication);
    applyObservedWorkbenchMenuAndRunState(document, evidence);
    expect(control(document, 'WorkbenchRunStop').state).toMatchObject({ visible: false, disabled: true });
    expect(document['penpot_review_same_frame_menu_run']).toMatchObject({ inventoryComplete: false,
      pending: evidence.arrangedInventoryPublication.pending, surfaceInventoryAudit: evidence.arrangedInventoryPublication.surfaces[0].inventoryAudit });
    expect(evidence.arrangedInventoryPublication).toEqual(before);
    for (const id of ['WorkbenchRunStop', 'WorkbenchToolbarCommandRow']) {
      const invalid = observation(), node = actualRow(invalid, id);
      node.surfaceNodeId = null; node.projected.surfaceNodeId = null;
      expect(() => applyObservedWorkbenchMenuAndRunState(cloneZuiDocument(host), invalid))
        .toThrow('actual runtime surface node ID');
    }
  });
});

describe('registered private AI source closure', () => {
  const sourcePath = 'zircon_plugins/ai/editor/behavior_tree.zui';
  const paths = [sourcePath, 'zircon_editor/assets/ui/theme/editor_base.zui',
    'zircon_editor/assets/ui/theme/editor_material.zui', theme];
  async function admitted() {
    const source = await readFile(`${repo}/${sourcePath}`, 'utf8');
    const expectedSources = await Promise.all(paths.map(async sourcePath => ({
      sourcePath, sha256: sha256(await readFile(`${repo}/${sourcePath}`, 'utf8')),
    })));
    return { source, original: parseZuiDocument(source).document, expectedSources };
  }
  it('preserves the original twelve controls and style closure without a LevelEditor frame', async () => {
    const { source, original, expectedSources } = await admitted();
    const prepared = await prepareRealOwnerInput(repo, `${repo}/${sourcePath}`,
      source, original, { expectedSources });
    expect(prepared.sourceFingerprints).toEqual([...expectedSources].sort((a, b) => a.sourcePath.localeCompare(b.sourcePath)));
    expect(prepared.input['penpot_original_source']).toBe(source);
    expect(prepared.input['penpot_original_source_path']).toBe(sourcePath);
    expect(prepared.input.imports).toEqual(original.imports);
    expect(prepared.managedPresentationPrepared).toBe(false);
    const projection = prepared.input['penpot_review_host'] as ZuiDocument;
    expect(Object.values(original.nodes ?? {}).filter(node => node.control_id)).toHaveLength(12);
    for (const [nodeId, node] of Object.entries(original.nodes ?? {})) {
      expect(projection.nodes?.[nodeId]?.control_id).toBe(node.control_id);
      expect(projection.nodes?.[nodeId]?.['penpot_review_source_path']).toBe(sourcePath);
      expect(projection.nodes?.[nodeId]?.['penpot_review_source_node_id']).toBe(nodeId);
    }
    expect(projection.nodes?.['behavior_tree_graph_canvas']?.component).toBe('Space');
    expect(projection['penpot_actual_normal_review_case']).toBeUndefined();
  });
  it('rejects missing, stale, incomplete and extra independent fingerprints', async () => {
    const { source, original, expectedSources } = await admitted();
    await expect(prepareRealOwnerInput(repo, `${repo}/${sourcePath}`, source, original))
      .rejects.toThrow('independently pinned source closure');
    for (const admittedSources of [
      expectedSources.map(item => item.sourcePath === sourcePath ? { ...item, sha256: '0'.repeat(64) } : item),
      expectedSources.filter(item => item.sourcePath !== theme),
      [...expectedSources, { sourcePath: button, sha256: sha256(await readFile(`${repo}/${button}`, 'utf8')) }],
    ]) {
      await expect(prepareRealOwnerInput(repo, `${repo}/${sourcePath}`, source, original,
        { expectedSources: admittedSources })).rejects.toThrow();
    }
  });
  it('rejects a rebound asset, root, imports or substituted theme despite matching owner bytes', async () => {
    const { source, original, expectedSources } = await admitted();
    for (const changed of [
      { ...original, asset: { ...original.asset, id: 'ai.other' } },
      { ...original, root: { node: 'different_root' } },
      { ...original, imports: { styles: ['res://ui/editor/theme/editor_tokens.zui'] } },
      { ...original, imports: { ...original.imports, widgets: ['unadmitted.zui#Widget'] } },
    ]) {
      await expect(prepareRealOwnerInput(repo, `${repo}/${sourcePath}`, source, changed,
        { expectedSources })).rejects.toThrow(/Private source/);
    }
    await expect(prepareRealOwnerInput(repo, `${repo}/${sourcePath}`, source, original,
      { expectedSources, themeSourcePath: theme })).rejects.toThrow('theme substitution');
  });

  it('admits only the original private source root scalar gap without mutating it', async () => {
    const { original } = await admitted();
    const before = cloneZuiDocument(original);
    expect(prepareRealOwnerGapEdit(sourcePath, original)).toEqual({
      sourcePath, sourceNodeId: 'behavior_tree_root', controlId: 'AiBehaviorTreeRoot',
      instancePath: [], beforeGap: 8, afterGap: 9,
    });
    expect(original).toEqual(before);
    expect(() => prepareRealOwnerGapEdit(levelEditorWindow, original)).toThrow('registered private AI owner');
    expect(() => prepareRealOwnerGapEdit(sourcePath, { ...original, root: { node: 'other' } }))
      .toThrow('registered owner');
  });

  it.each([NaN, Infinity, -1, '$editor.gap', undefined])(
    'rejects a non-finite, negative, token or missing source scalar: %s', async gap => {
      const { original } = await admitted();
      original.nodes!['behavior_tree_root'].layout!['container'] = { kind: 'HorizontalBox', gap };
      expect(() => prepareRealOwnerGapEdit(sourcePath, original)).toThrow('finite scalar literal');
    });

  it('resolves only one visible attached exact source identity with direct ancestry', async () => {
    const { original } = await admitted();
    const target = prepareRealOwnerGapEdit(sourcePath, original);
    type MountedNode = Parameters<typeof sourceOwnerSpacingNode>[0][number];
    const node: MountedNode = {
      nodeId: 'behavior_tree_root', shapeId: '4f1bc15c-3bf7-49fa-a506-289538bcc34b',
      sourcePath, sourceNodeId: target.sourceNodeId, controlId: target.controlId, instancePath: '[]',
      parentNodeId: null, component: 'HorizontalGroup', visible: true, detached: false,
      bounds: { x: 0, y: 0, width: 1280, height: 800 }, text: '',
    };
    expect(sourceOwnerSpacingNode([node], target)).toBe(node);
    const mismatches: Array<Partial<MountedNode>> = [
      { sourcePath: levelEditorWindow }, { sourceNodeId: 'other' }, { controlId: 'Other' },
      { visible: false }, { detached: true },
      { instancePath: '[{"sourcePath":"other.zui","sourceNodeId":"caller"}]' },
    ];
    for (const mismatch of mismatches) {
      const wrong = { ...node, ...mismatch };
      expect(() => sourceOwnerSpacingNode([wrong], target)).toThrow('exact source-owned spacing target');
      expect(sourceOwnerSpacingNode([wrong, node], target)).toBe(node);
    }
    expect(() => sourceOwnerSpacingNode([node, { ...node, shapeId: 'another' }], target))
      .toThrow('exact source-owned spacing target');
  });

  it('reconciles the single raw gap through the official owner codec in memory and rejects additional deltas', async () => {
    const { source, original, expectedSources } = await admitted();
    const canonical = Object.fromEntries(await Promise.all(paths.map(async path =>
      [path, await readFile(`${repo}/${path}`, 'utf8')] as const)));
    const prepared = await prepareRealOwnerInput(repo, `${repo}/${sourcePath}`,
      source, original, { expectedSources });
    const target = prepareRealOwnerGapEdit(sourcePath, original);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(reviewHost(prepared.input)!));
    const root = snapshot.shapes.find(shape => shape.sourcePath === sourcePath &&
      shape.sourceNodeId === target.sourceNodeId && shape.controlId === target.controlId)!;
    expect(root).toBeDefined();
    expect(root.current.container.direction).toBe('row');
    root.current.container.columnGap = target.afterGap;
    const exported = reconcileReviewSource(prepared.input, snapshot);
    const manifest = parseSourceExportManifest(exported.document['penpot_source_exports']);
    expect(validateRealOwnerGapExport(manifest, expectedSources, sha256(source), target)).toBe(4);
    if (typeof exported.source !== 'string') throw new Error('Official owner export did not produce source text');
    const applied = applyZuiExportToSourceFiles(exported.source, canonical, expectedSources);
    expect(applied.appliedPathsBySource).toEqual({
      [sourcePath]: ['nodes.behavior_tree_root.layout.container.gap'],
    });
    expect(applied.outputs.map(output => output.sourcePath)).toEqual([sourcePath]);
    const actual = parseZuiDocument(applied.outputs[0]!.source).document;
    const expected = cloneZuiDocument(original);
    (expected.nodes!['behavior_tree_root'].layout!['container'] as Record<string, unknown>)['gap'] = 9;
    expect(normalizeZuiDocument(actual)).toEqual(normalizeZuiDocument(expected));
    expect(Object.keys(actual.nodes!)).toEqual(Object.keys(original.nodes!));
    expect(Object.values(actual.nodes!).map(node => node.control_id))
      .toEqual(Object.values(original.nodes!).map(node => node.control_id));
    expect(Object.values(actual.nodes!).filter(node => node.control_id)).toHaveLength(12);
    expect(Object.values(actual.nodes!).flatMap(node => node.events ?? [])).toEqual([]);
    expect(actual.imports).toEqual(original.imports);
    for (const path of paths) expect(await readFile(`${repo}/${path}`, 'utf8')).toBe(canonical[path]);
    const firstEdit = manifest.edits[0]!;
    for (const edits of [
      [{ ...firstEdit, value: 10 }],
      [{ ...firstEdit, controlId: 'Other' }],
      [{ ...firstEdit, sourceNodeId: 'other' }],
      [...manifest.edits, { ...firstEdit, fieldPath: ['props', 'font_size'], value: 20 }],
    ]) expect(() => validateRealOwnerGapExport({ ...manifest, edits }, expectedSources,
      sha256(source), target)).toThrow('exactly the admitted root scalar gap delta');
  });
});

describe('material component authored source ancestry', () => {
  const sourcePath =
    'zircon_editor/assets/ui/editor/modules/core/rendering/material_workspace.zui';
  const paths = [
    sourcePath, button, theme,
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/field.zui',
    'zircon_editor/assets/ui/editor/components/workbench/primitives/data/list_row.zui',
    'zircon_editor/assets/ui/editor/workbench/inputs/property_editor_row.zui',
    'zircon_editor/assets/ui/editor/workbench/primitives/chrome/section_title.zui',
    'zircon_editor/assets/ui/editor/workbench/primitives/data/property_row.zui',
    'zircon_editor/assets/ui/editor/workbench/primitives/data/table_row.zui',
    'zircon_editor/assets/ui/editor/workbench/primitives/inputs/dropdown.zui',
  ].sort();
  type Step = { sourcePath: string; sourceNodeId: string };
  async function preparedMaterial() {
    const canonical = Object.fromEntries(await Promise.all(paths.map(async path =>
      [path, await readFile(repo + '/' + path, 'utf8')] as const)));
    const source = canonical[sourcePath]!;
    const original = parseZuiDocument(source).document;
    const expectedSources = paths.map(path => ({ sourcePath: path, sha256: sha256(canonical[path]!) }));
    const prepared = await prepareRealOwnerInput(repo, repo + '/' + sourcePath,
      source, original, { expectedSources });
    return { canonical, source, original, expectedSources, prepared, host: reviewHost(prepared.input)! };
  }

  it('keeps only canonical callsites in the ten-source Material closure without altering original visibility, slots or events', async () => {
    const { canonical, source, original, expectedSources, prepared, host } = await preparedMaterial();
    expect(prepared.sourceFingerprints).toEqual(expectedSources);
    expect(prepared.input['penpot_original_source']).toBe(source);
    expect(prepared.input['penpot_original_source_path']).toBe(sourcePath);
    expect(prepared.input.nodes).toEqual(original.nodes);
    expect(prepared.input.imports).toEqual(original.imports);
    expect(prepared.reviewCase).toBeUndefined();
    expect(prepared.managedPresentationPrepared).toBe(false);
    const root = host.nodes![host.root!.node]!;
    expect(root['penpot_review_source_path']).toBe(sourcePath);
    expect(root['penpot_review_source_node_id']).toBe('material_workspace');
    expect(root['penpot_review_instance_path']).toBe('[]');
    expect(root.props?.['visibility']).toBe('collapsed');
    expect(root['penpot_prefab_source']).toBe(sourcePath + '#WorkbenchMaterialWorkspace');
    const documents = Object.fromEntries(paths.map(path =>
      [path, parseZuiDocument(canonical[path]!).document]));
    const authored = Object.values(host.nodes!).filter(node =>
      typeof node['penpot_review_source_path'] === 'string');
    expect(authored.length).toBeGreaterThan(29);
    for (const node of authored) {
      const owner = node['penpot_review_source_path'] as string;
      const rawId = node['penpot_review_source_node_id'] as string;
      expect(documents[owner]?.nodes?.[rawId]).toBeDefined();
      for (const step of JSON.parse(node['penpot_review_instance_path'] as string) as Step[])
        expect(documents[step.sourcePath]?.nodes?.[step.sourceNodeId]).toBeDefined();
    }
    const field = authored.find(node => node.control_id === 'WorkbenchMaterialPreviewField')!;
    expect(field).toBeDefined();
    expect(JSON.parse(field['penpot_review_instance_path'] as string)).toContainEqual({
      sourcePath, sourceNodeId: 'material_preview_field',
    });
    for (const id of ['material_domain_property_row', 'material_blend_property_row', 'material_preview_property_row'])
      expect(prepared.input.nodes![id]!.children).toEqual(original.nodes![id]!.children);
    const events = authored.flatMap(node => node.events ?? []);
    const originalEvents = Object.values(original.nodes!).flatMap(node => node.events ?? []);
    expect(originalEvents).toHaveLength(13);
    for (const event of originalEvents) expect(events).toContainEqual(event);
    expect(Object.values(original.nodes!).filter(node => node.control_id)).toHaveLength(27);
    expect(Object.keys(original.nodes!)).toHaveLength(29);
  });

  it('reconciles one Material header spacing edit through the original owner codec in memory and preserves all dependency sources', async () => {
    const { canonical, source, original, expectedSources, prepared, host } = await preparedMaterial();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(host));
    const noOp = reconcileReviewSource(prepared.input, snapshot);
    expect(noOp.source).toBe(source);
    const header = snapshot.shapes.find(shape => shape.sourcePath === sourcePath &&
      shape.sourceNodeId === 'material_center_header')!;
    expect(header).toBeDefined();
    expect(header.current.container.direction).toBe('row');
    const gap = header.current.container.columnGap + 1;
    header.current.container.columnGap = gap;
    const exported = reconcileReviewSource(prepared.input, snapshot);
    const manifest = parseSourceExportManifest(exported.document['penpot_source_exports']);
    expect(validateRealOwnerExportSources(manifest, sourcePath, expectedSources, sha256(source))).toBe(10);
    expect(manifest.edits).toEqual([{
      exportNodeId: header.nodeId, sourcePath, sourceNodeId: 'material_center_header',
      controlId: 'WorkbenchMaterialCenterHeader', instancePath: '[]',
      fieldPath: ['layout', 'container', 'gap'], value: gap,
    }]);
    if (typeof exported.source !== 'string') throw new Error('Material owner export did not produce source text');
    const applied = applyZuiExportToSourceFiles(exported.source, canonical, expectedSources);
    expect(applied.appliedPathsBySource).toEqual({
      [sourcePath]: ['nodes.material_center_header.layout.container.gap'],
    });
    expect(applied.outputs.map(output => output.sourcePath)).toEqual([sourcePath]);
    const actual = parseZuiDocument(applied.outputs[0]!.source).document;
    const expected = cloneZuiDocument(original);
    (expected.nodes!['material_center_header']!.layout!['container'] as Record<string, unknown>)['gap'] = gap;
    expect(normalizeZuiDocument(actual)).toEqual(normalizeZuiDocument(expected));
    expect(actual.nodes!['material_workspace']!.props?.['visibility']).toBe('collapsed');
    expect(actual.imports).toEqual(original.imports);
    expect(actual.components).toEqual(original.components);
    expect(Object.values(actual.nodes!).map(node => node.control_id))
      .toEqual(Object.values(original.nodes!).map(node => node.control_id));
    expect(Object.values(actual.nodes!).flatMap(node => node.events ?? []))
      .toEqual(Object.values(original.nodes!).flatMap(node => node.events ?? []));
    for (const path of paths) expect(await readFile(repo + '/' + path, 'utf8')).toBe(canonical[path]);
  });

  it('still rejects a generated review callsite if it is reintroduced into the changed export ancestry', async () => {
    const { canonical, expectedSources, prepared, host } = await preparedMaterial();
    for (const node of Object.values(host.nodes!)) {
      const rawPath = node['penpot_review_instance_path'];
      if (typeof rawPath === 'string')
        node['penpot_review_instance_path'] = JSON.stringify([
          { sourcePath, sourceNodeId: 'review_component' },
          ...JSON.parse(rawPath) as Step[],
        ]);
    }
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(host));
    const header = snapshot.shapes.find(shape => shape.sourcePath === sourcePath &&
      shape.sourceNodeId === 'material_center_header')!;
    expect(header).toBeDefined();
    header.current.container.columnGap += 1;
    const exported = reconcileReviewSource(prepared.input, snapshot);
    if (typeof exported.source !== 'string') throw new Error('Material owner export did not produce source text');
    expect(() => applyZuiExportToSourceFiles(exported.source!, canonical, expectedSources))
      .toThrow('no instance call-site node review_component');
  });
});
