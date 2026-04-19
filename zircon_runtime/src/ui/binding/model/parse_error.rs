use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingParseError {
    #[error("binding is missing event separator ':'")]
    MissingEventSeparator,
    #[error("binding is missing control separator '/'")]
    MissingControlSeparator,
    #[error("binding contains empty view or control id")]
    EmptyPathSegment,
    #[error("binding contains unexpected trailing input")]
    TrailingInput,
    #[error("unknown ui event kind {0}")]
    UnknownEventKind(String),
    #[error("unexpected end of input")]
    UnexpectedEnd,
    #[error("expected '{expected}' but found '{found}'")]
    ExpectedCharacter { expected: char, found: char },
    #[error("invalid binding call symbol")]
    InvalidCallSymbol,
    #[error("invalid numeric literal")]
    InvalidNumber,
    #[error("unterminated string literal")]
    UnterminatedString,
    #[error("invalid escape sequence")]
    InvalidEscape,
}
