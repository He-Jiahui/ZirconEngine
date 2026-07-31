//! Bounded-memory structural evidence for transaction artifacts.
//!
//! The migration stage already produces canonical TOML and persists its digest
//! before a live target can change. Recovery must still reject forged artifact
//! bytes, but it cannot retain a whole 1 GiB artifact merely to inspect it.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub(super) fn stream_toml_file_digest(path: &Path, buffer_bytes: usize) -> io::Result<String> {
    TomlEvidenceReader::new(buffer_bytes).stream_file_digest(path)
}

/// Reuses one bounded I/O buffer while recovery verifies multiple artifacts.
pub(super) struct TomlEvidenceReader {
    buffer: Vec<u8>,
}

impl TomlEvidenceReader {
    pub(super) fn new(buffer_bytes: usize) -> Self {
        Self {
            buffer: vec![0_u8; buffer_bytes.max(1)],
        }
    }

    pub(super) fn stream_file_digest(&mut self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = blake3::Hasher::new();
        let mut validator = TomlStructureValidator::default();

        loop {
            let read = file.read(&mut self.buffer)?;
            if read == 0 {
                break;
            }
            let chunk = &self.buffer[..read];
            hasher.update(chunk);
            validator.push(chunk)?;
        }

        validator.finish()?;
        Ok(hasher.finalize().to_hex().to_string())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum LineKind {
    #[default]
    Empty,
    Assignment,
    HeaderOpen,
    HeaderClosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringKind {
    OpeningBasic { quotes: u8 },
    Basic { escaped: bool },
    MultilineBasic { escaped: bool, quotes: u8 },
    OpeningLiteral { quotes: u8 },
    Literal,
    MultilineLiteral { quotes: u8 },
}

/// Validates the structural subset that recovery needs before trusting a
/// digest-bound artifact. Typed document parsing remains the authority when a
/// migration document is first read; this scanner prevents recovery from
/// accepting arbitrary non-TOML bytes without retaining the payload.
#[derive(Default)]
struct TomlStructureValidator {
    line_kind: LineKind,
    line_has_content: bool,
    array_depth: u32,
    inline_table_depth: u32,
    string: Option<StringKind>,
    string_closed: bool,
    comment: bool,
    utf8_tail: [u8; 3],
    utf8_tail_len: usize,
}

impl TomlStructureValidator {
    fn push(&mut self, mut bytes: &[u8]) -> io::Result<()> {
        if self.utf8_tail_len > 0 {
            let tail_len = self.utf8_tail_len;
            let mut joined = [0_u8; 4];
            joined[..tail_len].copy_from_slice(&self.utf8_tail[..tail_len]);
            let needed = (4 - tail_len).min(bytes.len());
            joined[tail_len..tail_len + needed].copy_from_slice(&bytes[..needed]);
            let expected = utf8_sequence_len(joined[0])?;
            if tail_len + needed < expected {
                self.utf8_tail[tail_len..tail_len + needed].copy_from_slice(&bytes[..needed]);
                self.utf8_tail_len = tail_len + needed;
                return Ok(());
            }
            std::str::from_utf8(&joined[..expected])
                .map_err(|error| invalid_data(error.to_string()))?;
            for byte in &joined[..expected] {
                self.push_valid_byte(*byte)?;
            }
            self.utf8_tail_len = 0;
            bytes = &bytes[expected - tail_len..];
        }

        while !bytes.is_empty() {
            match std::str::from_utf8(bytes) {
                Ok(_) => {
                    for byte in bytes {
                        self.push_valid_byte(*byte)?;
                    }
                    break;
                }
                Err(error) => {
                    let valid = error.valid_up_to();
                    for byte in &bytes[..valid] {
                        self.push_valid_byte(*byte)?;
                    }
                    let Some(error_len) = error.error_len() else {
                        let tail = &bytes[valid..];
                        if tail.len() > self.utf8_tail.len() {
                            return Err(invalid_data("incomplete UTF-8 sequence"));
                        }
                        self.utf8_tail[..tail.len()].copy_from_slice(tail);
                        self.utf8_tail_len = tail.len();
                        break;
                    };
                    return Err(invalid_data(format!(
                        "invalid UTF-8 sequence of {error_len} byte(s)"
                    )));
                }
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        if self.utf8_tail_len > 0 {
            return Err(invalid_data("incomplete UTF-8 sequence at end of file"));
        }
        match self.string.take() {
            Some(StringKind::OpeningBasic { quotes: 2 })
            | Some(StringKind::OpeningLiteral { quotes: 2 })
            | None => {}
            Some(_) => return Err(invalid_data("unterminated TOML string")),
        }
        if self.array_depth != 0 || self.inline_table_depth != 0 {
            return Err(invalid_data("unterminated TOML container"));
        }
        self.finish_line()
    }

    fn push_valid_byte(&mut self, byte: u8) -> io::Result<()> {
        if self.string.is_some() {
            return self.push_string_byte(byte);
        }
        if byte == b'\n' {
            self.comment = false;
            if self.array_depth == 0 && self.inline_table_depth == 0 {
                self.finish_line()?;
            }
            return Ok(());
        }
        if self.comment {
            return Ok(());
        }
        if self.string_closed
            && !byte.is_ascii_whitespace()
            && !matches!(byte, b'#' | b',' | b']' | b'}' | b'=' | b'.')
        {
            return Err(invalid_data("unexpected token after TOML string"));
        }
        if byte == b'#' {
            self.comment = true;
            return Ok(());
        }
        if !byte.is_ascii_whitespace() {
            self.line_has_content = true;
        }
        if matches!(byte, b',' | b']' | b'}' | b'=' | b'.') {
            self.string_closed = false;
        }
        match byte {
            b'"' => {
                self.string_closed = false;
                self.string = Some(StringKind::OpeningBasic { quotes: 1 });
            }
            b'\'' => {
                self.string_closed = false;
                self.string = Some(StringKind::OpeningLiteral { quotes: 1 });
            }
            b'=' if self.line_kind == LineKind::Empty => self.line_kind = LineKind::Assignment,
            b'=' if self.line_kind != LineKind::Assignment => {
                return Err(invalid_data(
                    "TOML table header cannot contain an assignment",
                ));
            }
            b'[' if self.line_kind == LineKind::Empty && self.line_has_content => {
                self.line_kind = LineKind::HeaderOpen;
                self.array_depth = self
                    .array_depth
                    .checked_add(1)
                    .ok_or_else(|| invalid_data("TOML array nesting overflow"))?;
            }
            b'[' => {
                self.array_depth = self
                    .array_depth
                    .checked_add(1)
                    .ok_or_else(|| invalid_data("TOML array nesting overflow"))?
            }
            b']' => {
                self.array_depth = self
                    .array_depth
                    .checked_sub(1)
                    .ok_or_else(|| invalid_data("unmatched TOML array close"))?;
                if self.line_kind == LineKind::HeaderOpen && self.array_depth == 0 {
                    self.line_kind = LineKind::HeaderClosed;
                }
            }
            b'{' => {
                self.inline_table_depth = self
                    .inline_table_depth
                    .checked_add(1)
                    .ok_or_else(|| invalid_data("TOML inline-table nesting overflow"))?
            }
            b'}' => {
                self.inline_table_depth = self
                    .inline_table_depth
                    .checked_sub(1)
                    .ok_or_else(|| invalid_data("unmatched TOML inline-table close"))?
            }
            _ if self.line_kind == LineKind::HeaderClosed && !byte.is_ascii_whitespace() => {
                return Err(invalid_data("unexpected data after TOML table header"));
            }
            _ => {}
        }
        Ok(())
    }

    fn push_string_byte(&mut self, byte: u8) -> io::Result<()> {
        let Some(kind) = self.string.take() else {
            return Err(invalid_data("TOML string state was lost"));
        };
        match kind {
            StringKind::OpeningBasic { quotes } if byte == b'"' && quotes == 1 => {
                self.string = Some(StringKind::OpeningBasic { quotes: 2 });
            }
            StringKind::OpeningBasic { quotes: 2 } if byte == b'"' => {
                self.string = Some(StringKind::MultilineBasic {
                    escaped: false,
                    quotes: 0,
                });
            }
            StringKind::OpeningBasic { quotes: 2 } => {
                self.string_closed = true;
                return self.push_valid_byte(byte);
            }
            StringKind::OpeningBasic { .. } => {
                self.string = Some(StringKind::Basic { escaped: false });
                return self.push_string_byte(byte);
            }
            StringKind::Basic { .. } if byte == b'\n' => {
                return Err(invalid_data("newline in single-line TOML string"));
            }
            StringKind::Basic { escaped: true } => {
                self.string = Some(StringKind::Basic { escaped: false });
            }
            StringKind::Basic { escaped: false } if byte == b'\\' => {
                self.string = Some(StringKind::Basic { escaped: true });
            }
            StringKind::Basic { escaped: false } if byte != b'"' => {
                self.string = Some(StringKind::Basic { escaped: false });
            }
            StringKind::Basic { escaped: false } => self.string_closed = true,
            StringKind::OpeningLiteral { quotes } if byte == b'\'' && quotes == 1 => {
                self.string = Some(StringKind::OpeningLiteral { quotes: 2 });
            }
            StringKind::OpeningLiteral { quotes: 2 } if byte == b'\'' => {
                self.string = Some(StringKind::MultilineLiteral { quotes: 0 });
            }
            StringKind::OpeningLiteral { quotes: 2 } => {
                self.string_closed = true;
                return self.push_valid_byte(byte);
            }
            StringKind::OpeningLiteral { .. } => {
                self.string = Some(StringKind::Literal);
                return self.push_string_byte(byte);
            }
            StringKind::Literal if byte == b'\n' => {
                return Err(invalid_data("newline in single-line TOML string"));
            }
            StringKind::Literal if byte != b'\'' => {
                self.string = Some(StringKind::Literal);
            }
            StringKind::Literal => self.string_closed = true,
            StringKind::MultilineBasic { escaped: true, .. } => {
                self.string = Some(StringKind::MultilineBasic {
                    escaped: false,
                    quotes: 0,
                });
            }
            StringKind::MultilineBasic { quotes, .. } if byte == b'\\' => {
                self.string = Some(StringKind::MultilineBasic {
                    escaped: true,
                    quotes: 0,
                });
            }
            StringKind::MultilineBasic { quotes, .. } if byte == b'"' && quotes < 2 => {
                self.string = Some(StringKind::MultilineBasic {
                    escaped: false,
                    quotes: quotes + 1,
                });
            }
            StringKind::MultilineBasic { .. } if byte == b'"' => self.string_closed = true,
            StringKind::MultilineBasic { .. } => {
                self.string = Some(StringKind::MultilineBasic {
                    escaped: false,
                    quotes: 0,
                });
            }
            StringKind::MultilineLiteral { quotes } if byte == b'\'' && quotes < 2 => {
                self.string = Some(StringKind::MultilineLiteral { quotes: quotes + 1 });
            }
            StringKind::MultilineLiteral { .. } if byte == b'\'' => self.string_closed = true,
            StringKind::MultilineLiteral { .. } => {
                self.string = Some(StringKind::MultilineLiteral { quotes: 0 });
            }
        }
        Ok(())
    }

    fn finish_line(&mut self) -> io::Result<()> {
        if self.line_has_content && self.line_kind == LineKind::Empty {
            return Err(invalid_data(
                "TOML statement is missing an assignment or table header",
            ));
        }
        if self.line_kind == LineKind::HeaderOpen {
            return Err(invalid_data("unterminated TOML table header"));
        }
        self.line_kind = LineKind::Empty;
        self.line_has_content = false;
        self.string_closed = false;
        Ok(())
    }
}

fn utf8_sequence_len(first: u8) -> io::Result<usize> {
    match first {
        0x00..=0x7f => Ok(1),
        0xc2..=0xdf => Ok(2),
        0xe0..=0xef => Ok(3),
        0xf0..=0xf4 => Ok(4),
        _ => Err(invalid_data("invalid UTF-8 leading byte")),
    }
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_TEST_ARTIFACT_ID: AtomicU64 = AtomicU64::new(1);

    fn digest(source: &[u8], chunk_bytes: usize) -> io::Result<String> {
        let artifact_id = NEXT_TEST_ARTIFACT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "zircon-toml-evidence-{}-{}-{}",
            std::process::id(),
            artifact_id,
            chunk_bytes
        ));
        std::fs::write(&path, source)?;
        let result = stream_toml_file_digest(&path, chunk_bytes);
        let _ = std::fs::remove_file(path);
        result
    }

    #[test]
    fn transaction_toml_evidence_streams_chunked_valid_document() {
        let source = b"version = 2\nname = \"hero\"\n\n[shader]\nuuid = \"abc\"\n";
        assert_eq!(
            digest(source, 1).unwrap(),
            blake3::hash(source).to_hex().to_string()
        );
    }

    #[test]
    fn transaction_toml_evidence_reuses_one_reader_for_multiple_artifacts() {
        let artifact_id = NEXT_TEST_ARTIFACT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "zircon-toml-evidence-reader-{}-{artifact_id}",
            std::process::id(),
        ));
        let mut reader = TomlEvidenceReader::new(2);

        std::fs::write(&path, b"name = \"first\"\n").unwrap();
        assert_eq!(
            reader.stream_file_digest(&path).unwrap(),
            blake3::hash(b"name = \"first\"\n").to_hex().to_string()
        );
        std::fs::write(&path, b"name = \"second\"\n").unwrap();
        assert_eq!(
            reader.stream_file_digest(&path).unwrap(),
            blake3::hash(b"name = \"second\"\n").to_hex().to_string()
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn transaction_toml_evidence_keeps_payload_reads_bounded() {
        const SOURCE: &str = include_str!("toml_evidence.rs");

        assert!(SOURCE.contains("buffer_bytes.max(1)"));
        for forbidden in ["read_to_end(", "read_to_string(", "fs::read("] {
            assert!(
                !SOURCE.contains(forbidden),
                "transaction evidence must not reintroduce whole-payload {forbidden}"
            );
        }
    }

    #[test]
    fn transaction_toml_evidence_rejects_forged_non_toml_artifact() {
        let error = digest(b"attacker-controlled backup bytes", 7).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn transaction_toml_evidence_rejects_unclosed_containers_and_strings() {
        assert!(digest(b"value = [1, 2", 3).is_err());
        assert!(digest(b"value = \"unterminated", 5).is_err());
    }

    #[test]
    fn transaction_toml_evidence_rejects_tokens_after_closed_strings() {
        assert!(digest(b"empty = \"\" trailing\n", 1).is_err());
        assert!(digest(b"label = \"valid\" trailing\n", 1).is_err());
        assert!(digest(b"summary = \"\"\"\nline\n\"\"\" trailing\n", 1).is_err());
    }

    #[test]
    fn transaction_toml_evidence_preserves_quoted_dotted_keys() {
        let source = b"\"build\".\"target\" = \"valid\"\n";
        assert_eq!(
            digest(source, 1).unwrap(),
            blake3::hash(source).to_hex().to_string()
        );
    }

    #[test]
    fn transaction_toml_evidence_preserves_split_utf8_strings() {
        let source = "name = \"Zircon 渲染\"\n".as_bytes();
        assert_eq!(
            digest(source, 2).unwrap(),
            blake3::hash(source).to_hex().to_string()
        );
    }

    #[test]
    fn transaction_toml_evidence_preserves_chunked_multiline_strings() {
        let source =
            b"summary = \"\"\"\nfirst line\nsecond line\n\"\"\"\nlabel = '''\nliteral line\n'''\n";
        assert_eq!(
            digest(source, 1).unwrap(),
            blake3::hash(source).to_hex().to_string()
        );
    }

    #[test]
    fn transaction_toml_evidence_preserves_empty_and_escaped_string_delimiters() {
        let source = b"empty_basic = \"\"\nempty_literal = ''\nquoted = \"a \\\" quote\"\nmultiline = \"\"\"\nescaped \\\"\\\"\\\" delimiter\n\"\"\"\n";
        assert!(toml::from_str::<toml::Value>(std::str::from_utf8(source).unwrap()).is_ok());
        assert_eq!(
            digest(source, 1).unwrap(),
            blake3::hash(source).to_hex().to_string()
        );
    }

    #[test]
    fn transaction_toml_evidence_accepts_canonical_migration_output() {
        let value = toml::from_str::<toml::Value>(
            r#"
title = "Migration evidence"
enabled = true
weight = 1.25
created = 2026-07-28T12:00:00Z
tags = ["runtime", "render"]

[render]
limits = { width = 1920, height = 1080 }
"#,
        )
        .unwrap();
        let source = toml::to_string_pretty(&value).unwrap();
        assert_eq!(
            digest(source.as_bytes(), 3).unwrap(),
            blake3::hash(source.as_bytes()).to_hex().to_string()
        );
    }
}
