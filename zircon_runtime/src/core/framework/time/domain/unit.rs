/// 时钟域值的静态解释标签，用于区分持续时长与 UTC 日历时间戳。
///
/// [ClockDomainRegistry](crate::core::ClockDomainRegistry) 将它附到描述符和 stamp，
/// 标签本身不持有数值、不做单位换算，也不执行跨域校验。
/// 当前内建 Time 时钟提供 Duration；UnixTimestamp 是 WallUtc 的分类，不能据标签推断已有日历采样值。
///
/// Unit carried by a clock-domain value so duration and calendar values cannot be interchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClockDomainUnit {
    /// 经过时间或派生时间步的类别；内建 Time 以 std::time::Duration 表示数值。
    Duration,
    /// WallUtc 的日历时间戳类别；标签不指定序列化精度，也不提供 UTC 采样值。
    UnixTimestamp,
}
