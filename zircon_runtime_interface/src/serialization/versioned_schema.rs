use super::{MigrationChain, SchemaId};

/// Declares the current version and complete forward migration chain for a payload.
/// 迁移链须覆盖从 0 到当前版本的每一步；读取当前版本也会校验整条链。
///
/// # 使用时机与生命周期
/// 每种参与持久化的具体类型实现该 trait，包括需要独立序列化的借用视图。读写入口处理文档
/// 时读取这些常量和静态迁移链；已写出的 schema 身份与历史迁移步骤必须继续可用，才能读取旧数据。
///
/// # 约束
/// `VERSION` 是当前 wire schema 版本；`migrations()` 必须返回从版本 0 到该版本的完整、
/// 有序迁移链。即使输入已是当前版本，读取仍会验证整条链。
///
/// # 参考
/// [`SchemaId`](super::SchemaId)、[`MigrationChain`](super::MigrationChain) 和
/// [`load_versioned`](super::load_versioned)。Runtime 与 Editor 的持久文档通过此 trait 声明
/// 各自身份；具体读取策略由其 loader 调用决定。
pub trait VersionedSchema: Sized {
    /// 跨写入版本保持稳定、用于 envelope 匹配的载荷族身份。
    const SCHEMA: SchemaId;
    /// 当前可写出且可直接解码的 schema 版本。
    const VERSION: u32;

    /// 返回该类型完整的前向迁移表；引用须在整个进程内有效。
    fn migrations() -> &'static MigrationChain<Self>;
}
