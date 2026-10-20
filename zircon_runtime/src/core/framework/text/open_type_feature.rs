use serde::{Deserialize, Serialize};

/// An OpenType feature setting supplied through the neutral text service contract.
///
/// 自有的四字节标签和值，不携带输入借用或文本作用范围；构造此值不会校验标签、查询字体支持或应用功能。
/// 功能列表的排序、重复标签处理及实际作用范围由整形实现负责。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextOpenTypeFeature {
    /// 四个原始标签字节；此处不进行字符串转换、ASCII 校验或大小写规范化。
    pub tag: [u8; 4],
    /// 功能参数的完整 `u32` 值；含义取决于标签，并非所有功能都只接受布尔开关。
    pub value: u32,
}

impl TextOpenTypeFeature {
    /// 按值接收 `tag` 和 `value`，将二者原样组成并返回一个自有设置。
    ///
    /// 此构造器只打包数值，不校验标签、查询字体支持，也不对设置列表排序或去重。
    pub const fn new(tag: [u8; 4], value: u32) -> Self {
        Self { tag, value }
    }
}
