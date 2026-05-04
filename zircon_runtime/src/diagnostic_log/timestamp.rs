pub(super) fn current_log_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}
