from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PARITY_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "ui" / "surface" / "render" / "parity.rs"
CONTRACTS_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "tests" / "render_parity_performance_contracts.rs"


def test_renderer_parity_maps_source_indices_in_one_pass() -> None:
    source = PARITY_RS.read_text(encoding="utf-8")

    assert "batch_indices_by_source_index(&plan.batches, elements.len())" in source
    assert "for (batch_index, batch) in batches.iter().enumerate()" in source
    assert ".position(|batch| batch.source_indices.contains(&paint_index))" not in source


def test_release_benchmark_keeps_scale_and_threshold_contract() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "RUNTIME_INTERFACE03_RENDER_PARITY_BATCH_MAPPING_BENCH_V1" in source
    assert "const ELEMENT_COUNT: usize = 4_096;" in source
    assert "const SAMPLE_COUNT: usize = 11;" in source
    assert "indexed_samples[p95].saturating_mul(5)" in source


def test_mapping_preserves_first_batch_and_ignores_invalid_source_indices() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "first_batch_wins_for_duplicate_source_indices" in source
    assert "ignores_out_of_range_source_indices" in source
