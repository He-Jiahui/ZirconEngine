//! 报告会话锁已发布，以及发布后的目录同步是否存在耐久不确定性；调用方不能把已发布等同于已完全落盘。

/// Whether the latest lock mutation was published and whether its directory sync was uncertain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionLockDurability {
    Published,
    PublishedWithDurabilityUncertainty,
}
