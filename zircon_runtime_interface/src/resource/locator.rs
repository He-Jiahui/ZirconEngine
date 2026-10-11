use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::path::{Component, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 区分资源地址的逻辑来源，让解析和解析后的读取端选择各自的地址规则。
///
/// # 方案与职责
///
/// `Res`、`Library`、`Package`、`Builtin`、`Memory` 对应规范文本中的 `res`、`lib`、`package`、`builtin`、`mem`。方案不保存物理根；包根和项目根由读取端登记。
///
/// # 参考
///
/// [`crate::resource::ResourceLocator::parse`]、[`crate::resource::ResourceLocator::package_id`]、[`crate::resource::ResourceId::from_locator`]。
pub enum ResourceScheme {
    Res,
    Library,
    Package,
    Builtin,
    Memory,
}

impl ResourceScheme {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "res" => Some(Self::Res),
            "lib" => Some(Self::Library),
            "package" => Some(Self::Package),
            "builtin" => Some(Self::Builtin),
            "mem" => Some(Self::Memory),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Res => "res",
            Self::Library => "lib",
            Self::Package => "package",
            Self::Builtin => "builtin",
            Self::Memory => "mem",
        }
    }
}

/// 资源边界共用的规范位置：路径归一化后参与查找、显示和稳定身份推导。
/// `package` 路径首段是包标识，其余部分是包内相对路径。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 保存经过词法折叠的逻辑资源地址，作为引用、注册表键、显示文本和持久化地址的共同表示。
///
/// # 结构与使用时机
///
/// `scheme` 决定地址来源，`path` 是逻辑路径，`label` 可区分同一路径中的标记。字段通过构造方法建立，结构比较会区分路径和标签。
///
/// # 构造与验证边界
///
/// 文本入口先按方案与首个标签分隔符解析；分字段入口直接规范化已提供的路径。两者只处理词法组件，不确认磁盘文件存在或实际包含关系。包定位符另保留包 ID 与包内路径的边界。
///
/// # 生命周期
///
/// 定位符拥有路径与标签字符串；访问方法返回借用，使用后续物理路径前仍需经过对应读取端的检查。
///
/// # 参考
///
/// [`Self::parse`]、[`Self::new`]、[`Self::package_id`]、[`crate::resource::ResourceId::from_locator`]。
///
/// # 文本与序列化
///
/// 显示与 Serde 保存使用方案、路径和可选标签的单一文本表示；文本读取再经过解析入口。不要把显示文本视为物理文件路径。
pub struct ResourceLocator {
    scheme: ResourceScheme,
    path: String,
    label: Option<String>,
}

impl ResourceLocator {
    /// 解析并拒绝越出逻辑根目录的路径；调用方应使用返回的规范值作索引键。
    /// 解析外部文本资源地址，并建立后续身份派生和地址解析使用的词法定位符。
    ///
    /// # 参数
    ///
    /// `value` 使用规范方案和 `://` 分隔符。首个 `#` 后是非空标签；其后的 `#` 保留为标签内容。反斜线作为路径分隔符参与规范化。
    ///
    /// # 返回与错误
    ///
    /// 成功返回拥有路径和标签的定位符。缺少方案、不支持的方案、空路径或标签、路径越过词法根、包地址格式不成立时返回解析错误；失败不改变外部状态。
    ///
    /// # 职责边界
    ///
    /// 成功只证明本定位符的词法规则成立。实际文件包含关系、注册状态与读取结果由对应资源入口检查。
    ///
    /// # 参考
    ///
    /// [`Self::new`]、[`crate::resource::ResourceLocatorError`]、[`crate::resource::ResourceId::from_locator`]。
    pub fn parse(value: &str) -> Result<Self, ResourceLocatorError> {
        let Some((scheme, remainder)) = value.split_once("://") else {
            return Err(ResourceLocatorError::MissingScheme(value.to_string()));
        };
        let Some(scheme) = ResourceScheme::parse(scheme) else {
            return Err(ResourceLocatorError::UnsupportedScheme(scheme.to_string()));
        };
        let (path, label) = split_label(remainder)?;
        Self::new(scheme, path, label)
    }

    /// 从已分离的方案、路径和标签构造词法定位符，供别名入口和分字段调用者使用。
    ///
    /// # 参数
    ///
    /// - `scheme`：已选定的逻辑来源。
    /// - `path`：移入规范化流程的逻辑路径；包方案须含包 ID 与非空包内路径。
    /// - `label`：已分离的可选标签，Some 时不得为空；本入口不会从路径中再次拆出标签。
    ///
    /// # 返回与错误
    ///
    /// 返回规范化后的定位符。路径为空、越过词法根、包格式不成立或标签为空时返回错误；不注册资源、不访问文件。
    ///
    /// # 调用边界
    ///
    /// 需要解析完整地址文本时使用 `parse`。分字段入口与文本入口的分隔语义不同，调用端应在各自入口处理标签；不能从成功构造推断磁盘访问或结构往返已通过额外验证。
    ///
    /// # 参考
    ///
    /// [`Self::parse`]、[`Self::package_id`]、[`crate::resource::ResourceLocatorError`]。
    pub fn new(
        scheme: ResourceScheme,
        path: impl Into<String>,
        label: Option<String>,
    ) -> Result<Self, ResourceLocatorError> {
        let raw_path = path.into();
        let normalized_path = match scheme {
            ResourceScheme::Package => normalize_package_resource_path(&raw_path)?,
            _ => normalize_resource_path(&raw_path)?,
        };
        let normalized_label = match label {
            Some(value) if value.is_empty() => return Err(ResourceLocatorError::EmptyLabel),
            Some(value) => Some(value),
            None => None,
        };
        Ok(Self {
            scheme,
            path: normalized_path,
            label: normalized_label,
        })
    }

    pub fn scheme(&self) -> ResourceScheme {
        self.scheme
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Compares this locator's canonical display form without allocating it.
    /// 在无需分配显示字符串的比较路径中，检查输入是否恰为该定位符的规范显示文本。
    ///
    /// # 参数
    ///
    /// `value` 应为完整的规范显示串；本方法不解析别名、折叠路径，也不调整分隔符。
    ///
    /// # 返回
    ///
    /// 方案、路径和完整标签后缀逐段相同才返回 true；无标签时要求路径后没有多余字符。返回值仅表示文本匹配，不表示资源存在或已就绪。
    ///
    /// # 参考
    ///
    /// [`Self::parse`]、[`Self::label`]。
    pub fn matches_display(&self, value: &str) -> bool {
        let Some(remainder) = value
            .strip_prefix(self.scheme.as_str())
            .and_then(|value| value.strip_prefix("://"))
        else {
            return false;
        };
        let Some(suffix) = remainder.strip_prefix(self.path.as_str()) else {
            return false;
        };
        match self.label.as_deref() {
            Some(label) => suffix.strip_prefix('#') == Some(label),
            None => suffix.is_empty(),
        }
    }

    /// 为包资源读取端提取选择已登记包根所需的逻辑包 ID。
    ///
    /// # 返回与生命周期
    ///
    /// 包方案返回规范路径第一段的借用，其余方案返回 None。借用随定位符存活，不确认对应包已登记或可打开。
    ///
    /// # 参考
    ///
    /// [`Self::package_path`]、[`crate::resource::ResourceScheme`]。
    pub fn package_id(&self) -> Option<&str> {
        if self.scheme != ResourceScheme::Package {
            return None;
        }
        self.path.split_once('/').map(|(package_id, _)| package_id)
    }

    /// 提取包 ID 之后的包内相对路径，供读取端与已登记的包根配对解析。
    ///
    /// # 返回与生命周期
    ///
    /// 包方案返回路径借用，其余方案返回 None。包内路径已经过独立的词法折叠，返回值本身不证明磁盘路径仍位于包根内。
    ///
    /// # 参考
    ///
    /// [`Self::package_id`]、[`Self::new`]。
    pub fn package_path(&self) -> Option<&str> {
        if self.scheme != ResourceScheme::Package {
            return None;
        }
        self.path
            .split_once('/')
            .map(|(_, package_path)| package_path)
    }
}

impl Display for ResourceLocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.scheme.as_str(), self.path)?;
        if let Some(label) = &self.label {
            write!(f, "#{label}")?;
        }
        Ok(())
    }
}

impl Serialize for ResourceLocator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceLocator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 说明逻辑资源地址在解析或构造阶段被拒绝的原因，供调用端选择报告或拒绝输入。
///
/// # 错误边界
///
/// 错误覆盖方案分隔、支持范围、空值、词法根与包结构；它不表示文件读取失败、资源缺失或载荷不可用。Serde 读取会将解析原因交给反序列化错误。
///
/// # 参考
///
/// [`crate::resource::ResourceLocator::parse`]、[`crate::resource::ResourceLocator::new`]。
pub enum ResourceLocatorError {
    MissingScheme(String),
    UnsupportedScheme(String),
    EmptyPath,
    MissingPackagePath(String),
    EscapeAttempt(String),
    EmptyLabel,
}

impl Display for ResourceLocatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingScheme(value) => write!(f, "resource locator is missing scheme: {value}"),
            Self::UnsupportedScheme(value) => write!(f, "unsupported resource scheme: {value}"),
            Self::EmptyPath => write!(f, "resource locator path cannot be empty"),
            Self::MissingPackagePath(value) => {
                write!(
                    f,
                    "package resource locator requires package id and path: {value}"
                )
            }
            Self::EscapeAttempt(value) => write!(f, "resource locator escapes root: {value}"),
            Self::EmptyLabel => write!(f, "resource locator label cannot be empty"),
        }
    }
}

impl std::error::Error for ResourceLocatorError {}

/// 在完整地址的文本入口中分离首个标签分隔符，保持后续标签字符原样。
///
/// # 参数
///
/// `value` 是已经去除方案前缀的文本。
///
/// # 返回与错误
///
/// 返回拥有的路径和可选标签；首个 `#` 后为空时返回 EmptyLabel。没有分隔符时保留完整路径；不在此处规范化路径。
///
/// # 参考
///
/// [`crate::resource::ResourceLocator::parse`]、[`crate::resource::ResourceLocator::new`]。
fn split_label(value: &str) -> Result<(String, Option<String>), ResourceLocatorError> {
    match value.split_once('#') {
        Some((_path, "")) => Err(ResourceLocatorError::EmptyLabel),
        Some((path, label)) => Ok((path.to_string(), Some(label.to_string()))),
        None => Ok((value.to_string(), None)),
    }
}

/// 为非包方案建立统一分隔符的相对逻辑路径，供分字段构造入口消费。
///
/// # 参数
///
/// `path` 可含反斜线及 `.`、`..`；标签应由调用入口另行处理。
///
/// # 返回与错误
///
/// 返回折叠并以 `/` 连接的路径。结果为空或组件不满足相对根规则时返回错误；此处不访问磁盘，也不重新解释标签分隔符。
///
/// # 参考
///
/// [`crate::resource::ResourceLocator::new`]、[`crate::resource::ResourceLocator::parse`]。
fn normalize_resource_path(path: &str) -> Result<String, ResourceLocatorError> {
    let original = path.replace('\\', "/");
    let normalized = normalize_relative_path_components(&original)?;

    if normalized.is_empty() {
        return Err(ResourceLocatorError::EmptyPath);
    }

    Ok(normalized.join("/"))
}

/// 分开保留包 ID 与包内相对路径，使包内折叠不能回退到包 ID。
///
/// # 参数
///
/// `path` 应含一个非空包 ID 与非空包内路径；包 ID 必须是单个普通词法组件。
///
/// # 返回与错误
///
/// 返回包 ID 与折叠后包内路径。缺少分段、包 ID 不成立、包内为空或越过内部词法根时返回错误；不检查包注册或实际磁盘根。
///
/// # 参考
///
/// [`crate::resource::ResourceLocator::new`]、[`crate::resource::ResourceLocator::package_id`]、[`crate::resource::ResourceLocator::package_path`]。
fn normalize_package_resource_path(path: &str) -> Result<String, ResourceLocatorError> {
    let original = path.replace('\\', "/");
    let Some((package_id, package_path)) = original.split_once('/') else {
        return Err(ResourceLocatorError::MissingPackagePath(original));
    };
    if package_id.is_empty() || package_path.is_empty() {
        return Err(ResourceLocatorError::MissingPackagePath(original));
    }
    if !matches!(
        Path::new(package_id)
            .components()
            .collect::<Vec<_>>()
            .as_slice(),
        [Component::Normal(_)]
    ) {
        return Err(ResourceLocatorError::EscapeAttempt(original));
    }
    let normalized_package_path = normalize_relative_path_components(package_path)?;
    if normalized_package_path.is_empty() {
        return Err(ResourceLocatorError::MissingPackagePath(original));
    }

    Ok(format!(
        "{package_id}/{}",
        normalized_package_path.join("/")
    ))
}

/// 在词法组件栈中折叠相对路径，为普通路径和包内路径共同提供根边界检查。
///
/// # 参数
///
/// `path` 由上层处理分隔符后传入，按当前主机的 Path 组件规则解释。
///
/// # 返回与错误
///
/// 普通组件保留，`.` 忽略，`..` 消去已有组件；越过栈底或出现根、前缀时拒绝。返回的组件可能为空，由调用者决定是否允许；不检查文件、符号链接或注册根。
///
/// # 参考
///
/// [`crate::resource::ResourceLocator::new`]、[`crate::resource::ResourceLocatorError`]。
fn normalize_relative_path_components(path: &str) -> Result<Vec<String>, ResourceLocatorError> {
    let mut normalized = Vec::new();

    for component in Path::new(path).components() {
        match component {
            Component::Normal(segment) => normalized.push(segment.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized.pop().is_none() {
                    return Err(ResourceLocatorError::EscapeAttempt(path.to_string()));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(ResourceLocatorError::EscapeAttempt(path.to_string()));
            }
        }
    }

    Ok(normalized)
}

#[cfg(test)]
#[path = "tests/locator.rs"]
mod tests;
