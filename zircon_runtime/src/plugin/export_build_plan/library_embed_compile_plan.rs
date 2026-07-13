use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::{
    core::framework::project::ExportBuildMode, core::framework::project::ExportPackagingStrategy,
};

use super::{ExportBuildPlan, ExportLinkedRuntimeCrate, ExportRuntimeCrateRegistrationKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct LibraryEmbedLinkedRuntimeCrate {
    pub crate_name: String,
    pub path: String,
    pub registration_kind: LibraryEmbedCompileHostTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_package_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    pub fn binary_for_target_mode(target_mode: RuntimeTargetMode) -> &'static str {
        target_for_mode(target_mode).binary
    }

    pub fn cargo_profile_for_build_mode(build_mode: ExportBuildMode) -> &'static str {
        match build_mode {
            ExportBuildMode::Debug => "debug",
            ExportBuildMode::Release => "release",
        }
    }
}

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
    let target_dir = Path::new("stages")
        .join("compile_host")
        .join("target")
        .display()
        .to_string()
        .replace('\\', "/");
    let manifest_path = PathBuf::from("Cargo.toml").display().to_string();
    let release = plan.profile.build_mode == ExportBuildMode::Release;
    let mut command = vec![
        "cargo".to_string(),
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.clone(),
        "-p".to_string(),
        target.package.to_string(),
        "--bin".to_string(),
        target.binary.to_string(),
        "--no-default-features".to_string(),
        "--features".to_string(),
        target.app_feature.to_string(),
        "--target-dir".to_string(),
        target_dir.clone(),
    ];
    if release {
        command.push("--release".to_string());
    }

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
