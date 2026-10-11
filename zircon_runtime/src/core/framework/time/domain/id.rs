/// 时间流的域类别，用于选择快照中的时钟和解释其单位。
///
/// # 所有权边界
///
/// 外帧快照提供 MonotonicReal；World 快照另外提供 WorldVirtual 与 WorldFixed。
/// 其余类别参与静态分类，但这两类快照当前不返回它们的 stamp。
/// 同一类别可由多个 World 使用；该枚举及显式数值不标识具体 World 或时钟实例。
///
/// # 参考
///
/// 单位映射见 [ClockDomainRegistry::descriptor](crate::core::ClockDomainRegistry::descriptor)，
/// 运行时代际见 [ClockDomainStamp](crate::core::ClockDomainStamp)。
///
/// Stable identities for every time source and derived time stream exposed by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ClockDomainId {
    /// 外层 Runtime 采样的单调真实时间；各 World 从外帧快照继承其来源代际。
    MonotonicReal = 1,
    /// UTC 日历时间类别，静态单位为 UnixTimestamp；当前外帧和 World 快照均不提供它。
    WallUtc = 2,
    /// World 根据真实时间、速率和暂停状态派生的虚拟时间。
    WorldVirtual = 3,
    /// World 用虚拟时间积累步进债务、按预算逐步提交的固定步时间。
    WorldFixed = 4,
    Input = 5,
    Render = 6,
    Audio = 7,
    Network = 8,
    Media = 9,
    EditorPreview = 10,
}
