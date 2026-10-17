use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::CoreError;
use crate::core::ServiceKind;

/// 服务注册表的规范键，格式为 `<module namespace>.<kind>.<service>`。
///
/// 模块命名空间可分段；验证后的完整字符串同时决定相等、哈希与序列化身份。
/// 每个模块段及服务本地名都须非空且首尾无 Unicode 空白；不会清理内部空白或改变大小写。
/// 种类段只接受 Driver、Manager、Plugin；名称有效不等于服务已注册或当前准入开放。
#[derive(Clone, Debug)]
pub struct RegistryName {
    // `value` remains the equality, hash, borrow, and serde authority; the
    // cached fields only avoid re-parsing the validated registry string.
    value: Arc<str>,
    module_end: usize,
    service_start: usize,
    kind: ServiceKind,
}

impl RegistryName {
    /// 验证来自配置或常量的完整名称；非法名称返回错误，不进入注册表。
    /// 成功返回独立拥有规范字符串的名称；失败保留原输入于 InvalidRegistryName。
    /// 模块段可含多个点分段，但每段都须满足上述边界；不做注册表查找。
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let Some((module_end, kind_end)) = registry_separator_offsets(&value) else {
            return Err(CoreError::InvalidRegistryName(value));
        };
        if !is_canonical_module_namespace(&value[..module_end]) {
            return Err(CoreError::InvalidRegistryName(value));
        }

        let kind_start = module_end + 1;
        let kind_segment = &value.as_bytes()[kind_start..kind_end];
        let Some(kind) = ServiceKind::from_registry_segment_bytes(kind_segment) else {
            return Err(CoreError::InvalidRegistryName(value));
        };

        let service_start = kind_end + 1;
        let service = &value[service_start..];
        if !is_canonical_segment(service) {
            return Err(CoreError::InvalidRegistryName(value));
        }

        Ok(Self {
            value: Arc::from(value),
            module_end,
            service_start,
            kind,
        })
    }

    /// 为已知合法的内部片段构造名称；不合法片段会触发断言。
    /// `module` 可为多段命名空间，`service` 须为单个无点段，且各段非空、首尾无空白。
    /// 成功直接生成拥有式名称；非法片段触发断言。外部不可信字符串宜使用 [`Self::new`]。
    pub fn from_parts(module: &str, kind: ServiceKind, service: &str) -> Self {
        assert!(
            is_canonical_module_namespace(module),
            "registry name module namespaces must contain non-empty, trim-clean segments"
        );
        assert!(
            is_canonical_dot_free_segment(service),
            "registry name service segments must be non-empty, trim-clean, and dot-free"
        );
        let kind_segment = kind.as_str();
        let service_start = module.len() + kind_segment.len() + 2;
        let mut value =
            String::with_capacity(module.len() + kind_segment.len() + service.len() + 2);
        value.push_str(module);
        value.push('.');
        value.push_str(kind_segment);
        value.push('.');
        value.push_str(service);
        Self {
            value: Arc::from(value),
            module_end: module.len(),
            service_start,
            kind,
        }
    }

    /// 借用完整规范键；该视图的生命期受当前名称借用约束。
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// 借用种类段之前的完整模块命名空间。
    pub fn module_name(&self) -> &str {
        &self.value[..self.module_end]
    }

    /// 返回构造时已校验的服务种类，不再次解析字符串。
    pub fn service_kind(&self) -> ServiceKind {
        self.kind
    }

    /// 借用最后一段服务本地名，不包含模块或种类。
    pub fn service_name(&self) -> &str {
        &self.value[self.service_start..]
    }
}

impl PartialEq for RegistryName {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for RegistryName {}

impl Hash for RegistryName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Borrow<str> for RegistryName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Serialize for RegistryName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RegistryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match Self::new(value) {
            Ok(name) => Ok(name),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

impl fmt::Display for RegistryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

fn is_canonical_segment(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_whitespace() {
        return false;
    }
    let last = match chars.next_back() {
        Some(last) => last,
        None => first,
    };
    !last.is_whitespace()
}

fn is_canonical_dot_free_segment(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_whitespace() || first == '.' {
        return false;
    }

    let mut last = first;
    for ch in chars {
        if ch == '.' {
            return false;
        }
        last = ch;
    }

    !last.is_whitespace()
}

fn is_canonical_module_namespace(value: &str) -> bool {
    let mut segment_start = 0;
    for (index, ch) in value.char_indices() {
        if ch != '.' {
            continue;
        }
        if !is_canonical_dot_free_segment(&value[segment_start..index]) {
            return false;
        }
        segment_start = index + 1;
    }
    is_canonical_dot_free_segment(&value[segment_start..])
}

// 从右侧确定服务种类与本地名边界，左侧保留可分段的模块命名空间。
fn registry_separator_offsets(value: &str) -> Option<(usize, usize)> {
    let bytes = value.as_bytes();
    let mut kind_end = None;
    let mut index = bytes.len();

    while index > 0 {
        index -= 1;
        if bytes[index] == b'.' {
            if let Some(kind_end) = kind_end {
                return Some((index, kind_end));
            }
            kind_end = Some(index);
        }
    }
    None
}

#[cfg(test)]
#[path = "tests/registry_name.rs"]
mod tests;
