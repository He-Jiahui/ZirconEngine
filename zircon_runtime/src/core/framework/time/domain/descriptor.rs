use super::{ClockDomainId, ClockDomainUnit};

/// 从内建时钟分类取得的域类别与单位描述。
///
/// 调用者可先取得描述，再选择时间值的解释方式；查询包含尚无运行时实例的域，
/// 因此描述符本身不证明某个时钟已经创建或推进。
///
/// # 构造与生命周期
///
/// 字段不对外开放，由 [ClockDomainRegistry](crate::core::ClockDomainRegistry) 的静态映射组装。
/// 返回值可独立拷贝，不借用注册表或时钟实例。
///
/// Immutable entry in the canonical clock-domain taxonomy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClockDomainDescriptor {
    id: ClockDomainId,
    unit: ClockDomainUnit,
}

impl ClockDomainDescriptor {
    // 内建注册表在调用处选定匹配的域与单位；此构造器不另行校验配对关系。
    pub(crate) const fn new(id: ClockDomainId, unit: ClockDomainUnit) -> Self {
        Self { id, unit }
    }

    pub const fn id(self) -> ClockDomainId {
        self.id
    }

    pub const fn unit(self) -> ClockDomainUnit {
        self.unit
    }
}
