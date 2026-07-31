use std::cell::Cell;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Write};

use serde_json::{json, value::RawValue};

use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};

use super::super::binary::{MAX_BINARY_BODY_BYTES, MAX_BINARY_STRING_BYTES};
use super::super::text::canonical_writer::write_canonical_text_with_limit_for_test;
use super::super::text::wire::MAX_TEXT_DOCUMENT_BYTES;
use super::super::{
    load_versioned, write_canonical_text_to, write_versioned, write_versioned_text,
    write_versioned_text_to, CanonicalTextWriteError, Format, MigrationChain, SchemaId,
    VersionedSchema, WriteError,
};
use super::FixtureDocument;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FloatProbe {
    precise: f64,
}

impl VersionedSchema for FloatProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.float-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<FloatProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[derive(Serialize)]
struct OversizedBinaryProbe {
    chunks: Vec<String>,
}

#[derive(Serialize)]
struct OversizedTextProbe {
    contents: String,
}

struct RepeatedChunkProbe {
    repetitions: usize,
    chunk: String,
}

impl Serialize for RepeatedChunkProbe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.repetitions))?;
        for _ in 0..self.repetitions {
            sequence.serialize_element(&self.chunk)?;
        }
        sequence.end()
    }
}

impl VersionedSchema for OversizedTextProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.oversized-text-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<OversizedTextProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl VersionedSchema for OversizedBinaryProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.oversized-binary-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<OversizedBinaryProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CanonicalMapProbe {
    entries: BTreeMap<String, u32>,
}

impl VersionedSchema for CanonicalMapProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.canonical-map-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<CanonicalMapProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

struct DuplicateKeyProbe;

impl VersionedSchema for DuplicateKeyProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.duplicate-key-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<DuplicateKeyProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl Serialize for DuplicateKeyProbe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("duplicate", &1_u32)?;
        map.serialize_entry("duplicate", &2_u32)?;
        map.end()
    }
}

struct NumericKeyProbe;

impl VersionedSchema for NumericKeyProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.numeric-key-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<NumericKeyProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl Serialize for NumericKeyProbe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(&1.0_f64, "float key")?;
        map.serialize_entry("1", "string key")?;
        map.end()
    }
}

struct InvalidMapKeyProbe;

impl VersionedSchema for InvalidMapKeyProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.invalid-map-key-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<InvalidMapKeyProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl Serialize for InvalidMapKeyProbe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&[1_u8, 2], &"unsupported key")?;
        map.end()
    }
}

struct CustomSerializeFailure;

impl VersionedSchema for CustomSerializeFailure {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.custom-serialize-failure");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<CustomSerializeFailure> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl Serialize for CustomSerializeFailure {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(<S::Error as serde::ser::Error>::custom(
            "fixture serializer rejected the value",
        ))
    }
}

struct ByteProbe([u8; 3]);

impl VersionedSchema for ByteProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.byte-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<ByteProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl Serialize for ByteProbe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

#[derive(Serialize)]
struct WideIntegerProbe {
    signed: i128,
    unsigned: u128,
}

impl VersionedSchema for WideIntegerProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.wide-integer-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<WideIntegerProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[derive(Serialize)]
struct RawValueProbe {
    raw: Box<RawValue>,
}

impl VersionedSchema for RawValueProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.raw-value-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<RawValueProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

struct RawValueMapKeyProbe {
    raw: Box<RawValue>,
}

impl VersionedSchema for RawValueMapKeyProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.raw-value-map-key-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<RawValueMapKeyProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl Serialize for RawValueMapKeyProbe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.raw, &1_u8)?;
        map.end()
    }
}

struct ChunkedDisplayProbe<'counter> {
    emitted_chunks: &'counter Cell<usize>,
    total_chunks: usize,
    chunk: &'static str,
}

impl fmt::Display for ChunkedDisplayProbe<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.total_chunks {
            self.emitted_chunks.set(self.emitted_chunks.get() + 1);
            formatter.write_str(self.chunk)?;
        }
        Ok(())
    }
}

impl Serialize for ChunkedDisplayProbe<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

struct KeyHeavyProbe<'counter> {
    generated_keys: &'counter Cell<usize>,
    total_keys: usize,
}

impl Serialize for KeyHeavyProbe<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.total_keys))?;
        for index in 0..self.total_keys {
            self.generated_keys.set(self.generated_keys.get() + 1);
            let key = format!("key-{index:04}-{}", "x".repeat(24));
            map.serialize_entry(&key, &0_u8)?;
        }
        map.end()
    }
}

#[test]
fn canonical_text_writer_orders_nested_keys_and_uses_one_trailing_newline() {
    let document = FixtureDocument {
        label: "stable".to_string(),
        count: 7,
    };
    let first = write_versioned_text(&document).expect("fixture should encode");
    let second = write_versioned_text(&document).expect("fixture should encode identically");

    assert_eq!(first, second);
    assert_eq!(
        first,
        concat!(
            "{\n",
            "  \"$zircon\": {\n",
            "    \"header\": {\n",
            "      \"schema_id\": \"zircon.tests.fixture-document\",\n",
            "      \"schema_version\": 2\n",
            "    },\n",
            "    \"payload\": {\n",
            "      \"count\": 7,\n",
            "      \"label\": \"stable\"\n",
            "    }\n",
            "  }\n",
            "}\n",
        ),
        "streaming output must remain byte-identical to the canonical text wire format",
    );
    assert!(first.ends_with('\n'));
    assert!(!first.ends_with("\n\n"));
    assert!(first.find("\"header\"").unwrap() < first.find("\"payload\"").unwrap());
    assert!(first.find("\"count\"").unwrap() < first.find("\"label\"").unwrap());

    let loaded = load_versioned::<FixtureDocument>(first.as_bytes(), Format::Text).unwrap();
    assert_eq!(loaded.value, document);
    assert_eq!(loaded.migrated_from, None);
}

#[test]
fn canonical_text_writer_uses_shortest_roundtrip_float_spelling() {
    let encoded = write_versioned_text(&FloatProbe { precise: 0.1 }).unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["$zircon"]["payload"], json!({ "precise": 0.1 }));
    assert!(encoded.contains("\"precise\": 0.1"));
}

#[test]
fn canonical_text_writer_serializes_bytes_through_the_sequence_contract() {
    let encoded = write_versioned_text(&ByteProbe([0, 127, 255])).unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();

    assert_eq!(value["$zircon"]["payload"], json!([0, 127, 255]));
}

#[test]
fn text_and_binary_writers_reject_out_of_range_json_integers_consistently() {
    let document = WideIntegerProbe {
        signed: i128::from(i64::MAX) + 1,
        unsigned: u128::from(u64::MAX) + 1,
    };

    for format in [Format::Text, Format::Binary] {
        assert!(matches!(
            write_versioned(&document, format),
            Err(WriteError::PayloadEncode { .. })
        ));
    }
}

#[test]
fn text_and_binary_writers_reject_raw_value_private_marker_consistently() {
    let document = RawValueProbe {
        raw: RawValue::from_string(r#"{ "zeta": 2, "alpha": 1 }"#.to_string())
            .expect("raw JSON fixture should be valid"),
    };

    for format in [Format::Text, Format::Binary] {
        assert!(matches!(
            write_versioned(&document, format),
            Err(WriteError::PayloadValidation { reason, .. })
                if reason.contains("RawValue")
        ));
    }
}

#[test]
fn canonical_text_preaggregation_rejects_raw_value_map_keys_as_payload_validation() {
    let document = RawValueMapKeyProbe {
        raw: RawValue::from_string(r#""raw""#.to_string())
            .expect("raw JSON string fixture should be valid"),
    };

    assert!(matches!(
        write_versioned_text(&document),
        Err(WriteError::PayloadValidation { reason, .. }) if reason.contains("RawValue")
    ));
}

#[test]
fn canonical_text_preaggregation_stops_chunked_display_before_all_chunks_are_formatted() {
    const TOTAL_CHUNKS: usize = 8;
    let emitted_chunks = Cell::new(0);
    let document = ChunkedDisplayProbe {
        emitted_chunks: &emitted_chunks,
        total_chunks: TOTAL_CHUNKS,
        chunk: "0123456789abcdef",
    };

    let error = write_canonical_text_with_limit_for_test(&document, &mut io::sink(), 32)
        .expect_err("the small wire budget must reject the display stream");

    assert!(matches!(
        error,
        CanonicalTextWriteError::OutputTooLarge { max: 32, .. }
    ));
    assert!(
        emitted_chunks.get() < TOTAL_CHUNKS,
        "the writer must stop Display before it aggregates every chunk"
    );
}

#[test]
fn canonical_text_preaggregation_stops_key_generation_before_the_full_map_is_retained() {
    const TOTAL_KEYS: usize = 32;
    let generated_keys = Cell::new(0);
    let document = KeyHeavyProbe {
        generated_keys: &generated_keys,
        total_keys: TOTAL_KEYS,
    };

    let error = write_canonical_text_with_limit_for_test(&document, &mut io::sink(), 64)
        .expect_err("the small wire budget must reject the key-heavy map");

    assert!(matches!(
        error,
        CanonicalTextWriteError::OutputTooLarge { max: 64, .. }
    ));
    assert!(
        generated_keys.get() < TOTAL_KEYS,
        "the writer must reject before retaining every canonical map key"
    );
}

#[test]
fn canonical_text_preaggregation_counts_a_duplicate_map_key_only_once() {
    let expected = "{\n  \"duplicate\": 2\n}\n";
    let mut output = Vec::new();

    let written =
        write_canonical_text_with_limit_for_test(&DuplicateKeyProbe, &mut output, expected.len())
            .expect("last-write-wins replacement must reuse the retained key budget");

    assert_eq!(written, expected.len());
    assert_eq!(output, expected.as_bytes());
}

#[test]
fn current_text_writer_streams_a_borrowed_payload_with_canonical_map_order() {
    let document = CanonicalMapProbe {
        entries: BTreeMap::from([("alpha".to_string(), 1), ("zeta".to_string(), 2)]),
    };

    let encoded = write_versioned_text(&document).expect("fixture should encode");
    assert!(encoded.find("\"alpha\"").unwrap() < encoded.find("\"zeta\"").unwrap());
    assert_eq!(
        load_versioned::<CanonicalMapProbe>(encoded.as_bytes(), Format::Text)
            .unwrap()
            .value,
        document
    );

    let write_source = include_str!("../write.rs");
    let text_writer = write_source
        .split("pub fn write_versioned_text_to")
        .nth(1)
        .and_then(|body| body.split("fn encode_binary_payload_value").next())
        .expect("text and binary writers must stay separate");
    assert!(
        text_writer.contains("write_canonical_text(&document, sink)"),
        "the current text payload must stream through the canonical text owner"
    );
    assert!(
        !text_writer.contains("serde_json::to_value"),
        "current text encoding must not materialize a payload Value"
    );
    let canonical_writer = include_str!("../text/canonical_writer.rs");
    assert!(canonical_writer.contains("value.serialize(&mut serializer)"));
    assert!(canonical_writer.contains("COPY_BUFFER_BYTES"));
    assert!(!canonical_writer.contains("serde_json::to_value"));
}

#[test]
fn streaming_text_writer_matches_the_owned_text_adapter_through_short_writes() {
    let document = FixtureDocument {
        label: "streamed".to_string(),
        count: 19,
    };
    let expected = write_versioned_text(&document).expect("owned adapter should encode");
    let mut sink = ChunkedSink::new(3, None);

    assert_eq!(
        write_versioned(&document, Format::Text).expect("byte adapter should encode"),
        expected.as_bytes(),
        "write_versioned text output must remain an adapter over the streaming canonical writer"
    );

    let written = write_versioned_text_to(&document, &mut sink)
        .expect("streaming writer should retry short writes");

    assert_eq!(written, expected.len());
    assert_eq!(sink.bytes, expected.into_bytes());
}

#[test]
fn streaming_text_writer_preserves_the_downstream_write_failure() {
    let document = FixtureDocument {
        label: "failure".to_string(),
        count: 23,
    };
    let mut sink = ChunkedSink::new(2, Some(11));

    let error = write_versioned_text_to(&document, &mut sink)
        .expect_err("streaming writer must expose a sink failure");

    assert!(matches!(
        error,
        WriteError::TextWrite {
            schema_id,
            schema_version: 2,
            source,
            ..
        } if schema_id == "zircon.tests.fixture-document"
            && source.kind() == io::ErrorKind::BrokenPipe
    ));
}

#[test]
fn canonical_text_writer_matches_json_key_normalization_and_error_categories() {
    let duplicate = write_versioned_text(&DuplicateKeyProbe).unwrap();
    assert_eq!(
        duplicate.matches("\"duplicate\"").count(),
        1,
        "canonical output must emit only the last value for a duplicate key"
    );
    let duplicate: serde_json::Value = serde_json::from_str(&duplicate).unwrap();
    assert_eq!(duplicate["$zircon"]["payload"]["duplicate"], json!(2));
    assert_eq!(
        duplicate["$zircon"]["payload"].as_object().unwrap().len(),
        1
    );

    let numeric = write_versioned_text(&NumericKeyProbe).unwrap();
    let numeric: serde_json::Value = serde_json::from_str(&numeric).unwrap();
    assert_eq!(numeric["$zircon"]["payload"]["1.0"], json!("float key"));
    assert_eq!(numeric["$zircon"]["payload"]["1"], json!("string key"));

    assert!(matches!(
        write_versioned_text(&InvalidMapKeyProbe),
        Err(WriteError::PayloadEncode { .. })
    ));
    assert!(matches!(
        write_versioned_text(&CustomSerializeFailure),
        Err(WriteError::PayloadValidation { .. })
    ));
}

#[test]
fn writers_reject_non_finite_floats_instead_of_normalizing_them_to_null() {
    for precise in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for format in [Format::Text, Format::Binary] {
            let error = write_versioned(&FloatProbe { precise }, format)
                .expect_err("non-finite values must not become JSON null");

            assert!(matches!(
                error,
                WriteError::NonFiniteFloat {
                    schema_id,
                    schema_version: 0,
                    ..
                } if schema_id == "zircon.tests.float-probe"
            ));
        }
    }
}

#[test]
fn binary_writer_rejects_a_body_larger_than_the_reader_limit() {
    let document = OversizedBinaryProbe {
        chunks: (0..4)
            .map(|_| "x".repeat(MAX_BINARY_STRING_BYTES))
            .collect(),
    };

    let error = write_versioned(&document, Format::Binary)
        .expect_err("writer must not emit a body that the same wire reader rejects");

    assert!(matches!(
        error,
        WriteError::BinaryPayloadTooLarge {
            schema_id,
            schema_version: 0,
            max,
        } if schema_id == "zircon.tests.oversized-binary-probe"
            && max == MAX_BINARY_BODY_BYTES
    ));
}

#[test]
fn text_writer_rejects_a_document_larger_than_the_reader_limit() {
    let document = OversizedTextProbe {
        contents: "x".repeat(MAX_TEXT_DOCUMENT_BYTES),
    };

    let error = write_versioned_text(&document)
        .expect_err("writer must not emit a document that the text reader rejects");

    assert!(matches!(
        error,
        WriteError::TextDocumentTooLarge {
            schema_id,
            schema_version: 0,
            max,
            found,
        } if schema_id == "zircon.tests.oversized-text-probe"
            && max == MAX_TEXT_DOCUMENT_BYTES
            && found > MAX_TEXT_DOCUMENT_BYTES
    ));
}

#[test]
fn raw_streaming_writer_is_not_limited_by_the_versioned_text_wire_budget() {
    let document = OversizedTextProbe {
        contents: "x".repeat(MAX_TEXT_DOCUMENT_BYTES),
    };
    let mut sink = io::sink();

    let written = write_canonical_text_to(&document, &mut sink)
        .expect("raw archive streaming must not inherit the 64 MiB versioned text limit");

    assert!(written > MAX_TEXT_DOCUMENT_BYTES);
}

#[test]
fn raw_streaming_writer_handles_a_one_mebibyte_direct_array_without_text_aggregation() {
    let document = RepeatedChunkProbe {
        repetitions: 1,
        chunk: "x".repeat(1024 * 1024),
    };
    let mut sink = io::sink();

    let written = write_canonical_text_to(&document, &mut sink)
        .expect("direct arrays must stream without creating an owned text document");

    assert!(written > 1024 * 1024);
}

#[test]
fn raw_streaming_writer_bounds_each_sink_request_to_the_staging_size() {
    let document = RepeatedChunkProbe {
        repetitions: 1,
        chunk: "x".repeat(1024 * 1024),
    };
    let mut sink = RequestTrackingSink::default();

    write_canonical_text_to(&document, &mut sink)
        .expect("direct arrays must write through fixed-size staging requests");

    assert!(
        sink.largest_request <= 64 * 1024,
        "canonical text must not hand an unbounded string slice to its sink"
    );
}

#[test]
#[ignore = "high-volume 512 MiB streaming contract"]
fn raw_streaming_writer_handles_a_512_mebibyte_direct_array_with_fixed_staging() {
    let document = RepeatedChunkProbe {
        repetitions: 512,
        chunk: "x".repeat(1024 * 1024),
    };
    let mut sink = io::sink();

    let written = write_canonical_text_to(&document, &mut sink)
        .expect("512 MiB direct arrays must not inherit the versioned text wire limit");

    assert!(written > 512 * 1024 * 1024);
}

struct ChunkedSink {
    bytes: Vec<u8>,
    chunk_size: usize,
    fail_after: Option<usize>,
}

impl ChunkedSink {
    fn new(chunk_size: usize, fail_after: Option<usize>) -> Self {
        Self {
            bytes: Vec::new(),
            chunk_size,
            fail_after,
        }
    }
}

impl Write for ChunkedSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .fail_after
            .is_some_and(|limit| self.bytes.len() >= limit)
        {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "fixture downstream sink rejected the stream",
            ));
        }
        let allowed = self
            .fail_after
            .map(|limit| limit.saturating_sub(self.bytes.len()))
            .unwrap_or(bytes.len());
        let written = self.chunk_size.min(allowed).min(bytes.len());
        if written == 0 {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "fixture downstream sink rejected the stream",
            ));
        }
        self.bytes.extend_from_slice(&bytes[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct RequestTrackingSink {
    largest_request: usize,
}

impl Write for RequestTrackingSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.largest_request = self.largest_request.max(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
