//! 把 LibraryEmbed 或 NativeDynamic 的编译宿主需求写成可审核的计划；Python CompileHost 消费命令与链接 crate 元数据。
use serde::{Deserialize, Serialize};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::{
    core::framework::project::ExportBuildMode, core::framework::project::ExportPackagingStrategy,
};

use super::{ExportBuildPlan, ExportLinkedRuntimeCrate, ExportRuntimeCrateRegistrationKind};

const MANIFEST_PATH: &str = "Cargo.toml";
const TARGET_DIR: &str = "stages/compile_host/target";
const BASE_COMMAND_ARGUMENT_COUNT: usize = 13;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Validate 报告交付给 CompileHost 的声明；路径以该 stage 约定的工作目录解析。
pub struct LibraryEmbedCompileHostPlan {
    pub package: String,
    pub binary: String,
    pub manifest_path: String,
    pub target_dir: String,
    pub cargo_profile: String,
    pub release: bool,
    pub app_features: Vec<String>,
    pub runtime_features: Vec<String>,
    pub expected_runtime_plugins: Vec<String>,
    pub linked_runtime_crates: Vec<LibraryEmbedLinkedRuntimeCrate>,
    pub command: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 编译宿主需链接的 crate 与注册角色；外部特征 provider 身份随计划保留。
pub struct LibraryEmbedLinkedRuntimeCrate {
    pub crate_name: String,
    pub path: String,
    pub registration_kind: LibraryEmbedCompileHostTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_package_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 区分整包注册和特征注册，后续宿主生成不能把两种 provider 当成同一入口。
pub enum LibraryEmbedCompileHostTarget {
    RuntimePlugin,
    RuntimeFeature,
}

impl ExportBuildPlan {
    pub(crate) fn set_library_embed_compile_host_plan(
        &mut self,
        plan: Option<LibraryEmbedCompileHostPlan>,
    ) {
        self.library_embed_compile_host = plan;
    }
}

impl LibraryEmbedCompileHostPlan {
    /// 提供与计划命令一致的宿主二进制选择，供导出工具核对目标模式。
    pub fn binary_for_target_mode(target_mode: RuntimeTargetMode) -> &'static str {
        target_for_mode(target_mode).binary
    }

    /// 提供 Cargo 产物目录约定，供编译及打包阶段寻找相应构建模式的文件。
    pub fn cargo_profile_for_build_mode(build_mode: ExportBuildMode) -> &'static str {
        match build_mode {
            ExportBuildMode::Debug => "debug",
            ExportBuildMode::Release => "release",
        }
    }
}

/// 不执行 Cargo；仅为 CompileHost stage 提供与目标模式、构建模式一致的命令和预期插件集合。
pub(super) fn library_embed_compile_host_plan(
    plan: &ExportBuildPlan,
    linked_runtime_crates: &[ExportLinkedRuntimeCrate],
) -> Option<LibraryEmbedCompileHostPlan> {
    if !compile_host_strategy_enabled(plan) {
        return None;
    }

    let target = target_for_mode(plan.profile.target_mode);
    let cargo_profile =
        LibraryEmbedCompileHostPlan::cargo_profile_for_build_mode(plan.profile.build_mode);
    let target_dir = TARGET_DIR.to_string();
    let manifest_path = MANIFEST_PATH.to_string();
    let release = plan.profile.build_mode == ExportBuildMode::Release;
    let command =
        library_embed_command(&target, manifest_path.clone(), target_dir.clone(), release);

    Some(LibraryEmbedCompileHostPlan {
        package: target.package.to_string(),
        binary: target.binary.to_string(),
        manifest_path,
        target_dir,
        cargo_profile: cargo_profile.to_string(),
        release,
        app_features: vec![target.app_feature.to_string()],
        runtime_features: vec![target.runtime_feature.to_string()],
        expected_runtime_plugins: plan.enabled_runtime_plugins.clone(),
        linked_runtime_crates: linked_runtime_crates
            .iter()
            .map(LibraryEmbedLinkedRuntimeCrate::from_linked_crate)
            .collect(),
        command,
    })
}

fn library_embed_command(
    target: &LibraryEmbedTarget,
    manifest_path: String,
    target_dir: String,
    release: bool,
) -> Vec<String> {
    let mut command = Vec::with_capacity(BASE_COMMAND_ARGUMENT_COUNT + usize::from(release));
    command.extend([
        "cargo".to_string(),
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path,
        "-p".to_string(),
        target.package.to_string(),
        "--bin".to_string(),
        target.binary.to_string(),
        "--no-default-features".to_string(),
        "--features".to_string(),
        target.app_feature.to_string(),
        "--target-dir".to_string(),
        target_dir,
    ]);
    if release {
        command.push("--release".to_string());
    }
    command
}

fn compile_host_strategy_enabled(plan: &ExportBuildPlan) -> bool {
    plan.profile
        .uses_strategy(ExportPackagingStrategy::LibraryEmbed)
        || plan
            .profile
            .uses_strategy(ExportPackagingStrategy::NativeDynamic)
}

impl LibraryEmbedLinkedRuntimeCrate {
    fn from_linked_crate(linked_crate: &ExportLinkedRuntimeCrate) -> Self {
        Self {
            crate_name: linked_crate.crate_name.clone(),
            path: linked_crate.path.clone(),
            registration_kind: match linked_crate.registration_kind {
                ExportRuntimeCrateRegistrationKind::RuntimePlugin => {
                    LibraryEmbedCompileHostTarget::RuntimePlugin
                }
                ExportRuntimeCrateRegistrationKind::RuntimeFeature => {
                    LibraryEmbedCompileHostTarget::RuntimeFeature
                }
            },
            provider_package_id: linked_crate.provider_package_id.clone(),
        }
    }
}

struct LibraryEmbedTarget {
    package: &'static str,
    binary: &'static str,
    app_feature: &'static str,
    runtime_feature: &'static str,
}

fn target_for_mode(target_mode: RuntimeTargetMode) -> LibraryEmbedTarget {
    match target_mode {
        RuntimeTargetMode::ClientRuntime => LibraryEmbedTarget {
            package: "zircon_app",
            binary: "zircon_runtime",
            app_feature: "target-client",
            runtime_feature: "target-client",
        },
        RuntimeTargetMode::ServerRuntime => LibraryEmbedTarget {
            package: "zircon_app",
            binary: "zircon_runtime",
            app_feature: "target-server",
            runtime_feature: "target-server",
        },
        RuntimeTargetMode::EditorHost => LibraryEmbedTarget {
            package: "zircon_app",
            binary: "zircon_editor",
            app_feature: "target-editor-host",
            runtime_feature: "target-editor-host",
        },
    }
}

#[cfg(test)]
#[path = "tests/library_embed_compile_plan.rs"]
mod tests;
