//! 以流式方式写出规范 JSON 字符串，并把引号、原文和转义字节交给计数 writer。
//! 对象键已提前预留完整编码长度时使用预记账入口，避免写出阶段重复占用文档预算。

use std::fmt;
use std::io::Write;

use super::super::super::write_error::CanonicalTextWriteError;
use super::CountingWriter;

/// 写出带引号的 JSON 字符串，并逐段计入实际输出字节。
pub(super) fn write_json_string<W>(
    output: &mut CountingWriter<'_, '_, W>,
    value: &str,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    output.write_counted(b"\"")?;
    write_json_string_content(value, |bytes| output.write_counted(bytes))?;
    output.write_counted(b"\"")
}

/// 写出带引号的 JSON 字符串；调用方必须已为完整编码长度预留文档预算。
pub(super) fn write_json_string_preaccounted<W>(
    output: &mut CountingWriter<'_, '_, W>,
    value: &str,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    output.write_preaccounted(b"\"")?;
    write_json_string_content(value, |bytes| output.write_preaccounted(bytes))?;
    output.write_preaccounted(b"\"")
}

/// 将 `Display` 结果作为带引号 JSON 字符串流式格式化；预算错误原样返回，格式化器拒绝则转为验证错误。
pub(super) fn write_json_display<W, T>(
    output: &mut CountingWriter<'_, '_, W>,
    value: &T,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
    T: ?Sized + fmt::Display,
{
    output.write_counted(b"\"")?;
    let (display_result, write_error) = {
        let mut writer = CanonicalJsonDisplayWriter {
            output,
            error: None,
        };
        let result = fmt::write(&mut writer, format_args!("{value}"));
        (result, writer.error.take())
    };
    if let Some(error) = write_error {
        return Err(error);
    }
    if display_result.is_err() {
        return Err(CanonicalTextWriteError::PayloadValidation {
            reason: "Display implementation rejected canonical text formatting".to_string(),
        });
    }
    output.write_counted(b"\"")
}

struct CanonicalJsonDisplayWriter<'writer, 'sink, 'budget, W: Write + ?Sized> {
    output: &'writer mut CountingWriter<'sink, 'budget, W>,
    error: Option<CanonicalTextWriteError>,
}

impl<W> fmt::Write for CanonicalJsonDisplayWriter<'_, '_, '_, W>
where
    W: Write + ?Sized,
{
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let result = write_json_string_content(value, |bytes| self.output.write_counted(bytes));
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                self.error = Some(error);
                Err(fmt::Error)
            }
        }
    }
}

// 调用方负责引号；未转义片段直接输出 UTF-8，控制字符则写为 JSON 转义序列。
fn write_json_string_content<F>(value: &str, mut write: F) -> Result<(), CanonicalTextWriteError>
where
    F: FnMut(&[u8]) -> Result<(), CanonicalTextWriteError>,
{
    let mut literal_start = 0;
    for (index, character) in value.char_indices() {
        let escaped = match character {
            '\"' => Some(b"\\\"" as &[u8]),
            '\\' => Some(b"\\\\" as &[u8]),
            '\u{08}' => Some(b"\\b" as &[u8]),
            '\u{0C}' => Some(b"\\f" as &[u8]),
            '\n' => Some(b"\\n" as &[u8]),
            '\r' => Some(b"\\r" as &[u8]),
            '\t' => Some(b"\\t" as &[u8]),
            control if control <= '\u{1F}' => {
                write(value[literal_start..index].as_bytes())?;
                let code = control as u32;
                let unicode = [
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    hex_digit((code >> 4) as u8),
                    hex_digit(code as u8),
                ];
                write(&unicode)?;
                literal_start = index + character.len_utf8();
                continue;
            }
            _ => None,
        };
        if let Some(escaped) = escaped {
            write(value[literal_start..index].as_bytes())?;
            write(escaped)?;
            literal_start = index + character.len_utf8();
        }
    }
    write(value[literal_start..].as_bytes())
}

fn hex_digit(value: u8) -> u8 {
    match value & 0x0F {
        0..=9 => b'0' + (value & 0x0F),
        digit => b'a' + (digit - 10),
    }
}
