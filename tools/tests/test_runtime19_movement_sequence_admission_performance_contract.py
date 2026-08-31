from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / (
    "examples/woc/native/crates/woc_protocol/src/movement_input.rs"
)
INTEGRATION_TEST = REPO_ROOT / (
    "examples/woc/native/crates/woc_protocol/tests/movement_input.rs"
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


def test_movement_dispositions_distinguish_applied_duplicate_and_stale() -> None:
    source = _source()
    enum_body = _function_body(source, "pub enum MovementFrameDisposition")

    assert "Applied" in enum_body
    assert "Duplicate" in enum_body
    assert "Stale" in enum_body


def test_sequence_admission_uses_one_btree_entry_lookup_per_frame() -> None:
    body = _function_body(_source(), "pub fn apply_batch(")

    assert ".entry(key)" in body
    assert ".get(&key)" not in body
    assert "self.inputs.insert(" not in body


def test_duplicate_and_stale_frames_do_not_mutate_retained_input() -> None:
    body = _function_body(_source(), "pub fn apply_batch(")
    occupied_branch = body[body.index("Entry::Occupied") :]

    assert "Ordering::Greater" in occupied_branch
    assert "Ordering::Equal" in occupied_branch
    assert "Ordering::Less" in occupied_branch
    assert occupied_branch.index("Ordering::Greater") < occupied_branch.index("input.flags =")
    assert occupied_branch.index("Ordering::Equal") > occupied_branch.index("input.flags =")


def test_newer_frame_preserves_facing_when_update_omits_it() -> None:
    body = _function_body(_source(), "pub fn apply_batch(")
    compact_body = "".join(body.split())

    assert "ifletSome(facing)=frame.facing" in compact_body
    assert "input.facing=Some(facing)" in compact_body


def test_release_evidence_benchmarks_the_real_relay_entry_path() -> None:
    source = INTEGRATION_TEST.read_text(encoding="utf-8")

    assert "RUNTIME19_MOVEMENT_SEQUENCE_ADMISSION_BENCH_V1" in source
    assert ".apply_batch(51, black_box(batch))" in source
    assert "legacy_repeated_lookup" in source
    assert ".div_ceil(100)" in source


def test_release_evidence_keeps_probe_and_latency_gates() -> None:
    source = INTEGRATION_TEST.read_text(encoding="utf-8")

    assert "optimized_probes.saturating_mul(5) <= legacy_probes.saturating_mul(2)" in source
    assert "optimized_p50.saturating_mul(100) <= legacy_p50.saturating_mul(60)" in source
    assert "optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(60)" in source
