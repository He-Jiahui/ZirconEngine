use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

/// 可跨宿主传递的数值编辑约束；这里只校验有限边界、边界顺序和正步长，字段类型兼容性由运行时注册端复核。
///
/// # serde 线格式
///
/// 序列化对象包含 `min`、`max`、`step`、`precision` 四项；反序列化拒绝未知字段，并经 [`Self::new`] 复用边界与步长校验。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReflectNumericRange {
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
    precision: Option<u8>,
}

impl ReflectNumericRange {
    /// 创建数值编辑约束，并拒绝无法形成有效区间的边界和步长。
    ///
    /// # 参数
    /// `min`、`max` 和 `step` 为可选的有限数值；`precision` 原样保存为编辑提示。
    ///
    /// # 返回值
    /// 校验通过时返回约束值。
    ///
    /// # 错误
    /// 边界或步长非有限、上下界倒置、或步长非正时返回对应错误。字段值类型与约束兼容性由运行时注册校验。
    pub fn new(
        min: Option<f32>,
        max: Option<f32>,
        step: Option<f32>,
        precision: Option<u8>,
    ) -> Result<Self, ReflectNumericRangeError> {
        if min.is_some_and(|value| !value.is_finite()) {
            return Err(ReflectNumericRangeError::NonFiniteMin);
        }
        if max.is_some_and(|value| !value.is_finite()) {
            return Err(ReflectNumericRangeError::NonFiniteMax);
        }
        if step.is_some_and(|value| !value.is_finite()) {
            return Err(ReflectNumericRangeError::NonFiniteStep);
        }
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                return Err(ReflectNumericRangeError::Inverted { min, max });
            }
        }
        if let Some(step) = step.filter(|step| *step <= 0.0) {
            return Err(ReflectNumericRangeError::NonPositiveStep { step });
        }
        Ok(Self {
            min,
            max,
            step,
            precision,
        })
    }

    /// 返回可选的最小值；不存在下界时返回 `None`。
    pub const fn min(&self) -> Option<f32> {
        self.min
    }

    /// 返回可选的最大值；不存在上界时返回 `None`。
    pub const fn max(&self) -> Option<f32> {
        self.max
    }

    /// 返回可选的编辑步长；构造后若存在则为有限正数。
    pub const fn step(&self) -> Option<f32> {
        self.step
    }

    /// 返回可选的小数精度提示；此类型不对该提示应用额外范围校验。
    pub const fn precision(&self) -> Option<u8> {
        self.precision
    }
}

// 线格式也经 new 校验，避免反序列化绕过与本地构造相同的数值约束。
impl<'de> Deserialize<'de> for ReflectNumericRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ReflectNumericRangeWire::deserialize(deserializer)?;
        Self::new(wire.min, wire.max, wire.step, wire.precision).map_err(D::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
// 线格式仅用于反序列化，随后必须经 `ReflectNumericRange::new` 执行同一组约束检查。
struct ReflectNumericRangeWire {
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
    precision: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// 构造数值编辑约束时遇到的具体无效条件。
pub enum ReflectNumericRangeError {
    /// 最小值不是有限数。
    NonFiniteMin,
    /// 最大值不是有限数。
    NonFiniteMax,
    /// 步长不是有限数。
    NonFiniteStep,
    /// 最小值大于最大值；变体保留冲突的两个边界。
    Inverted { min: f32, max: f32 },
    /// 步长为零或负数；变体保留实际步长。
    NonPositiveStep { step: f32 },
}

impl fmt::Display for ReflectNumericRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteMin => formatter.write_str("reflection numeric range min is not finite"),
            Self::NonFiniteMax => formatter.write_str("reflection numeric range max is not finite"),
            Self::NonFiniteStep => {
                formatter.write_str("reflection numeric range step is not finite")
            }
            Self::Inverted { min, max } => write!(
                formatter,
                "reflection numeric range min {min} exceeds max {max}"
            ),
            Self::NonPositiveStep { step } => write!(
                formatter,
                "reflection numeric range step must be positive, found {step}"
            ),
        }
    }
}

impl std::error::Error for ReflectNumericRangeError {}
