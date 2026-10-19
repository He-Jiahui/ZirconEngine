use super::{TextDirection, TextFontRequest, TextOpenTypeFeature, TextWritingMode};

/// 一次文本形状计算的借用输入；语言、方向、OpenType 特性与字体选择共同决定结果。
///
/// 调用方应在调用期间保持文本和特性切片有效；服务返回拥有型字形结果。
///
/// # 借用与约束
/// `'a` 同时约束文本、语言、特性切片以及字体条件中的字符串借用；复制请求仍共享这些输入。
/// 请求可在调用前保存，其借用必须一直有效；构造和字段赋值本身不执行输入校验或成形。
/// 结果不携带这些输入借用。参见 [`TextLayoutService::shape`](super::TextLayoutService::shape)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextShapeRequest<'a> {
    /// 待成形的原文借用；结果的源范围以这段文本起点为零，单位为 UTF-8 字节。
    pub text: &'a str,
    /// 可选语言标签借用；此处保留原值，具体服务入口再决定规范化与校验。
    pub language: Option<&'a str>,
    /// 请求的基础方向；`Auto` 留给服务按文本解析。
    pub direction: TextDirection,
    /// 水平或纵排成形方式；后续 UI 软折行和溢出策略由布局层处理。
    pub writing_mode: TextWritingMode,
    /// 行高配置原值；`new` 只在构造时按传入字号计算初值。
    pub line_height: f32,
    /// 制表间距相对空格宽度的倍数原值，具体间距由布局消费者解析。
    pub tab_size: f32,
    /// 是否请求字距调整；最终处理与显式特性组合由具体后端实施。
    pub include_kerning: bool,
    /// 借用的 OpenType 特性列表；存储时保留调用方顺序和值。
    pub features: &'a [TextOpenTypeFeature],
    /// 按值保存的字体条件；其中家族列表和资源名仍借用 `'a` 输入。
    pub font: TextFontRequest<'a>,
}

impl<'a> TextShapeRequest<'a> {
    /// 用借用文本和字体条件创建带默认配置的请求。
    ///
    /// # 参数与返回
    /// `text` 是本次请求的原文；`font` 按值保存，其中的字符串借用继续受 `'a` 约束。
    /// 返回请求保留这些借用，默认自动解析方向并使用水平成形、字距调整和空特性列表。
    ///
    /// # 默认值与约束
    /// 行高仅在这里计算为 `font.size * 1.2`，随后修改字号不会自动重算；制表倍数初值为 `4.0`。
    /// 本构造不检查输入合法性，也不执行成形；实际消费入口为 [`TextLayoutService::shape`](super::TextLayoutService::shape)。
    pub const fn new(text: &'a str, font: TextFontRequest<'a>) -> Self {
        Self {
            text,
            language: None,
            direction: TextDirection::Auto,
            writing_mode: TextWritingMode::HorizontalTopToBottom,
            line_height: font.size * 1.2,
            tab_size: 4.0,
            include_kerning: true,
            features: &[],
            font,
        }
    }

    /// 按值替换请求的特性切片并返回请求，其余字段原样保留。
    ///
    /// `features` 的共享借用随返回请求保留，受 `'a` 约束；本方法不复制元素、合并列表或规范化特性。
    pub const fn with_features(mut self, features: &'a [TextOpenTypeFeature]) -> Self {
        self.features = features;
        self
    }
}
