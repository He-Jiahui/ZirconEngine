use std::borrow::Cow;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

/// 线格式允许的 schema 身份最大字节数；语法本身只接受 ASCII，因此字节数也等于字符数。
pub const MAX_SCHEMA_ID_BYTES: usize = 128;

/// Stable identity for one versioned payload family.
/// 构造和反序列化共用便携的 ASCII 命名语法；已写出的标识须跨版本保持稳定。
///
/// # 约束
/// 标识由至少两个以点分隔的片段组成；片段以小写 ASCII 字母开头，后续只含小写字母、
/// 数字或非末尾连字符，整体不超过 [`MAX_SCHEMA_ID_BYTES`] 字节。持久数据写出后，调用方
/// 应继续使用同一身份读取该数据。
///
/// # 参考
/// [`VersionedSchema`](super::VersionedSchema) 将该值作为载荷族身份；版本化 loader 用它拒绝
/// 与目标类型不匹配的 envelope。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SchemaId(Cow<'static, str>);

impl SchemaId {
    /// 在编译期或运行期构造静态 schema 身份；非法字面量会触发 panic。
    ///
    /// # 参数
    /// - `value`：须符合 [`SchemaId`] 的便携命名语法。
    ///
    /// # 返回值
    /// 返回借用静态字符串的身份值，无需为内层字符串分配内存。
    ///
    /// # Panics
    /// 当 `value` 不符合 [`SchemaId`] 的语法或超过 [`MAX_SCHEMA_ID_BYTES`] 字节时触发；
    /// 若在常量上下文调用，该非法值会使常量求值失败。
    pub const fn new(value: &'static str) -> Self {
        match validate_schema_id(value) {
            Ok(()) => Self(Cow::Borrowed(value)),
            Err(_) => panic!("invalid SchemaId literal"),
        }
    }

    /// 返回身份的规范字符串，用于序列化、诊断和 schema 匹配。
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<String> for SchemaId {
    type Error = SchemaIdError;

    /// 校验并取得已有字符串的所有权，供反序列化等动态输入路径使用。
    ///
    /// # 参数
    /// - `value`：待校验的拥有型 schema 名称。
    ///
    /// # 返回值
    /// 成功时保留该字符串；不符合语法或字节上限时返回带有错误类别和字节位置的
    /// [`SchemaIdError`]。
    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate_schema_id(&value)?;
        Ok(Self(Cow::Owned(value)))
    }
}

impl TryFrom<&str> for SchemaId {
    type Error = SchemaIdError;

    /// 校验动态字符串切片并创建拥有型身份。
    ///
    /// # 参数
    /// - `value`：待校验的 schema 名称；该切片不必具有静态生命周期。
    ///
    /// # 返回值
    /// 返回持有名称副本的身份值，或指出首个不合法位置的 [`SchemaIdError`]。
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_schema_id(value)?;
        Ok(Self(Cow::Owned(value.to_string())))
    }
}

impl<'de> Deserialize<'de> for SchemaId {
    /// 先读取拥有型字符串，再套用与 [`SchemaId::new`] 相同的命名校验。
    /// 返回值不借用反序列化输入缓冲区，之后可独立于该输入继续使用。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_from(value).map_err(D::Error::custom)
    }
}

/// Schema 身份语法或长度检查失败的具体原因；其中 `index` 以输入字节为单位。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaIdError {
    /// 名称为空。
    Empty,
    /// 名称超过上限；`found` 与 `max` 都以字节计。
    TooLong { max: usize, found: usize },
    /// 名称没有至少两个命名空间片段。
    MissingNamespace,
    /// 点分隔符两侧出现空片段；`index` 是该片段起点或输入末尾。
    EmptySegment { index: usize },
    /// 片段首字节不是小写 ASCII 字母。
    InvalidSegmentStart { index: usize, found: char },
    /// 片段以连字符结束。
    InvalidSegmentEnd { index: usize, found: char },
    /// ASCII 输入中出现片段语法不支持的字符；非 ASCII 输入由 [`NonAscii`](Self::NonAscii) 报告。
    InvalidCharacter { index: usize, found: char },
    /// 输入包含非 ASCII 数据；`index` 指向首个非 ASCII 字符的起始字节。
    NonAscii { index: usize },
}

impl fmt::Display for SchemaIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("schema ID cannot be empty"),
            Self::TooLong { max, found } => {
                write!(formatter, "schema ID exceeds {max} bytes (found {found})")
            }
            Self::MissingNamespace => {
                formatter.write_str("schema ID must contain at least two namespace segments")
            }
            Self::EmptySegment { index } => {
                write!(formatter, "schema ID has an empty segment at byte {index}")
            }
            Self::InvalidSegmentStart { index, found } => write!(
                formatter,
                "schema ID segment at byte {index} must start with a lowercase ASCII letter, found {found:?}"
            ),
            Self::InvalidSegmentEnd { index, found } => write!(
                formatter,
                "schema ID segment cannot end with {found:?} at byte {index}"
            ),
            Self::InvalidCharacter { index, found } => write!(
                formatter,
                "schema ID contains invalid character {found:?} at byte {index}"
            ),
            Self::NonAscii { index } => {
                write!(formatter, "schema ID contains non-ASCII data at byte {index}")
            }
        }
    }
}

impl std::error::Error for SchemaIdError {}

const fn validate_schema_id(value: &str) -> Result<(), SchemaIdError> {
    let bytes = value.as_bytes();
    // 先拒绝空值和超长输入，再逐字节解析，保证后续错误索引始终对应 wire 字节位置。
    if bytes.is_empty() {
        return Err(SchemaIdError::Empty);
    }
    if bytes.len() > MAX_SCHEMA_ID_BYTES {
        return Err(SchemaIdError::TooLong {
            max: MAX_SCHEMA_ID_BYTES,
            found: bytes.len(),
        });
    }

    let mut index = 0;
    let mut segment_start = true;
    let mut segment_ends_with_hyphen = false;
    let mut has_namespace_separator = false;
    while index < bytes.len() {
        let byte = bytes[index];
        // 点号结束一个非空片段；连字符不能成为片段末字符，下一轮仍从新片段起点校验。
        if !byte.is_ascii() {
            return Err(SchemaIdError::NonAscii { index });
        }
        if byte == b'.' {
            if segment_start {
                return Err(SchemaIdError::EmptySegment { index });
            }
            if segment_ends_with_hyphen {
                return Err(SchemaIdError::InvalidSegmentEnd {
                    index: index - 1,
                    found: '-',
                });
            }
            has_namespace_separator = true;
            segment_start = true;
            segment_ends_with_hyphen = false;
            index += 1;
            continue;
        }
        // 每个片段的首字节单独限制为小写字母，后续扫描只接受字母、数字和连字符。
        if segment_start {
            if byte < b'a' || byte > b'z' {
                return Err(SchemaIdError::InvalidSegmentStart {
                    index,
                    found: byte as char,
                });
            }
            segment_start = false;
            index += 1;
            continue;
        }
        if (byte >= b'a' && byte <= b'z') || (byte >= b'0' && byte <= b'9') {
            segment_ends_with_hyphen = false;
        } else if byte == b'-' {
            segment_ends_with_hyphen = true;
        } else {
            return Err(SchemaIdError::InvalidCharacter {
                index,
                found: byte as char,
            });
        }
        index += 1;
    }

    // 收尾检查处理尾随点/连字符，并确保名称确实带有命名空间分隔符。
    if segment_start {
        return Err(SchemaIdError::EmptySegment { index: bytes.len() });
    }
    if segment_ends_with_hyphen {
        return Err(SchemaIdError::InvalidSegmentEnd {
            index: bytes.len() - 1,
            found: '-',
        });
    }
    if !has_namespace_separator {
        return Err(SchemaIdError::MissingNamespace);
    }
    Ok(())
}
