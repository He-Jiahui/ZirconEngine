use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderCapability {
    Primitive,
    Text,
    Image,
    Vector,
    Clip,
    Scroll,
    Canvas,
    VirtualizedLayout,
}
