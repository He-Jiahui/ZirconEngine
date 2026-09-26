use serde::{Deserialize, Serialize};

/// 节点级失效域；UiTree 汇总后由 Runtime 分别安排布局、命中、绘制等更新。
/// 多个域可同时置位；`any` 仅判断是否还有待处理工作，不能代替各域的消费判断。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDirtyFlags {
    pub layout: bool,
    pub hit_test: bool,
    pub render: bool,
    pub style: bool,
    pub text: bool,
    pub input: bool,
    pub visible_range: bool,
}

impl UiDirtyFlags {
    pub const fn any(self) -> bool {
        self.layout
            || self.hit_test
            || self.render
            || self.style
            || self.text
            || self.input
            || self.visible_range
    }
}
