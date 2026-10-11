use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

use super::stable_uuid_from_components;

/// 资源目录与引用共用的身份；有稳定来源标签时可重建，否则通过 `new` 分配随机 UUID。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 标识资产本身的持久 UUID，可随资产地址变化保留，用于元数据、引用和注册表查询。
///
/// # 构造与生命周期
///
/// 首次建立侧车时可生成随机身份；只有稳定标签的场景可显式派生身份。二者服务不同入口，读取已有侧车时不应以当前位置重新生成 UUID。
///
/// # 验证边界
///
/// UUID 的格式解析不证明资产已注册或引用关系正确；需要注册表解析时应同时携带当前定位符。
///
/// # 参考
///
/// [`Self::new`]、[`Self::from_stable_label`]、[`crate::resource::AssetReference`]。
pub struct AssetUuid(pub(crate) Uuid);

impl AssetUuid {
    /// 为没有稳定来源标签的新资源随机生成 v4 身份。
    /// 为尚无持久元数据的资产分配随机 UUID。
    ///
    /// # 返回与使用时机
    ///
    /// 返回 UUIDv4 身份；重复调用会重新分配，不是按路径去重。已有侧车 UUID 时应读取并保留，随后将该身份写入元数据或引用。
    ///
    /// # 参考
    ///
    /// [`crate::resource::AssetReference::new`]、[`Self::from_stable_label`]。
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从稳定来源标签派生可重建身份，例如资源 locator，使重新扫描不依赖发现顺序。
    /// 在只有稳定标签的入口中派生可重复的资产 UUID，并与资源 ID 的派生域分离。
    ///
    /// # 参数
    ///
    /// `label` 按原始 UTF-8 字节进入资产命名空间；本方法不解析或规范化标签。地址标签应先通过定位符的构造边界。
    ///
    /// # 返回与生命周期
    ///
    /// 相同算法版本和相同标签得到相同 UUID。标签或算法版本改变会改变身份；此结果不能代替已有侧车的随机 UUID。
    ///
    /// # 参考
    ///
    /// [`crate::resource::AssetReference::from_locator`]、[`crate::resource::STABLE_UUID_ALGORITHM_VERSION`]、[`crate::resource::ResourceId::from_stable_label`]。
    pub fn from_stable_label(label: &str) -> Self {
        Self(stable_uuid_from_components("zircon-asset-uuid", &[label]))
    }

    /// Returns the UUID bytes for allocation-free deterministic ordering.
    /// 借用 UUID 的原始 16 字节作为确定性排序键，供注册表引用查询使用。
    ///
    /// # 返回与生命周期
    ///
    /// 返回与 `self` 同寿命的字节数组引用，不分配字符串。排序比较的是 UUID 字节顺序，不表示资产路径顺序或注册时间。
    ///
    /// # 参考
    ///
    /// [`crate::resource::AssetUuid`]。
    pub fn binary_key(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Default for AssetUuid {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for AssetUuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AssetUuid {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}
