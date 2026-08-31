from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ECS_RS = REPO_ROOT / "zircon_runtime_interface" / "src" / "ui" / "ecs.rs"
FOCUSED_IMPACT_RS = (
    REPO_ROOT
    / "zircon_runtime_interface"
    / "src"
    / "ui"
    / "ecs"
    / "focused_impact.rs"
)
CONTRACTS_RS = (
    REPO_ROOT
    / "zircon_runtime_interface"
    / "src"
    / "tests"
    / "ui_ecs_node_lookup_contracts.rs"
)


def test_snapshot_and_delta_query_only_the_requested_domain() -> None:
    source = ECS_RS.read_text(encoding="utf-8")
    impact_bodies = source.split(
        "pub fn dirty_domain_impact(\n        &self,\n        domain: UiEcsDirtyDomainKind,"
    )[1:]

    assert len(impact_bodies) == 2
    for body in impact_bodies:
        body = body.split("\n    }", 1)[0]
        assert "projection_dirty_domain_impact_from_" in body
        assert "self.dirty_domain_impacts()" not in body


def test_single_domain_aggregation_does_not_build_all_domain_buckets() -> None:
    source = FOCUSED_IMPACT_RS.read_text(encoding="utf-8")
    start = source.index("fn projection_dirty_domain_impact_from_domains")
    body = source[start : source.index("\n}", start)]

    assert "UiEcsDirtyDomainKind::ordered()" not in body
    assert "node_ids_by_domain" not in body


def test_release_benchmark_keeps_scale_and_threshold_contract() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "RUNTIME_INTERFACE03_SINGLE_DOMAIN_IMPACT_BENCH_V1" in source
    assert "const NODE_COUNT: u64 = 4_096;" in source
    assert "const PROBE_COUNT: usize = 100;" in source
    assert "focused_samples[p95].saturating_mul(5)" in source
