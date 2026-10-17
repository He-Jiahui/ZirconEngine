use serde::{Deserialize, Serialize};

/// Stable runtime stage contract shared by the lifecycle kernel, plugins, and scene scheduler.
///
/// 声明调度位置，serde 保留变体原名，Rust 枚举布局不作为原生插件编码。
/// 显式 `ORDER` 序号由 ABI 解码使用，固定步组由 WorldDriver 在每个已规划固定步内执行。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemStage {
    /// 帧调度最前的位置。
    First,
    /// 普通更新前的位置，排在固定步组之前。
    PreUpdate,
    /// 每个固定步组的首阶段。
    FixedFirst,
    /// 每个固定步组的主更新阶段。
    FixedUpdate,
    /// 每个固定步组的后置阶段。
    FixedPostUpdate,
    /// 普通更新位置，排在固定步组之后。
    Update,
    /// 普通更新后的调度位置。
    PostUpdate,
    /// 渲染提取之前的末尾更新位置。
    Last,
    /// 渲染提取的调度位置。
    RenderExtract,
}

impl SystemStage {
    /// 内建阶段总数，与显式顺序表长度一致。
    pub const COUNT: usize = 9;
    // 此顺序同时规定 Native Host API 的 u32 阶段编号；重排会改变既有插件的阶段含义。
    pub const ORDER: [Self; Self::COUNT] = [
        Self::First,
        Self::PreUpdate,
        Self::FixedFirst,
        Self::FixedUpdate,
        Self::FixedPostUpdate,
        Self::Update,
        Self::PostUpdate,
        Self::Last,
        Self::RenderExtract,
    ];
    /// WorldDriver 在每个固定步内完整运行这三个阶段，提交或回滚以整步为边界。
    pub const FIXED_LOOP: [Self; 3] = [Self::FixedFirst, Self::FixedUpdate, Self::FixedPostUpdate];

    /// 返回本阶段在显式顺序表中的零起始序号，供阶段分组和排序使用。
    pub const fn rank(self) -> usize {
        match self {
            Self::First => 0,
            Self::PreUpdate => 1,
            Self::FixedFirst => 2,
            Self::FixedUpdate => 3,
            Self::FixedPostUpdate => 4,
            Self::Update => 5,
            Self::PostUpdate => 6,
            Self::Last => 7,
            Self::RenderExtract => 8,
        }
    }

    /// 判断是否属于固定步的三个阶段，该方法不推进时钟或执行阶段。
    pub const fn is_fixed_loop(self) -> bool {
        matches!(
            self,
            Self::FixedFirst | Self::FixedUpdate | Self::FixedPostUpdate
        )
    }
}
