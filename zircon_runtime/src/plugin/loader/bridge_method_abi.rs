//! 入口桥方法表的原生转换边界；这里检查表形状，安装阶段再核对包清单。

use super::abi_declarations::{
    NativePluginBridgeMethodTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
};
use super::bridge_method_bindings::{NativeBridgeMethodBinding, NativeBridgeMethodFn};
use super::native_strings::read_required_c_string;

pub(super) type NativeBridgeMethodAbiResult<T> = std::result::Result<T, NativeBridgeMethodAbiError>;

#[derive(Debug)]
pub(super) enum NativeBridgeMethodAbiError {
    UnsupportedTableAbiVersion {
        actual: u32,
        expected: u32,
    },
    MissingMethodsPointerWithCount {
        method_count: usize,
    },
    InvalidRequiredField {
        field_name: &'static str,
        source: String,
    },
    MissingCallback {
        interface_id: String,
        method_name: String,
    },
}

impl std::fmt::Display for NativeBridgeMethodAbiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedTableAbiVersion { actual, expected } => write!(
                formatter,
                "unsupported native bridge method table ABI version {actual}; expected {expected}"
            ),
            Self::MissingMethodsPointerWithCount { .. } => formatter.write_str(
                "native bridge method table declared methods but methods pointer was null",
            ),
            Self::InvalidRequiredField { source, .. } => formatter.write_str(source),
            Self::MissingCallback {
                interface_id,
                method_name,
            } => write!(
                formatter,
                "native bridge method `{interface_id}.{method_name}` declared no callback"
            ),
        }
    }
}

impl std::error::Error for NativeBridgeMethodAbiError {}

/// 将可选 v3 方法表转换为宿主绑定；空表表示未导出桥方法。
///
/// # Safety
/// 非空 table 与 methods 在转换期间须有效、对齐、可读且不被改写，名称是有效 C 字符串；
/// 回调地址在后续动态库代际存活期间保持有效。
pub(super) unsafe fn bridge_method_bindings_from_abi_v3(
    table: *const NativePluginBridgeMethodTableV3,
) -> NativeBridgeMethodAbiResult<Vec<NativeBridgeMethodBinding>> {
    if table.is_null() {
        return Ok(Vec::new());
    }
    // SAFETY: 非空 table 是当前已加载插件提供的可读 v3 表，调用方维持其借用期。
    let table = unsafe { &*table };
    if table.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        return Err(NativeBridgeMethodAbiError::UnsupportedTableAbiVersion {
            actual: table.abi_version,
            expected: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        });
    }
    if table.methods.is_null() {
        if table.method_count == 0 {
            return Ok(Vec::new());
        }
        return Err(NativeBridgeMethodAbiError::MissingMethodsPointerWithCount {
            method_count: table.method_count,
        });
    }

    // SAFETY: 插件提供的非空数组指针与数量须一致；空指针加非零数量已拒绝。
    let methods = unsafe { std::slice::from_raw_parts(table.methods, table.method_count) };
    let mut bindings = Vec::with_capacity(methods.len());
    for method in methods {
        let interface_id =
            unsafe { required_bridge_method_field(method.interface_id, "interface_id")? };
        let method_name =
            unsafe { required_bridge_method_field(method.method_name, "method_name")? };
        let Some(callback) = method.method else {
            return Err(NativeBridgeMethodAbiError::MissingCallback {
                interface_id,
                method_name,
            });
        };
        bindings.push(NativeBridgeMethodBinding::new(
            interface_id,
            method_name,
            NativeBridgeMethodFn::from_abi_v3(callback, method.user_data),
        ));
    }
    Ok(bindings)
}

unsafe fn required_bridge_method_field(
    value: *const std::ffi::c_char,
    field_name: &'static str,
) -> NativeBridgeMethodAbiResult<String> {
    unsafe { read_required_c_string(value, field_name) }.map_err(|source| {
        NativeBridgeMethodAbiError::InvalidRequiredField {
            field_name,
            source: source.to_string(),
        }
    })
}

#[cfg(test)]
#[path = "tests/bridge_method_abi.rs"]
mod tests;
