/// Scene-facing marker for values stored in the runtime world's typed resource table.
///
/// The neutral declaration remains owned by `core::framework::scene`; this leaf module owns the
/// ECS vocabulary without duplicating the trait or making the structural `resource/mod.rs` root
/// carry behavior.
pub use crate::core::framework::scene::SceneResource as Resource;
