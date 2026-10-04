//! 项目脚本清单只声明包根和启动包；项目加载传递这些作者设置，脚本宿主决定实际发现与启动顺序。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScriptManifest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub package_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub startup_packages: Vec<String>,
}

impl ProjectScriptManifest {
    pub fn is_empty(&self) -> bool {
        self.package_roots.is_empty() && self.startup_packages.is_empty()
    }
}
