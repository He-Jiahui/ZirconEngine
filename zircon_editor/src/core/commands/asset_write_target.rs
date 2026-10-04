//! 把命令参数中的资产类型与定位字段接入写权限解析；描述符只声明取值位置，运行时仍须校验参数并取得资产写权限。

use serde::{Deserialize, Serialize};

/// Names the invocation arguments used to resolve an asset mutation target.
/// 用于贡献声明与宿主共用的参数映射；传入的是字段名，不能把声明本身当作资产写权限。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetWriteTargetDescriptor {
    asset_type_argument: String,
    locator_argument: String,
}

impl AssetWriteTargetDescriptor {
    pub fn new(
        asset_type_argument: impl Into<String>,
        locator_argument: impl Into<String>,
    ) -> Self {
        Self {
            asset_type_argument: asset_type_argument.into(),
            locator_argument: locator_argument.into(),
        }
    }

    pub fn asset_type_argument(&self) -> &str {
        &self.asset_type_argument
    }

    pub fn locator_argument(&self) -> &str {
        &self.locator_argument
    }
}
