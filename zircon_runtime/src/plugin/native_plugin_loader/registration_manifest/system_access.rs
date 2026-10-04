//! 将原生注册清单中的访问声明编译为 ECS 调度冲突集。
//! 清单解析先限制声明形式，注册前的 authority 校验所有权与授权，World 建立时才解析稳定 ID。

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::scene::ecs::{QueryAccess, SceneSystemThreadAffinity, SystemParamAccess};
use crate::scene::World;

mod authority;
mod error;

pub(in crate::plugin::native_plugin_loader) use authority::NativeSystemAccessAuthority;
pub(in crate::plugin::native_plugin_loader) use error::{
    NativeSystemAccessAuthorityError, NativeSystemAccessContractError,
    NativeSystemAccessResolveError,
};

pub(in crate::plugin::native_plugin_loader) const NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY: &str =
    "runtime.native.system.worker_safe";

/// 原生系统声明的线程契约；工作线程执行必须同时提供显式访问和宿主授予的能力。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::plugin::native_plugin_loader) enum NativePluginRegistrationThreadAffinity {
    #[default]
    MainThreadOnly,
    WorkerSafe,
}

impl NativePluginRegistrationThreadAffinity {
    const fn runtime_affinity(self) -> SceneSystemThreadAffinity {
        match self {
            Self::MainThreadOnly => SceneSystemThreadAffinity::MainThreadOnly,
            Self::WorkerSafe => SceneSystemThreadAffinity::WorkerSafe,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessDomain {
    Component,
    Resource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) struct NativeSystemAccessDeclaration {
    pub(in crate::plugin::native_plugin_loader) mode: NativeSystemAccessMode,
    pub(in crate::plugin::native_plugin_loader) domain: NativeSystemAccessDomain,
    pub(in crate::plugin::native_plugin_loader) stable_id: String,
}

/// 注册与调度之间传递的访问计划；稳定 ID 保留到具体 World 建立时再解析。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) struct NativeSystemAccessPlan {
    affinity: SceneSystemThreadAffinity,
    declarations: Vec<NativeSystemAccessDeclaration>,
    conservative_world_access: bool,
}

impl NativeSystemAccessPlan {
    /// 解析主线程保守访问或显式组件/资源声明；工作线程缺少显式访问和声明能力时拒绝注册。
    pub(in crate::plugin::native_plugin_loader) fn from_manifest(
        thread_affinity: NativePluginRegistrationThreadAffinity,
        raw_access: &[String],
        capabilities: &[String],
    ) -> Result<Self, NativeSystemAccessContractError> {
        let affinity = thread_affinity.runtime_affinity();
        // 旧清单未声明细粒度访问时按独占 World 参与调度，不能把它当作可并行系统。
        if raw_access.is_empty() || raw_access == ["write:world"] {
            if affinity == SceneSystemThreadAffinity::WorkerSafe {
                return Err(NativeSystemAccessContractError::WorkerSafeRequiresExplicitAccess);
            }
            return Ok(Self {
                affinity,
                declarations: Vec::new(),
                conservative_world_access: true,
            });
        }
        if raw_access.iter().any(|access| access == "write:world") {
            return Err(NativeSystemAccessContractError::WorldAccessMustBeExclusive);
        }
        if affinity == SceneSystemThreadAffinity::WorkerSafe
            && !capabilities
                .iter()
                .any(|capability| capability == NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY)
        {
            return Err(NativeSystemAccessContractError::MissingWorkerSafeCapability);
        }

        let mut declarations = Vec::with_capacity(raw_access.len());
        let mut declared = BTreeMap::new();
        for raw in raw_access {
            let declaration = parse_access_declaration(raw)?;
            let key = (declaration.domain, declaration.stable_id.clone());
            if let Some(existing) = declared.insert(key, declaration.mode) {
                if existing == declaration.mode {
                    return Err(NativeSystemAccessContractError::DuplicateAccess {
                        declaration: raw.clone(),
                    });
                }
                return Err(NativeSystemAccessContractError::ConflictingAccess {
                    domain: declaration.domain,
                    stable_id: declaration.stable_id,
                });
            }
            declarations.push(declaration);
        }
        declarations.sort_by(|left, right| {
            left.domain
                .cmp(&right.domain)
                .then(left.stable_id.cmp(&right.stable_id))
                .then(left.mode.cmp(&right.mode))
        });
        Ok(Self {
            affinity,
            declarations,
            conservative_world_access: false,
        })
    }

    pub(in crate::plugin::native_plugin_loader) const fn affinity(
        &self,
    ) -> SceneSystemThreadAffinity {
        self.affinity
    }

    pub(in crate::plugin::native_plugin_loader) fn declarations(
        &self,
    ) -> &[NativeSystemAccessDeclaration] {
        &self.declarations
    }

    pub(in crate::plugin::native_plugin_loader) const fn has_conservative_world_access(
        &self,
    ) -> bool {
        self.conservative_world_access
    }

    /// 注册闭包在具体 World 初始化期间把稳定 ID 解析为调度 ID；未安装的组件必须失败。
    /// 资源 ID 是宿主状态的冲突身份，不表示此处在 World 中插入了资源值。
    pub(in crate::plugin::native_plugin_loader) fn compile(
        &self,
        world: &mut World,
    ) -> Result<SystemParamAccess, NativeSystemAccessResolveError> {
        let mut access = SystemParamAccess::default();
        if self.conservative_world_access {
            access.add_conservative_world_access();
            return Ok(access);
        }

        let mut component_access = QueryAccess::default();
        for declaration in &self.declarations {
            match declaration.domain {
                NativeSystemAccessDomain::Component => {
                    let component_id = world
                        .registered_dynamic_component_id(&declaration.stable_id)
                        .ok_or_else(|| NativeSystemAccessResolveError::UnknownComponent {
                            stable_id: declaration.stable_id.clone(),
                        })?;
                    let result = match declaration.mode {
                        NativeSystemAccessMode::Read => component_access.add_read(component_id),
                        NativeSystemAccessMode::Write => component_access.add_write(component_id),
                    };
                    result.map_err(|error| NativeSystemAccessResolveError::ConflictingAccess {
                        stable_id: declaration.stable_id.clone(),
                        message: error.to_string(),
                    })?;
                }
                NativeSystemAccessDomain::Resource => {
                    let resource_id = world.external_resource_id(&declaration.stable_id);
                    let result = match declaration.mode {
                        NativeSystemAccessMode::Read => access.add_resource_read(resource_id),
                        NativeSystemAccessMode::Write => access.add_resource_write(resource_id),
                    };
                    result.map_err(|error| NativeSystemAccessResolveError::ConflictingAccess {
                        stable_id: declaration.stable_id.clone(),
                        message: error.to_string(),
                    })?;
                }
            }
        }
        access
            .add_query_access(&component_access)
            .map_err(|error| NativeSystemAccessResolveError::ConflictingAccess {
                stable_id: "component access set".to_string(),
                message: error.to_string(),
            })?;
        Ok(access)
    }
}

impl NativeSystemAccessDomain {
    const fn label(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::Resource => "resource",
        }
    }
}

impl NativeSystemAccessMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

fn parse_access_declaration(
    declaration: &str,
) -> Result<NativeSystemAccessDeclaration, NativeSystemAccessContractError> {
    let mut fields = declaration.splitn(3, ':');
    let mode = match fields.next() {
        Some("read") => NativeSystemAccessMode::Read,
        Some("write") => NativeSystemAccessMode::Write,
        _ => return invalid_declaration(declaration),
    };
    let domain = match fields.next() {
        Some("component") => NativeSystemAccessDomain::Component,
        Some("resource") => NativeSystemAccessDomain::Resource,
        _ => return invalid_declaration(declaration),
    };
    let Some(stable_id) = fields.next() else {
        return invalid_declaration(declaration);
    };
    if stable_id.is_empty()
        || stable_id.trim() != stable_id
        || stable_id.chars().any(char::is_whitespace)
    {
        return Err(NativeSystemAccessContractError::InvalidStableId {
            stable_id: stable_id.to_string(),
        });
    }
    Ok(NativeSystemAccessDeclaration {
        mode,
        domain,
        stable_id: stable_id.to_string(),
    })
}

fn invalid_declaration<T>(declaration: &str) -> Result<T, NativeSystemAccessContractError> {
    Err(NativeSystemAccessContractError::InvalidDeclaration {
        declaration: declaration.to_string(),
    })
}

#[cfg(test)]
#[path = "tests/system_access.rs"]
mod tests;
