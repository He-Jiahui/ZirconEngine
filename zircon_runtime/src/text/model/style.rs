use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::shaped_run::OpenTypeFeature;

/// 行内对齐策略；Start/End 随解析后的书写方向决定起止侧，Left/Right 保持物理侧语义。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    /// 物理左对齐，不随书写方向改侧；也是默认值。
    #[default]
    Left,
    /// 请求行内内容居中。
    Center,
    /// 物理右对齐，不随书写方向改侧。
    Right,
    /// 随解析后的书写方向选择起侧。
    Start,
    /// 随解析后的书写方向选择结束侧。
    End,
    /// 请求两端对齐，具体伸展由布局阶段决定。
    Justify,
}

/// 软折行策略；硬分隔符仍独立切分物理段落，WordSmart 再约束词尾标点的断行位置。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextWrap {
    /// 不请求软折行；硬分隔符仍由段落或物理行处理。
    None,
    /// 使用普通词边界折行，也是默认值。
    #[default]
    Word,
    /// 在词边界策略上增加词尾标点的断行约束。
    WordSmart,
    /// 选择按整形结果校正边界的逐字形折行路径。
    Glyph,
}

/// 富文本解析与缓存身份中的版本化语法选择；格式版本是编译产物语义的一部分。
/// 调用方须显式选择受支持的子集，不能把外部任意 HTML 或 Markdown 当成完整文档语法。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextFormat {
    /// 纯文本身份，不选择标记语法解析器；也是默认值。
    #[default]
    Plain,
    /// 第 1 版行内 Markdown 子集的解析与工件身份。
    #[serde(rename = "markdown_inline_v1")]
    MarkdownInlineV1,
    /// 第 1 版 BBCode 子集的解析与工件身份。
    #[serde(rename = "bbcode_v1")]
    BbCodeV1,
    /// 第 1 版受限 HTML 语法的解析与工件身份。
    #[serde(rename = "html_subset_v1")]
    HtmlSubsetV1,
}

/// UI 样式投影后供测量、整形和字形产物共同消费的中性文本样式。
/// 可序列化字段不表示已经规范化；语言、字体选择、特性与几何约束在请求入口继续校验。
/// 样式拥有字符串并共享特性切片；默认构造、字段补缺省和请求规范化分别发生。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TextStyle {
    /// 可选字体资源身份原值，不能套用家族名称的空白规范化规则。
    pub font: Option<String>,
    /// 可选家族名原值；字体查询与整形消费者另取去空白的非空视图。
    pub font_family: Option<String>,
    /// 可选语言标记原值；规范化与有效性检查由具体请求入口决定。
    pub language: Option<String>,
    /// 字重原值；构造、赋值或反序列化不会自动调用限幅函数。
    pub font_weight: u16,
    /// 斜体请求；缺字段的反序列化输入显式按 false 补全。
    #[serde(default)]
    pub italic: bool,
    /// 共享的 OpenType 特性原列表；缺字段时为空，不在此排序或消除重复。
    #[serde(default)]
    pub features: Arc<[OpenTypeFeature]>,
    /// 字号原值；有限性与正值等约束由具体请求入口决定。
    pub font_size: f32,
    /// 行高原值；默认构造使用字号比例，具体消费者可再施加下限。
    pub line_height: f32,
    /// 制表间距相对空格宽度的倍数原值；有效间距由布局消费者解析。
    pub tab_size: f32,
    /// 行内对齐请求；起止侧仍需结合解析后的书写方向。
    pub text_align: TextAlign,
    /// 软折行请求，不消除输入中的硬分隔符。
    pub wrap: TextWrap,
}

impl TextStyle {
    pub(crate) const DEFAULT_FONT_SIZE: f32 = 16.0;
    pub(crate) const DEFAULT_FONT_WEIGHT: u16 = 400;
    pub(crate) const DEFAULT_LINE_HEIGHT_SCALE: f32 = 1.2;
    pub(crate) const DEFAULT_TAB_SIZE: f32 = 4.0;
    pub(crate) const MIN_FONT_WEIGHT: u16 = 1;
    pub(crate) const MAX_FONT_WEIGHT: u16 = 1000;

    /// 按输入字号乘以默认比例返回行高；不更改输入，也不校验有限性或正值。
    /// 默认构造显式调用此函数；外部给出的行高不会因此自动重算。
    pub(crate) fn default_line_height(font_size: f32) -> f32 {
        font_size * Self::DEFAULT_LINE_HEIGHT_SCALE
    }

    /// 返回位于 `1..=1000` 闭区间的字重副本；超界值取相应端点，不改写样式。
    /// 字体查询、整形与缓存键在需要时显式调用；样式构造或反序列化不触发它。
    pub(crate) const fn normalized_font_weight(font_weight: u16) -> u16 {
        if font_weight < Self::MIN_FONT_WEIGHT {
            Self::MIN_FONT_WEIGHT
        } else if font_weight > Self::MAX_FONT_WEIGHT {
            Self::MAX_FONT_WEIGHT
        } else {
            font_weight
        }
    }
}

/// 返回固定默认参数组成的样式；资源身份为空，特性为空，对齐与折行使用各自默认变体。
/// 这里只构造值，不解析字体，也不验证资源、语言或几何参数。
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: None,
            font_family: None,
            language: None,
            font_weight: Self::DEFAULT_FONT_WEIGHT,
            italic: false,
            features: Arc::from([]),
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::default_line_height(Self::DEFAULT_FONT_SIZE),
            tab_size: Self::DEFAULT_TAB_SIZE,
            text_align: TextAlign::default(),
            wrap: TextWrap::default(),
        }
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
