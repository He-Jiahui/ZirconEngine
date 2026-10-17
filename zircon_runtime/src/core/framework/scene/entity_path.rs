use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

/// 动画目标和世界属性访问共享的层级定位符；结构化段用于查找，文本用于外部引用。
///
/// 同时拥有显示串与层级分段，不借用 World，也不保证实体当前存在。
/// 派生 serde 直接读写这两种表示，不会调用构造器或校验两者一致性。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityPath {
    raw: String,
    segments: Vec<String>,
}

impl EntityPath {
    /// 接管层级分段并拼接显示串。
    ///
    /// # 参数、返回值与约束
    /// `segments` 须非空且每段均非空白，否则返回 `PathParseError`。
    /// 保留每段原文，不裁剪空白；段内含层级分隔符 `/` 时返回错误，避免文本回读改变层级。
    pub fn new(segments: Vec<String>) -> Result<Self, PathParseError> {
        if segments.is_empty() {
            return Err(PathParseError::new(
                "entity path must contain at least one segment",
            ));
        }
        if segments.iter().any(|segment| segment.trim().is_empty()) {
            return Err(PathParseError::new(
                "entity path segments must not be empty or whitespace",
            ));
        }

        if segments.iter().any(|segment| segment.contains('/')) {
            return Err(PathParseError::new(
                "entity path segments must not contain the hierarchy separator '/'",
            ));
        }

        Ok(Self {
            raw: segments.join("/"),
            segments,
        })
    }

    /// 把斜线分隔的文本解析为拥有型路径。
    ///
    /// # 参数与返回值
    /// `path` 仅在调用期借用，每段先裁剪空白并跳过空段，再交给构造器。
    /// 有效段为空时返回错误；成功只表示路径构造成功，实体查找另由 World 完成。
    pub fn parse(path: &str) -> Result<Self, PathParseError> {
        let mut segments = Vec::with_capacity(path.len().saturating_div(2).max(1));
        segments.extend(
            path.split('/')
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .map(ToOwned::to_owned),
        );

        Self::new(segments)
    }

    /// 借用保存的层级分段，引用有效期随当前路径的共享借用。
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// 借用保存的显示串；显示及该访问器均不重新解析分段。
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

#[cfg(test)]
#[path = "entity_path/tests/single_scan_parse.rs"]
mod single_scan_parse_tests;

impl fmt::Display for EntityPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// 属性绑定的结构化键；动态组件类型名可能包含点号，调用端须保留 component 与属性段。
///
/// 拥有组件键、属性分段和显示串，不检查组件登记或字段可写性。
/// 派生 serde 直接恢复各字段，绕过构造器的非空检查及显示串重建。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentPropertyPath {
    raw: String,
    component: String,
    property_segments: Vec<String>,
}

impl ComponentPropertyPath {
    /// 接管组件键和属性分段，构造独立拥有的属性路径。
    ///
    /// # 参数与返回值
    /// `component` 与每个属性段均须非空白，`property_segments` 至少有一项，否则返回错误。
    /// 成功保留输入文本，按 UTF-8 字节长度预留显示串空间；不裁剪或转义段内字符。
    pub fn new(
        component: impl Into<String>,
        property_segments: Vec<String>,
    ) -> Result<Self, PathParseError> {
        let component = component.into();
        if component.trim().is_empty() {
            return Err(PathParseError::new(
                "component path must include a component name",
            ));
        }
        let property_len = validated_property_segments_len(&property_segments)?;

        Ok(Self {
            raw: component_property_path_raw(&component, &property_segments, property_len),
            component,
            property_segments,
        })
    }

    // TODO: [CR-FRAMEWORK-SCENE-0002] 明确动态类型路径的文本回读约定：new 可接收带点号的组件名，parse 却将首个点号当分隔符。
    /// 解析点分文本，各段裁剪空白并跳过空段。
    ///
    /// # 返回值与约束
    /// 首个有效段作为组件键，其余交给构造器校验；缺少组件或属性时返回错误。
    /// 含点组件名的往返边界见相邻 TODO，此解析器不会恢复完整组件键。
    pub fn parse(path: &str) -> Result<Self, PathParseError> {
        let mut segments = path
            .split('.')
            .map(str::trim)
            .filter(|segment| !segment.is_empty());
        let component = segments.next().ok_or_else(|| {
            PathParseError::new("component property path must use `<component>.<property>` syntax")
        })?;
        let property_segments = segments.map(ToOwned::to_owned).collect::<Vec<_>>();

        Self::new(component.to_string(), property_segments)
    }

    /// 借用保存的组件键，字段目录的匹配和规范化由消费方决定。
    pub fn component(&self) -> &str {
        &self.component
    }

    /// 借用保存的属性分段，引用受路径共享借用期约束。
    pub fn property_segments(&self) -> &[String] {
        &self.property_segments
    }

    /// 借用显示串，不查询组件存储或复核反序列化表示。
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

/// 一次检查属性列表及各段非空白，并累计保留原文的 UTF-8 字节数。
fn validated_property_segments_len(property_segments: &[String]) -> Result<usize, PathParseError> {
    if property_segments.is_empty() {
        return Err(PathParseError::new(
            "component path must include at least one property segment",
        ));
    }

    let mut property_len = 0;
    for segment in property_segments {
        if segment.trim().is_empty() {
            return Err(PathParseError::new(
                "component property segments must not be empty or whitespace",
            ));
        }
        property_len += segment.len();
    }

    Ok(property_len)
}

/// 按已校验分段及字节总长预留空间并拼接显示串，长度参数由上游校验助手提供。
fn component_property_path_raw(
    component: &str,
    property_segments: &[String],
    property_len: usize,
) -> String {
    let mut raw = String::with_capacity(component.len() + property_len + property_segments.len());
    raw.push_str(component);
    for segment in property_segments {
        raw.push('.');
        raw.push_str(segment);
    }
    raw
}

impl fmt::Display for ComponentPropertyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// 拥有路径构造或解析错误的说明文本，显示时直接输出该说明。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathParseError {
    message: String,
}

impl PathParseError {
    /// 接管错误说明并返回错误值，不执行路径解析。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PathParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for PathParseError {}

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for EntityPath {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::RetainedOwnerV1;
        self.raw.retained_bound_v1(m)?;
        self.segments.retained_bound_v1(m)
    }
}
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1 for EntityPath {}
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for ComponentPropertyPath {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::RetainedOwnerV1;
        self.raw.retained_bound_v1(m)?;
        self.component.retained_bound_v1(m)?;
        self.property_segments.retained_bound_v1(m)
    }
}
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1
    for ComponentPropertyPath
{
}
