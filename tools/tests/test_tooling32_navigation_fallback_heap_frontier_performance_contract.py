from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GRAPH_RS = ROOT / "zircon_plugins/navigation/native/src/fallback_query/graph.rs"


def _source() -> str:
    return GRAPH_RS.read_text(encoding="utf-8")


def test_fallback_route_uses_heap_frontier_instead_of_full_graph_rescan() -> None:
    source = _source()

    assert "BinaryHeap" in source
    assert "Reverse" in source
    assert ".min_by(" not in source


def test_fallback_route_skips_stale_frontier_entries() -> None:
    source = _source()

    assert "if cost > distances[current]" in source
    assert "continue;" in source


def test_fallback_route_frontier_has_stable_polygon_tie_break() -> None:
    source = _source()

    assert "Reverse((OrderedRouteCost::new(candidate), edge.to))" in source
    assert "Reverse((OrderedRouteCost::new(0.0), start))" in source

