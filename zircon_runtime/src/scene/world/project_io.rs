#[path = "project_io/camera.rs"]
mod camera;
#[path = "project_io/components.rs"]
mod components;
#[path = "project_io/document.rs"]
mod document;
#[path = "project_io/mesh.rs"]
mod mesh;
#[path = "project_io/physics.rs"]
mod physics;
#[path = "project_io/post_process.rs"]
mod post_process;
#[path = "project_io/prefab.rs"]
mod prefab;
#[path = "project_io/references.rs"]
mod references;
#[path = "project_io/render2d.rs"]
mod render2d;
#[path = "project_io/scene_asset.rs"]
mod scene_asset;
#[path = "project_io/script.rs"]
mod script;
#[path = "project_io/source_document.rs"]
mod source_document;
#[path = "project_io/transform.rs"]
mod transform;

pub use components::SceneComponentSerializer;
pub(crate) use components::SceneComponentSerializerRegistry;
pub use document::SceneProjectError;
pub use source_document::{LoadedSceneDocument, ScenePublicationReceipt, SceneSourceRevision};

const BUILTIN_CUBE: &str = "builtin://cube";
const BUILTIN_DEFAULT_MATERIAL: &str = "builtin://material/default";
const BUILTIN_MISSING_MODEL: &str = "builtin://missing-model";
const BUILTIN_MISSING_MATERIAL: &str = "builtin://missing-material";
const BUILTIN_MISSING_TEXTURE: &str = "builtin://missing-texture";
const BUILTIN_QUAD: &str = "builtin://quad";
const PREFAB_INSTANCE_COMPONENT: &str = "zircon.prefab.instance";
const SCRIPT_BINDINGS_COMPONENT: &str = "script.bindings";
