use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::asset::AssetTypeRegistry;
use crate::core::editor_extension::EditorExtensionRegistration;
use crate::ui::control::EditorUiControlService;
use crate::ui::host::EditorManager;
use crate::ui::workbench::reflection::EditorTransientUiState;
use crate::ui::workbench::state::EditorState;

/// UI-only workbench authority removed from the headless editor core.
pub(crate) struct WorkbenchShellState {
    state: Mutex<WorkbenchShellStateData>,
}

pub(crate) struct WorkbenchShellStateData {
    pub(crate) state: EditorState,
    pub(crate) manager: Arc<EditorManager>,
    pub(crate) transient: EditorTransientUiState,
    pub(crate) control_service: EditorUiControlService,
    pub(crate) editor_extensions: Vec<EditorExtensionRegistration>,
    pub(crate) editor_template_generation: u64,
    pub(crate) asset_type_registry_cache: AssetTypeRegistryGenerationCache,
}

impl WorkbenchShellStateData {
    pub(crate) fn template_contributions_changed(&mut self) {
        self.editor_template_generation = self.editor_template_generation.saturating_add(1);
    }

    pub(crate) fn extension_registered(&mut self) {
        self.template_contributions_changed();
        self.asset_type_registry_cache.extension_registered();
    }
}

#[derive(Default)]
pub(crate) struct AssetTypeRegistryGenerationCache {
    extension_generation: u64,
    cached: Option<CachedAssetTypeRegistry>,
    cache_hits: u64,
    materializations: u64,
}

struct CachedAssetTypeRegistry {
    extension_generation: u64,
    enabled_capabilities: Vec<String>,
    registry: Arc<AssetTypeRegistry>,
}

impl AssetTypeRegistryGenerationCache {
    pub(crate) fn extension_registered(&mut self) {
        self.extension_generation = self.extension_generation.saturating_add(1);
        self.cached = None;
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
        self.cached = Some(CachedAssetTypeRegistry {
            extension_generation: self.extension_generation,
            enabled_capabilities,
            registry,
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
                editor_extensions: Vec::new(),
                editor_template_generation: 0,
                asset_type_registry_cache: AssetTypeRegistryGenerationCache::default(),
            }),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, WorkbenchShellStateData> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
