//! 供运行时与 Native 构建面共享的插件包声明。
//! 运行时转换由 feature 控制，包元数据仍是各投影生成的唯一来源。
#[cfg(feature = "runtime")]
use crate::runtime::RuntimePluginDeclaration;
#[cfg(feature = "runtime")]
use zircon_runtime::builtin::RuntimePluginId;
#[cfg(feature = "runtime")]
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
#[cfg(feature = "runtime")]
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};
#[cfg(feature = "runtime")]
use zircon_runtime::core::ModuleDescriptor;
#[cfg(feature = "runtime")]
use zircon_runtime::plugin::{PluginMaturity as RuntimePluginMaturity, RuntimePluginDescriptor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 插件声明希望接入的运行环境；该枚举会映射为运行时 descriptor 的目标模式。
pub enum PluginTarget {
    ClientRuntime,
    ServerRuntime,
    EditorHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 包声明支持的导出平台；runtime 功能开启时会转换为导出系统的平台枚举。
pub enum PluginPlatform {
    Windows,
    Linux,
    Macos,
    Android,
    Ios,
    WebGpu,
    Wasm,
    Headless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 包的成熟度等级，投影到运行时插件成熟度字段。
pub enum PluginMaturityLevel {
    Core,
    Stable,
    Beta,
    Experimental,
    Externalized,
    Stub,
    Deprecated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 包可采用的默认分发形式，映射到导出打包策略。
pub enum PluginPackaging {
    SourceTemplate,
    LibraryEmbed,
    NativeDynamic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 标明 capability 由哪个注册路径提供；运行时投影只收录运行时提供的角色。
pub enum PluginCapabilityRole {
    RuntimeRegistration,
    EditorRegistration,
    RuntimeEditorRegistration,
    RequestedOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 区分生产包、开发工具、示例与测试夹具，并作为包用途元数据参与投影。
pub enum PluginPackageRole {
    Production,
    DeveloperTool,
    Sample,
    TestFixture,
}

impl PluginCapabilityRole {
    const fn is_runtime_provided(self) -> bool {
        matches!(
            self,
            Self::RuntimeRegistration | Self::RuntimeEditorRegistration
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Native 入口名及其 NUL 终止形式，供生成的 ABI 静态声明导出。
pub struct NativePluginEntryDeclaration {
    name: &'static str,
    cstr: &'static [u8],
}

impl NativePluginEntryDeclaration {
    pub const fn new(name: &'static str, cstr: &'static [u8]) -> Self {
        Self { name, cstr }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn cstr(self) -> &'static [u8] {
        self.cstr
    }
}

#[cfg(feature = "runtime")]
impl From<PluginTarget> for RuntimeTargetMode {
    fn from(target: PluginTarget) -> Self {
        match target {
            PluginTarget::ClientRuntime => Self::ClientRuntime,
            PluginTarget::ServerRuntime => Self::ServerRuntime,
            PluginTarget::EditorHost => Self::EditorHost,
        }
    }
}

#[cfg(feature = "runtime")]
impl From<PluginPlatform> for ExportTargetPlatform {
    fn from(platform: PluginPlatform) -> Self {
        match platform {
            PluginPlatform::Windows => Self::Windows,
            PluginPlatform::Linux => Self::Linux,
            PluginPlatform::Macos => Self::Macos,
            PluginPlatform::Android => Self::Android,
            PluginPlatform::Ios => Self::Ios,
            PluginPlatform::WebGpu => Self::WebGpu,
            PluginPlatform::Wasm => Self::Wasm,
            PluginPlatform::Headless => Self::Headless,
        }
    }
}

#[cfg(feature = "runtime")]
impl From<PluginMaturityLevel> for RuntimePluginMaturity {
    fn from(maturity: PluginMaturityLevel) -> Self {
        match maturity {
            PluginMaturityLevel::Core => Self::Core,
            PluginMaturityLevel::Stable => Self::Stable,
            PluginMaturityLevel::Beta => Self::Beta,
            PluginMaturityLevel::Experimental => Self::Experimental,
            PluginMaturityLevel::Externalized => Self::Externalized,
            PluginMaturityLevel::Stub => Self::Stub,
            PluginMaturityLevel::Deprecated => Self::Deprecated,
        }
    }
}

#[cfg(feature = "runtime")]
impl From<PluginPackaging> for ExportPackagingStrategy {
    fn from(packaging: PluginPackaging) -> Self {
        match packaging {
            PluginPackaging::SourceTemplate => Self::SourceTemplate,
            PluginPackaging::LibraryEmbed => Self::LibraryEmbed,
            PluginPackaging::NativeDynamic => Self::NativeDynamic,
        }
    }
}

#[cfg(feature = "runtime")]
impl From<PluginPackageRole> for zircon_runtime::plugin::PluginPackageRole {
    fn from(role: PluginPackageRole) -> Self {
        match role {
            PluginPackageRole::Production => Self::Production,
            PluginPackageRole::DeveloperTool => Self::DeveloperTool,
            PluginPackageRole::Sample => Self::Sample,
            PluginPackageRole::TestFixture => Self::TestFixture,
        }
    }
}

/// Metadata that belongs to a plugin package rather than its runtime behavior.
///
/// The declaration intentionally contains no registration callbacks or importer
/// logic. Those remain in the plugin crate, while this value supplies the
/// descriptor fields that must agree with generated package metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginDeclaration {
    id: &'static str,
    display_name: &'static str,
    category: &'static str,
    module_name: &'static str,
    module_description: &'static str,
    target_modes: &'static [PluginTarget],
    supported_platforms: &'static [PluginPlatform],
    capabilities: &'static [&'static str],
    capability_roles: &'static [PluginCapabilityRole],
    maturity: PluginMaturityLevel,
    default_packaging: &'static [PluginPackaging],
    package_role: PluginPackageRole,
}

impl PluginDeclaration {
    /// 构造静态包元数据；能力名与角色按索引配对。
    ///
    /// 两个切片的长度在生成运行时声明时校验，必须保持一致。
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: &'static str,
        display_name: &'static str,
        category: &'static str,
        module_name: &'static str,
        module_description: &'static str,
        target_modes: &'static [PluginTarget],
        supported_platforms: &'static [PluginPlatform],
        capabilities: &'static [&'static str],
        capability_roles: &'static [PluginCapabilityRole],
        maturity: PluginMaturityLevel,
        default_packaging: &'static [PluginPackaging],
    ) -> Self {
        Self {
            id,
            display_name,
            category,
            module_name,
            module_description,
            target_modes,
            supported_platforms,
            capabilities,
            capability_roles,
            maturity,
            default_packaging,
            package_role: PluginPackageRole::Production,
        }
    }

    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn display_name(self) -> &'static str {
        self.display_name
    }

    pub const fn category(self) -> &'static str {
        self.category
    }

    pub const fn module_name(self) -> &'static str {
        self.module_name
    }

    pub const fn declared_targets(self) -> &'static [PluginTarget] {
        self.target_modes
    }

    pub const fn declared_platforms(self) -> &'static [PluginPlatform] {
        self.supported_platforms
    }

    pub const fn capabilities(self) -> &'static [&'static str] {
        self.capabilities
    }

    pub const fn capability_roles(self) -> &'static [PluginCapabilityRole] {
        self.capability_roles
    }

    pub const fn declared_maturity(self) -> PluginMaturityLevel {
        self.maturity
    }

    pub const fn declared_packaging(self) -> &'static [PluginPackaging] {
        self.default_packaging
    }

    pub const fn package_role(self) -> PluginPackageRole {
        self.package_role
    }

    pub const fn with_package_role(mut self, package_role: PluginPackageRole) -> Self {
        self.package_role = package_role;
        self
    }

    #[cfg(feature = "runtime")]
    pub fn target_modes(self) -> Vec<RuntimeTargetMode> {
        self.target_modes.iter().copied().map(Into::into).collect()
    }

    #[cfg(feature = "runtime")]
    pub fn supported_platforms(self) -> Vec<ExportTargetPlatform> {
        self.supported_platforms
            .iter()
            .copied()
            .map(Into::into)
            .collect()
    }

    #[cfg(feature = "runtime")]
    pub fn maturity(self) -> RuntimePluginMaturity {
        self.maturity.into()
    }

    #[cfg(feature = "runtime")]
    pub fn default_packaging(self) -> Vec<ExportPackagingStrategy> {
        self.default_packaging
            .iter()
            .copied()
            .map(Into::into)
            .collect()
    }

    #[cfg(feature = "runtime")]
    pub fn module_descriptor(self) -> ModuleDescriptor {
        ModuleDescriptor::new(self.module_name, self.module_description)
    }

    /// 从包元数据构造运行时声明，仅把运行时提供的 capability 写入 descriptor。
    ///
    /// 插件 ID 必须是规范的 RuntimePluginId key；能力名与角色数量不等时会 panic。
    #[cfg(feature = "runtime")]
    pub fn runtime_declaration(self, crate_name: impl Into<String>) -> RuntimePluginDeclaration {
        let runtime_id = RuntimePluginId::new(self.id);
        assert_eq!(
            runtime_id.key(),
            self.id,
            "plugin declaration id `{}` must be a canonical RuntimePluginId key",
            self.id
        );
        let declaration =
            RuntimePluginDeclaration::new(self.id, self.display_name, runtime_id, crate_name)
                .with_category(self.category)
                .with_target_modes(self.target_modes())
                .with_maturity(self.maturity())
                .with_default_packaging(self.default_packaging())
                .with_package_role(self.package_role.into());

        assert_eq!(
            self.capabilities.len(),
            self.capability_roles.len(),
            "plugin declaration capability names and roles must remain aligned"
        );
        self.capabilities
            .iter()
            .copied()
            .zip(self.capability_roles.iter().copied())
            .filter(|(_, role)| role.is_runtime_provided())
            .fold(declaration, |declaration, (capability, _)| {
                declaration.with_capability(capability)
            })
    }

    /// 补入本插件的模块描述，生成可供运行时注册与模块安装使用的 descriptor。
    #[cfg(feature = "runtime")]
    pub fn runtime_descriptor(self, crate_name: impl Into<String>) -> RuntimePluginDescriptor {
        self.runtime_declaration(crate_name)
            .with_module_descriptor(self.module_descriptor())
            .into_descriptor()
    }
}

mod macros;

#[cfg(test)]
#[path = "declaration/tests/cases.rs"]
mod tests;
