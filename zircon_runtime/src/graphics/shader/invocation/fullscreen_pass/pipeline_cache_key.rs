use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FullscreenPipelineCacheKey {
    pub shader: AssetReference,
    pub fragment_entry: String,
    pub option_bits: u32,
    pub content_hash: u64,
}

impl FullscreenPipelineCacheKey {
    pub fn canonical_string(&self) -> String {
        format!(
            "shader_fullscreen_pipeline_v1|shader={}|fragment={}|options={:#010x}|content={:#018x}",
            self.shader, self.fragment_entry, self.option_bits, self.content_hash
        )
    }
}
