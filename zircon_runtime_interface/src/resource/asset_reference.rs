use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};

use super::{AssetUuid, ResourceLocator};

/// 跨资产文档传递 UUID 身份与可读定位符；定位符可随迁移变化，UUID 仍用于持久引用。
/// 序列化字段 `url` 保持既有资产文档格式。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// 把资产的持久 UUID 与当前定位符作为同一引用传给场景、项目注册表和持久化数据。
///
/// # 字段与使用时机
///
/// - `uuid` 关联既有资产身份；项目资产移动时通常保留侧车中已有的 UUID。
/// - `locator` 描述当前地址，JSON 与二进制引用都使用字段名 `url`。
/// - 已有注册信息时使用 `new` 保留身份对；只有地址的引用可使用 `from_locator` 派生身份。
///
/// # 构造与验证边界
///
/// 构造和反序列化不会查询注册表以确认两字段指向同一资产；加载端仍须按当前项目或内置资源规则解析。显示只输出定位符，不能替代保存完整引用。
///
/// # 参考
///
/// [`Self::new`]、[`Self::from_locator`]、[`crate::resource::AssetUuid`]、[`crate::resource::ResourceLocator`]。
pub struct AssetReference {
    pub uuid: AssetUuid,
    pub locator: ResourceLocator,
}

impl AssetReference {
    pub fn new(uuid: AssetUuid, locator: ResourceLocator) -> Self {
        Self { uuid, locator }
    }

    /// 为仅有定位符的引用推导身份；已有资产 UUID 应用 `new` 保留。
    /// 为只有定位符的引用生成可重复的资产 UUID；已有持久资产身份时应保留原 UUID。
    ///
    /// # 参数
    ///
    /// `locator` 被移入结果，完整显示串（含标签）参与资产 UUID 派生。
    ///
    /// # 返回与使用时机
    ///
    /// 返回派生 UUID 与原定位符的身份对，不查询项目侧车。此处对任一定位符采用稳定资产身份派生，不能据此推断内存资源的注册表 ID 分配方式。
    ///
    /// # 参考
    ///
    /// [`Self::new`]、[`crate::resource::AssetUuid::from_stable_label`]、[`crate::resource::ResourceId::from_locator`]。
    pub fn from_locator(locator: ResourceLocator) -> Self {
        let uuid = AssetUuid::from_stable_label(&locator.to_string());
        Self::new(uuid, locator)
    }
}

impl Display for AssetReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.locator)
    }
}

impl Serialize for AssetReference {
    /// 把完整资产引用写入 Serde，以相同的 UUID 与地址语义服务 JSON 和二进制数据。
    ///
    /// # 参数
    ///
    /// `serializer` 的可读性标志选择字段表示：可读格式借用强类型字段；不可读格式按 `uuid`、`url` 的顺序写入两个字符串。
    ///
    /// # 返回与错误
    ///
    /// 返回序列化器的完成结果，错误由序列化器传回。这里只写引用数据，不加载资源，也不检查 UUID 与地址的注册关系。
    ///
    /// # 参考
    ///
    /// [`crate::resource::AssetReference`]、[`crate::resource::ResourceLocator`]。
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Repr<'a> {
            uuid: AssetUuid,
            url: &'a ResourceLocator,
        }

        if !serializer.is_human_readable() {
            #[derive(Serialize)]
            struct BinaryRepr {
                uuid: String,
                url: String,
            }

            return BinaryRepr {
                uuid: self.uuid.to_string(),
                url: self.locator.to_string(),
            }
            .serialize(serializer);
        }

        Repr {
            uuid: self.uuid,
            url: &self.locator,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AssetReference {
    /// 按配对的 Serde 字段布局恢复资产身份对，并重新验证文本 UUID 和定位符。
    ///
    /// # 参数
    ///
    /// `deserializer` 的可读性标志必须与写入格式匹配。不可读格式先读取 `uuid`、`url` 两个字符串；可读格式读取强类型字段。
    ///
    /// # 返回与错误
    ///
    /// 字段读取或任一文本解析失败时返回反序列化错误；成功时保留读出的身份对，不重新派生 UUID，不查询资源注册表。
    ///
    /// # 参考
    ///
    /// [`crate::resource::AssetReference::new`]、[`crate::resource::ResourceLocator::parse`]。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Repr {
            uuid: AssetUuid,
            url: ResourceLocator,
        }

        if !deserializer.is_human_readable() {
            #[derive(Deserialize)]
            struct BinaryRepr {
                uuid: String,
                url: String,
            }

            let BinaryRepr { uuid, url } = BinaryRepr::deserialize(deserializer)?;
            let uuid = uuid.parse::<AssetUuid>().map_err(D::Error::custom)?;
            let url = ResourceLocator::parse(&url).map_err(D::Error::custom)?;
            return Ok(Self::new(uuid, url));
        }

        let Repr { uuid, url } = Repr::deserialize(deserializer)?;
        Ok(Self::new(uuid, url))
    }
}
