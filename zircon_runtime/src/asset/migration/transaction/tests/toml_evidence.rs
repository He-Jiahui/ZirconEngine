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
    const SOURCE: &str = include_str!("../toml_evidence.rs");
    let production_source = SOURCE
        .split_once("#[cfg(test)]")
        .expect("test module follows production code")
        .0;

    assert!(production_source.contains("buffer_bytes.max(1)"));
    for forbidden in ["read_to_end(", "read_to_string(", "fs::read("] {
        assert!(
            !production_source.contains(forbidden),
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
fn transaction_toml_evidence_preserves_four_basic_closing_quotes_at_eof() {
    let source = b"value = \"\"\"a\"\"\"\"";
    let parsed = toml::from_str::<toml::Value>(std::str::from_utf8(source).unwrap())
        .expect("valid multiline closing-quote TOML");
    assert_eq!(parsed["value"].as_str(), Some("a\""));
    let expected_digest = blake3::hash(source).to_hex().to_string();

    for chunk_bytes in [1, 2, 3, 7] {
        assert_eq!(digest(source, chunk_bytes).unwrap(), expected_digest);
    }
}

#[test]
fn transaction_toml_evidence_preserves_five_basic_closing_quotes_before_newline() {
    let source = b"value = \"\"\"a\"\"\"\"\"\nnext = 7\n";
    let parsed = toml::from_str::<toml::Value>(std::str::from_utf8(source).unwrap())
        .expect("valid multiline closing-quote TOML");
    assert_eq!(parsed["value"].as_str(), Some("a\"\""));
    assert_eq!(parsed["next"].as_integer(), Some(7));
    let expected_digest = blake3::hash(source).to_hex().to_string();

    for chunk_bytes in [1, 2, 3, 7] {
        assert_eq!(digest(source, chunk_bytes).unwrap(), expected_digest);
    }
}

#[test]
fn transaction_toml_evidence_preserves_four_literal_closing_quotes_before_comment() {
    let source = b"value = '''a'''' # trailing comment\n";
    let parsed = toml::from_str::<toml::Value>(std::str::from_utf8(source).unwrap())
        .expect("valid multiline closing-quote TOML");
    assert_eq!(parsed["value"].as_str(), Some("a'"));
    let expected_digest = blake3::hash(source).to_hex().to_string();

    for chunk_bytes in [1, 2, 3, 7] {
        assert_eq!(digest(source, chunk_bytes).unwrap(), expected_digest);
    }
}

#[test]
fn transaction_toml_evidence_preserves_five_literal_closing_quotes_before_array_comma() {
    let source = b"values = ['''a''''', \"tail\"]\n";
    let parsed = toml::from_str::<toml::Value>(std::str::from_utf8(source).unwrap())
        .expect("valid multiline closing-quote TOML");
    let values = parsed["values"].as_array().expect("string array");
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].as_str(), Some("a''"));
    assert_eq!(values[1].as_str(), Some("tail"));
    let expected_digest = blake3::hash(source).to_hex().to_string();

    for chunk_bytes in [1, 2, 3, 7] {
        assert_eq!(digest(source, chunk_bytes).unwrap(), expected_digest);
    }
}

#[test]
fn transaction_toml_evidence_rejects_excessive_quotes_and_tokens_after_multiline_close() {
    let sources: [&[u8]; 6] = [
        b"value = \"\"\"a\"\"\"\"\"\"\n",
        b"value = '''a''''''\n",
        b"value = \"\"\"a\"\"\"\\nmore\"\"\"\n",
        b"value = \"\"\"a\"\"\"\"\\nmore\"\"\"\n",
        b"value = \"\"\"a\"\"\"\"\"\\nmore\"\"\"\n",
        b"value = '''a'''' trailing\n",
    ];

    for source in sources {
        assert!(toml::from_str::<toml::Value>(std::str::from_utf8(source).unwrap()).is_err());
        for chunk_bytes in [1, 2, 3, 7] {
            let error = digest(source, chunk_bytes).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
    }
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
