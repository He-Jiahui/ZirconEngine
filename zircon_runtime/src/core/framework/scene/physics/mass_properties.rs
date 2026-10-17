use serde::{Deserialize, Serialize};

use crate::core::math::Real;

/// 场景资产保留质量模式；物理插件创建刚体前验证，并选择形状推导或显式惯量。
///
/// serde 以 `mode` 内部标记保存蛇形小写模式，回读不会自动调用 `is_valid`。
/// 本声明不持有模拟对象，形状支持和派生质量仍由插件创建链处理。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum PhysicsMassProperties {
    /// 使用外部刚体质量声明，默认选择此模式且不指定惯性张量。
    Explicit {
        /// 可选惯性张量，缺失时由插件按形状处理，指定时仍受后端支持范围约束。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inertia_tensor: Option<[[Real; 3]; 3]>,
    },
    /// 请求由碰撞形状体积和密度派生质量，形状是否受支持由插件判断。
    AutoFromShape {
        /// 密度输入，本类型的布尔预检要求其有限且严格为正。
        density: Real,
    },
}

impl Default for PhysicsMassProperties {
    fn default() -> Self {
        Self::Explicit {
            inertia_tensor: None,
        }
    }
}

impl PhysicsMassProperties {
    /// 供刚体接纳路径使用；显式张量必须是有限、近似对称且正定的物理惯量。
    ///
    /// # 返回值与约束
    /// 消费可复制的声明并返回本模块的数值预检结果，不会修改场景或创建刚体。
    /// 自动模式检查密度，未指定张量的显式模式直接通过，指定张量时检查有限值、对称误差及顺序主子式。
    /// 本方法不检查独立的刚体质量、形状支持或派生质量/密度，调用方仍须遵守插件创建约束。
    pub fn is_valid(self) -> bool {
        match self {
            Self::AutoFromShape { density } => density.is_finite() && density > 0.0,
            Self::Explicit {
                inertia_tensor: None,
            } => true,
            Self::Explicit {
                inertia_tensor: Some(tensor),
            } => inertia_tensor_is_positive_definite(tensor),
        }
    }
}

/// 按绝对对称误差及三个顺序主子式判定输入，所有运算均使用 `Real`，没有额外精度提升。
fn inertia_tensor_is_positive_definite(tensor: [[Real; 3]; 3]) -> bool {
    const SYMMETRY_EPSILON: Real = 1.0e-5;
    if tensor.iter().flatten().any(|value| !value.is_finite()) {
        return false;
    }
    if (tensor[0][1] - tensor[1][0]).abs() > SYMMETRY_EPSILON
        || (tensor[0][2] - tensor[2][0]).abs() > SYMMETRY_EPSILON
        || (tensor[1][2] - tensor[2][1]).abs() > SYMMETRY_EPSILON
    {
        return false;
    }
    let leading_minor_2 = tensor[0][0] * tensor[1][1] - tensor[0][1] * tensor[1][0];
    let determinant = tensor[0][0] * (tensor[1][1] * tensor[2][2] - tensor[1][2] * tensor[2][1])
        - tensor[0][1] * (tensor[1][0] * tensor[2][2] - tensor[1][2] * tensor[2][0])
        + tensor[0][2] * (tensor[1][0] * tensor[2][1] - tensor[1][1] * tensor[2][0]);
    tensor[0][0] > 0.0 && leading_minor_2 > 0.0 && determinant > 0.0
}
