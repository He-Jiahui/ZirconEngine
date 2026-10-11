use super::{ClockDomainDescriptor, ClockDomainId, ClockDomainUnit};

/// 内建时钟域与数值单位的静态分类入口。
///
/// # 使用时机
///
/// 查询某一域的单位，或为初始 stamp 取得一致的描述；查询不要求该域已有运行时时钟。
/// 此零大小值无可变注册状态和分配，不负责采样、推进、暂停或重置时间。
/// 运行时代际保存在 [ClockDomainStamp](crate::core::ClockDomainStamp)，与本分类版本分别管理。
///
/// Versioned static taxonomy for engine clock domains.
///
/// The registry deliberately contains no mutable state or allocation. Runtime clock instances
/// carry their own epoch and source-generation stamp instead of consulting a global service.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockDomainRegistry;

impl ClockDomainRegistry {
    /// 当前编译版本的内建域分类版本，与时钟的 epoch 和来源代际无关。
    /// 读取该常量不会进行协议协商或兼容性检查。
    pub const VERSION: u16 = 1;

    /// 取得内建静态分类，供后续 [descriptor](Self::descriptor) 查询使用。
    ///
    /// # 返回与生命周期
    ///
    /// 返回可拷贝的零大小值；无需初始化、注册或释放服务，查询也不借用全局可变状态。
    pub const fn builtin() -> Self {
        Self
    }

    pub const fn version(self) -> u16 {
        Self::VERSION
    }

    /// 查询域类别对应的静态单位描述。
    ///
    /// # 参数
    ///
    /// - id：待查询的域类别，包括尚未由当前快照提供的类别。
    ///
    /// # 返回与边界
    ///
    /// 穷尽匹配所有内建类别并返回 ClockDomainDescriptor；WallUtc 对应 UnixTimestamp，其余对应 Duration。
    /// 返回成功只证明静态映射存在，不证明运行时时钟存在，也不产生时间值或单位换算。
    /// 结果不借用分类入口，可独立保留和拷贝。
    pub const fn descriptor(self, id: ClockDomainId) -> ClockDomainDescriptor {
        match id {
            ClockDomainId::MonotonicReal => {
                ClockDomainDescriptor::new(id, ClockDomainUnit::Duration)
            }
            ClockDomainId::WallUtc => {
                ClockDomainDescriptor::new(id, ClockDomainUnit::UnixTimestamp)
            }
            ClockDomainId::WorldVirtual
            | ClockDomainId::WorldFixed
            | ClockDomainId::Input
            | ClockDomainId::Render
            | ClockDomainId::Audio
            | ClockDomainId::Network
            | ClockDomainId::Media
            | ClockDomainId::EditorPreview => {
                ClockDomainDescriptor::new(id, ClockDomainUnit::Duration)
            }
        }
    }
}
