use std::fs;
use std::path::Path;

/// 源码检出的注册和构建前置状态；仅确认仓库形态，不证明工具链或产物可用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceEngineValidation {
    Valid,
    MissingRoot,
    MissingWorkspaceManifest,
    MissingRuntimeWorkspaceMember,
    MissingBuildTool,
}

impl SourceEngineValidation {
    pub fn summary(self) -> &'static str {
        match self {
            Self::Valid => "Source engine is ready",
            Self::MissingRoot => "Source checkout directory is missing",
            Self::MissingWorkspaceManifest => "Source checkout is missing Cargo.toml",
            Self::MissingRuntimeWorkspaceMember => {
                "Source checkout workspace is missing zircon_runtime member"
            }
            Self::MissingBuildTool => "Source checkout is missing tools/zircon_build.py",
        }
    }

    pub fn recovery_hint(self) -> &'static str {
        match self {
            Self::Valid => "No recovery action is required",
            Self::MissingRoot => {
                "Locate an existing ZirconEngine checkout or update Settings > Source Checkout"
            }
            Self::MissingWorkspaceManifest => {
                "Select the ZirconEngine repository root that contains the workspace Cargo.toml"
            }
            Self::MissingRuntimeWorkspaceMember => {
                "Select the ZirconEngine repository root whose Cargo workspace includes zircon_runtime"
            }
            Self::MissingBuildTool => {
                "Select a complete ZirconEngine checkout with tools/zircon_build.py before building"
            }
        }
    }
}

/// 设置登记与执行构建前共用的轻量门禁；调用方将状态映射为可恢复的 Hub 提示。
pub fn validate_source_engine(path: impl AsRef<Path>) -> SourceEngineValidation {
    let path = path.as_ref();
    if !path.is_dir() {
        return SourceEngineValidation::MissingRoot;
    }
    if !path.join("Cargo.toml").is_file() {
        return SourceEngineValidation::MissingWorkspaceManifest;
    }
    if !workspace_includes_runtime_member(&path.join("Cargo.toml")) {
        return SourceEngineValidation::MissingRuntimeWorkspaceMember;
    }
    if !path.join("tools").join("zircon_build.py").is_file() {
        return SourceEngineValidation::MissingBuildTool;
    }
    SourceEngineValidation::Valid
}

fn workspace_includes_runtime_member(manifest_path: &Path) -> bool {
    let Ok(text) = fs::read_to_string(manifest_path) else {
        return false;
    };
    let Ok(value) = toml::from_str::<toml::Value>(&text) else {
        return false;
    };
    value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(|members| members.as_array())
        .is_some_and(|members| {
            members
                .iter()
                .filter_map(|member| member.as_str())
                .any(member_path_references_runtime)
        })
}

fn member_path_references_runtime(member: &str) -> bool {
    let member = member.trim().replace('\\', "/");
    member == "zircon_runtime"
        || member.ends_with("/zircon_runtime")
        || member.contains("/zircon_runtime/")
}

#[cfg(test)]
#[path = "tests/validation.rs"]
mod tests;
