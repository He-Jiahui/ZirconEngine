//! 作者编写的头像掩码 TOML 声明；编辑器视图仅投影规则，运行时权重仍须与目标骨架编译。
use serde::{Deserialize, Serialize};
use zircon_runtime::core::framework::animation::AnimationAvatarMask;
use zircon_runtime::core::math::Real;

use super::AvatarMaskError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AvatarMaskAsset {
    pub id: String,
    #[serde(default)]
    pub default_weight: Real,
    #[serde(default)]
    pub rules: Vec<AvatarMaskRule>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AvatarMaskRule {
    pub target: String,
    pub weight: Real,
    #[serde(default = "default_inherit")]
    pub inherit: bool,
    #[serde(default)]
    pub boundary_weights: Vec<Real>,
}

impl AvatarMaskAsset {
    pub fn from_toml(source: &str) -> Result<Self, AvatarMaskError> {
        toml::from_str(source).map_err(|error| AvatarMaskError::Parse {
            message: error.to_string(),
        })
    }

    /// 为作者界面提供规则摘要；边界渐变和继承展开仍需在具体骨架上编译。
    pub fn editor_view(&self) -> AnimationAvatarMask {
        AnimationAvatarMask {
            id: self.id.clone(),
            included_target_ids: self.rules.iter().map(|rule| rule.target.clone()).collect(),
            excluded_target_ids: self
                .rules
                .iter()
                .filter(|rule| rule.weight == 0.0)
                .map(|rule| rule.target.clone())
                .collect(),
            weight: self.default_weight,
        }
    }
}

fn default_inherit() -> bool {
    true
}
