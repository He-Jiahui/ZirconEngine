use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationSkinningBackend {
    #[default]
    Cpu,
    Gpu,
    Hybrid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationGpuSkinningReadiness {
    pub enabled: bool,
    pub backend: AnimationSkinningBackend,
    pub skinned_entities: u32,
    pub mesh_targets: u32,
    pub bone_palette_bytes: u64,
    pub morph_target_bytes: u64,
    pub missing_gpu_resources: Vec<String>,
    pub diagnostics: Vec<String>,
}

impl Default for AnimationGpuSkinningReadiness {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: AnimationSkinningBackend::Cpu,
            skinned_entities: 0,
            mesh_targets: 0,
            bone_palette_bytes: 0,
            morph_target_bytes: 0,
            missing_gpu_resources: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

impl AnimationGpuSkinningReadiness {
    pub fn ready_for_gpu_skinning(&self) -> bool {
        self.enabled
            && matches!(
                self.backend,
                AnimationSkinningBackend::Gpu | AnimationSkinningBackend::Hybrid
            )
            && self.missing_gpu_resources.is_empty()
    }

    pub fn with_missing_gpu_resource(mut self, resource: impl Into<String>) -> Self {
        self.missing_gpu_resources.push(resource.into());
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostics.push(diagnostic.into());
        self
    }
}
