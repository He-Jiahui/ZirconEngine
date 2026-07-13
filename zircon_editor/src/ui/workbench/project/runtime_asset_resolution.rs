pub(in crate::ui::workbench::project) fn invalid_data(
    message: impl Into<String>,
) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into())
}
