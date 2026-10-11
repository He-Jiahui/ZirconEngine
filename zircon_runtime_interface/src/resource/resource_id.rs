use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

use super::{stable_uuid_from_components, AssetUuid, ResourceLocator, ResourceScheme};

/// 注册表使用的资源身份，与可更名的定位符分离；项目资产可由资产 UUID 推导。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 作为记录、句柄和事件共同使用的资源注册表身份，与可变定位符分离。
///
/// # 身份来源
///
/// 临时内存入口可随机分配；持久定位符可按规范显示文本派生；项目资产可按已保存的资产 UUID 派生。不同派生入口使用不同命名空间。
///
/// # 使用边界
///
/// UUID 值或其文本格式不证明资源存在。需要保持项目资产移动后的身份时，应沿用资产 UUID 的派生入口，而非从新地址重新生成。
///
/// # 参考
///
/// [`Self::from_locator`]、[`Self::from_asset_uuid`]、[`crate::resource::ResourceRecord`]。
pub struct ResourceId(Uuid);

impl ResourceId {
    /// 为需要独立身份的临时资源分配随机资源 ID。
    ///
    /// # 返回与使用时机
    ///
    /// 返回 UUIDv4 身份；每次调用重新分配，不登记记录、不由地址去重。需要可重复的持久身份时选择相应派生入口。
    ///
    /// # 参考
    ///
    /// [`Self::from_locator`]、[`Self::from_asset_uuid`]。
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 持久方案由规范定位符稳定推导；`mem` 每次分配新身份以避免偶然复用。
    /// 为只有定位符的资源入口选择内存随机身份或持久地址派生身份。
    ///
    /// # 参数
    ///
    /// `locator` 是已构造的逻辑地址。持久方案的完整显示串（含标签）参与派生；此方法不再规范化地址。
    ///
    /// # 返回与生命周期
    ///
    /// mem 每次分配新 ID；res、lib、package、builtin 按同一版本与显示文本稳定派生。返回身份不注册或加载资源；项目资产移动时需要保留原身份可使用资产 UUID 入口。
    ///
    /// # 参考
    ///
    /// [`Self::from_asset_uuid`]、[`Self::from_stable_label`]、[`crate::resource::ResourceLocator`]。
    pub fn from_locator(locator: &ResourceLocator) -> Self {
        match locator.scheme() {
            ResourceScheme::Memory => Self::new(),
            ResourceScheme::Res
            | ResourceScheme::Library
            | ResourceScheme::Package
            | ResourceScheme::Builtin => Self::from_stable_label(&locator.to_string()),
        }
    }

    /// 在资源标签命名空间中生成可重复的注册表身份。
    ///
    /// # 参数
    ///
    /// `label` 以原始 UTF-8 字节参与派生，本入口不解析路径。若标签来自地址，应先建立定位符的规范表示。
    ///
    /// # 返回与生命周期
    ///
    /// 相同算法版本与标签返回相同 ID，标签变化会改变身份；资源标签域与资产 UUID 域、资产 UUID 到资源 ID 的域分别隔离。
    ///
    /// # 参考
    ///
    /// [`Self::from_locator`]、[`crate::resource::AssetUuid::from_stable_label`]、[`crate::resource::STABLE_UUID_ALGORITHM_VERSION`]。
    pub fn from_stable_label(label: &str) -> Self {
        Self(stable_uuid_from_components(
            "zircon-resource-id/stable-label",
            &[label],
        ))
    }

    /// 用项目资产已有的持久 UUID 派生资源 ID，使地址变化后仍可维持身份。
    ///
    /// # 参数
    ///
    /// `uuid` 应是侧车或注册表中已有的资产身份；本方法不读取侧车、不检查资产地址，也不重新生成 UUID。
    ///
    /// # 返回与生命周期
    ///
    /// 返回资产 UUID 专用命名空间中的资源 ID。相同 UUID 与算法版本得到相同 ID，其值不是原资产 UUID，也不等于按同文本作标签派生的 ID。
    ///
    /// # 参考
    ///
    /// [`crate::resource::AssetUuid`]、[`crate::resource::AssetReference::new`]、[`Self::from_locator`]。
    pub fn from_asset_uuid(uuid: AssetUuid) -> Self {
        Self(stable_uuid_from_components(
            "zircon-resource-id/asset-uuid",
            &[uuid.to_string().as_str()],
        ))
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ResourceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}
