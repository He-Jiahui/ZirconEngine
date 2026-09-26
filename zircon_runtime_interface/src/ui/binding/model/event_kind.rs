use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::UiBindingParseError;

/// 原生绑定语法中的事件标识；`native_name` 与 `FromStr` 共同约定 `onX` 字面量。
/// Runtime 的旧模板与 v2 构建器还据此推断节点的输入能力，扩展事件时须同步两条路径。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiEventKind {
    Click,
    DoubleClick,
    Hover,
    Press,
    Release,
    Change,
    Submit,
    Toggle,
    Focus,
    Blur,
    // BUG: [CR-UITREE-0003] 两条 Runtime 推断路径只为 Scroll 设置 receives_input；
    // 无其他交互能力的 onScroll 节点不满足命中表准入，无法成为滚轮目标。
    Scroll,
    Resize,
    DragBegin,
    DragUpdate,
    DragEnd,
    Drop,
}

impl UiEventKind {
    pub fn native_name(self) -> &'static str {
        match self {
            Self::Click => "onClick",
            Self::DoubleClick => "onDoubleClick",
            Self::Hover => "onHover",
            Self::Press => "onPress",
            Self::Release => "onRelease",
            Self::Change => "onChange",
            Self::Submit => "onSubmit",
            Self::Toggle => "onToggle",
            Self::Focus => "onFocus",
            Self::Blur => "onBlur",
            Self::Scroll => "onScroll",
            Self::Resize => "onResize",
            Self::DragBegin => "onDragBegin",
            Self::DragUpdate => "onDragUpdate",
            Self::DragEnd => "onDragEnd",
            Self::Drop => "onDrop",
        }
    }
}

impl fmt::Display for UiEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.native_name())
    }
}

impl FromStr for UiEventKind {
    type Err = UiBindingParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "onClick" => Ok(Self::Click),
            "onDoubleClick" => Ok(Self::DoubleClick),
            "onHover" => Ok(Self::Hover),
            "onPress" => Ok(Self::Press),
            "onRelease" => Ok(Self::Release),
            "onChange" => Ok(Self::Change),
            "onSubmit" => Ok(Self::Submit),
            "onToggle" => Ok(Self::Toggle),
            "onFocus" => Ok(Self::Focus),
            "onBlur" => Ok(Self::Blur),
            "onScroll" => Ok(Self::Scroll),
            "onResize" => Ok(Self::Resize),
            "onDragBegin" => Ok(Self::DragBegin),
            "onDragUpdate" => Ok(Self::DragUpdate),
            "onDragEnd" => Ok(Self::DragEnd),
            "onDrop" => Ok(Self::Drop),
            other => Err(UiBindingParseError::UnknownEventKind(other.to_string())),
        }
    }
}
