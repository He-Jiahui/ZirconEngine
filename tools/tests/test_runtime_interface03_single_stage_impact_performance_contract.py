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


def test_snapshot_and_delta_query_only_the_requested_stage() -> None:
    source = ECS_RS.read_text(encoding="utf-8")
    schedule_impact_bodies = source.split(
        "pub fn schedule_impact(&self, stage: UiPipelineStage)"
    )[1:]

    assert len(schedule_impact_bodies) == 2
    for body in schedule_impact_bodies:
        body = body.split("\n    }", 1)[0]
        assert "projection_schedule_impact_from_" in body
        assert "self.schedule_impacts()" not in body


def test_single_stage_aggregation_does_not_build_all_stage_buckets() -> None:
    source = FOCUSED_IMPACT_RS.read_text(encoding="utf-8")
    start = source.index("fn projection_schedule_impact_from_domains")
    body = source[start : source.index("\n}", start)]

    assert "UiPipelineStage::ordered()" not in body
    assert "ProjectionScheduleImpactBucket" not in body


def test_release_benchmark_keeps_scale_and_threshold_contract() -> None:
    source = CONTRACTS_RS.read_text(encoding="utf-8")

    assert "RUNTIME_INTERFACE03_SINGLE_STAGE_IMPACT_BENCH_V1" in source
    assert "const NODE_COUNT: u64 = 4_096;" in source
    assert "const PROBE_COUNT: usize = 100;" in source
    assert "focused_samples[p95].saturating_mul(5)" in source
