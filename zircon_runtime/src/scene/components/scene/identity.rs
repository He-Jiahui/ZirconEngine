use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Empty,
    Camera,
    Cube,
    Mesh,
    AmbientLight,
    DirectionalLight,
    PointLight,
    RectLight,
    SpotLight,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::Name",
    script_visibility = "public"
)]
pub struct Name(
    #[zr_reflect(name = "value", value_type_path = "String", editor_hint = "String")] pub String,
);
