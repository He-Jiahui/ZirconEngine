use crate::scene::EntityId;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
    zircon_reflect_derive::ZrReflect,
)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::Hierarchy",
    serialization = "none",
    serializable = false,
    script_visibility = "public"
)]
pub struct Hierarchy {
    #[zr_reflect(
        value_type_path = "Entity",
        editor_hint = "Entity",
        serializable = false
    )]
    pub parent: Option<EntityId>,
}
