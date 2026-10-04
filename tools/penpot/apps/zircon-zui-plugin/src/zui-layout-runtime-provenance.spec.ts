import { describe, expect, it } from 'vitest';
import { canonicalSha256 } from '../tools/zui-layout-review-contract';
import {
  buildBrowserRuntimeLoadedSourcesAudit,
  capturePreparedWorkbenchLocaleAudit,
  capturePreparedEditorTokenReceipt,
  deriveExpectedRuntimeSourceFiles,
  expectedWorkbenchRuntimeDocumentPaths,
  runtimeLoadedSourceErrors,
  runtimeTokenEvidenceErrors,
  workbenchLocaleEvidenceErrors,
} from '../tools/zui-layout-runtime-provenance';

const paletteFields = [
  'surface_recessed',
  'surface_hover',
  'surface_selected',
  'surface_disabled',
  'accent',
  'accent_soft',
  'border',
  'border_disabled',
  'separator_strong',
  'separator_soft',
  'text_primary',
  'text_secondary',
  'text_disabled',
  'success',
  'success_container',
  'info',
  'info_container',
  'warning',
  'warning_container',
  'error',
  'error_container',
  'popup',
  'track',
  'focus_ring',
  'shadow',
];
const densityFields = [
  'gap_xsmall',
  'gap_tight',
  'gap_small',
  'gap_regular',
  'gap_medium',
  'gap_large',
  'gap_group',
  'drawer_padding',
  'panel_padding',
  'toolbar_action_width',
  'toolbar_wide_action_width',
  'ui_asset_action_min_width',
  'ui_asset_action_preferred_width',
  'ui_asset_action_max_width',
  'ui_asset_side_min_width',
  'ui_asset_side_preferred_width',
  'ui_asset_center_min_width',
  'ui_asset_center_preferred_width',
  'ui_asset_header_kind_min_width',
  'ui_asset_header_kind_preferred_width',
  'ui_asset_header_kind_max_width',
  'ui_asset_tool_min_width',
  'ui_asset_tool_preferred_width',
  'ui_asset_tool_max_width',
  'command_palette_min_width',
  'command_palette_preferred_width',
  'command_palette_max_width',
  'command_palette_min_height',
  'command_palette_preferred_height',
  'command_palette_max_height',
  'dialog_min_width',
  'dialog_preferred_width',
  'dialog_max_width',
  'dialog_min_height',
  'dialog_preferred_height',
  'dialog_max_height',
  'confirm_dialog_preferred_width',
  'confirm_dialog_max_width',
  'confirm_dialog_min_height',
  'confirm_dialog_preferred_height',
  'confirm_dialog_max_height',
  'notification_panel_min_width',
  'notification_panel_preferred_width',
  'notification_panel_max_width',
  'notification_panel_min_height',
  'notification_panel_preferred_height',
  'notification_panel_max_height',
  'caption_min_height',
  'caption_preferred_height',
  'caption_max_height',
  'label_min_height',
  'label_preferred_height',
  'label_max_height',
  'chip_min_width',
  'chip_preferred_width',
  'chip_max_width',
  'axis_value_field_min_width',
  'axis_value_field_preferred_width',
  'axis_value_field_max_width',
  'row_height',
  'left_drawer_width',
  'right_drawer_width',
  'bottom_output_height',
  'breakpoint_ultra_width',
  'breakpoint_narrow_width',
  'breakpoint_wide_width',
  'compact_side_width',
  'ultra_compact_side_width',
  'compact_left_drawer_max_width',
  'compact_right_drawer_max_width',
  'compact_side_min_width',
  'minimum_document_width_fraction',
  'ultra_compact_left_drawer_max_width',
  'ultra_compact_right_drawer_max_width',
  'compact_bottom_available_height',
  'compact_bottom_max_height',
  'compact_bottom_max_available_fraction',
  'compact_bottom_min_height',
  'ultra_compact_bottom_available_height',
  'ultra_compact_bottom_max_height',
  'ultra_compact_bottom_max_available_fraction',
  'ultra_compact_bottom_min_height',
  'minimum_window_width',
  'minimum_window_height',
  'ultra_minimum_window_width',
  'ultra_minimum_window_height',
];
const chromeFields = [
  'top_bar_height',
  'host_bar_height',
  'workbench_toolbar_height',
  'workbench_toolbar_command_row_height',
  'workbench_toolbar_popup_command_offset_y',
  'workbench_toolbar_popup_module_offset_y',
  'status_bar_height',
  'panel_header_height',
  'document_header_height',
  'viewport_toolbar_height',
  'activity_rail_width',
  'separator_thickness',
  'splitter_hit_size',
];

const numericRecord = (keys: string[]) =>
  Object.fromEntries(keys.map((key) => [key, 1]));
const color = () => ({ red: 0.2, green: 0.3, blue: 0.4, alpha: 1 });
const activeDesignTokens = () => {
  const tokens = {
    id: 'zircon.editor.workbench',
    palette: {
      surface: [color(), color(), color(), color()],
      ...Object.fromEntries(paletteFields.map((key) => [key, color()])),
    },
    typography: {
      ui_family: 'Fira Sans',
      ui_strong_family: 'Fira Sans',
      code_family: 'Fira Mono',
      utility_tab_text_role: 'ui',
      font_smoothing: 'grayscale',
      body_size: 14,
      caption_size: 12,
      overlay_size: 12,
      heading_size: 16,
      title_size: 20,
      body_weight: 400,
      medium_weight: 500,
      strong_weight: 600,
      emphasis_weight: 700,
      code_weight: 400,
      line_height: 1.4,
    },
    controls: {
      large_height: 48,
      default_height: 32,
      compact_height: 32,
      dense_height: 28,
      small_radius: 4,
      control_radius: 4,
      large_radius: 6,
      panel_radius: 8,
      pill_radius: 999,
      border_width: 1,
    },
    density: numericRecord(densityFields),
    chrome: numericRecord(chromeFields),
    state_roles: {
      default: 'surface1',
      hovered: 'surface2',
      pressed: 'surface3',
      selected: 'surface_selected',
      focused: 'surface1',
      disabled: 'text_disabled',
      loading: 'accent',
    },
  };
  return {
    complete: true,
    sha256: canonicalSha256(tokens),
    tokens,
  };
};

const tokenReceipt = (tokens: Record<string, string | number>) => ({
  complete: true,
  sha256: canonicalSha256(tokens),
  tokens,
});

const sourceFile = {
  sourcePath: 'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
  sha256: 'a'.repeat(64),
};
const loadedSource = (overrides: Record<string, unknown> = {}) => ({
  assetId: 'workbench-window',
  sourcePath: sourceFile.sourcePath,
  resourceUri: 'res://ui/editor/windows/workbench_window.zui',
  physicalPath: 'E:\\Git\\ZirconEngine\\zircon_editor\\assets\\ui\\editor\\windows\\workbench_window.zui',
  sha256: sourceFile.sha256,
  catalogMatches: true,
  currentFileMatches: true,
  ...overrides,
});
const geometryWithLoadedSource = (overrides: Record<string, unknown> = {}) => ({
  sourceIdentityProvenance: {
    runtimeLoadedSources: {
      complete: true,
      documentIds: ['res://ui/editor/windows/workbench_window.zui'],
      files: [loadedSource()],
      unresolvedImports: [],
      ...overrides,
    },
  },
});

describe('dual renderer runtime token and source provenance', () => {
  it('records actual prepared locale and requires native i18n locale agreement', () => {
    const browserLocale = capturePreparedWorkbenchLocaleAudit(
      { activeLocale: 'en' },
      'en-US',
    );
    const penpot = { localeAudit: browserLocale };
    const engine = {
      localeAudit: {
        complete: true,
        actualLocale: 'en',
        expectedLocale: 'en',
        source: 'editor-i18n-service',
      },
    };
    expect(browserLocale).toEqual({
      complete: true,
      actualLocale: 'en',
      expectedLocale: 'en',
      source: 'penpot-prepared-workbench-presentation',
    });
    expect(workbenchLocaleEvidenceErrors(penpot, engine, 'en-US')).toEqual([]);
    expect(
      workbenchLocaleEvidenceErrors(
        penpot,
        { localeAudit: { ...engine.localeAudit, actualLocale: 'zh-CN' } },
        'en-US',
      ),
    ).toContain('engine: active locale differs from the case expectation');
    expect(
      capturePreparedWorkbenchLocaleAudit({ activeLocale: 'zh-CN' }, 'en-US')
        .complete,
    ).toBe(false);
  });

  it('derives active source document IDs from the factual product snapshot', () => {
    const snapshot = {
      documents: {
        activeId: 'scene',
        items: [
          { id: 'scene', sourcePath: null },
          {
            id: 'hierarchy',
            sourcePath: 'zircon_editor/assets/ui/editor/panes/hierarchy.zui',
          },
        ],
      },
      drawers: [
        {
          visible: false,
          activeTabId: 'hierarchy',
          tabs: [
            {
              id: 'hierarchy',
              sourcePath:
                'zircon_editor/assets/ui/editor/panes/hierarchy.zui',
            },
          ],
        },
      ],
    };
    expect(expectedWorkbenchRuntimeDocumentPaths(snapshot)).toEqual({
      sourcePaths: [
        'zircon_editor/assets/ui/editor/host/workbench_shell.zui',
        'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
      ],
      errors: [],
    });

    const paneSnapshot = structuredClone(snapshot) as typeof snapshot;
    paneSnapshot.documents.activeId = 'hierarchy';
    const panePaths = expectedWorkbenchRuntimeDocumentPaths(paneSnapshot);
    expect(panePaths.errors).toEqual([]);
    expect(panePaths.sourcePaths).toContain(
      'zircon_editor/assets/ui/editor/panes/hierarchy.zui',
    );
    const drawerSnapshot = structuredClone(snapshot) as typeof snapshot;
    drawerSnapshot.drawers[0]!.visible = true;
    const drawerPaths = expectedWorkbenchRuntimeDocumentPaths(drawerSnapshot);
    expect(drawerPaths.errors).toEqual([]);
    expect(drawerPaths.sourcePaths).toContain(
      'zircon_editor/assets/ui/editor/panes/hierarchy.zui',
    );
  });

  it('seals browser-loaded files to current catalog fingerprints and active roots', () => {
    const expected = [
      {
        sourcePath: 'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
        sha256: 'a'.repeat(64),
      },
    ];
    const loaded = [
      {
        assetId: 'zircon.editor.workbench.window',
        sourcePath: expected[0]!.sourcePath,
        resourceUri: 'res://ui/editor/windows/workbench_window.zui',
        physicalPath:
          'E:\\Git\\ZirconEngine\\zircon_editor\\assets\\ui\\editor\\windows\\workbench_window.zui',
        sha256: expected[0]!.sha256,
        catalogMatches: true,
        currentFileMatches: true,
      },
    ];
    const captured = buildBrowserRuntimeLoadedSourcesAudit(
      expected,
      loaded,
      [expected[0]!.sourcePath],
    );
    expect(captured.audit.complete).toBe(true);
    expect(captured.audit.files[0]?.assetId).toBe('zircon.editor.workbench.window');
    expect(captured.audit.documentIds).toEqual([
      'res://ui/editor/windows/workbench_window.zui',
    ]);
    expect(captured.errors).toEqual([]);
    expect(
      buildBrowserRuntimeLoadedSourcesAudit(
        expected,
        [{ ...loaded[0]!, currentFileMatches: false }],
        [expected[0]!.sourcePath],
      ).audit.complete,
    ).toBe(false);
  });

  it('uses only actual prepared editor tokens and resolves state role aliases against that map', () => {
    const captured = capturePreparedEditorTokenReceipt({
      'editor.surface.1': '#242424',
      'editor.state.default': 'surface_1',
      '--editor-surface-1': '#242424',
      'workbench.viewport.background': '#151515',
    });
    expect(captured.errors).toEqual([]);
    expect(captured.receipt.tokens).toEqual({
      'editor.surface.1': '#242424',
      'editor.state.default': '$editor.surface.1',
    });
    expect(captured.receipt.sha256).toBe(
      canonicalSha256(captured.receipt.tokens),
    );

    const unresolved = capturePreparedEditorTokenReceipt({
      'editor.state.default': 'missing_role',
    });
    expect(unresolved.receipt.complete).toBe(false);
    expect(unresolved.errors.join(' ')).toContain(
      'state-role reference cannot be resolved',
    );
  });

  it('compares the complete canonical editor token inventory with bounded numeric conversion', () => {
    const browserTokens = {
      'editor.surface.0': '#151515',
      'editor.density.gap_regular': 14.4,
      'editor.typography.ui_family': 'Fira Sans',
    };
    const nativeTokens = {
      ...browserTokens,
      'editor.density.gap_regular': 14.399999618530273,
    };
    const browser = { consumedTokens: tokenReceipt(browserTokens) };
    const native = {
      consumedTokens: tokenReceipt(nativeTokens),
      activeDesignTokens: activeDesignTokens(),
    };
    expect(runtimeTokenEvidenceErrors(browser, native)).toEqual([]);

    const missingToken = {
      consumedTokens: tokenReceipt({
        'editor.surface.0': '#151515',
        'editor.density.gap_regular': 14.4,
      }),
    };
    expect(runtimeTokenEvidenceErrors(missingToken, native).join(' ')).toContain(
      'canonical editor token inventory differs',
    );

    const differentValue = {
      consumedTokens: tokenReceipt({
        ...browserTokens,
        'editor.density.gap_regular': 14.4002,
      }),
    };
    expect(
      runtimeTokenEvidenceErrors(differentValue, native).join(' '),
    ).toContain('consumed editor token differs');

    const staleHash = {
      consumedTokens: {
        ...tokenReceipt(browserTokens),
        sha256: 'b'.repeat(64),
      },
    };
    expect(runtimeTokenEvidenceErrors(staleHash, native).join(' ')).toContain(
      'missing, incomplete, or stale consumed token receipt',
    );

    const incompleteSnapshot = activeDesignTokens();
    delete (incompleteSnapshot.tokens.controls as Record<string, unknown>)[
      'border_width'
    ];
    incompleteSnapshot.sha256 = canonicalSha256(incompleteSnapshot.tokens);
    expect(
      runtimeTokenEvidenceErrors(browser, {
        ...native,
        activeDesignTokens: incompleteSnapshot,
      }).join(' '),
    ).toContain('active design-token controls are incomplete or invalid');
  });

  it('requires identical loaded source closures and current catalog-backed files on both renderers', () => {
    const browser = geometryWithLoadedSource();
    const native = geometryWithLoadedSource();
    expect(
      runtimeLoadedSourceErrors(browser, native, [sourceFile]),
    ).toEqual([]);

    const differentAssetIdentity = geometryWithLoadedSource({
      files: [loadedSource({ assetId: 'different-workbench-window' })],
    });
    expect(
      runtimeLoadedSourceErrors(browser, differentAssetIdentity, [sourceFile]).join(' '),
    ).toContain('loaded source file identities differ between renderers');

    const pending = geometryWithLoadedSource({ complete: false });
    expect(
      runtimeLoadedSourceErrors(pending, native, [sourceFile]).join(' '),
    ).toContain('runtime source closure is incomplete');

    const missing = geometryWithLoadedSource({ files: [] });
    expect(
      runtimeLoadedSourceErrors(missing, native, [sourceFile]).join(' '),
    ).toContain('loaded source closure differs');

    const stale = geometryWithLoadedSource({
      files: [loadedSource({ currentFileMatches: false })],
    });
    expect(
      runtimeLoadedSourceErrors(stale, native, [sourceFile]).join(' '),
    ).toContain('loaded source is not current');

    const unresolved = geometryWithLoadedSource({
      unresolvedImports: [{ sourcePath: sourceFile.sourcePath, reference: 'missing' }],
    });
    expect(
      runtimeLoadedSourceErrors(unresolved, native, [sourceFile]).join(' '),
    ).toContain('unresolved runtime source imports');

    const extra = geometryWithLoadedSource({
      files: [
        loadedSource(),
        loadedSource({
          assetId: 'unused',
          sourcePath: 'zircon_editor/assets/ui/editor/unused.zui',
          resourceUri: 'res://ui/editor/unused.zui',
        }),
      ],
    });
    expect(
      runtimeLoadedSourceErrors(extra, native, [sourceFile]).join(' '),
    ).toContain('loaded source closure differs');
  });

  it('derives required source files from prepared imports, authored owners, and the actual theme root', () => {
    const imported = 'zircon_editor/assets/ui/editor/panes/hierarchy.zui';
    const theme = 'zircon_editor/assets/ui/editor/themes/workbench.zui';
    const presentationRoot = sourceFile.sourcePath;
    const projection = {
      penpot_dependency_sources: [imported, theme],
      penpot_review_workbench_presentation: {
        sourceFingerprint: { sourcePath: presentationRoot },
      },
      nodes: {
        authored: { penpot_review_source_path: imported },
      },
    };
    const catalogSources = [
      sourceFile,
      { sourcePath: imported, sha256: 'b'.repeat(64) },
      { sourcePath: theme, sha256: 'c'.repeat(64) },
      {
        sourcePath: 'zircon_editor/assets/ui/editor/modules/workbench_window.zui',
        sha256: 'd'.repeat(64),
      },
    ];
    const derived = deriveExpectedRuntimeSourceFiles(
      projection,
      ['zircon_editor/assets/ui/editor/modules/workbench_window.zui'],
      catalogSources,
    );
    expect(derived.errors).toEqual([]);
    expect(derived.sources.map(({ sourcePath }) => sourcePath)).toEqual([
      presentationRoot,
      theme,
      imported,
      'zircon_editor/assets/ui/editor/modules/workbench_window.zui',
    ].sort());

    const missingCatalogSource = deriveExpectedRuntimeSourceFiles(
      projection,
      [],
      [sourceFile],
    );
    expect(missingCatalogSource.errors.join(' ')).toContain(
      'prepared source is absent from catalog fingerprints',
    );
  });
});
