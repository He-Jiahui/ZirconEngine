//! 向构建矩阵暴露各逻辑 profile 的 Cargo feature 要求，数据由统一 TOML 生成。
//! 此表描述编译组合；运行时模块与提供者可用性由装配描述符另行解释。

use crate::core::framework::project::RuntimeProfileId;

/// Compile-time Cargo feature requirements for one logical runtime profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeProfileFeaturePreset {
    pub id: RuntimeProfileId,
    pub name: &'static str,
    pub cargo_feature: &'static str,
    pub runtime_features: &'static [&'static str],
    pub app_features: &'static [&'static str],
}

// 构建脚本从同一预设源生成表；Cargo feature 一致性另由构建矩阵校验。
include!(concat!(
    env!("OUT_DIR"),
    "/runtime_profile_feature_presets_generated.rs"
));
