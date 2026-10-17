//! 字体面的选择属性、源索引与可变轴声明值。
//! 限幅方法只处理数值；字体源解析和实例归一由消费方负责。

use serde::{Deserialize, Serialize};

use crate::asset::assets::FontFamilyName;

/// 字体选面使用的字重尺度；经 clamped 导入的元数据限制在传统 100–900 区间。
/// 此包装本身不验证直接构造或反序列化的数值。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const NORMAL: Self = Self(400);
    pub const BOLD: Self = Self(700);

    /// 按本模型的字重范围组装包装值。
    ///
    /// # 参数与返回
    /// `value` 为待截断的字重；返回 `100..=900` 内的值，不查找或验证字体面。
    pub fn clamped(value: u16) -> Self {
        Self(value.clamp(100, 900))
    }
}

// TODO: [CR-TEXT-LAYOUT-0001] 确认 Oblique 参数的单位及选面意图；style_distance 忽略角度而选面缓存按浮点位区分，缺少角度行为测试；下一步核对字体元数据与斜体合成消费端。
/// 字体选择的样式分类，供字体描述与选面查询共享。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique(f32),
}

/// 以正常宽度为 100 的字体宽度偏好；选面按宽度距离排序，不是布局阶段的缩放倍率。
/// 公开字段直接构造和派生反序列化不会调用 [`Self::clamped`]；
/// 只有显式调用该方法时才按本模块的范围截断。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FontStretch(pub u16);

impl FontStretch {
    pub const NORMAL: Self = Self(100);

    /// 按本模型的宽度范围组装包装值。
    ///
    /// # 参数与返回
    /// `value` 以正常宽度 `100` 为基准；返回截断到 `50..=200` 的值，不执行布局缩放。
    pub fn clamped(value: u16) -> Self {
        Self(value.clamp(50, 200))
    }
}

/// 字体集合文件内部的字体面索引，随源文件一起定位字体；与数据库面身份不同。
pub type FaceIndex = u32;

/// OpenType 轴标签与设计坐标；进入实例登记时才规范顺序、重复标签和有限值。
/// 调用方应让整形与栅格消费同一有效实例，避免相同字形号落在不同轴变体。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VariationCoords(pub Vec<(u32, f32)>);

/// 字体源登记时使用的作者或元数据描述；不拥有字体字节，也不自动证明字符覆盖。
/// face_index 定位集合内的字体面，variations 作为后续有效实例的默认轴坐标。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontFaceDescriptor {
    pub family: FontFamilyName,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub face_index: FaceIndex,
    #[serde(default)]
    pub variations: VariationCoords,
}

impl FontFaceDescriptor {
    /// 组装常规样式的字体面声明。
    ///
    /// # 参数与返回
    /// `family` 转为 `FontFamilyName`；返回字重 `400`、`Normal` 样式、正常宽度 `100`、集合索引 `0` 及空轴坐标。
    /// 返回值未读取字体源或登记字体面，也不证明该集合索引在实际文件中存在。
    pub fn regular(family: impl Into<FontFamilyName>) -> Self {
        Self {
            family: family.into(),
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
            face_index: 0,
            variations: VariationCoords::default(),
        }
    }
}
