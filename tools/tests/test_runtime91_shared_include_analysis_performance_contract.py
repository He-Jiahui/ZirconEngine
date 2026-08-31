import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / (
    "zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs"
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


def test_incremental_batch_builds_one_shared_include_analysis() -> None:
    body = _function_body(
        _source(),
        "fn shader_sources_with_module_dependency_hashes_and_changed_paths(",
    )

    assert body.count("let analysis = dag.analyze();") == 1
    assert "dag.reverse_changed_source_closure(&sources, changed_paths, &analysis)" in body
    assert "dag.topology_hashes_by_source(&sources, &analysis)?" in body


def test_shared_analysis_builds_scc_and_component_graph_once() -> None:
    body = _function_body(_source(), "fn analyze(&self) -> IndexedIncludeAnalysis")

    assert body.count("self.strongly_connected_components()") == 1
    assert body.count("self.component_graph(") == 1


def test_include_analysis_owns_the_reusable_component_projection() -> None:
    source = _source()

    assert "struct IndexedIncludeAnalysis" in source
    assert "components: Vec<Vec<usize>>" in source
    assert "component_for_source: Vec<usize>" in source
    assert "graph: IndexedIncludeComponentGraph" in source


def test_analysis_consumers_do_not_rebuild_graph_structure() -> None:
    source = _source()
    consumer_bodies = (
        _function_body(source, "fn topology_hashes_by_source("),
        _function_body(source, "fn reverse_changed_source_closure("),
    )

    for body in consumer_bodies:
        assert "strongly_connected_components" not in body
        assert "component_graph(" not in body


class Runtime91SharedIncludeAnalysisPerformanceContractTests(unittest.TestCase):
    def test_incremental_batch_builds_one_shared_include_analysis(self) -> None:
        test_incremental_batch_builds_one_shared_include_analysis()

    def test_shared_analysis_builds_scc_and_component_graph_once(self) -> None:
        test_shared_analysis_builds_scc_and_component_graph_once()

    def test_include_analysis_owns_the_reusable_component_projection(self) -> None:
        test_include_analysis_owns_the_reusable_component_projection()

    def test_analysis_consumers_do_not_rebuild_graph_structure(self) -> None:
        test_analysis_consumers_do_not_rebuild_graph_structure()


if __name__ == "__main__":
    unittest.main()
