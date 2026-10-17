//! 原生绑定语法和绑定值准入错误的公共分类。
//!
//! 解析器在相应分支构造这些错误，Editor 转换层继续传递；类型本身不持有输入借用。
//! 构造器诊断拥有名称与原因，值错误通过自动转换保留原有分类。

use thiserror::Error;

use super::UiBindingValueValidationError;

/// 原生绑定文本的解析错误；构造值校验失败保留原始错误，Editor 绑定转换层直接透传此类型。
///
/// # 返回边界
/// 错误描述语法或值准入失败，不代表路由查找与动作执行结果，也不携带输入位置。
/// 格式化文本由错误派生实现生成；拥有的诊断字段可在输入文本释放后保存。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingParseError {
    /// 路由头中未找到事件分隔符。
    #[error("binding is missing event separator ':'")]
    MissingEventSeparator,
    /// 事件分隔符之前未找到控件分隔符。
    #[error("binding is missing control separator '/'")]
    MissingControlSeparator,
    /// 视图或控件标识为空。
    #[error("binding contains empty view or control id")]
    EmptyPathSegment,
    /// 完成可选动作后仍有未消费的非空白内容。
    #[error("binding contains unexpected trailing input")]
    TrailingInput,
    /// 事件名称不在静态事件种类映射中，载荷保留原名称。
    #[error("unknown ui event kind {0}")]
    UnknownEventKind(String),
    /// 需要下一个字符或值时已到输入末尾。
    #[error("unexpected end of input")]
    UnexpectedEnd,
    /// 指定字符与当前字符不同；两个字符均按值保留。
    #[error("expected '{expected}' but found '{found}'")]
    ExpectedCharacter { expected: char, found: char },
    /// 动作或构造器符号位置未消费到允许的字符。
    #[error("invalid binding call symbol")]
    InvalidCallSymbol,
    /// 数字格式、数字类型转换或当前值起始形式不能被此入口接受。
    #[error("invalid numeric literal")]
    InvalidNumber,
    /// 字符串在关闭引号之前结束。
    #[error("unterminated string literal")]
    UnterminatedString,
    /// 转义标记之后缺少字符或该转义未受支持。
    #[error("invalid escape sequence")]
    InvalidEscape,
    /// 构造器名称、参数数量、参数类型或领域约束不受接受。
    #[error("invalid binding value constructor `{constructor}`: {reason}")]
    InvalidValueConstructor { constructor: String, reason: String },
    /// 已构造参数值的校验失败；自动转换保留原始值错误。
    #[error(transparent)]
    InvalidValue(#[from] UiBindingValueValidationError),
}
