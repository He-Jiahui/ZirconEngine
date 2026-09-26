/// 宿主实际观察到的基础能力，供运行时报告与静态能力目录交叉验证。
/// 配置启用或后端自称支持，并不能替代此观测事实。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformHostObservedCapabilities {
    event_loop: bool,
    windowing: bool,
    display_topology: bool,
}

impl PlatformHostObservedCapabilities {
    pub const fn new(event_loop: bool, windowing: bool, display_topology: bool) -> Self {
        Self {
            event_loop,
            windowing,
            display_topology,
        }
    }

    pub const fn event_loop(self) -> bool {
        self.event_loop
    }

    pub const fn windowing(self) -> bool {
        self.windowing
    }

    pub const fn display_topology(self) -> bool {
        self.display_topology
    }
}
