use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDirtyFlags {
    pub layout: bool,
    pub hit_test: bool,
    pub render: bool,
    pub style: bool,
    pub input: bool,
    pub visible_range: bool,
}

impl UiDirtyFlags {
    pub const fn any(self) -> bool {
        self.layout
            || self.hit_test
            || self.render
            || self.style
            || self.input
            || self.visible_range
    }
}
