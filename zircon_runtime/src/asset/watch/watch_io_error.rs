pub(super) fn watch_io_error(error: notify::Error) -> std::io::Error {
    std::io::Error::other(error.to_string())
}
