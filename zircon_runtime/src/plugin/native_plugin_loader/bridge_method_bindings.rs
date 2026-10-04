//! 插件按名称导出桥回调，包清单决定宿主最终接口与方法槽位。
//! 安装时校验一一对应，调用时桥 scope 持有动态库代际与回调租约。

use std::collections::BTreeMap;

use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus};

use crate::plugin::{PluginInterfaceMethodManifest, PluginPackageManifest};

use super::abi_declarations::{NativePluginBridgeMethodCallV3, NativePluginBridgeMethodFnV3};

#[cfg(test)]
#[path = "bridge_method_bindings/tests/capacity_tests.rs"]
mod capacity_tests;

/// 槽位已解析的单次桥调用；负载和输出缓冲不可跨回调保存。
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeBridgeCall {
    pub interface_slot: u32,
    pub method_slot: u32,
    pub payload: ZrByteSlice,
    pub output: ZrByteBufferRef,
}

/// 区分进程内 Rust 回调与动态库 ABI 回调，后者需加载代际保活。
#[derive(Clone, Copy)]
pub struct NativeBridgeMethodFn {
    callable: NativeBridgeMethodCallable,
}

impl NativeBridgeMethodFn {
    pub const fn from_rust(method: fn(NativeBridgeCall) -> ZrStatus) -> Self {
        Self {
            callable: NativeBridgeMethodCallable::Rust(method),
        }
    }

    pub(super) const fn from_abi_v3(method: NativePluginBridgeMethodFnV3, user_data: u64) -> Self {
        Self {
            callable: NativeBridgeMethodCallable::AbiV3 { method, user_data },
        }
    }

    /// 桥 scope 在接口启用、槽位命中并取得动态库回调租约后调用。
    pub(super) fn call(self, call: NativeBridgeCall) -> ZrStatus {
        match self.callable {
            NativeBridgeMethodCallable::Rust(method) => method(call),
            // SAFETY: 生产分派由桥 scope 取得加载代际租约；本模块测试使用静态回调。
            // payload/output 在同步调用内有效，user_data 原样来自插件表。
            NativeBridgeMethodCallable::AbiV3 { method, user_data } => unsafe {
                method(NativePluginBridgeMethodCallV3 {
                    interface_slot: call.interface_slot,
                    method_slot: call.method_slot,
                    payload: call.payload,
                    output: call.output,
                    user_data,
                })
            },
        }
    }

    pub(super) const fn requires_loaded_generation_owner(self) -> bool {
        matches!(self.callable, NativeBridgeMethodCallable::AbiV3 { .. })
    }
}

impl std::fmt::Debug for NativeBridgeMethodFn {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.callable {
            NativeBridgeMethodCallable::Rust(method) => formatter
                .debug_tuple("NativeBridgeMethodFn::Rust")
                .field(&(method as usize))
                .finish(),
            NativeBridgeMethodCallable::AbiV3 { method, user_data } => formatter
                .debug_struct("NativeBridgeMethodFn::AbiV3")
                .field("method", &(method as usize))
                .field("user_data", &user_data)
                .finish(),
        }
    }
}

impl From<fn(NativeBridgeCall) -> ZrStatus> for NativeBridgeMethodFn {
    fn from(method: fn(NativeBridgeCall) -> ZrStatus) -> Self {
        Self::from_rust(method)
    }
}

#[derive(Clone, Copy)]
enum NativeBridgeMethodCallable {
    Rust(fn(NativeBridgeCall) -> ZrStatus),
    AbiV3 {
        method: NativePluginBridgeMethodFnV3,
        user_data: u64,
    },
}

/// 已从包清单取得槽位的安装项，随后与冻结接口表组合。
#[derive(Clone)]
pub struct NativeBridgeMethodDescriptor {
    interface_id: String,
    method_slot: u32,
    method: NativeBridgeMethodFn,
}

impl NativeBridgeMethodDescriptor {
    pub fn new(
        interface_id: impl Into<String>,
        method_slot: u32,
        method: impl Into<NativeBridgeMethodFn>,
    ) -> Self {
        Self {
            interface_id: interface_id.into(),
            method_slot,
            method: method.into(),
        }
    }

    pub fn interface_id(&self) -> &str {
        &self.interface_id
    }

    pub const fn method_slot(&self) -> u32 {
        self.method_slot
    }

    pub const fn method(&self) -> NativeBridgeMethodFn {
        self.method
    }

    pub fn from_manifest_method(
        interface_id: &str,
        method: &PluginInterfaceMethodManifest,
        method_fn: impl Into<NativeBridgeMethodFn>,
    ) -> Self {
        Self::new(interface_id, method.method_slot, method_fn)
    }
}

/// 插件提交的接口 ID 和方法名候选；此时尚未获清单槽位。
#[derive(Clone, Debug)]
pub struct NativeBridgeMethodBinding {
    pub(super) interface_id: String,
    pub(super) method_name: String,
    pub(super) method: NativeBridgeMethodFn,
}

impl NativeBridgeMethodBinding {
    pub fn new(
        interface_id: impl Into<String>,
        method_name: impl Into<String>,
        method: impl Into<NativeBridgeMethodFn>,
    ) -> Self {
        Self {
            interface_id: interface_id.into(),
            method_name: method_name.into(),
            method: method.into(),
        }
    }

    pub fn interface_id(&self) -> &str {
        &self.interface_id
    }

    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    pub(super) const fn requires_loaded_generation_owner(&self) -> bool {
        self.method.requires_loaded_generation_owner()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeBridgeMethodManifestError {
    DuplicateBinding {
        interface_id: String,
        method_name: String,
    },
    MissingBinding {
        interface_id: String,
        method_name: String,
    },
    UnknownBinding {
        interface_id: String,
        method_name: String,
    },
}

impl std::fmt::Display for NativeBridgeMethodManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateBinding {
                interface_id,
                method_name,
            } => write!(
                formatter,
                "duplicate native bridge method binding `{interface_id}.{method_name}`"
            ),
            Self::MissingBinding {
                interface_id,
                method_name,
            } => write!(
                formatter,
                "native bridge method `{interface_id}.{method_name}` is declared but has no binding"
            ),
            Self::UnknownBinding {
                interface_id,
                method_name,
            } => write!(
                formatter,
                "native bridge method binding `{interface_id}.{method_name}` is not declared by the package manifest"
            ),
        }
    }
}

impl std::error::Error for NativeBridgeMethodManifestError {}

/// 安装前按清单匹配每个绑定，拒绝重复、缺失和未声明方法。
/// 返回顺序与槽位来自清单，避免插件表自行决定宿主分派编号。
pub fn native_bridge_method_descriptors_from_manifest(
    manifest: &PluginPackageManifest,
    bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
) -> Result<Vec<NativeBridgeMethodDescriptor>, NativeBridgeMethodManifestError> {
    let descriptor_capacity = manifest.bridge_methods().count();
    let mut bindings_by_method = BTreeMap::new();
    for binding in bindings {
        let key = (binding.interface_id, binding.method_name);
        if bindings_by_method
            .insert(key.clone(), binding.method)
            .is_some()
        {
            return Err(NativeBridgeMethodManifestError::DuplicateBinding {
                interface_id: key.0,
                method_name: key.1,
            });
        }
    }

    let mut descriptors = Vec::with_capacity(descriptor_capacity);
    for (interface, method) in manifest.bridge_methods() {
        let key = (interface.id.clone(), method.name.clone());
        let Some(method_fn) = bindings_by_method.remove(&key) else {
            return Err(NativeBridgeMethodManifestError::MissingBinding {
                interface_id: key.0,
                method_name: key.1,
            });
        };
        descriptors.push(NativeBridgeMethodDescriptor::from_manifest_method(
            &interface.id,
            method,
            method_fn,
        ));
    }

    if let Some(((interface_id, method_name), _)) = bindings_by_method.into_iter().next() {
        return Err(NativeBridgeMethodManifestError::UnknownBinding {
            interface_id,
            method_name,
        });
    }

    Ok(descriptors)
}
