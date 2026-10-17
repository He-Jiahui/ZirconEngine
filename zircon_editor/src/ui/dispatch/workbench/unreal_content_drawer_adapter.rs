//! Live asset-browser body data for the Unreal content drawer.
//!
//! The drawer owns the mount point (`ContentDrawerAssetBrowser`), while the
//! asset workspace remains the source of truth for rows, selection, search,
//! view mode, and the published surface frame.  Keep this adapter at that
//! boundary so the Unreal component does not grow a second asset model or a
//! static list of controls.

use std::sync::Arc;

use zircon_runtime_interface::ui::{layout::UiSize, surface::UiSurfaceFrame};

use crate::ui::layouts::{
    views::{asset_browser_pane_data, ViewTemplateNodeData},
    windows::workbench_host_window::AssetBrowserPaneViewData,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

/// The already-projected asset-browser body that the Unreal drawer mounts
/// below `ContentDrawerAssetBrowser`.
///
/// `nodes` and `render_source_frame` are intentionally the same pair emitted
/// for a normal AssetBrowser pane.  This preserves the existing node IDs,
/// route bindings, pointer hit frames, and runtime paint-frame ownership.
#[derive(Clone, Default)]
pub(crate) struct UnrealContentDrawerAssetBrowserData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
    pub render_source_frame: Option<Arc<UiSurfaceFrame>>,
}

impl From<AssetBrowserPaneViewData> for UnrealContentDrawerAssetBrowserData {
    fn from(data: AssetBrowserPaneViewData) -> Self {
        Self {
            nodes: data.nodes,
            render_source_frame: data.render_source_frame,
        }
    }
}

/// Project the live asset workspace into the drawer body.
///
/// Callers should pass the drawer's content size and mount the returned pair
/// as one publication.  The projection cache in `asset_browser_pane_data`
/// keeps this cheap when the snapshot and size are unchanged.
pub(crate) fn unreal_content_drawer_asset_browser_data(
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) -> UnrealContentDrawerAssetBrowserData {
    asset_browser_pane_data(snapshot, size).into()
}

use super::super::virtual_rows::{
    TemplateBridgeVirtualRowBinding, TemplateBridgeVirtualRowSequence,
};
use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};
use crate::ui::binding::{
    AssetCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use zircon_runtime::ui::surface::UiVirtualListItemKey;
use zircon_runtime::ui::tree::UiRuntimeTreeScrollExt;
use zircon_runtime_interface::ui::{
    component::UiValue,
    layout::UiContainerKind,
    surface::{UiPointerEventKind, UiPointerRoute},
};

const BODY: &str = "ContentDrawerAssetBrowser";
const SELECT: &str = "workbench.content_drawer.select_asset";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn sync_content_browser_find_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for node in self.control_node_ids_with_descendants("WorkbenchToolbarAssets") {
            self.mutate_node_bool(node, "enabled", enabled)?;
        }
        Ok(())
    }

    pub(crate) fn prepare_content_drawer_assets(
        &mut self,
        snapshot: &AssetWorkspaceSnapshot,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.content_drawer_assets = snapshot.clone();
        self.mutate_control_property(
            "SearchEdited",
            "value_text",
            UiValue::String(snapshot.search_query.clone()),
        )?;
        self.mutate_control_property(
            "SetViewMode",
            "value",
            UiValue::String(
                match snapshot.view_mode {
                    crate::ui::workbench::snapshot::AssetViewMode::List => "list",
                    crate::ui::workbench::snapshot::AssetViewMode::Thumbnail => "thumbnail",
                }
                .into(),
            ),
        )?;
        self.mutate_control_property(
            "SetViewMode",
            "value_text",
            UiValue::String(
                match snapshot.view_mode {
                    crate::ui::workbench::snapshot::AssetViewMode::List => "List",
                    crate::ui::workbench::snapshot::AssetViewMode::Thumbnail => "Thumbnails",
                }
                .into(),
            ),
        )?;
        if !self.content_drawer_is_open() {
            return Ok(());
        }
        self.reconcile_content_drawer_assets()
    }

    fn reconcile_content_drawer_assets(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let rows = TemplateBridgeVirtualRowSequence::from_surface_repeat(
            &self.template_surface.surface,
            BODY,
        )?;
        let assets = self.content_drawer_assets.visible_assets.clone();
        let reconcile = rows.reconcile_with_keys(
            &mut self.template_surface.surface,
            assets.len(),
            |index| asset_key(&assets[index].uuid),
            |metadata, _| metadata,
        )?;
        if reconcile.topology_changed {
            self.template_surface.refresh_control_node_index()?;
        }
        for binding in rows.bindings(&self.template_surface.surface)? {
            self.sync_content_drawer_asset_row(&binding)?;
        }
        Ok(())
    }

    fn sync_content_drawer_asset_row(
        &mut self,
        binding: &TemplateBridgeVirtualRowBinding,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let asset = self
            .content_drawer_assets
            .visible_assets
            .get(binding.logical_index)
            .cloned();
        let Some(asset) = asset else {
            self.set_visible(&binding.control_id, false)?;
            return Ok(());
        };
        self.mutate_control_property(
            &binding.control_id,
            "text",
            UiValue::String(asset.display_name.clone()),
        )?;
        self.mutate_control_property(
            &binding.control_id,
            "tooltip",
            UiValue::String(asset.locator.clone()),
        )?;
        self.mutate_control_property(
            &binding.control_id,
            "content_browser_asset_uuid",
            UiValue::String(asset.uuid.clone()),
        )?;
        self.mutate_control_property(
            &binding.control_id,
            "content_browser_asset_locator",
            UiValue::String(asset.locator.clone()),
        )?;
        self.mutate_control_property(
            &binding.control_id,
            "icon",
            UiValue::String(asset.asset_type.icon_name.clone()),
        )?;
        self.set_selected(
            &binding.control_id,
            asset.selected
                || self.content_drawer_assets.selected_asset_uuid.as_deref()
                    == Some(asset.uuid.as_str()),
        )?;
        self.set_visible(&binding.control_id, true)?;
        Ok(())
    }

    pub(crate) fn refresh_content_drawer_assets_after_scroll(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.content_drawer_is_open()
            || route.kind != UiPointerEventKind::Scroll
            || route.scroll_delta == 0.0
        {
            return Ok(false);
        }
        let Some(body) = self.control_node_id(BODY) else {
            return Ok(false);
        };
        if !route.routing_root_to_leaf().contains(&body)
            && !route.hit_path.root_to_leaf.contains(&body)
        {
            return Ok(false);
        }
        // AssetView owns focus while its physical virtual rows are recycled.
        if self
            .template_surface
            .surface
            .focus
            .focused
            .is_some_and(|focus| {
                focus != body
                    && self
                        .control_node_ids_with_descendants(BODY)
                        .contains(&focus)
            })
        {
            self.template_surface.surface.focus_node(body)?;
        }
        self.reconcile_content_drawer_assets()?;
        Ok(self
            .template_surface
            .surface
            .pending_invalidation_changed_node_count()
            > 0)
    }

    pub(crate) fn owns_content_drawer_control(&self, control: &str) -> bool {
        self.control_node_id(control)
            .is_some_and(|id| self.content_drawer_contains(id))
    }

    pub(crate) fn content_drawer_asset_selection_binding(
        &self,
        control: &str,
    ) -> Option<EditorUiBinding> {
        if !self.content_drawer_is_open() || !self.owns_content_drawer_control(control) {
            return None;
        }
        let uuid = self.control_string(control, "content_browser_asset_uuid")?;
        let index = self
            .content_drawer_assets
            .visible_assets
            .selected_index(&uuid)?;
        let asset = self.content_drawer_assets.visible_assets.get(index)?;
        if self
            .control_string(control, "content_browser_asset_locator")?
            .as_str()
            != asset.locator
        {
            return None;
        }
        Some(EditorUiBinding::new(
            "Unreal",
            "SelectContentAsset",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::asset_command(AssetCommand::SelectItem { asset_uuid: uuid }),
        ))
    }

    pub(crate) fn is_content_drawer_asset_select_action(action: &str) -> bool {
        action == SELECT
    }

    pub(crate) fn reveal_content_drawer_selection(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if !self.content_drawer_is_open() {
            self.toggle_content_drawer()?;
        }
        let Some(body) = self.control_node_id(BODY) else {
            return Ok(());
        };
        let Some(index) = self
            .content_drawer_assets
            .selected_asset_uuid
            .as_deref()
            .and_then(|uuid| {
                self.content_drawer_assets
                    .visible_assets
                    .selected_index(uuid)
            })
        else {
            return Ok(());
        };
        // Consume the resolved authored extent and gap in logical units. The
        // Runtime owns scroll clamping, virtual slots, geometry and hit frames.
        let step = self
            .template_surface
            .surface
            .tree
            .node(body)
            .and_then(|node| match node.container {
                UiContainerKind::ScrollableBox(config) => {
                    config.virtualization.and_then(|virtualization| {
                        authored_asset_row_step(virtualization.item_extent, config.gap)
                    })
                }
                _ => None,
            });
        if let Some(step) = step {
            self.template_surface.surface.focus_node(body)?;
            self.template_surface
                .surface
                .tree
                .set_scroll_offset(body, index as f32 * step)?;
            self.reconcile_content_drawer_assets()?;
        }
        self.refresh_prepared_state_change()?;
        Ok(())
    }

    pub(crate) fn content_drawer_search_binding(
        &mut self,
        control: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if control != "SearchEdited"
            || binding_id != "AssetSurface/SearchEdited"
            || !self.content_drawer_is_open()
            || !self.owns_content_drawer_control(control)
        {
            return Ok(None);
        }
        self.mutate_control_property(control, "value_text", UiValue::String(value.into()))?;
        self.refresh_prepared_state_change()?;
        Ok(Some(EditorUiBinding::new(
            "AssetSurface",
            "SearchEdited",
            EditorUiEventKind::Change,
            EditorUiBindingPayload::asset_command(AssetCommand::SetSearchQuery {
                query: value.into(),
            }),
        )))
    }
}

// This consumes the already resolved ScrollableBox configuration in logical
// units. Runtime's internal step helper has the same finite/clamping rules;
// its visibility remains internal to Runtime.
fn authored_asset_row_step(item_extent: f32, gap: f32) -> Option<f32> {
    if !item_extent.is_finite() || item_extent <= 0.0 {
        return None;
    }
    let gap = if gap.is_finite() { gap.max(0.0) } else { 0.0 };
    Some(((item_extent as f64 + gap as f64).min(f32::MAX as f64)) as f32)
}

fn asset_key(uuid: &str) -> UiVirtualListItemKey {
    // UUID is the authoritative row key. Index is never used across generations.
    let key = uuid
        .parse::<uuid::Uuid>()
        .map(|id| id.as_u128())
        .unwrap_or_else(|_| {
            uuid.bytes().fold(
                0x6c62_272e_07bb_0142_62b8_2175_6295_c58d_u128,
                |hash, byte| {
                    (hash ^ u128::from(byte))
                        .wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013b)
                },
            )
        });
    UiVirtualListItemKey::new(key)
}

#[cfg(test)]
mod content_drawer_tests {
    use super::*;
    use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetTypeProjectionSnapshot};
    use zircon_runtime_interface::resource::ResourceKind;
    #[test]
    fn drawer_reveal_uses_logical_authored_extent_gap_and_finite_scroll_steps() {
        assert_eq!(authored_asset_row_step(28.0, 2.0), Some(30.0));
        assert_eq!(authored_asset_row_step(28.0, -2.0), Some(28.0));
        assert_eq!(authored_asset_row_step(28.0, f32::INFINITY), Some(28.0));
        assert_eq!(authored_asset_row_step(f32::MAX, f32::MAX), Some(f32::MAX));
        assert_eq!(authored_asset_row_step(0.0, 0.0), None);
        assert_eq!(authored_asset_row_step(f32::NAN, 0.0), None);
    }

    fn snapshot(count: usize) -> AssetWorkspaceSnapshot {
        AssetWorkspaceSnapshot {
            visible_assets: (0..count)
                .map(|index| AssetItemSnapshot {
                    uuid: format!("00000000-0000-0000-0000-{index:012}"),
                    locator: format!("res://models/{index}.model"),
                    display_name: format!("Catalog asset {index}"),
                    file_name: format!("{index}.model"),
                    extension: "model".into(),
                    kind: ResourceKind::Model,
                    asset_type: AssetTypeProjectionSnapshot::from_resource_kind(
                        ResourceKind::Model,
                    ),
                    preview_artifact_path: String::new(),
                    dirty: false,
                    diagnostics: vec![],
                    selected: index == 0,
                    resource_state: None,
                    resource_revision: None,
                })
                .collect(),
            ..Default::default()
        }
    }
    #[test]
    fn actual_unreal_drawer_rows_are_source_owned_virtual_catalog_rows_with_current_identity() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .prepare_content_drawer_assets(&snapshot(500))
            .unwrap();
        assert!(!bridge.content_drawer_is_open());
        bridge.toggle_content_drawer().unwrap();
        let rows =
            TemplateBridgeVirtualRowSequence::from_surface_repeat(bridge.surface(), BODY).unwrap();
        let bindings = rows.bindings(bridge.surface()).unwrap();
        assert!(!bindings.is_empty());
        assert!(
            bindings.len() < 100,
            "the virtual drawer must not materialize 500 rows"
        );
        let row = &bindings[0];
        let binding = bridge
            .content_drawer_asset_selection_binding(&row.control_id)
            .unwrap();
        assert!(
            matches!(binding.payload(), EditorUiBindingPayload::AssetCommand(AssetCommand::SelectItem { asset_uuid }) if asset_uuid == "00000000-0000-0000-0000-000000000000")
        );
        assert!(bridge
            .host_projection()
            .node_by_control_id(&row.control_id)
            .is_some());
        assert!(bridge.control_frame(&row.control_id).unwrap().height > 0.0);
        assert!(bridge
            .content_drawer_asset_selection_binding("WorkbenchToolbarAssets")
            .is_none());
        bridge.prepare_content_drawer_assets(&snapshot(0)).unwrap();
        assert!(bridge
            .content_drawer_asset_selection_binding(&row.control_id)
            .is_none());
    }
    #[test]
    fn drawer_search_uses_the_actual_edit_and_excludes_foreign_controls() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge.toggle_content_drawer().unwrap();
        let binding = bridge
            .content_drawer_search_binding("SearchEdited", "AssetSurface/SearchEdited", "ship")
            .unwrap()
            .unwrap();
        assert!(
            matches!(binding.payload(), EditorUiBindingPayload::AssetCommand(AssetCommand::SetSearchQuery { query }) if query == "ship")
        );
        assert_eq!(
            bridge
                .host_projection()
                .node_by_control_id("SearchEdited")
                .unwrap()
                .value_text
                .as_deref(),
            Some("ship")
        );
        assert!(bridge
            .content_drawer_search_binding(
                "ContentDrawerClose",
                "AssetSurface/SearchEdited",
                "ship"
            )
            .unwrap()
            .is_none());
        bridge.dismiss_content_drawer().unwrap();
        assert!(bridge
            .content_drawer_search_binding("SearchEdited", "AssetSurface/SearchEdited", "ship")
            .unwrap()
            .is_none());
    }

    #[test]
    fn drawer_reopen_and_filter_reset_publish_the_current_search_and_view_mode_caption() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        let mut assets = snapshot(3);
        assets.search_query = "current query".into();
        assets.view_mode = crate::ui::workbench::snapshot::AssetViewMode::Thumbnail;
        bridge.prepare_content_drawer_assets(&assets).unwrap();
        bridge.toggle_content_drawer().unwrap();
        assert_eq!(
            bridge
                .host_projection()
                .node_by_control_id("SearchEdited")
                .unwrap()
                .value_text
                .as_deref(),
            Some("current query")
        );
        assert_eq!(
            bridge
                .host_projection()
                .node_by_control_id("SetViewMode")
                .unwrap()
                .value_text
                .as_deref(),
            Some("Thumbnails")
        );
        bridge.dismiss_content_drawer().unwrap();
        assets.search_query.clear();
        assets.view_mode = crate::ui::workbench::snapshot::AssetViewMode::List;
        bridge.prepare_content_drawer_assets(&assets).unwrap();
        bridge.toggle_content_drawer().unwrap();
        assert_eq!(
            bridge
                .host_projection()
                .node_by_control_id("SearchEdited")
                .unwrap()
                .value_text
                .as_deref(),
            Some("")
        );
        assert_eq!(
            bridge
                .host_projection()
                .node_by_control_id("SetViewMode")
                .unwrap()
                .value_text
                .as_deref(),
            Some("List")
        );
    }

    #[test]
    fn actual_find_binding_and_enabled_projection_follow_the_current_controller_decision() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        let binding = bridge
            .binding_for_control(
                "WorkbenchToolbarAssets",
                zircon_runtime_interface::ui::binding::UiEventKind::Click,
            )
            .unwrap();
        assert!(
            matches!(binding.payload(), EditorUiBindingPayload::MenuAction { action_id } if action_id == "workbench.content_drawer.sync_selection")
        );
        let nodes = bridge.control_node_ids_with_descendants("WorkbenchToolbarAssets");
        assert!(!nodes.is_empty());
        bridge.sync_content_browser_find_enabled(false).unwrap();
        assert!(nodes.iter().all(|node| !bridge
            .surface()
            .tree
            .node(*node)
            .unwrap()
            .state_flags
            .enabled));
        bridge.sync_content_browser_find_enabled(true).unwrap();
        assert!(nodes.iter().all(|node| bridge
            .surface()
            .tree
            .node(*node)
            .unwrap()
            .state_flags
            .enabled));
    }

    #[test]
    fn find_reveals_a_far_catalog_item_using_the_authored_virtual_extent_and_focuses_its_view() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        let mut assets = snapshot(500);
        assets.selected_asset_uuid = Some("00000000-0000-0000-0000-000000000399".into());
        assets.visible_assets = assets.visible_assets.project_items(|asset| {
            asset.selected = asset.uuid == "00000000-0000-0000-0000-000000000399";
        });
        bridge.prepare_content_drawer_assets(&assets).unwrap();
        bridge.reveal_content_drawer_selection().unwrap();
        let rows =
            TemplateBridgeVirtualRowSequence::from_surface_repeat(bridge.surface(), BODY).unwrap();
        let bindings = rows.bindings(bridge.surface()).unwrap();
        assert!(bindings.iter().any(|binding| binding.logical_index == 399));
        assert!(bindings.len() < 100);
        assert_eq!(bridge.surface().focus.focused, bridge.control_node_id(BODY));
    }
}
