use thiserror::Error;

use super::{TimePolicy, TimePolicyError, TimePolicyTransaction};

/// 用于选择产品时间预设并标识配置来源的角色类别。
///
/// # 选择边界
///
/// 内建配置由 [ProductTimePolicies::for_profile](crate::core::runtime::ProductTimePolicies::for_profile)
/// 生成；动态会话将运行模式映射到此类别后，分别执行配置准备与 Runtime 提交。
/// [ProductTimePolicy::new] 仍可接收自定义值和预算，类别本身不强制它们等于内建预设。
/// 显式判别值还参与配置摘要；它表示角色，不是某个会话或 World 的运行时身份。
/// Product role used to select a versioned runtime time-policy preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProductTimeProfile {
    Client = 1,
    Headless = 2,
    Editor = 3,
    Test = 4,
}

/// 带产品角色、版本与每外帧固定步预算的时间策略值。
///
/// # 使用时机
///
/// 宿主选择配置后，在创建会话或进入 Runtime 前调用 [validate](Self::validate)
/// 或 [time_policy_transaction](Self::time_policy_transaction)。值可独立拷贝，不借用活动时钟。
///
/// # 责任边界
///
/// 版本必须等于 [VERSION](Self::VERSION)，预算必须非零，底层 [TimePolicy] 必须满足值域约束。
/// 构造不选择预设，校验不要求自定义值与角色的内建预设相同。
/// 转换后的低层事务只携带时钟值；角色、版本和预算仍由宿主分别保留与使用。
/// Versioned product-time policy contract for runtime time and fixed-step budgeting.
///
/// `TimePolicy` owns the clock values. This type adds the profile-selected fixed-step
/// execution budget and the schema version needed to identify the configuration at a
/// BuildSet, replay, or diagnostics boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProductTimePolicy {
    version: u16,
    profile: ProductTimeProfile,
    time_policy: TimePolicy,
    max_fixed_steps_per_frame: u32,
}

impl ProductTimePolicy {
    /// 当前产品时间配置的契约版本；[validate](Self::validate) 仅接受完全相等的版本值。
    pub const VERSION: u16 = 1;

    /// 构造待验证的产品时间配置。
    ///
    /// # 参数
    ///
    /// - `version`：待检查的契约版本。
    /// - `profile`：产品角色标识；不会据此自动选择其余参数。
    /// - `time_policy`：虚拟增量、倍率与固定步长的低层值。
    /// - `max_fixed_steps_per_frame`：宿主为每个外帧选择的固定步提交预算。
    ///
    /// # 返回与约束
    ///
    /// 仅保存参数，不校验版本、预算或时钟值；需后续 [validate](Self::validate)。
    /// 返回值不持有 Runtime 或 World，构造成功不能证明配置已被采纳。
    pub const fn new(
        version: u16,
        profile: ProductTimeProfile,
        time_policy: TimePolicy,
        max_fixed_steps_per_frame: u32,
    ) -> Self {
        Self {
            version,
            profile,
            time_policy,
            max_fixed_steps_per_frame,
        }
    }

    pub const fn version(self) -> u16 {
        self.version
    }

    pub const fn profile(self) -> ProductTimeProfile {
        self.profile
    }

    pub const fn time_policy(self) -> TimePolicy {
        self.time_policy
    }

    pub const fn max_fixed_steps_per_frame(self) -> u32 {
        self.max_fixed_steps_per_frame
    }

    /// 检查产品时间契约的版本、预算及底层时钟值。
    ///
    /// # 返回与错误
    ///
    /// 成功返回 `()`；依次拒绝不等于 [VERSION](Self::VERSION) 的版本、零固定步预算，
    /// 然后传播 [TimePolicy::validate] 的值域错误。
    /// 不会按角色重新生成配置，也不检查某个 World 的活动步或待偿欠债；全程不应用配置。
    pub fn validate(self) -> Result<(), ProductTimePolicyError> {
        if self.version != Self::VERSION {
            return Err(ProductTimePolicyError::UnsupportedVersion {
                requested: self.version,
            });
        }
        if self.max_fixed_steps_per_frame == 0 {
            return Err(ProductTimePolicyError::MaxFixedStepsPerFrameZero);
        }
        self.time_policy.validate()?;
        Ok(())
    }

    /// 验证产品配置并提取底层时钟策略请求。
    ///
    /// # 返回与错误
    ///
    /// 成功返回只携带 [TimePolicy] 的 [TimePolicyTransaction]；失败返回
    /// [validate](Self::validate) 的产品级拒绝，不产生部分提交。
    ///
    /// # 宿主责任
    ///
    /// 角色、版本和每帧预算不会进入返回事务。调用者应保留原产品配置，另行使用预算，
    /// 并通过 Runtime 或 World 的事务入口完成应用；本转换不取得状态所有权或生成提交回执。
    /// Returns the already-validated low-level transaction that clock authority accepts.
    pub fn time_policy_transaction(self) -> Result<TimePolicyTransaction, ProductTimePolicyError> {
        self.validate()?;
        Ok(TimePolicyTransaction::new(self.time_policy))
    }
}

/// 产品版本、预算或底层时间策略的拒绝原因。
///
/// 版本拒绝保留提交的版本号，零预算拒绝要求选择非零预算。
/// `TimePolicy` 分支保存原 [TimePolicyError]；生成的转换也可包装由所有者返回的状态冲突。
/// 因此单凭此枚举无法判断失败发生在产品验证还是后续应用阶段，宿主应保留调用上下文。
/// Typed rejection for a product-level time-policy configuration.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProductTimePolicyError {
    #[error("unsupported product time-policy version {requested}")]
    UnsupportedVersion { requested: u16 },
    #[error("product time-policy max fixed steps per frame must be non-zero")]
    MaxFixedStepsPerFrameZero,
    #[error(transparent)]
    TimePolicy(#[from] TimePolicyError),
}

#[cfg(test)]
#[path = "tests/product_policy.rs"]
mod tests;
