from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ECS_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "ui" / "ecs.rs"
CONTRACTS_RS = (
    REPO_ROOT
    / "zircon_runtime_interface"
    / "src"
    / "tests"
    / "ui_ecs_node_lookup_contracts.rs"
)


def _node_lookup_body() -> str:
    source = ECS_RS.read_text(encoding="utf-8")
    start = source.index("    pub fn node(&self, node_id: UiNodeId)")
    end = source.index("\n    }", start)
    return source[start:end]


def test_sorted_ecs_projection_uses_logarithmic_node_lookup() -> None:
    body = _node_lookup_body()

    assert "binary_search_by_key" in body
    assert ".get(index)" in body


def test_unsorted_ecs_projection_keeps_compatible_fallback() -> None:
    body = _node_lookup_body()

    assert ".iter().find(" in body


def test_release_benchmark_keeps_scale_and_threshold_contract() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "RUNTIME_INTERFACE03_ECS_NODE_LOOKUP_BENCH_V1" in source
    assert "const NODE_COUNT: u64 = 4_096;" in source
    assert "const PROBE_COUNT: usize = 100_000;" in source
    assert "indexed_samples[p95].saturating_mul(5)" in source
