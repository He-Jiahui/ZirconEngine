use std::sync::OnceLock;

use tracing_subscriber::layer::SubscriberExt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TracySinkStatus {
    pub installed: bool,
    pub message: String,
}

/// Installs the Tracy tracing layer for the current linked image.
/// 这是进程全局且只尝试一次的订阅器安装；已有全局订阅器时返回失败状态供宿主判断。
pub fn initialize_tracy_sink() -> TracySinkStatus {
    static STATUS: OnceLock<TracySinkStatus> = OnceLock::new();
    STATUS.get_or_init(install_tracy_sink).clone()
}

fn install_tracy_sink() -> TracySinkStatus {
    let subscriber = tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default());
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(()) => TracySinkStatus {
            installed: true,
            message: "Tracy profiling sink installed".to_string(),
        },
        Err(error) => TracySinkStatus {
            installed: false,
            message: format!("Tracy profiling sink was not installed: {error}"),
        },
    }
}

/// profile_frame! 创建的作用域标记；离开帧作用域时向 Tracy 跟踪层发送完成事件。
#[derive(Debug)]
pub struct TracyFrameScope {
    stream: &'static str,
    name: &'static str,
}

impl TracyFrameScope {
    pub fn enter(stream: &'static str, name: &'static str) -> Self {
        Self { stream, name }
    }
}

impl Drop for TracyFrameScope {
    fn drop(&mut self) {
        tracing::info!(
            target: "zircon.profile.frame",
            stream = self.stream,
            name = self.name,
            tracy.frame_mark = true,
            "finished frame"
        );
    }
}
