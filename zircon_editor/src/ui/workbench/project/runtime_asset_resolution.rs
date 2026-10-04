// TODO: [CR-EDITOR-WORKBENCH-0010] 确认此项目内错误转换辅助的长期用途；当前模块仍装配，但未找到生产或测试消费者，宿主预览使用另一处同名实现。
/// 项目内InvalidData转换辅助；当前没有接入的消费链，不能视作宿主预览错误入口。
pub(in crate::ui::workbench::project) fn invalid_data(
    message: impl Into<String>,
) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into())
}
