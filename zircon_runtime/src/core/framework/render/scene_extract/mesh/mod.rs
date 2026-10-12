#[path = "lod_selection.rs"]
mod lod_selection;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "stable_key.rs"]
mod stable_key;
#[path = "static_state.rs"]
mod static_state;
#[path = "../../mesh/transform_revision.rs"]
mod transform_revision;

pub use lod_selection::RenderMeshLodSelection;
pub use snapshot::RenderMeshSnapshot;
pub use stable_key::{
    render_mesh_stable_instance_key, RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL,
    RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS,
};
pub use static_state::RenderMeshStaticState;
pub use transform_revision::render_mesh_transform_revision;
