use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const ASSET_FOLDER_ROWS: &[&str] = &["WorkbenchAssetsForestRow", "WorkbenchAssetsMaterialRow"];
const ASSET_TABLE_ROWS: &[&str] = &[
    "WorkbenchAssetsTableRow01",
    "WorkbenchAssetsTableRow02",
    "WorkbenchAssetsTableRow03",
];
const ASSET_TABLE_ACTIONS: &[&str] = &[
    "workbench.module.assets.table_tree.select",
    "workbench.module.assets.table_material.select",
    "workbench.module.assets.table_texture.select",
];

static FOREST_ASSETS: [AssetProfile; 3] = [
    AssetProfile {
        name: "SM_Tree_Oak_01",
        row_text: "SM_Tree_Oak_01     Static Mesh     2.4 MB     Today",
        row_value: "asset_01",
        columns: ["SM_Tree_Oak_01", "Static Mesh", "2.4 MB", "Today"],
        asset_type: "Static Mesh",
        path: "/Game/Environment/Forest/SM_Tree_Oak_01",
        source: "glTF importer",
    },
    AssetProfile {
        name: "M_Bark_Master",
        row_text: "M_Bark_Master      Material        512 KB     10m ago",
        row_value: "asset_02",
        columns: ["M_Bark_Master", "Material", "512 KB", "10m ago"],
        asset_type: "Material",
        path: "/Game/Environment/Forest/M_Bark_Master",
        source: "Material Editor",
    },
    AssetProfile {
        name: "T_Leaf_N",
        row_text: "T_Leaf_N           Texture         1.2 MB     1h ago",
        row_value: "asset_03",
        columns: ["T_Leaf_N", "Texture", "1.2 MB", "1h ago"],
        asset_type: "Texture 2D",
        path: "/Game/Environment/Forest/T_Leaf_N",
        source: "Texture importer",
    },
];

static MATERIAL_ASSETS: [AssetProfile; 3] = [
    AssetProfile {
        name: "M_Rock_Cliff",
        row_text: "M_Rock_Cliff       Material Inst.  384 KB     Today",
        row_value: "asset_01",
        columns: ["M_Rock_Cliff", "Material Inst.", "384 KB", "Today"],
        asset_type: "Material Instance",
        path: "/Game/Materials/M_Rock_Cliff",
        source: "Material Editor",
    },
    AssetProfile {
        name: "M_Metal_Brushed",
        row_text: "M_Metal_Brushed    Material        620 KB     Today",
        row_value: "asset_02",
        columns: ["M_Metal_Brushed", "Material", "620 KB", "Today"],
        asset_type: "Material",
        path: "/Game/Materials/M_Metal_Brushed",
        source: "Material Editor",
    },
    AssetProfile {
        name: "T_Noise_Detail",
        row_text: "T_Noise_Detail     Texture         1.0 MB     2h ago",
        row_value: "asset_03",
        columns: ["T_Noise_Detail", "Texture", "1.0 MB", "2h ago"],
        asset_type: "Texture 2D",
        path: "/Game/Materials/T_Noise_Detail",
        source: "Texture importer",
    },
];

static ASSET_FOLDERS: [AssetFolderProfile; 2] = [
    AssetFolderProfile {
        action_id: "workbench.module.assets.forest_row.select",
        row_control_id: "WorkbenchAssetsForestRow",
        title: "Content/Environment/Forest",
        assets: &FOREST_ASSETS,
    },
    AssetFolderProfile {
        action_id: "workbench.module.assets.material_row.select",
        row_control_id: "WorkbenchAssetsMaterialRow",
        title: "Content/Materials",
        assets: &MATERIAL_ASSETS,
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_assets_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.project_asset_folder(&ASSET_FOLDERS[0])
    }

    pub(super) fn apply_assets_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.assets.import.invoke" {
            self.apply_asset_import_feedback()?;
            return Ok(true);
        }
        if let Some(folder) = ASSET_FOLDERS
            .iter()
            .find(|folder| folder.action_id == action_id)
        {
            self.project_asset_folder(folder)?;
            return Ok(true);
        }
        let Some(asset_index) = ASSET_TABLE_ACTIONS
            .iter()
            .position(|candidate| *candidate == action_id)
        else {
            return Ok(false);
        };
        let folder = self.selected_asset_folder();
        self.project_asset(folder.assets, asset_index)?;
        Ok(true)
    }

    fn selected_asset_folder(&self) -> &'static AssetFolderProfile {
        ASSET_FOLDERS
            .iter()
            .find(|folder| self.control_bool(folder.row_control_id, "selected"))
            .unwrap_or(&ASSET_FOLDERS[0])
    }

    fn project_asset_folder(
        &mut self,
        folder: &'static AssetFolderProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(ASSET_FOLDER_ROWS, folder.row_control_id)?;
        self.set_asset_string("WorkbenchAssetsCenterTitle", "text", folder.title)?;
        for (control_id, asset) in ASSET_TABLE_ROWS.iter().zip(folder.assets) {
            self.set_asset_string(control_id, "text", asset.row_text)?;
            self.set_asset_string(control_id, "value", asset.row_value)?;
            self.mutate_control_property(
                control_id,
                "options",
                UiValue::Array(
                    asset
                        .columns
                        .iter()
                        .map(|column| UiValue::String((*column).to_string()))
                        .collect(),
                ),
            )?;
        }
        self.project_asset(folder.assets, 0)
    }

    fn project_asset(
        &mut self,
        assets: &'static [AssetProfile; 3],
        index: usize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let profile = &assets[index];
        self.select_exclusive_selected(ASSET_TABLE_ROWS, ASSET_TABLE_ROWS[index])?;
        for (control_id, value) in [
            ("WorkbenchAssetsTypeField", profile.asset_type),
            ("WorkbenchAssetsPathField", profile.path),
            ("WorkbenchAssetsOwnerField", profile.source),
        ] {
            self.set_asset_string(control_id, "value", value)?;
        }
        self.set_asset_string(
            "WorkbenchAssetsOutputRow",
            "text",
            format!("Selected: {}   {}", profile.name, profile.asset_type),
        )
    }

    fn apply_asset_import_feedback(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let asset_type = self
            .control_string("WorkbenchAssetsTypeField", "value")
            .unwrap_or_default();
        let path = self
            .control_string("WorkbenchAssetsPathField", "value")
            .unwrap_or_default();
        let owner = self
            .control_string("WorkbenchAssetsOwnerField", "value")
            .unwrap_or_default();
        let asset_name = path
            .rsplit('/')
            .find(|segment| !segment.is_empty())
            .unwrap_or(path.as_str());
        self.set_asset_string("WorkbenchStatusReady", "text", "Asset import queued")?;
        self.set_asset_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_asset_string(
            "WorkbenchAssetsOutputRow",
            "text",
            format!("Import: {asset_name}   {asset_type} / {owner}"),
        )
    }

    fn set_asset_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct AssetFolderProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    title: &'static str,
    assets: &'static [AssetProfile; 3],
}

struct AssetProfile {
    name: &'static str,
    row_text: &'static str,
    row_value: &'static str,
    columns: [&'static str; 4],
    asset_type: &'static str,
    path: &'static str,
    source: &'static str,
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn folder_asset_and_import_actions_share_one_projection() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");

        assert!(bridge.control_bool("WorkbenchAssetsForestRow", "selected"));
        assert!(bridge.control_bool("WorkbenchAssetsTableRow01", "selected"));
        bridge
            .dispatch_control_state("WorkbenchAssetsMaterialRow", UiEventKind::Click)
            .expect("materials folder should dispatch")
            .expect("materials folder should bind");
        assert!(bridge.control_bool("WorkbenchAssetsMaterialRow", "selected"));
        assert!(bridge.control_bool("WorkbenchAssetsTableRow01", "selected"));
        assert_eq!(
            Some("Content/Materials".to_string()),
            bridge.control_string("WorkbenchAssetsCenterTitle", "text")
        );
        assert_eq!(
            Some("Material Instance".to_string()),
            bridge.control_string("WorkbenchAssetsTypeField", "value")
        );

        bridge
            .dispatch_control_state("WorkbenchAssetsTableRow02", UiEventKind::Click)
            .expect("second material should dispatch")
            .expect("second material should bind");
        assert_eq!(
            Some("/Game/Materials/M_Metal_Brushed".to_string()),
            bridge.control_string("WorkbenchAssetsPathField", "value")
        );
        for (control_id, value) in [
            ("WorkbenchAssetsTypeField", "Custom Material"),
            ("WorkbenchAssetsPathField", "/Game/Custom/M_Custom"),
            ("WorkbenchAssetsOwnerField", "Custom Importer"),
        ] {
            bridge
                .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
                .expect("asset metadata should edit");
        }
        bridge
            .dispatch_control_state("WorkbenchAssetsImportButton", UiEventKind::Click)
            .expect("asset import should dispatch")
            .expect("asset import should bind");
        assert_eq!(
            Some("Import: M_Custom   Custom Material / Custom Importer".to_string()),
            bridge.control_string("WorkbenchAssetsOutputRow", "text")
        );
    }
}
