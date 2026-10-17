use super::{ClockDomainId, ClockDomainRegistry, ClockDomainUnit};

/// 随时间观测值传递的域类别、单位及两级代际快照。
///
/// # 代际含义
///
/// - epoch：内建 World 时钟的虚拟速率、钳制上限或固定步长策略发生变化时递增。
///   暂停、恢复和普通帧推进不改变该值；饱和到 u64::MAX 后也不再增加。
/// - source_generation：从外帧继承的来源版本，用于观察底层时钟重基线。
///   设置者负责传入已接受的版本，本值不验证单调性。
///
/// # 比较与生命周期
///
/// 可拷贝并随不可变快照保留；不包含时间数值、帧索引或 World 身份。
/// 只有已知属于同一时钟所有者时，代际比较才能说明配置或来源是否变化；
/// 不同 World 可以持有完全相等的 stamp。取得 stamp 本身不推进或重新采样时钟。
///
/// Runtime identity attached to a time value.
///
/// Explicit World Reset/Restore also advance epochs with a checked, non-reusing
/// timeline boundary. Existing policy-only changes retain their saturating contract.
///
/// `epoch` changes when that domain's semantics are reconfigured. `source_generation` changes
/// when the underlying source is rebased, such as after activation or a lifecycle discontinuity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClockDomainStamp {
    id: ClockDomainId,
    unit: ClockDomainUnit,
    epoch: u64,
    source_generation: u64,
}

impl ClockDomainStamp {
    // 框架时钟和测试上下文从静态分类建立初始身份；后续代际由各自的时钟所有者更新。
    pub(crate) const fn initial(id: ClockDomainId) -> Self {
        let descriptor = ClockDomainRegistry::builtin().descriptor(id);
        Self {
            id: descriptor.id(),
            unit: descriptor.unit(),
            epoch: 0,
            source_generation: 0,
        }
    }

    pub const fn id(self) -> ClockDomainId {
        self.id
    }

    pub const fn unit(self) -> ClockDomainUnit {
        self.unit
    }

    /// 观察域配置的版本，用于识别同一 World 时钟的语义配置变化。
    ///
    /// 内建 World 在虚拟速率、钳制上限或固定步长改变后更新对应值；暂停和普通推进不更新。
    /// 该值在 u64::MAX 饱和，不能作为无限递增的唯一序号，也不代表帧数。
    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    /// 观察底层时钟来源的版本，以识别同一时钟所有者的重基线边界。
    ///
    /// 内建 Runtime 从重基线回执取得版本，World 再随外帧传播到虚拟与固定步时钟。
    /// 它不是 World 的配置版本或帧索引；读取旧快照仍得到旧快照记录的版本。
    pub const fn source_generation(self) -> u64 {
        self.source_generation
    }

    // World 策略提交仅更新受影响时钟的配置代际；饱和后不能继续区分新的配置变化。
    pub(crate) fn bump_epoch(&mut self) {
        self.epoch = self.epoch.saturating_add(1);
    }

    // A timeline replacement must not reuse an earlier epoch. Return a copy
    // only after checking both the current stamp and the caller's epoch floor.
    pub(crate) fn checked_epoch_after(self, floor: u64) -> Option<Self> {
        let epoch = self.epoch.max(floor).checked_add(1)?;
        Some(Self { epoch, ..self })
    }

    // 接收 Runtime 已接受的外帧来源版本；此处不递增或校验单调性，World 每帧复制该值。
    pub(crate) fn set_source_generation(&mut self, source_generation: u64) {
        self.source_generation = source_generation;
    }
}
