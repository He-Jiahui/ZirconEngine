/// 文档生成时选择标题、能力和空章节输出；标题层级由渲染器钳制到 1..=6。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptHostInterfaceMarkdownOptions {
    pub title: String,
    pub heading_level: usize,
    pub include_capabilities: bool,
    pub include_empty_sections: bool,
}

impl Default for ScriptHostInterfaceMarkdownOptions {
    fn default() -> Self {
        Self {
            title: "ZrVM Host Interface".to_string(),
            heading_level: 1,
            include_capabilities: true,
            include_empty_sections: false,
        }
    }
}
