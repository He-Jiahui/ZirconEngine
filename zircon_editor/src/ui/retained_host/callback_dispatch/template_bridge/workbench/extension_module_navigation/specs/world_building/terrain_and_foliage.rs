use super::{ActionControl, ExtensionNavigationSpec, action, spec};

const TERRAIN_EDITOR_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionTerrainEditorSculptTab",
    "WorkbenchExtensionTerrainEditorPaintTab",
    "WorkbenchExtensionTerrainEditorStreamingTab",
];
const TERRAIN_EDITOR_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.terrain_editor.sculpt_tab.select",
        "WorkbenchExtensionTerrainEditorSculptTab",
    ),
    action(
        "workbench.extension.terrain_editor.paint_tab.select",
        "WorkbenchExtensionTerrainEditorPaintTab",
    ),
    action(
        "workbench.extension.terrain_editor.streaming_tab.select",
        "WorkbenchExtensionTerrainEditorStreamingTab",
    ),
];
const TERRAIN_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionTerrainEditorLandscapeRow",
    "WorkbenchExtensionTerrainEditorHeightfieldRow",
    "WorkbenchExtensionTerrainEditorLayerRockRow",
    "WorkbenchExtensionTerrainEditorCellA1208Row",
    "WorkbenchExtensionTerrainEditorCellA1209Row",
    "WorkbenchExtensionTerrainEditorRockLayerTableRow",
    "WorkbenchExtensionTerrainEditorStreamingCellTableRow",
    "WorkbenchExtensionTerrainEditorOutputRow",
];
const TERRAIN_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.terrain_editor.landscape_row.select",
        "WorkbenchExtensionTerrainEditorLandscapeRow",
    ),
    action(
        "workbench.extension.terrain_editor.heightfield_row.select",
        "WorkbenchExtensionTerrainEditorHeightfieldRow",
    ),
    action(
        "workbench.extension.terrain_editor.layer_rock_row.select",
        "WorkbenchExtensionTerrainEditorLayerRockRow",
    ),
    action(
        "workbench.extension.terrain_editor.cell_a_1208_table_row.select",
        "WorkbenchExtensionTerrainEditorCellA1208Row",
    ),
    action(
        "workbench.extension.terrain_editor.cell_a_1209_table_row.select",
        "WorkbenchExtensionTerrainEditorCellA1209Row",
    ),
    action(
        "workbench.extension.terrain_editor.rock_layer_table_row.select",
        "WorkbenchExtensionTerrainEditorRockLayerTableRow",
    ),
    action(
        "workbench.extension.terrain_editor.streaming_cell_table_row.select",
        "WorkbenchExtensionTerrainEditorStreamingCellTableRow",
    ),
    action(
        "workbench.extension.terrain_editor.output.select",
        "WorkbenchExtensionTerrainEditorOutputRow",
    ),
];
const TERRAIN_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsTerrainEditorButton",
    "WorkbenchExtensionTerrainEditorPreviewButton",
    "WorkbenchExtensionTerrainEditorBuildButton",
];
const TERRAIN_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.terrain_editor.open",
        "WorkbenchAssetsTerrainEditorButton",
    ),
    action(
        "workbench.extension.terrain_editor.preview.invoke",
        "WorkbenchExtensionTerrainEditorPreviewButton",
    ),
    action(
        "workbench.extension.terrain_editor.build.invoke",
        "WorkbenchExtensionTerrainEditorBuildButton",
    ),
];
const TERRAIN_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.terrain_editor.brush.edit",
    "workbench.extension.terrain_editor.brush.commit",
    "workbench.extension.terrain_editor.radius.edit",
    "workbench.extension.terrain_editor.radius.commit",
    "workbench.extension.terrain_editor.strength.edit",
    "workbench.extension.terrain_editor.strength.commit",
];

pub(in super::super) const TERRAIN_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.terrain_editor.open",
    "WorkbenchExtensionTerrainEditorWorkspace",
    TERRAIN_EDITOR_TAB_CONTROLS,
    TERRAIN_EDITOR_TAB_ACTIONS,
    TERRAIN_EDITOR_ROW_CONTROLS,
    TERRAIN_EDITOR_ROW_ACTIONS,
    TERRAIN_EDITOR_COMMAND_CONTROLS,
    TERRAIN_EDITOR_COMMAND_ACTIONS,
    TERRAIN_EDITOR_FIELD_ACTIONS,
);

const FOLIAGE_EDITOR_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionFoliageEditorPaintTab",
    "WorkbenchExtensionFoliageEditorEraseTab",
    "WorkbenchExtensionFoliageEditorClustersTab",
];
const FOLIAGE_EDITOR_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.foliage_editor.paint_tab.select",
        "WorkbenchExtensionFoliageEditorPaintTab",
    ),
    action(
        "workbench.extension.foliage_editor.erase_tab.select",
        "WorkbenchExtensionFoliageEditorEraseTab",
    ),
    action(
        "workbench.extension.foliage_editor.clusters_tab.select",
        "WorkbenchExtensionFoliageEditorClustersTab",
    ),
];
const FOLIAGE_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionFoliageEditorForestRow",
    "WorkbenchExtensionFoliageEditorOakRow",
    "WorkbenchExtensionFoliageEditorBiomeRow",
    "WorkbenchExtensionFoliageEditorForestA12Row",
    "WorkbenchExtensionFoliageEditorForestA13Row",
    "WorkbenchExtensionFoliageEditorRiver02Row",
    "WorkbenchExtensionFoliageEditorCliff01Row",
    "WorkbenchExtensionFoliageEditorOutputRow",
];
const FOLIAGE_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.foliage_editor.forest_row.select",
        "WorkbenchExtensionFoliageEditorForestRow",
    ),
    action(
        "workbench.extension.foliage_editor.oak_row.select",
        "WorkbenchExtensionFoliageEditorOakRow",
    ),
    action(
        "workbench.extension.foliage_editor.biome_row.select",
        "WorkbenchExtensionFoliageEditorBiomeRow",
    ),
    action(
        "workbench.extension.foliage_editor.forest_a_12_table_row.select",
        "WorkbenchExtensionFoliageEditorForestA12Row",
    ),
    action(
        "workbench.extension.foliage_editor.forest_a_13_table_row.select",
        "WorkbenchExtensionFoliageEditorForestA13Row",
    ),
    action(
        "workbench.extension.foliage_editor.river_02_table_row.select",
        "WorkbenchExtensionFoliageEditorRiver02Row",
    ),
    action(
        "workbench.extension.foliage_editor.cliff_01_table_row.select",
        "WorkbenchExtensionFoliageEditorCliff01Row",
    ),
    action(
        "workbench.extension.foliage_editor.output.select",
        "WorkbenchExtensionFoliageEditorOutputRow",
    ),
];
const FOLIAGE_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsFoliageEditorButton",
    "WorkbenchExtensionFoliageEditorPreviewButton",
    "WorkbenchExtensionFoliageEditorBuildButton",
];
const FOLIAGE_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.foliage_editor.open",
        "WorkbenchAssetsFoliageEditorButton",
    ),
    action(
        "workbench.extension.foliage_editor.preview.invoke",
        "WorkbenchExtensionFoliageEditorPreviewButton",
    ),
    action(
        "workbench.extension.foliage_editor.build.invoke",
        "WorkbenchExtensionFoliageEditorBuildButton",
    ),
];
const FOLIAGE_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.foliage_editor.type.edit",
    "workbench.extension.foliage_editor.type.commit",
    "workbench.extension.foliage_editor.density.edit",
    "workbench.extension.foliage_editor.density.commit",
    "workbench.extension.foliage_editor.radius.edit",
    "workbench.extension.foliage_editor.radius.commit",
];

pub(in super::super) const FOLIAGE_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.foliage_editor.open",
    "WorkbenchExtensionFoliageEditorWorkspace",
    FOLIAGE_EDITOR_TAB_CONTROLS,
    FOLIAGE_EDITOR_TAB_ACTIONS,
    FOLIAGE_EDITOR_ROW_CONTROLS,
    FOLIAGE_EDITOR_ROW_ACTIONS,
    FOLIAGE_EDITOR_COMMAND_CONTROLS,
    FOLIAGE_EDITOR_COMMAND_ACTIONS,
    FOLIAGE_EDITOR_FIELD_ACTIONS,
);
