use super::{
    TextDirection, TextFontRequest, TextLayoutError, TextRenderMode, TextShapeRequest,
    TextShapeResult,
};

/// 对外的后端中立排版接口；具体字体集合和 shaping backend 留在 text 域。
///
/// 调用方传入借用请求，取得可供布局和渲染使用的拥有型结果或类型化错误。
///
/// 方法同步接收调用期借用；`shape` 的请求虽按值传入，其中的文本与字体条件仍是借用。
/// 返回的结果或错误不携带这些输入的生命周期；拥有结果也不等于持有字体资源的存活凭证。
/// `Send + Sync` 允许服务跨线程转移和共享，不承诺自动并行执行或无锁访问。
/// 两个解析方法可独立调用，具体实现的默认策略不构成所有实现必须采用的算法。
pub trait TextLayoutService: Send + Sync {
    /// 根据字体条件查询渲染模式；`self` 与 `request` 均在调用期借用，返回枚举不携带输入借用。
    ///
    /// 共享文本服务将 `Auto` 解析为 `Native`，其余模式保持原值；这是该实现的策略。
    fn resolve_render_mode(&self, request: &TextFontRequest<'_>) -> TextRenderMode;

    /// 根据借用的 `text` 和按值传入的 `requested` 查询基础方向，返回值不携带文本借用。
    ///
    /// 基础方向不等于逐字形视觉顺序；`Auto`、`Mixed` 的解析方式由实现约定，
    /// 也不能用此查询替代实际整形结果中的方向信息。
    fn resolve_direction(&self, text: &str, requested: TextDirection) -> TextDirection;

    /// 将一次请求投影到中立字形和度量；实现应在同一字体代际内完成结果投影。
    ///
    /// 按值接收包含借用的 `request`；调用期间可读取输入，返回自有字形、度量或类型化错误。
    /// 成功结果中的源区间使用本次请求文本的 UTF-8 字节坐标与半开范围，不是字符序号。
    /// 实现需维持上述字体代际投影约束；字体覆盖、后续光栅化与界面换行不由此返回类型保证。
    fn shape(&self, request: TextShapeRequest<'_>) -> Result<TextShapeResult, TextLayoutError>;
}
