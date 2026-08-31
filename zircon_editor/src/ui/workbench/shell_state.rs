use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::asset::AssetTypeRegistry;
use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::core::editor_extension::EditorContributionHandle;
use crate::core::extension::{ContributionStore, ContributionTicket};
use crate::ui::control::EditorUiControlService;
use crate::ui::host::{EditorError, EditorManager};
use crate::ui::workbench::reflection::EditorTransientUiState;
use crate::ui::workbench::snapshot::AssetWorkspaceItemGeneration;
use crate::ui::workbench::state::EditorState;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};
use crate::ui::workbench::ActivityLogConsoleProjection;

const GAME_VIEW_DESCRIPTOR_ID: &str = "editor.game";

/// UI-only workbench authority removed from the headless editor core.
pub(crate) struct WorkbenchShellState {
    state: Mutex<WorkbenchShellStateData>,
}

pub(crate) struct WorkbenchShellStateData {
    pub(crate) state: EditorState,
    pub(crate) manager: Arc<EditorManager>,
    pub(crate) transient: EditorTransientUiState,
    pub(crate) control_service: EditorUiControlService,
    pub(crate) contributions: ContributionStore,
    pub(crate) contribution_owners: Vec<OwnedContribution>,
    pub(crate) asset_type_registry_cache: AssetTypeRegistryGenerationCache,
    pub(crate) activity_log_console_projection: ActivityLogConsoleProjection,
    pub(crate) console_message_filter: ConsoleMessageFilter,
    pub(crate) console_source_filter: ConsoleSourceFilter,
    play_preview_descriptor: ViewDescriptorId,
    play_preview_restore_view: Option<ViewInstanceId>,
}

impl WorkbenchShellStateData {
    pub(crate) fn contributions_changed(&mut self) {
        self.asset_type_registry_cache.contributions_changed();
    }

    pub(crate) fn set_console_source_filter(&mut self, filter: ConsoleSourceFilter) -> bool {
        let changed = self.console_source_filter != filter;
        self.console_source_filter = filter;
        changed
    }

    pub(crate) fn set_console_message_filter(&mut self, filter: ConsoleMessageFilter) -> bool {
        let changed = self.console_message_filter != filter;
        self.console_message_filter = filter;
        changed
    }

    pub(crate) fn focus_play_preview_view(&mut self) -> Result<bool, EditorError> {
        let previous = self.manager.current_focused_view();
        let descriptor = self.play_preview_descriptor.clone();
        let existing = self
            .manager
            .current_view_instances()
            .into_iter()
            .find(|instance| instance.descriptor_id == descriptor)
            .map(|instance| instance.instance_id);
        let (game_view, opened) = match existing {
            Some(instance) => (instance, false),
            None => (self.manager.open_view(descriptor, None)?, true),
        };
        if previous.as_ref() == Some(&game_view) {
            return Ok(false);
        }
        let changed = if opened {
            true
        } else {
            self.manager.focus_view(&game_view)?
        };
        if changed {
            self.play_preview_restore_view = previous;
        }
        Ok(changed)
    }

    pub(crate) fn restore_pre_play_view(&mut self) -> Result<bool, EditorError> {
        let Some(instance) = self.play_preview_restore_view.take() else {
            return Ok(false);
        };
        self.manager.focus_view(&instance)
    }

    pub(crate) fn play_preview_view_focused(&self) -> bool {
        self.manager
            .current_focused_view_matches(&self.play_preview_descriptor)
    }
}

pub(crate) struct OwnedContribution {
    handle: EditorContributionHandle,
}

impl OwnedContribution {
    pub(crate) fn new(handle: EditorContributionHandle) -> Self {
        Self { handle }
    }

    pub(crate) fn handle(&self) -> &EditorContributionHandle {
        &self.handle
    }

    pub(crate) fn owner_id(&self) -> &str {
        self.handle.owner_id()
    }

    pub(crate) fn ticket(&self) -> ContributionTicket {
        self.handle.ticket()
    }

    pub(crate) const fn owner_generation(&self) -> crate::core::tools::ToolOwnerGeneration {
        self.handle.owner_generation()
    }
}

#[derive(Default)]
pub(crate) struct AssetTypeRegistryGenerationCache {
    extension_generation: u64,
    cached: Option<CachedAssetTypeRegistry>,
    cache_hits: u64,
    materializations: u64,
    cached_asset_item_projection: Option<CachedAssetItemProjection>,
}

struct CachedAssetTypeRegistry {
    extension_generation: u64,
    enabled_capabilities: Vec<String>,
    registry: Arc<AssetTypeRegistry>,
}

struct CachedAssetItemProjection {
    source: AssetWorkspaceItemGeneration,
    registry: Arc<AssetTypeRegistry>,
    projected: AssetWorkspaceItemGeneration,
}

impl AssetTypeRegistryGenerationCache {
    pub(crate) fn contributions_changed(&mut self) {
        self.extension_generation = self.extension_generation.saturating_add(1);
        self.cached = None;
        self.cached_asset_item_projection = None;
    }

    pub(crate) fn get(
        &mut self,
        enabled_capabilities: &[String],
    ) -> Option<Arc<AssetTypeRegistry>> {
        let cached = self.cached.as_ref().filter(|cached| {
            cached.extension_generation == self.extension_generation
                && cached.enabled_capabilities.as_slice() == enabled_capabilities
        })?;
        let registry = Arc::clone(&cached.registry);
        self.cache_hits = self.cache_hits.saturating_add(1);
        Some(registry)
    }

    pub(crate) fn store(
        &mut self,
        enabled_capabilities: Vec<String>,
        registry: Arc<AssetTypeRegistry>,
    ) {
        self.materializations = self.materializations.saturating_add(1);
        self.cached_asset_item_projection = None;
        self.cached = Some(CachedAssetTypeRegistry {
            extension_generation: self.extension_generation,
            enabled_capabilities,
            registry,
        });
    }

    pub(crate) fn projected_asset_items(
        &self,
        source: &AssetWorkspaceItemGeneration,
        registry: &Arc<AssetTypeRegistry>,
    ) -> Option<AssetWorkspaceItemGeneration> {
        let cached = self
            .cached_asset_item_projection
            .as_ref()
            .filter(|cached| {
                cached.source.shares_items_with(source) && Arc::ptr_eq(&cached.registry, registry)
            })?;
        Some(cached.projected.clone())
    }

    pub(crate) fn previous_asset_item_projection(
        &self,
        registry: &Arc<AssetTypeRegistry>,
    ) -> Option<(AssetWorkspaceItemGeneration, AssetWorkspaceItemGeneration)> {
        let cached = self
            .cached_asset_item_projection
            .as_ref()
            .filter(|cached| Arc::ptr_eq(&cached.registry, registry))?;
        Some((cached.source.clone(), cached.projected.clone()))
    }

    pub(crate) fn store_projected_asset_items(
        &mut self,
        source: AssetWorkspaceItemGeneration,
        registry: Arc<AssetTypeRegistry>,
        projected: AssetWorkspaceItemGeneration,
    ) {
        self.cached_asset_item_projection = Some(CachedAssetItemProjection {
            source,
            registry,
            projected,
        });
    }

    #[cfg(test)]
    pub(crate) fn counts(&self) -> (u64, u64) {
        (self.cache_hits, self.materializations)
    }
}

impl WorkbenchShellState {
    pub(crate) fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        Self {
            state: Mutex::new(WorkbenchShellStateData {
                state,
                manager,
                transient: EditorTransientUiState::default(),
                control_service: EditorUiControlService::default(),
                contributions: ContributionStore::default(),
                contribution_owners: Vec::new(),
                asset_type_registry_cache: AssetTypeRegistryGenerationCache::default(),
                activity_log_console_projection: ActivityLogConsoleProjection::default(),
                console_message_filter: ConsoleMessageFilter::default(),
                console_source_filter: ConsoleSourceFilter::default(),
                play_preview_descriptor: ViewDescriptorId::new(GAME_VIEW_DESCRIPTOR_ID),
                play_preview_restore_view: None,
            }),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, WorkbenchShellStateData> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
