mod camera;
mod document;
mod mesh;
mod physics;
mod post_process;
mod prefab;
mod references;
mod scene_asset;
mod script;
mod transform;

pub use document::SceneProjectError;

const BUILTIN_CUBE: &str = "builtin://cube";
const BUILTIN_DEFAULT_MATERIAL: &str = "builtin://material/default";
const BUILTIN_MISSING_MODEL: &str = "builtin://missing-model";
const BUILTIN_MISSING_MATERIAL: &str = "builtin://missing-material";
const PREFAB_INSTANCE_COMPONENT: &str = "zircon.prefab.instance";
const SCRIPT_BINDINGS_COMPONENT: &str = "script.bindings";
