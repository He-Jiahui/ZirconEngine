//! 变换轴视觉接受文字和图标形角色，明确排除可交互 Button；链接操作事件不由绘制身份推断。

const AXIS_LABEL_ROLE: &str = "Label";
const AXIS_ICON_ROLE: &str = "Icon";
const AXIS_SVG_ICON_ROLE: &str = "SvgIcon";

pub(super) fn is_axis_label_role(role: &str) -> bool {
    role == AXIS_LABEL_ROLE || role == AXIS_ICON_ROLE || role == AXIS_SVG_ICON_ROLE
}

#[cfg(test)]
#[path = "tests/roles.rs"]
mod tests;
