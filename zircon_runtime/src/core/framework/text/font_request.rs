use super::TextRenderMode;

/// 排版调用方借用的字体选择条件；由文本服务映射到实现层样式和字体集合。
///
/// `'a` 同时约束家族列表、列表中的名称及可选资源标识的借用；复制请求只复制这些共享引用。
/// 此值不拥有字体资源，也不证明目标字体已加载、可用或覆盖待排文本。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFontRequest<'a> {
    /// 借用的家族名称列表；共享文本服务当前只将首项投影为家族名，不按整个列表逐项回退。
    pub families: &'a [&'a str],
    /// 借用的可选字体资源标识；具体解释由实现决定，字段本身不检查资源是否存在。
    pub asset: Option<&'a str>,
    /// 原始字号，默认 `16.0`；共享文本服务在整形前要求该值有限且大于零。
    pub size: f32,
    /// 字重请求，默认 `400`；此数据对象不校验或裁剪数值范围。
    pub weight: u16,
    /// 伸展请求，默认 `100`；共享文本服务当前不将它投影到整形样式，赋值不代表已生效。
    pub stretch: u16,
    /// 是否请求斜体，默认 `false`；实际匹配的字体由服务实现选择。
    pub italic: bool,
    /// 渲染策略请求，默认 `Auto`；可经 [`super::TextLayoutService::resolve_render_mode`] 单独解析。
    /// 字段本身不持有光栅资源，具体整形与渲染处理仍由实现决定。
    pub render_mode: TextRenderMode,
}

impl Default for TextFontRequest<'_> {
    /// 构造空家族列表、无资源标识、字号 `16.0`、字重 `400`、伸展 `100`、非斜体及 `Auto` 策略。
    ///
    /// 返回仅含选择条件的请求，不加载字体或验证其可用性；默认值也不证明字体匹配成功。
    fn default() -> Self {
        Self {
            families: &[],
            asset: None,
            size: 16.0,
            weight: 400,
            stretch: 100,
            italic: false,
            render_mode: TextRenderMode::Auto,
        }
    }
}
