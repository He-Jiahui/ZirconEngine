from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PARITY_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "ui" / "surface" / "render" / "parity.rs"
CONTRACTS_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "tests" / "render_parity_key_reuse_performance_contracts.rs"


def test_parity_rows_reuse_the_already_computed_batch_key_metadata() -> None:
    source = PARITY_RS.read_text(encoding="utf-8")

    assert "let resource = batch_key.resource.clone();" in source
    assert "let text_render_mode = batch_key.text_backend;" in source
    assert "fn paint_resource_key(" not in source
    assert "fn paint_text_render_mode(" not in source


def test_release_benchmark_keeps_scale_and_threshold_contract() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "RUNTIME_INTERFACE03_RENDER_PARITY_KEY_REUSE_BENCH_V1" in source
    assert "const ELEMENT_COUNT: usize = 4_096;" in source
    assert "const SAMPLE_COUNT: usize = 11;" in source
    assert "reused_samples[p95].saturating_mul(5)" in source


def test_snapshot_contract_covers_resource_and_text_metadata() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "parity_rows_preserve_resource_and_text_metadata" in source
    assert "row.resource, row.batch_key.resource" in source
    assert "row.text_render_mode, row.batch_key.text_backend" in source
