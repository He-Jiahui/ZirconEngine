from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / (
    "examples/woc/native/crates/woc_protocol/src/command_value.rs"
)
INTEGRATION_TEST = REPO_ROOT / (
    "examples/woc/native/crates/woc_protocol/tests/command_value.rs"
)


def _source() -> str:
    return SOURCE.read_text(encoding="utf-8")


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening_brace = source.index("{", start)
    depth = 0
    for index in range(opening_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening_brace + 1 : index]
    raise AssertionError(f"unclosed function body for {signature}")


def test_command_value_imports_btree_entry_api() -> None:
    source = _source()

    assert "btree_map::Entry" in source


def test_object_constructor_moves_unique_keys_into_vacant_entries() -> None:
    body = _function_body(_source(), "pub fn object(")

    assert "values.entry(key)" in body
    assert "Entry::Vacant(vacant)" in body
    assert "vacant.insert(value)" in body
    assert "key.clone()" not in body


def test_object_decoder_moves_unique_keys_into_vacant_entries() -> None:
    body = _function_body(_source(), "fn read_value(")
    object_branch = body[body.index("TAG_OBJECT =>") :]

    assert "values.entry(key)" in object_branch
    assert "Entry::Vacant(vacant)" in object_branch
    assert "vacant.insert(value)" in object_branch
    assert "key.clone()" not in object_branch


def test_duplicate_error_clones_only_the_occupied_key() -> None:
    source = _source()

    assert source.count("occupied.key().clone()") == 2
    assert "values.insert(key.clone(), value)" not in source


def test_release_evidence_benchmarks_the_real_object_constructor() -> None:
    source = INTEGRATION_TEST.read_text(encoding="utf-8")

    assert "RUNTIME19_COMMAND_VALUE_OBJECT_KEYS_BENCH_V1" in source
    assert "CommandValue::object(entries)" in source
    assert "legacy_clone_then_insert" in source
    assert ".div_ceil(100)" in source


def test_release_evidence_keeps_allocation_and_latency_gates() -> None:
    source = INTEGRATION_TEST.read_text(encoding="utf-8")

    assert "optimized_allocations.saturating_mul(5) <= legacy_allocations" in source
    assert "optimized_bytes.saturating_mul(100) <= legacy_bytes.saturating_mul(60)" in source
    assert "optimized_p50.saturating_mul(100) <= legacy_p50.saturating_mul(85)" in source
    assert "optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(90)" in source
