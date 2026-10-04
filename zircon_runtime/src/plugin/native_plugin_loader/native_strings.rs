//! 原生 ABI 文本转换边界；复制 C 字符串并保留 TOML 解析错误的类型来源。

use std::collections::HashSet;
use std::ffi::{c_char, CStr};

use crate::plugin::PluginPackageManifest;

pub(super) type NativeStringResult<T> = std::result::Result<T, NativeStringError>;

#[derive(Debug)]
pub(super) enum NativeStringError {
    MissingRequiredField {
        field_name: String,
    },
    InvalidPackageManifest {
        message: String,
        source: toml::de::Error,
    },
}

impl std::fmt::Display for NativeStringError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequiredField { field_name } => write!(
                formatter,
                "native plugin descriptor field {field_name} is null or invalid"
            ),
            Self::InvalidPackageManifest { message, source } => {
                write!(formatter, "{message}: {source}")
            }
        }
    }
}

impl std::error::Error for NativeStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingRequiredField { .. } => None,
            Self::InvalidPackageManifest { source, .. } => Some(source),
        }
    }
}

/// 构造供动态库查找的 NUL 结尾符号名；入口名应不含内嵌 NUL。
pub(super) fn native_symbol_name(symbol_name: &str) -> Vec<u8> {
    let mut bytes = symbol_name.as_bytes().to_vec();
    if !bytes.ends_with(&[0]) {
        bytes.push(0);
    }
    bytes
}

/// 必填字段缺失时保留字段名，让上层归因到对应探测或入口阶段。
///
/// # Safety
/// 非空指针须在读取期间指向可读、以 NUL 结束且未被并发改写的 C 字符串。
pub(super) unsafe fn read_required_c_string(
    value: *const c_char,
    field_name: &str,
) -> NativeStringResult<String> {
    read_optional_c_string(value).ok_or_else(|| NativeStringError::MissingRequiredField {
        field_name: field_name.to_string(),
    })
}

/// 在当前调用内复制可选文本；空指针及非法 UTF-8 当前均视为缺省。
///
/// # Safety
/// 非空指针须在读取期间指向可读、以 NUL 结束且未被并发改写的 C 字符串。
pub(super) unsafe fn read_optional_c_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    // TODO: [CR-PLUGIN-NATIVE-0601] 确认可选字段的非法 UTF-8 是否可与未提供等价；
    // 描述符入口名和清单也经此处，当前会丢失坏文本的原始诊断。
    // SAFETY: 非空指针的有效性及 NUL 终止性由调用方 ABI 前提提供；本次只复制文本。
    CStr::from_ptr(value).to_str().ok().map(str::to_string)
}

/// 空文本代表未携带内嵌清单；非空 TOML 错误留给加载阶段报告。
pub(super) fn package_manifest_from_toml(
    manifest_toml: &str,
    invalid_message: &str,
) -> NativeStringResult<Option<PluginPackageManifest>> {
    if manifest_toml.trim().is_empty() {
        return Ok(None);
    }
    toml::from_str::<PluginPackageManifest>(manifest_toml)
        .map(Some)
        .map_err(|source| NativeStringError::InvalidPackageManifest {
            message: invalid_message.to_string(),
            source,
        })
}

/// 统一解析能力与诊断标签列表，并按首次出现顺序去重。
pub(super) fn parse_native_string_list(value: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for entry in value
        .split(|character| matches!(character, '\n' | ',' | ';'))
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        if seen.insert(entry) {
            entries.push(entry.to_string());
        }
    }
    entries
}

#[cfg(test)]
#[path = "tests/native_strings.rs"]
mod tests;

#[cfg(test)]
#[path = "native_strings/tests/string_list_dedup_tests.rs"]
mod string_list_dedup_tests;
