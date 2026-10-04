/// 为日志候选目录后缀和每批日志行前缀生成秒级本地时间字符串。
pub(super) fn current_log_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}
