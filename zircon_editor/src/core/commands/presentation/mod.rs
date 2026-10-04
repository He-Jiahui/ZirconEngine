//! 汇集命令展示声明的类型边界；稳定菜单身份和本地化键由各自类型校验，运行时词包绑定由贡献准入负责。

mod command_localization_source;
mod command_menu_path;
mod command_menu_segment;
mod command_menu_segment_id;
mod command_presentation;

pub use command_localization_source::EditorCommandLocalizationSource;
pub use command_menu_path::EditorCommandMenuPath;
pub use command_menu_segment::EditorCommandMenuSegment;
pub use command_menu_segment_id::EditorCommandMenuSegmentId;
pub use command_presentation::EditorCommandPresentation;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
