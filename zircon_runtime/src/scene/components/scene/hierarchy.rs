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
/// 节点的父子关系源数据；重设父节点应走 World 的受检入口，以同时维护拓扑、派生变换和绑定失效代。
pub struct Hierarchy {
    #[zr_reflect(
        value_type_path = "Entity",
        editor_hint = "Entity",
        serializable = false
    )]
    pub parent: Option<EntityId>,
}
