import {
  describeChange,
  describeDefects,
  isSourceLayoutPath,
  classifySourcePath,
  isWorkbenchWindowReviewDocument,
  isStructuralWorkbenchHostDocument,
  renderIndex,
  sourceAssetRoot,
  collectComponentDependencies,
  trackedZuiPaths,
  discoverZuiSources,
  type CatalogEntry,
} from '../tools/zui-layout-catalog.js';
import { mkdtemp, mkdir, writeFile, rm, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import {
  appendEmbeddedScrollReviewCases,
  defaultReviewCases,
  caseSha256,
  casesSha256,
  validateLayoutReviewCase,
  validateLayoutReviewCases,
} from '../tools/zui-layout-review-contract';
import {
  bytesSha256,
  dependenciesSha256,
  dependencyFingerprints,
  dualEvidenceErrors,
  currentDualReview,
  verifyCurrentFiles,
} from '../tools/zui-layout-evidence';
import { recordReview } from '../tools/zui-layout-review';
import { parseZuiDocument } from './bridge/zui-document';

const testRepoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(process.cwd(), '../../../../'));

const projectOwnedReactBitsFixtures = [
  'zircon_runtime/tests/fixtures/ui/reactbits_agent_workspace.zui',
  'zircon_runtime/tests/fixtures/ui/reactbits_agent_native_components.zui',
  'zircon_runtime/tests/fixtures/ui/reactbits_workbench_interaction_surfaces.zui',
  'zircon_runtime/tests/fixtures/ui/reactbits_agent_workflow_components.zui',
  'zircon_runtime/tests/fixtures/ui/reactbits_data_surface_components.zui',
  'zircon_runtime/tests/fixtures/ui/reactbits_auth_onboarding_components.zui',
  'zircon_runtime/tests/fixtures/ui/reactbits_settings_form_components.zui',
  'zircon_runtime/tests/fixtures/ui/reactbits_kanban_scheduling_components.zui',
] as const;

describe('ZUI layout catalog reports', () => {
  it('inventories source layouts without recursively ingesting generated copies', () => {
    expect(isSourceLayoutPath('zircon_editor/assets/ui/new.zui')).toBe(true);
    expect(isSourceLayoutPath('docs/_data/layout/editor-ui/new/new.zui')).toBe(false);
    expect(isSourceLayoutPath('dev/reference/test.zui')).toBe(false);
    expect(
      isSourceLayoutPath(
        'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui',
      ),
    ).toBe(true);
    const wocTheme = defaultReviewCases(
      'examples/woc/assets/ui/theme/game_tokens.zui',
      'theme_tokens',
    );
    expect(wocTheme[0]).toMatchObject({
      host: 'woc',
      viewport: { width: 1920, height: 1080 },
    });
    expect(wocTheme.some((item) => item.state === 'disabled')).toBe(true);
    expect(isSourceLayoutPath('target/bundle/new.zui')).toBe(false);
    expect(isSourceLayoutPath('docs\\_data\\layout\\editor-ui\\new\\new.zui')).toBe(
      false,
    );
    expect(isSourceLayoutPath('./zircon_editor/assets/ui/new.zui')).toBe(true);
  });

  it('classifies authored sources independently from category and preserves asset roots', async () => {
    expect(
      classifySourcePath(
        'zircon_editor/assets/ui/editor/components/button.zui',
        'component',
      ),
    ).toBe('component');
    expect(
      classifySourcePath(
        'zircon_editor/assets/ui/editor/theme/tokens.zui',
        'theme_tokens',
      ),
    ).toBe('theme');
    expect(classifySourcePath('examples/woc/assets/ui/shell.zui', 'view')).toBe(
      'dynamic-host',
    );
    expect(
      classifySourcePath('examples/woc/assets/ui/hud/hud_theme.zui', 'style'),
    ).toBe('theme');
    expect(
      classifySourcePath(
        'zircon_editor/assets/ui/editor/project_overview.zui',
        'view',
      ),
    ).toBe('product-page');
    const workbenchHost = {
      asset: {
        kind: 'component' as const,
        id: 'res://workbench_module_workspace.zui',
        version: 2 as const,
      },
      nodes: {
        root: { component: 'Overlay' },
        body: { component: 'WorkbenchMaterialWorkspace' },
      },
    };
    expect(isStructuralWorkbenchHostDocument(workbenchHost)).toBe(true);
    const activityWindow = parseZuiDocument(
      await readFile(
        resolve(
          testRepoRoot,
          'zircon_editor/assets/ui/editor/components/workbench/shell/activity_drawer_window.zui',
        ),
        'utf8',
      ),
    ).document;
    // ActivityDrawerWindow is slot-only in its source component, but the
    // catalog has a deterministic product-window host for those slots.
    expect(isStructuralWorkbenchHostDocument(activityWindow)).toBe(false);
    const assetBrowser = parseZuiDocument(
      await readFile(
        resolve(
          testRepoRoot,
          'zircon_editor/assets/ui/editor/asset_browser.zui',
        ),
        'utf8',
      ),
    ).document;
    expect(
      assetBrowser.nodes?.['toolbar_title_row']?.layout?.['height'],
    ).toMatchObject({
      min: 24,
      preferred: 24,
      max: 24,
      stretch: 'Fixed',
    });
    expect(
      assetBrowser.nodes?.['toolbar_panel']?.layout?.['height'],
    ).toMatchObject({
      min: 112,
      preferred: 112,
      max: 112,
      stretch: 'Fixed',
    });
    expect(
      assetBrowser.nodes?.['content_header_path_text']?.props?.[
        'responsive_min_tier'
      ],
    ).toBe('wide');
    for (const nodeId of [
      'sources_title_text',
      'details_header_title_text',
      'reference_left_title_text',
      'reference_right_title_text',
    ]) {
      expect(assetBrowser.nodes?.[nodeId]?.layout?.['height']).toMatchObject({
        min: 24,
        preferred: 24,
        max: 24,
        stretch: 'Fixed',
      });
    }
    expect(
      assetBrowser.nodes?.['sources_subtitle_text']?.layout?.['position'],
    ).toMatchObject({
      y: 32,
    });
    expect(
      assetBrowser.nodes?.['details_header_selection_text']?.layout?.[
        'position'
      ],
    ).toMatchObject({ y: 32 });
    expect(
      assetBrowser.nodes?.['reference_left_scroll_body']?.layout?.['position'],
    ).toMatchObject({ y: 28 });
    expect(
      assetBrowser.nodes?.['reference_right_scroll_body']?.layout?.['position'],
    ).toMatchObject({ y: 28 });
    for (const [relativePath, nodeId] of [
      [
        'zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_perception_workspace.zui',
        'perception_map_row_01',
      ],
      [
        'zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui',
        'ability_graph_row',
      ],
      [
        'zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui',
        'render_graph_row_02',
      ],
    ] as const) {
      const source = parseZuiDocument(
        await readFile(resolve(testRepoRoot, relativePath), 'utf8'),
      ).document;
      expect(source.nodes?.[nodeId]?.layout?.['height']).toMatchObject({
        min: 48,
        preferred: 48,
        max: 48,
        stretch: 'Fixed',
      });
    }
    const blendSpace = parseZuiDocument(
      await readFile(
        resolve(
          testRepoRoot,
          'zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui',
        ),
        'utf8',
      ),
    ).document;
    expect(
      blendSpace.nodes?.['blend_space_preview_camera']?.layout?.['width'],
    ).toMatchObject({
      min: 96,
      preferred: 96,
      max: 112,
      stretch: 'Fixed',
    });
    for (const name of ['workbench_panel_header', 'workbench_diagnostic_row']) {
      const family = name === 'workbench_panel_header' ? 'chrome' : 'feedback';
      const path = resolve(
        testRepoRoot,
        `zircon_editor/assets/ui/editor/components/workbench/composites/${family}/${name}.zui`,
      );
      const source = parseZuiDocument(await readFile(path, 'utf8')).document;
      expect(isStructuralWorkbenchHostDocument(source)).toBe(false);
      expect(classifySourcePath(path, source.asset.kind, source)).toBe(
        'component',
      );
    }
    expect(
      classifySourcePath(
        'zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui',
        'component',
        workbenchHost,
      ),
    ).toBe('dynamic-host');
    expect(
      isStructuralWorkbenchHostDocument({
        ...workbenchHost,
        nodes: {
          ...workbenchHost.nodes,
          title: { component: 'Label', props: { text: 'Workspace' } },
        },
      }),
    ).toBe(false);
    expect(
      classifySourcePath(
        'zircon_runtime/assets/ui/runtime/fixtures/pause_menu.zui',
        'view',
      ),
    ).toBe('test-fixture');
    expect(
      classifySourcePath(
        'zircon_editor/src/tests/fixtures/ui_zui/editor/button.zui',
        'view',
      ),
    ).toBe('test-fixture');
    expect(
      classifySourcePath(
        'zircon_plugins/navigation/editor/agents_areas.zui',
        'view',
        {
          asset: { kind: 'view', id: 'navigation.agents_areas', version: 2 },
          nodes: {
            root: { component: 'VerticalGroup' },
            mount: { component: 'Space' },
          },
        },
      ),
    ).toBe('dynamic-host');
    expect(
      classifySourcePath('zircon_plugins/navigation/editor/bake.zui', 'view', {
        asset: { kind: 'view', id: 'navigation.bake', version: 2 },
        nodes: {
          root: { component: 'VerticalGroup' },
          title: { component: 'Label', props: { text: 'Bake' } },
        },
      }),
    ).toBe('product-page');
    expect(sourceAssetRoot('zircon_editor/assets/ui/editor/button.zui')).toBe(
      'zircon_editor/assets',
    );
    expect(
      sourceAssetRoot(
        'zircon_editor/src/tests/fixtures/ui_zui/editor/button.zui',
      ),
    ).toBe('zircon_editor/src/tests/fixtures/ui_zui');
  });

  it('gives the inspector layer label enough width to keep its text on one line', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    expect(source.nodes?.inspector_layer_label?.layout?.width).toEqual({
      min: 72.0,
      preferred: 72.0,
      max: 72.0,
      stretch: 'Fixed',
    });
    expect(source.nodes?.inspector_layer_mask?.layout?.width).toMatchObject({
      min: 96.0,
      preferred: 112.0,
      max: 120.0,
      stretch: 'Stretch',
    });
  });

  it('does not retain popup cases from a pruned Workbench module branch', async () => {
    const catalog = JSON.parse(
      await readFile(resolve(testRepoRoot, 'docs/_data/layout/catalog.json'), 'utf8'),
    ) as {
      entries: Array<{ sourcePath: string; cases?: Array<{ state: string }> }>;
    };
    const entry = catalog.entries.find(
      ({ sourcePath }) =>
        sourcePath ===
        'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui',
    );
    expect(entry?.cases?.some(({ state }) => state === 'open')).toBe(false);
    expect(entry?.cases?.some(({ state }) => state === 'closed')).toBe(false);
    expect(entry?.cases?.some(({ state }) => state === 'focus-return')).toBe(
      false,
    );
  });

  it('keeps wide status chips single-line inside the 24px status bar', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    for (const nodeId of ['status_grid', 'status_snap', 'status_zoom']) {
      expect(source.nodes?.[nodeId]?.props).toMatchObject({
        layout_padding_top: 0.0,
        layout_padding_bottom: 0.0,
      });
    }
    expect(source.nodes?.status_grid?.layout?.width).toEqual({
      min: 96.0,
      preferred: 96.0,
      max: 96.0,
      stretch: 'Fixed',
    });
    expect(source.nodes?.status_snap?.layout?.width).toEqual({
      min: 80.0,
      preferred: 80.0,
      max: 80.0,
      stretch: 'Fixed',
    });
  });

  it('gives compact showcase state values a content-driven two-line row', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/showcase/showcase_state_panel.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    for (const nodeId of [
      'last_control',
      'last_action',
      'current_value',
      'drag_payload',
    ]) {
      expect(source.nodes?.[nodeId]?.layout?.height).toEqual({
        min: 32.0,
        preferred: 48.0,
        max: 48.0,
        stretch: 'Fixed',
      });
    }
  });

  it('keeps the blend-space sample label inside its compact property row', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_blend_space_details.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    expect(source.nodes?.sample_position?.layout?.height).toEqual({
      min: 28.0,
      preferred: 48.0,
      max: 48.0,
      stretch: 'Fixed',
    });
  });

  it('uses a wide-only context drawer for the UI asset editor workspace', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    expect(source.nodes?.ui_asset_editor_workspace?.component).toBe('Overlay');
    expect(source.nodes?.ui_asset_editor_workspace_main?.component).toBe(
      'HorizontalGroup',
    );
    expect(
      source.nodes?.ui_asset_editor_right?.props?.responsive_min_tier,
    ).toBe('wide');
    expect(
      source.nodes?.ui_asset_editor_right_reserve?.props?.responsive_min_tier,
    ).toBe('wide');
    expect(source.nodes?.ui_asset_editor_center_header?.component).toBe(
      'ScrollableBox',
    );
    expect(source.nodes?.ui_asset_editor_table?.component).toBe(
      'ScrollableBox',
    );
  });

  it('keeps generated routes reachable through supported perpendicular scroll owners', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    const horizontal =
      source.nodes?.generated_bottom_route_table_horizontal_viewport;
    const vertical = source.nodes?.generated_bottom_route_table;
    expect(horizontal?.component).toBe('ScrollableBox');
    expect(horizontal?.layout?.container).toMatchObject({
      kind: 'ScrollableBox',
      axis: 'Horizontal',
      scrollbar_visibility: 'Auto',
    });
    expect(horizontal?.layout?.clip).toBe(true);
    expect(horizontal?.layout?.width).toEqual({ stretch: 'Stretch' });
    expect(horizontal?.layout?.height).toEqual({ stretch: 'Stretch' });
    expect(horizontal?.children).toEqual([
      { node: 'generated_bottom_route_table' },
    ]);
    expect(source.nodes?.generated_bottom_body?.children?.[0]).toEqual({
      node: 'generated_bottom_route_table_horizontal_viewport',
    });
    expect(vertical?.control_id).toBe('WorkbenchGeneratedBottomRouteTable');
    expect(vertical?.layout?.clip).toBe(true);
    expect(vertical?.layout?.height).toEqual({ stretch: 'Stretch' });
    expect(vertical?.layout?.width).toMatchObject({
      min: 360,
      preferred: 640,
      stretch: 'Stretch',
    });
    const rows = source.nodes?.generated_bottom_route_table_column?.children;
    expect(rows).toHaveLength(37);
    for (const { node } of rows ?? []) {
      expect(source.nodes?.[node]?.component).toBe('WorkbenchTableRow');
      expect(source.nodes?.[node]?.control_id).toBeTruthy();
    }
    expect(source.nodes?.generated_bottom_route_table?.component).toBe(
      'ScrollableBox',
    );
    expect(
      source.nodes?.generated_bottom_route_table?.layout?.container,
    ).toMatchObject({ kind: 'ScrollableBox', axis: 'Vertical' });
    expect(source.nodes?.generated_bottom_route_table?.children).toEqual([
      { node: 'generated_bottom_route_table_column' },
    ]);
    expect(source.nodes?.generated_bottom_route_table_rows).toBeUndefined();
    expect(
      source.nodes?.generated_bottom_route_table_column?.layout?.width,
    ).toMatchObject({ min: 360, preferred: 640, stretch: 'Stretch' });
    expect(
      source.nodes?.generated_bottom_route_table_column?.layout?.height,
    ).toEqual({ min: 1124, preferred: 1124, max: 1124, stretch: 'Fixed' });
  });

  it('keeps Material metadata readable at the 240px component width', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/material_components/data_display/material_chips.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    expect(source.nodes?.meta?.layout?.height).toMatchObject({
      min: 100,
      preferred: 100,
      max: 100,
      stretch: 'Fixed',
    });
    for (const nodeId of [
      'meta_group',
      'meta_response',
      'meta_variant',
      'meta_layout',
    ]) {
      expect(source.nodes?.[nodeId]?.layout?.height).toMatchObject({
        min: 48,
        preferred: 48,
        max: 48,
        stretch: 'Fixed',
      });
    }
    expect(source.nodes?.root?.layout?.height).toMatchObject({
      min: 336,
      preferred: 336,
      max: 336,
      stretch: 'Fixed',
    });
  });

  it('reserves the measured asset scope title slot in the workbench header', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    expect(source.nodes?.assets_center_title?.layout?.width).toEqual({
      min: 220.0,
      preferred: 220.0,
      max: 280.0,
      stretch: 'Fixed',
    });
  });

  it('keeps the component showcase main slot dominant at the compact shell tier', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/component_showcase.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    const body = source.nodes?.component_showcase_body;
    const children = body?.children ?? [];
    expect(children[0]?.slot?.layout?.width).toEqual({
      min: 120.0,
      preferred: 160.0,
      max: 180.0,
      stretch: 'Fixed',
    });
    expect(children[1]?.slot?.layout?.width).toEqual({
      min: 220.0,
      preferred: 360.0,
      stretch: 'Stretch',
    });
    expect(children[2]?.slot?.layout?.width).toEqual({
      min: 160.0,
      preferred: 240.0,
      max: 260.0,
      stretch: 'Fixed',
    });
  });

  it('reserves a wrapped heading row for the compact collections showcase', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/components/showcase/showcase_collections_section.zui',
    );
    const source = parseZuiDocument(
      await readFile(sourcePath, 'utf8'),
    ).document;
    expect(source.nodes?.section_title?.layout?.height).toEqual({
      min: 24.0,
      preferred: 44.0,
      max: 44.0,
      stretch: 'Fixed',
    });
  });

  it('records unique authored component references in stable order', () => {
    expect(
      collectComponentDependencies({
        asset: { kind: 'component', id: 'x', version: 2 },
        components: { Button: { root: 'a' } },
        nodes: {
          a: { component: 'Label' },
          b: { component: 'Label' },
          c: { component: 'Button' },
        },
      }),
    ).toEqual(['Button', 'Label']);
  });

  it('retains Workbench window findings after imported components are materialized', async () => {
    const path = resolve(
      testRepoRoot,
      'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
    );
    const source = parseZuiDocument(await readFile(path, 'utf8')).document;
    const expanded = {
      ...source,
      nodes: {
        ...source.nodes,
        top_toolbar: {
          ...source.nodes['top_toolbar'],
          component: 'VerticalGroup',
        },
      },
    };
    expect(isWorkbenchWindowReviewDocument(source)).toBe(true);
    expect(isWorkbenchWindowReviewDocument(expanded)).toBe(true);
    expect(
      isWorkbenchWindowReviewDocument({
        ...expanded,
        nodes: {
          ...expanded.nodes,
          top_toolbar: {
            ...expanded.nodes['top_toolbar'],
            control_id: 'UnrelatedToolbar',
          },
        },
      }),
    ).toBe(false);
  });

  it('exposes the ReactBits layout-mode mapping from the generated catalog index', () => {
    const index = renderIndex({
      schema: 'dev.zircon.zui.layout-catalog',
      version: 1,
      generatedAt: '2026-09-24T00:00:00.000Z',
      repoRoot: testRepoRoot,
      sourceCount: 0,
      entries: [],
    });
    expect(index).toContain(
      '[ReactBits App UI 布局模式](../ui-and-layout/reactbits-app-shell-layout-modes.md)',
    );
  });

  it('enumerates product sources plus each project-owned fixture once', async () => {
    const paths = await trackedZuiPaths();
    const bridgeFixture =
      'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';

    expect(paths).toHaveLength(328);
    expect(new Set(paths).size).toBe(paths.length);
    expect(paths).toEqual(
      [...paths].sort((left, right) => left.localeCompare(right, 'en')),
    );
    expect(paths.filter((path) => path === bridgeFixture)).toHaveLength(1);
    for (const fixture of projectOwnedReactBitsFixtures)
      expect(paths.filter((path) => path === fixture)).toHaveLength(1);
    // Windows scans both the index and the large unignored source tree.  Keep
    // the assertion bounded, but leave enough headroom for concurrent Cargo
    // work and antivirus/indexer I/O so a healthy inventory is not reported
    // as a flaky timeout.
  }, 30000);

  it('records the explicit bridge fixture separately from the Git source scan', async () => {
    const discovery = await discoverZuiSources();
    expect(discovery.metadata.scan).toBe(
      'git-ls-files-cached-others-exclude-standard',
    );
    expect(discovery.metadata.controlledOrUnignoredSourceCount).toBe(327);
    expect(discovery.metadata.projectOwnedScannedFixtures).toEqual(
      projectOwnedReactBitsFixtures.map((sourcePath) =>
        expect.objectContaining({ sourcePath }),
      ),
    );
    expect(discovery.metadata.explicitProjectFixtures).toEqual([
      expect.objectContaining({
        sourcePath:
          'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui',
      }),
    ]);
    expect(discovery.metadata.excludedRoots).toEqual(
      expect.arrayContaining(['docs/_data/layout/**', 'dev/**', 'target/**']),
    );
    expect(discovery.paths).toHaveLength(328);
  }, 15000);

  it('keeps agent workflow components source-owned across the native painter boundary', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_runtime/tests/fixtures/ui/reactbits_agent_workflow_components.zui',
    );
    const source = await readFile(sourcePath, 'utf8');
    const document = parseZuiDocument(source).document;
    const root = document.nodes?.['root'];
    expect(root?.props?.['reference_primary']).toBe(
      'https://pro.reactbits.dev/docs/app-ui/agent-plan',
    );
    expect(root?.props?.['reference_supporting']).toEqual(
      expect.arrayContaining([
        'https://pro.reactbits.dev/docs/app-ui/tool-calls',
        'https://pro.reactbits.dev/docs/app-ui/agent-approval',
        'https://pro.reactbits.dev/docs/app-ui/ai-usage',
      ]),
    );
    const workflowNodes = Object.values(document.nodes ?? {}).filter((node) =>
      ['AgentPlan', 'ToolCalls', 'AgentApproval', 'AIUsage'].includes(
        node.component,
      ),
    );
    expect(workflowNodes).toHaveLength(4);
    expect(workflowNodes.map((node) => node.props?.['component_role'])).toEqual(
      expect.arrayContaining([
        'mui-x-agent-plan',
        'mui-x-tool-calls',
        'mui-x-agent-approval',
        'mui-x-ai-usage',
      ]),
    );
    // The same Windows index/unignored-tree scan is intentionally exercised
    // twice in this suite; allow the second pass to run while Cargo is busy.
  }, 60000);

  it('keeps the workbench interaction fixture mapped to source-owned semantics', async () => {
    const sourcePath = resolve(
      testRepoRoot,
      'zircon_runtime/tests/fixtures/ui/reactbits_workbench_interaction_surfaces.zui',
    );
    const source = await readFile(sourcePath, 'utf8');
    const document = parseZuiDocument(source).document;
    const root = document.nodes?.['root'];
    expect(root?.props?.['reference_primary']).toBe(
      'https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4',
    );
    expect(root?.props?.['reference_supporting']).toEqual(
      expect.arrayContaining([
        'https://pro.reactbits.dev/docs/app-ui/command-menu',
        'https://pro.reactbits.dev/docs/app-ui/app-dialog',
        'https://pro.reactbits.dev/docs/app-ui/notifications',
        'https://pro.reactbits.dev/docs/app-ui/mobile',
      ]),
    );
    expect(
      Object.values(document.nodes ?? {})
        .map((node) => node.component)
        .filter((component) =>
          [
            'CommandPalette',
            'Dialog',
            'ConfirmDialog',
            'NotificationCenter',
            'DragOverlay',
            'WorkbenchToast',
          ].includes(component),
        ),
    ).toEqual(
      expect.arrayContaining([
        'CommandPalette',
        'Dialog',
        'ConfirmDialog',
        'NotificationCenter',
        'DragOverlay',
        'WorkbenchToast',
      ]),
    );
  });
  it('translates optimizer change ids into reviewable Penpot mappings', () => {
    expect(describeChange('fit-component-root-to-content')).toContain(
      '组件根节点',
    );
    expect(describeChange('apply-penpot-prefab-classes')).toContain('预制件');
    expect(describeChange('fit-auto-layout-to-content')).toContain('自动布局');
  });

  it('explains the original defect and foundation coverage', () => {
    const entry = {
      sourceFormat: 'legacy',
      changes: ['migrate-legacy-document', 'embed-penpot-prefab-tokens'],
      diagnostics: [],
      nodeCount: 4,
      projectedShapeCount: 4,
      prefabNodeCount: 4,
      prefabCoverage: 1,
    } as unknown as CatalogEntry;

    expect(describeDefects(entry).join(' ')).toContain('旧版');
    expect(describeDefects(entry).join(' ')).toContain('预制件');
  });

  it('reports undersized auto-layout containers as a source defect', () => {
    const entry = {
      sourceFormat: 'v2',
      changes: ['fit-auto-layout-to-content'],
      diagnostics: [],
    } as unknown as CatalogEntry;

    expect(describeDefects(entry).join(' ')).toContain('父容器');
  });

  it('validates the shared LayoutReviewCase envelope before hashing it', () => {
    const value = {
      id: 'default-360x520-dpi1',
      sourcePath: 'zircon_editor/assets/ui/editor/button.zui',
      host: 'component',
      viewport: { width: 360, height: 520 },
      dpi: 1.5,
      locale: 'en-US',
      state: 'default',
      data: {},
    };
    expect(validateLayoutReviewCase(value)).toEqual(value);
    expect(() =>
      validateLayoutReviewCase({
        ...value,
        viewport: { ...value.viewport, width: 0 },
      }),
    ).toThrow(/viewport\.width/);
    expect(() => validateLayoutReviewCase({ ...value, unknown: true })).toThrow(
      /Unknown LayoutReviewCase field/,
    );
    expect(() => validateLayoutReviewCases([value, { ...value }])).toThrow(
      /Duplicate LayoutReviewCase/,
    );
  });
});

describe('dual-renderer catalog review', () => {
  const makeEntry = (): CatalogEntry => {
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/components/button.zui',
      'component',
    );
    const sourceSha256 = bytesSha256('source');
    const dependencySha256 = dependenciesSha256([]);
    const evidence = cases.map((item) => ({
      caseId: item.id,
      status: 'passed' as const,
      screenshotPath: `evidence/${item.id}.png`,
      screenshotSha256: bytesSha256(item.id),
      rendererPath: 'renderer.js',
      rendererSha256: bytesSha256('renderer'),
      captureProgramFingerprints: [
        ['renderer.js', bytesSha256('renderer')],
      ] as Array<[string, string]>,
      geometryPath: `evidence/${item.id}.geometry.json`,
      geometrySha256: bytesSha256('geometry'),
      textPath: `evidence/${item.id}.text.json`,
      textSha256: bytesSha256('text'),
      sourceSha256,
      inputSha256: bytesSha256('projection'),
      dependencySha256,
      caseSha256: caseSha256(item),
    }));
    return {
      sourcePath: cases[0].sourcePath,
      outputPath: 'copy.zui',
      sourceSha256,
      outputSha256: sourceSha256,
      status: 'prepared',
      dependencyFingerprints: [],
      dependencySha256,
      penpotInputPath: 'evidence/penpot-input.zui',
      penpotInputSha256: bytesSha256('projection'),
      cases,
      penpotEvidence: structuredClone(evidence),
      engineEvidence: evidence.map((item) => ({
        ...item,
        inputSha256: sourceSha256,
        rendererKind: 'zircon-editor-retained-host',
      })),
    } as unknown as CatalogEntry;
  };

  it('defines responsive component, page, theme and fixture cases without inventing data', () => {
    const entry = makeEntry();
    expect(entry.cases?.map((item) => [item.viewport.width, item.dpi])).toEqual(
      [
        [360, 1],
        [360, 1.5],
        [240, 1],
        [480, 1],
      ],
    );
    expect(
      defaultReviewCases('examples/woc/assets/ui/shell.zui', 'view').map(
        (item) => item.viewport.width,
      ),
    ).toEqual([1920, 1920, 1280]);
    expect(
      defaultReviewCases(
        'zircon_editor/assets/ui/editor/toolbar.zui',
        'component',
      )[0].viewport.width,
    ).toBe(1280);
    expect(
      defaultReviewCases(
        'zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui',
        'component',
      ).map((item) => item.viewport.width),
    ).toEqual([1280, 1280, 900, 640]);
    expect(
      defaultReviewCases(
        'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui',
        'component',
      )[0].host,
    ).toBe('editor');
    for (const name of ['component_showcase', 'material_component_lab'])
      expect(
        defaultReviewCases(
          `zircon_editor/assets/ui/editor/views/${name}.zui`,
          'view',
        ).map((item) => item.viewport.width),
      ).toEqual([1280, 1280, 900, 640]);
    expect(
      defaultReviewCases('zircon_editor/assets/theme.zui', 'theme_tokens').some(
        (item) => item.state === 'disabled',
      ),
    ).toBe(true);
    const fixture = defaultReviewCases(
      'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui',
      'view',
    );
    expect(fixture[0].host).toBe('fixture');
    expect(fixture.every((item) => Object.keys(item.data).length === 0)).toBe(
      true,
    );
    const scrollCases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/scrolling_page.zui',
      'view',
      {
        asset: { kind: 'view', id: 'scrolling', version: 2 },
        root: { node: 'root' },
        nodes: {
          root: {
            component: 'ScrollableBox',
            layout: {
              container: { kind: 'ScrollableBox', axis: 'Vertical' },
            },
          },
        },
      },
    );
    expect(
      scrollCases
        .filter((item) => item.state.startsWith('scroll-'))
        .map((item) => [item.id, item.viewport.width, item.viewport.height]),
    ).toEqual([
      ['scroll-before-640x520-dpi1', 640, 520],
      ['scroll-after-640x520-dpi1', 640, 520],
    ]);
  });

  it('adds page scroll scenes when a real imported component owns the scroll region', () => {
    const sourcePath =
      'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
    const source = {
      asset: { kind: 'view' as const, id: 'window', version: 2 as const },
      root: { node: 'root' },
      nodes: { root: { component: 'VerticalGroup' } },
    };
    const original = defaultReviewCases(sourcePath, 'view', source);
    expect(original.some((item) => item.state === 'scroll-after')).toBe(false);
    const expanded = {
      ...source,
      nodes: {
        ...source.nodes,
        drawer_body: {
          component: 'ScrollableBox',
          layout: { container: { kind: 'ScrollableBox', axis: 'Vertical' } },
        },
        feedback_toast: {
          component: 'WorkbenchToast',
          props: {
            text: 'Operation completed',
            open: false,
            popup_open: false,
          },
        },
      },
    };
    const cases = appendEmbeddedScrollReviewCases(original, expanded);
    expect(
      cases
        .filter((item) => item.state.startsWith('scroll-'))
        .map(({ id, viewport, host, sourcePath: path }) => ({
          id,
          viewport,
          host,
          sourcePath: path,
        })),
    ).toEqual([
      {
        id: 'scroll-before-640x520-dpi1',
        viewport: { width: 640, height: 520 },
        host: 'editor',
        sourcePath,
      },
      {
        id: 'scroll-after-640x520-dpi1',
        viewport: { width: 640, height: 520 },
        host: 'editor',
        sourcePath,
      },
    ]);
    expect(appendEmbeddedScrollReviewCases(cases, expanded)).toEqual(cases);
    expect(cases).toContainEqual(
      expect.objectContaining({
        id: 'open-scroll-after-640x520-dpi1',
        sourcePath,
        host: 'editor',
        state: 'open',
        scrollPosition: 'end',
      }),
    );
    expect(appendEmbeddedScrollReviewCases(original, source)).toEqual(original);
  });

  it('adds an observable open-and-scroll scene when a page contains both behaviors', () => {
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/components/workbench/shell/showcase.zui',
      'view',
      {
        asset: { kind: 'view', id: 'showcase', version: 2 },
        root: { node: 'root' },
        nodes: {
          root: {
            component: 'ScrollableBox',
            layout: { container: { kind: 'ScrollableBox', axis: 'Vertical' } },
            children: [{ node: 'toast' }],
          },
          toast: {
            component: 'WorkbenchToast',
            props: {
              text: 'Operation completed',
              open: false,
              popup_open: false,
            },
          },
        },
      },
    );
    expect(cases).toContainEqual(
      expect.objectContaining({
        id: 'open-scroll-after-640x520-dpi1',
        state: 'open',
        scrollPosition: 'end',
        viewport: { width: 640, height: 520 },
      }),
    );
  });

  it('fingerprints case data and DPI with stable object-key ordering', () => {
    const value = makeEntry().cases![0];
    expect(caseSha256({ ...value, data: { a: 1, b: 2 } })).toBe(
      caseSha256({ ...value, data: { b: 2, a: 1 } }),
    );
    expect(caseSha256(value)).not.toBe(caseSha256({ ...value, dpi: 1.5 }));
    expect(casesSha256([value])).not.toBe(
      casesSha256([{ ...value, data: { fixture: 'populated' } }]),
    );
  });

  it('derives component states from declared capabilities regardless of file name', () => {
    for (const name of ['button', 'icon_button']) {
      const cases = defaultReviewCases(
        `zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_${name}.zui`,
        'component',
        {
          asset: { kind: 'component', id: 'button', version: 2 },
          components: { ButtonSpecimen: { root: 'root' } },
          nodes: {
            root: {
              component: name === 'button' ? 'Button' : 'IconButton',
              props: {
                input_hoverable: true,
                input_clickable: true,
                input_focusable: true,
                input_interactive: true,
              },
            },
          },
        },
      );
      expect([...new Set(cases.map((item) => item.state))]).toEqual([
        'default',
        'hover',
        'pressed',
        'focused',
        'disabled',
        'selected',
      ]);
      for (const state of new Set(cases.map((item) => item.state)))
        expect(
          cases
            .filter((item) => item.state === state)
            .map((item) => [item.viewport.width, item.dpi]),
        ).toEqual([
          [360, 1],
          [360, 1.5],
          [240, 1],
          [480, 1],
        ]);
    }
    expect(
      defaultReviewCases(
        'zircon_editor/assets/ui/editor/components/custom_button_host.zui',
        'component',
      ).every((item) => item.state === 'default'),
    ).toBe(true);
    expect(
      defaultReviewCases(
        'zircon_runtime/assets/ui/runtime/fixtures/button.zui',
        'view',
      )[0].host,
    ).toBe('fixture');
  });

  it.each([
    ['Dropdown', 'popup_open'],
    ['DropdownPopup', 'open'],
    ['ContextMenu', 'popup_open'],
    ['ContextActionMenu', 'popup_open'],
    ['CommandPalette', 'popup_open'],
    ['NotificationCenter', 'open'],
  ])(
    'registers both popup scenes for a %s with authored %s',
    (component, switchName) => {
      const cases = defaultReviewCases(
        'zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/unrelated-name.zui',
        'component',
        {
          asset: { kind: 'component', id: 'popup-review', version: 2 },
          components: { PopupReview: { root: 'root' } },
          nodes: {
            root: {
              component,
              props: { [switchName]: false },
            },
          },
        },
      );
      for (const state of ['open', 'closed', 'focus-return']) {
        expect(
          cases
            .filter((item) => item.state === state)
            .map((item) => [item.viewport.width, item.dpi]),
        ).toEqual([
          [360, 1],
          [360, 1.5],
          [240, 1],
          [480, 1],
        ]);
      }
      expect(cases[0].state).toBe('open');
    },
  );

  it('adds popup scenes for nested authored menu nodes on a page', () => {
    const source = {
      asset: { kind: 'view' as const, id: 'nested-popup', version: 2 as const },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'VerticalGroup',
          children: [{ node: 'menu' }],
        },
        menu: {
          component: 'ContextMenu',
          props: { popup_open: false, menu_items: ['Open'] },
        },
      },
    };
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/views/nested-popup.zui',
      'view',
      source,
    );
    expect(new Set(cases.map((item) => item.state))).toEqual(
      new Set(['default', 'open', 'closed', 'focus-return']),
    );
  });

  it('does not invent popup scenes for a non-popup root with an open switch', () => {
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/drag-overlay.zui',
      'component',
      {
        asset: { kind: 'component', id: 'drag', version: 2 },
        components: { Drag: { root: 'root' } },
        nodes: { root: { component: 'DragOverlay', props: { open: false } } },
      },
    );
    expect(cases.some((item) => item.state === 'closed')).toBe(false);
  });

  it('requires every case in both engines with current source and dependency hashes', () => {
    const entry = makeEntry();
    expect(dualEvidenceErrors(entry)).toEqual([]);
    entry.engineEvidence!.pop();
    expect(dualEvidenceErrors(entry).join(' ')).toContain('engineEvidence');
    const stale = makeEntry();
    stale.penpotEvidence![0].dependencySha256 = bytesSha256('old-theme');
    expect(dualEvidenceErrors(stale).join(' ')).toContain('stale');
  });

  it('rejects an opened scene that is pixel-identical to its scroll-position baseline', () => {
    const entry = makeEntry();
    const scroll = {
      ...entry.cases![0],
      id: 'scroll-after-640x520-dpi1',
      viewport: { width: 640, height: 520 },
      state: 'scroll-after',
      scrollPosition: 'end' as const,
    };
    const open = {
      ...scroll,
      id: 'open-scroll-after-640x520-dpi1',
      state: 'open',
    };
    entry.cases = [scroll, open];
    const penpot = entry.penpotEvidence![0];
    const engine = entry.engineEvidence![0];
    entry.penpotEvidence = [scroll, open].map((reviewCase) => ({
      ...penpot,
      caseId: reviewCase.id,
      caseSha256: caseSha256(reviewCase),
      screenshotPath: `evidence/${reviewCase.id}.png`,
      screenshotSha256: bytesSha256('unchanged popup image'),
    }));
    entry.engineEvidence = [scroll, open].map((reviewCase) => ({
      ...engine,
      caseId: reviewCase.id,
      caseSha256: caseSha256(reviewCase),
      screenshotPath: `evidence/native-${reviewCase.id}.png`,
      screenshotSha256: bytesSha256(reviewCase.id),
    }));
    expect(dualEvidenceErrors(entry)).toContain(
      'penpotEvidence: open-scroll-after-640x520-dpi1 is pixel-indistinguishable from scroll-after-640x520-dpi1',
    );
    entry.penpotEvidence[1].screenshotSha256 = bytesSha256('visible popup');
    expect(dualEvidenceErrors(entry)).toEqual([]);
  });

  it('fingerprints dependency media without interpreting label text as a resource', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-dependencies-'));
    try {
      await mkdir(resolve(root, 'app/assets/icons'), { recursive: true });
      const sourcePath = 'app/assets/source.zui';
      const dependencyPath = 'app/assets/theme.zui';
      await writeFile(
        resolve(root, sourcePath),
        '[nodes.label]\ncomponent="Label"\n[nodes.label.props]\ntext="not-a-resource.png"\n',
      );
      await writeFile(
        resolve(root, dependencyPath),
        '[nodes.icon]\ncomponent="Icon"\n[nodes.icon.props]\nsource="res://icons/panel.svg"\n',
      );
      await writeFile(resolve(root, 'app/assets/icons/panel.svg'), '<svg/>');
      const first = await dependencyFingerprints(root, sourcePath, {
        penpot_dependency_sources: [dependencyPath],
      });
      expect(first.map((item) => item.sourcePath)).toEqual([
        'app/assets/icons/panel.svg',
        dependencyPath,
      ]);
      await writeFile(
        resolve(root, 'app/assets/icons/panel.svg'),
        '<svg>changed</svg>',
      );
      const second = await dependencyFingerprints(root, sourcePath, {
        penpot_dependency_sources: [dependencyPath],
      });
      expect(dependenciesSha256(first)).not.toBe(dependenciesSha256(second));
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it('fingerprints authored font manifests and their binary sources', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-font-dependencies-'));
    try {
      await mkdir(resolve(root, 'app/assets/fonts'), { recursive: true });
      const sourcePath = 'app/assets/source.zui';
      await writeFile(
        resolve(root, sourcePath),
        '[nodes.label]\ncomponent="Label"\n',
      );
      await writeFile(
        resolve(root, 'app/assets/fonts/ui.font.toml'),
        'source = "ui.ttc"\nfamily = "Fira Sans"\n',
      );
      await writeFile(resolve(root, 'app/assets/fonts/ui.ttc'), 'font-bytes');
      const fingerprints = await dependencyFingerprints(root, sourcePath, {
        nodes: {
          label: {
            style: {
              self: {
                font: { asset: 'res://fonts/ui.font.toml' },
              },
            },
          },
        },
      });
      expect(fingerprints.map((item) => item.sourcePath)).toEqual([
        'app/assets/fonts/ui.font.toml',
        'app/assets/fonts/ui.ttc',
      ]);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it('rejects old single-renderer acceptance and archives it when recording a new decision', () => {
    const entry = makeEntry();
    entry.review = {
      status: 'accepted',
      screenshotSha256: 'old',
      outputSha256: entry.outputSha256,
      observations: 'old review',
      reviewedAt: '2026-01-01',
    };
    expect(currentDualReview(entry)).toBe(false);
    recordReview(entry, {
      sourcePath: entry.sourcePath,
      decision: 'accepted',
      observations: 'Both renderers checked in all cases.',
      caseReviews: entry.cases!.map((item) => ({
        caseId: item.id,
        status: 'accepted',
        observations:
          'Node and text bounds align; no clipped or missing content.',
        maxGeometryDeltaPx: 0,
        textMatches: true,
        visibilityMatches: true,
      })),
    });
    expect(entry.reviewHistory).toHaveLength(1);
    expect(currentDualReview(entry)).toBe(true);
    entry.engineEvidence![0].screenshotSha256 = bytesSha256('changed');
    expect(currentDualReview(entry)).toBe(false);
  });

  it('allows revision review of a failed screenshot but detects changed bytes and dependencies', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-review-'));
    try {
      const entry = makeEntry();
      entry.sourcePath = 'source.zui';
      entry.dependencyFingerprints = [
        { sourcePath: 'theme.zui', sha256: bytesSha256('theme') },
      ];
      entry.dependencySha256 = dependenciesSha256(entry.dependencyFingerprints);
      entry.engineEvidence = [];
      entry.penpotEvidence = [
        {
          ...entry.penpotEvidence![0],
          status: 'failed',
          screenshotPath: 'evidence/failed.png',
          screenshotSha256: bytesSha256('image'),
        },
      ];
      await mkdir(resolve(root, 'evidence'));
      await writeFile(resolve(root, 'source.zui'), 'source');
      await writeFile(resolve(root, 'copy.zui'), 'source');
      await writeFile(resolve(root, 'theme.zui'), 'theme');
      await writeFile(resolve(root, 'evidence/penpot-input.zui'), 'projection');
      await writeFile(resolve(root, 'evidence/failed.png'), 'image');
      await writeFile(resolve(root, 'renderer.js'), 'renderer');
      await writeFile(
        resolve(root, entry.penpotEvidence[0].geometryPath!),
        'geometry',
      );
      await writeFile(resolve(root, entry.penpotEvidence[0].textPath!), 'text');
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).resolves.toBeUndefined();
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'missing or failed',
      );
      await writeFile(resolve(root, 'theme.zui'), 'changed');
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).rejects.toThrow('theme.zui');
      await writeFile(resolve(root, 'theme.zui'), 'theme');
      await writeFile(resolve(root, 'evidence/failed.png'), 'changed');
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).rejects.toThrow('failed.png');
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
});
