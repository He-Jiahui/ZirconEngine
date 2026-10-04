use serde::{Deserialize, Serialize};

use super::face::{FontStretch, FontStyle, FontWeight};
use crate::asset::assets::FontFamilyName;

/// 字体数据库登记的一张字体面身份；只在对应数据库及其字库版本上下文中解析。
/// 相同数值不能跨独立数据库当作同一字体资源。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontFaceId(pub u64);

/// 一张字体面与有效可变轴坐标的实例身份，供整形和栅格缓存共享同一变体。
/// 实例仍依赖所属字库中的字体面；不能据此恢复字体文件或替代字库版本检查。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstancedFaceId(pub u64);

/// 字体选择请求；families 的顺序表示偏好，数据库可继续使用项目与运行时后备字体。
/// 选面与行度量使用同一请求，避免测量与实际整形选择不同字体。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontQuery {
    pub families: Vec<FontFamilyName>,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
}

impl FontQuery {
    pub fn single_family(family: impl Into<FontFamilyName>) -> Self {
        Self {
            families: vec![family.into()],
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
        }
    }
}

/// 选面结果及需要合成样式的元数据；结果不承诺覆盖请求中的每个字符。
/// 字素级覆盖与后备字体决议由后续整形解析完成。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontMatch {
    pub face: FontFaceId,
    pub synthetic_bold: bool,
    pub synthetic_oblique: bool,
}
