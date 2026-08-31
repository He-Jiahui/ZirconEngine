from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PARITY_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "ui" / "surface" / "render" / "parity.rs"
CONTRACTS_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "tests" / "render_parity_performance_contracts.rs"


def test_parity_stats_are_fused_into_row_projection() -> None:
    source = PARITY_RS.read_text(encoding="utf-8")

    assert "let mut clipped_paint_count = 0;" in source
    assert "let mut resource_bound_paint_count = 0;" in source
    assert "let mut text_paint_count = 0;" in source
    assert ".filter(|row| row.clip_frame.is_some())" not in source
    assert ".filter(|row| row.resource.is_some())" not in source


def test_release_benchmark_keeps_scale_and_threshold_contract() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "RUNTIME_INTERFACE03_RENDER_PARITY_STATS_FUSION_BENCH_V1" in source
    assert "const ROW_COUNT: usize = 65_536;" in source
    assert "const SAMPLE_COUNT: usize = 11;" in source
    assert "fused_samples[p95].saturating_mul(5)" in source


def test_fused_stats_match_separate_scan_contract() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "fused_stats_match_separate_scans" in source
    assert "separate_stats" in source
    assert "fused_stats" in source
