use super::*;

impl AssetWorkspaceState {
    pub(crate) fn content_browser_asset_uuids(
        &self,
        has_scene_selection: bool,
        resource_ids: &BTreeSet<String>,
        current_scene_uri: Option<&str>,
    ) -> Vec<String> {
        let Some(catalog) = self.catalog.as_ref() else {
            return Vec::new();
        };
        if has_scene_selection {
            // Only indexed top-level source assets are browsable. Missing or
            // builtin-only resources must not become invented catalog entries.
            // Referenced subresources have their own runtime ID. Resolve its
            // published locator, then browse the indexed top-level source.
            let source_locators = resource_ids
                .iter()
                .filter_map(|identity| {
                    identity
                        .parse::<zircon_runtime_interface::resource::ResourceId>()
                        .ok()
                })
                .filter_map(|identity| self.resources.row_by_id(identity))
                .filter_map(|resource| top_level_content_browser_locator(&resource.primary_locator))
                .collect::<BTreeSet<_>>();
            return catalog
                .assets
                .iter()
                .filter(|asset| {
                    resource_ids.contains(&asset.id)
                        || resource_ids.contains(&asset.uuid)
                        || source_locators.contains(&asset.locator)
                })
                .map(|asset| asset.uuid.clone())
                .collect();
        }
        current_scene_uri
            .and_then(|uri| catalog.asset_by_locator(uri))
            .filter(|asset| asset.kind == ResourceKind::Scene)
            .map(|asset| vec![asset.uuid.clone()])
            .unwrap_or_default()
    }

    pub(crate) fn sync_content_browser_assets(&mut self, uuids: &[String]) -> bool {
        let targets = uuids
            .iter()
            .filter_map(|uuid| self.asset_record(uuid))
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return false;
        }
        let selected: BTreeSet<String> = targets.iter().map(|asset| asset.uuid.clone()).collect();
        let primary = selected.iter().next().cloned();
        let parents = targets
            .iter()
            .map(|asset| parent_folder_id_for_locator(&asset.locator))
            .collect::<Vec<_>>();
        let folder = if parents.iter().all(|parent| parent == &parents[0]) {
            parents[0].clone()
        } else {
            "res://".into()
        };
        // UE PrepareToSyncItems clears its text search unconditionally.
        // Other filters are disabled only if they hide a requested asset.
        let clear_search = !self.search_query.is_empty();
        let clear_kind = targets
            .iter()
            .any(|asset| !asset_matches_filters(asset, "", self.kind_filter));
        let revealed = self.reveal_folder_ancestors(&folder);
        let changed = revealed
            || self.selected_folder_id != folder
            || self.selected_asset_uuid != primary
            || self.synced_asset_uuids != selected
            || clear_search
            || clear_kind;
        if !changed {
            return false;
        }
        if self.selected_asset_uuid != primary {
            self.selected_details = None;
        }
        self.selected_folder_id = folder;
        self.selected_asset_uuid = primary;
        self.synced_asset_uuids = selected;
        if clear_search {
            self.set_search_query(String::new());
        }
        if clear_kind {
            self.set_kind_filter(None);
        }
        self.item_generation_cache.borrow_mut().take();
        true
    }

    pub(super) fn asset_is_visible_in_selected_scope(
        &self,
        asset: &EditorAssetCatalogRecord,
    ) -> bool {
        asset_belongs_to_folder(asset, &self.selected_folder_id)
            || self.synced_asset_uuids.contains(&asset.uuid)
    }
}

fn top_level_content_browser_locator(locator: &str) -> Option<String> {
    let locator = AssetUri::parse(locator).ok()?;
    AssetUri::new(locator.scheme(), locator.path(), None)
        .ok()
        .map(|uri| uri.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::host::editor_asset_manager::EditorAssetCatalogSnapshotRecord;
    use zircon_runtime::asset::project::PreviewState;
    #[test]
    fn referenced_subresource_locator_browses_its_top_level_source_and_rejects_invalid_locators() {
        assert_eq!(
            top_level_content_browser_locator("res://models/ship.model#Mesh0"),
            Some("res://models/ship.model".into())
        );
        assert_eq!(
            top_level_content_browser_locator("res://models/ship.model"),
            Some("res://models/ship.model".into())
        );
        assert!(top_level_content_browser_locator("not-a-resource-locator").is_none());
    }

    fn asset(uuid: &str, id: &str, locator: &str, kind: ResourceKind) -> EditorAssetCatalogRecord {
        EditorAssetCatalogRecord {
            uuid: uuid.into(),
            id: id.into(),
            locator: locator.into(),
            kind,
            display_name: uuid.into(),
            file_name: uuid.into(),
            extension: String::new(),
            preview_state: PreviewState::Dirty,
            meta_path: String::new(),
            preview_artifact_path: String::new(),
            source_mtime_unix_ms: 0,
            source_hash: String::new(),
            dirty: false,
            diagnostics: vec![],
            direct_reference_uuids: vec![],
        }
    }
    fn workspace() -> AssetWorkspaceState {
        let mut state = AssetWorkspaceState::default();
        state.sync_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(
                EditorAssetCatalogSnapshotRecord {
                    assets: vec![
                        asset(
                            "model",
                            "runtime-model",
                            "res://models/ship.model",
                            ResourceKind::Model,
                        ),
                        asset(
                            "material",
                            "runtime-material",
                            "res://materials/ship.material",
                            ResourceKind::Material,
                        ),
                        asset(
                            "active",
                            "runtime-level",
                            "res://levels/active.scene",
                            ResourceKind::Scene,
                        ),
                        asset(
                            "default",
                            "runtime-default",
                            "res://levels/default.scene",
                            ResourceKind::Scene,
                        ),
                    ],
                    default_scene_uri: "res://levels/default.scene".into(),
                    ..Default::default()
                },
                1,
            ),
        ));
        state
    }
    #[test]
    fn selected_light_without_assets_does_not_fall_back_to_the_current_or_default_level() {
        let state = workspace();
        assert!(state
            .content_browser_asset_uuids(true, &BTreeSet::new(), Some("res://levels/active.scene"))
            .is_empty());
        assert_eq!(
            state.content_browser_asset_uuids(
                false,
                &BTreeSet::new(),
                Some("res://levels/active.scene")
            ),
            vec!["active"]
        );
        assert!(state
            .content_browser_asset_uuids(false, &BTreeSet::new(), None)
            .is_empty());
        assert!(state
            .content_browser_asset_uuids(false, &BTreeSet::new(), Some("res://untitled.scene"))
            .is_empty());
    }
    #[test]
    fn referenced_runtime_ids_resolve_to_catalog_sources_and_all_sync_targets_are_revealed() {
        let mut state = workspace();
        let targets = state.content_browser_asset_uuids(
            true,
            &BTreeSet::from([
                "runtime-model".into(),
                "runtime-material".into(),
                "missing".into(),
            ]),
            None,
        );
        assert_eq!(
            targets.iter().map(String::as_str).collect::<BTreeSet<_>>(),
            BTreeSet::from(["model", "material"])
        );
        state.set_search_query("cannot-match");
        state.set_kind_filter(Some(ResourceKind::Texture));
        assert!(state.sync_content_browser_assets(&targets));
        let snapshot = state.build_snapshot(AssetSurfaceMode::Explorer);
        assert!(snapshot.search_query.is_empty());
        assert_eq!(snapshot.kind_filter, None);
        assert_eq!(snapshot.selected_folder_id.as_deref(), Some("res://"));
        assert_eq!(
            snapshot
                .visible_assets
                .iter()
                .filter(|asset| asset.selected)
                .map(|asset| asset.uuid.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["material", "model"])
        );
        assert!(!state.sync_content_browser_assets(&targets));
        assert!(state.select_asset(Some("model".into())));
        assert!(state.synced_asset_uuids.is_empty());
    }
    #[test]
    fn sync_clears_text_search_but_preserves_a_compatible_kind_filter_and_missing_target_is_a_noop()
    {
        let mut state = workspace();
        state.set_search_query("model");
        state.set_kind_filter(Some(ResourceKind::Model));
        assert!(state.sync_content_browser_assets(&["model".into()]));
        assert!(state.search_query.is_empty());
        assert_eq!(state.kind_filter, Some(ResourceKind::Model));
        let selected = state.selected_asset_uuid.clone();
        state.set_search_query("unchanged");
        assert!(!state.sync_content_browser_assets(&["missing".into()]));
        assert_eq!(state.selected_asset_uuid, selected);
        assert_eq!(state.search_query, "unchanged");
    }

    #[test]
    fn selected_subresources_follow_the_current_published_management_generation_and_deduplicate_top_level_assets(
    ) {
        use zircon_runtime::core::resource::ResourceManager;
        use zircon_runtime_interface::resource::{ResourceId, ResourceLocator, ResourceRecord};

        let mut state = workspace();
        let first = ResourceLocator::parse("res://models/ship.model#Mesh0").unwrap();
        let second = ResourceLocator::parse("res://models/ship.model#Mesh1").unwrap();
        let first_id = ResourceId::from_locator(&first);
        let second_id = ResourceId::from_locator(&second);
        let references = BTreeSet::from([first_id.to_string(), second_id.to_string()]);
        assert!(state
            .content_browser_asset_uuids(true, &references, None)
            .is_empty());

        // Exercise the public production publisher rather than constructing a
        // private generation or assuming a top-level catalog ID is the mesh ID.
        let manager = ResourceManager::new();
        manager
            .register_lazy_records(vec![
                ResourceRecord::new(first_id, ResourceKind::Mesh, first),
                ResourceRecord::new(second_id, ResourceKind::Mesh, second),
            ])
            .unwrap();
        let published = manager.management_generation();
        assert_eq!(
            published
                .row_by_id(first_id)
                .unwrap()
                .primary_locator
                .as_ref(),
            "res://models/ship.model#Mesh0"
        );
        state.sync_resources(published);
        assert_eq!(
            state.content_browser_asset_uuids(true, &references, None),
            vec!["model"]
        );

        // Missing a reference in the currently published generation cannot
        // reuse an old source locator or fall back to the current level.
        state.sync_resources(Arc::new(ResourceManagementGeneration::default()));
        assert!(state
            .content_browser_asset_uuids(true, &references, Some("res://levels/active.scene"))
            .is_empty());
    }
}
