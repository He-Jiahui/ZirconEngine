use serde::{Deserialize, Serialize};

/// 区分产品、开发工具、示例和测试夹具；产品目录与安装准入接受 Production 和 DeveloperTool。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPackageRole {
    #[default]
    Production,
    DeveloperTool,
    Sample,
    TestFixture,
}

impl PluginPackageRole {
    /// 仅 Production 使用 serde 的默认省略规则。
    pub const fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// DeveloperTool 也可作为产品目录来源；Sample 和 TestFixture 只供样例/测试，不参与产品准入。
    pub const fn is_product_catalog_eligible(self) -> bool {
        matches!(self, Self::Production | Self::DeveloperTool)
    }
}
