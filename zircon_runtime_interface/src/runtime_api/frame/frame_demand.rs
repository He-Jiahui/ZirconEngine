//! Runtime tick 返回给宿主的下一帧调度建议。
//!
//! ABI 保持 `kind` 为整数，避免未知值在跨库边界被直接解释成 Rust 枚举。Runtime FFI 在
//! 成功 tick 后写出该值；App 与 Editor Gateway 分别校验版本、种类和字段组合，再将其转为
//! 各自事件循环使用的调度状态。

use crate::version::ZIRCON_RUNTIME_ABI_VERSION_V1;

/// 不要求仅因运行时帧需求而继续安排帧。
pub const ZR_RUNTIME_FRAME_DEMAND_IDLE_V1: u32 = 0;
/// 请求宿主尽快继续安排一帧。
pub const ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1: u32 = 1;
/// 请求宿主在 `delay_nanoseconds` 指定的等待后安排一帧。
pub const ZR_RUNTIME_FRAME_DEMAND_AFTER_V1: u32 = 2;

/// Raw ABI carrier for the runtime's next-frame request.
///
/// `kind` deliberately remains a `u32`. Consumers must check it before
/// constructing any crate-local Rust enum.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 每次 tick 后由运行时返回给宿主，决定继续渲染、等待指定时长或按需唤醒。
/// 空闲与立即请求的延时字段必须为零；延时请求的上限由消费端裁定。
///
/// 每次成功的动态 `tick_frame` 都通过输出指针返回此值。App 的
/// `RuntimeLibrarySession::tick_frame` 与 Editor Gateway 分别验证 ABI 版本及字段组合，并将
/// `After` 延时限制在各自的最长等待值；本结构本身不限制该延时，也不代表帧已被提交或展示。
///
/// `idle` 和 `immediate` 将延时设为零；`after` 原样保存纳秒数。消费者须先校验 `kind`，
/// 并且不能把 ABI 整数直接当作本地枚举。
pub struct ZrRuntimeFrameDemandV1 {
    /// 生产此值的 Runtime ABI 版本，构造器填入 V1 常量。
    pub abi_version: u32,
    /// 由 `ZR_RUNTIME_FRAME_DEMAND_*_V1` 定义的整数种类。
    pub kind: u32,
    /// `After` 的等待时长；`Idle` 与 `Immediate` 必须为零。
    pub delay_nanoseconds: u64,
}

impl ZrRuntimeFrameDemandV1 {
    /// 构造不要求继续调度帧的 V1 值。
    ///
    /// # 返回值
    ///
    /// 返回 V1 版本、`Idle` 种类和零延时的 ABI 值，可用于初始化 `tick_frame` 输出；它只编码
    /// Runtime 对下一帧的需求，不表示帧已提交、展示或宿主已停止其他唤醒源。
    pub const fn idle() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            kind: ZR_RUNTIME_FRAME_DEMAND_IDLE_V1,
            delay_nanoseconds: 0,
        }
    }

    /// 构造请求立即继续调度的 V1 值。
    ///
    /// # 返回值
    ///
    /// 返回 V1 版本、`Immediate` 种类和零延时的 ABI 值。消费者据种类安排下一次调度；构造此值
    /// 本身不会唤醒宿主或提交下一帧。
    pub const fn immediate() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            kind: ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
            delay_nanoseconds: 0,
        }
    }

    /// 构造延时调度请求；最大等待时间由 Runtime/宿主消费端限制。
    ///
    /// # 参数
    ///
    /// `delay_nanoseconds` 是请求等待的纳秒数；此构造器不执行截断或范围校验。
    ///
    /// # 返回值
    ///
    /// 返回保留该原始纳秒值的 V1 `After` ABI 请求；App 与 Editor Gateway 消费时各自将等待时间截断
    /// 至最长 60 秒，因此此方法的结果仍须由消费端校验并转换。
    pub const fn after(delay_nanoseconds: u64) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            kind: ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
            delay_nanoseconds,
        }
    }

    /// 判断 `kind` 是否为本接口版本定义的三种种类之一。
    ///
    /// # 返回值
    ///
    /// 仅检查整数种类，不检查 ABI 版本或延时字段；完整字段组合使用 `is_valid` 检查。
    pub const fn has_known_kind(self) -> bool {
        matches!(
            self.kind,
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1
                | ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1
                | ZR_RUNTIME_FRAME_DEMAND_AFTER_V1
        )
    }

    /// 检查 V1 版本以及种类对应的延时约束。
    ///
    /// `After` 接受任意 `u64` 纳秒值，限制等待上限的是消费端；未知种类、其他 ABI 版本，
    /// 或带非零延时的 `Idle`/`Immediate` 均返回 `false`。Editor Gateway 用此谓词验证值。
    pub const fn is_valid(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && match self.kind {
                ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 | ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 => {
                    self.delay_nanoseconds == 0
                }
                ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 => true,
                _ => false,
            }
    }
}
