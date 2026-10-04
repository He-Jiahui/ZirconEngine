use serde::{Deserialize, Serialize};

use crate::asset::assets::FontFamilyName;

use super::face::FontFaceDescriptor;

/// 可序列化的字体家族描述，把家族名与各字体面声明集中表达。
/// 该值不含字体源字节，构造它本身不会登记字体面或创建可用实例。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontFamilyDescriptor {
    pub name: FontFamilyName,
    #[serde(default)]
    pub faces: Vec<FontFaceDescriptor>,
}
