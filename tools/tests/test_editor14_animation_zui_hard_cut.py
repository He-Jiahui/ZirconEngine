from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
OLD_ASSET = "animation_editor.zui"
SEQUENCE_ASSET = "host/animation_sequence_body.zui"
GRAPH_ASSET = "host/animation_graph_body.zui"


def read(relative: str) -> str:
    return (REPO / relative).read_text(encoding="utf-8")


def test_animation_editor_projection_uses_canonical_host_assets() -> None:
    projection = read("zircon_editor/src/ui/layouts/views/animation_editor.rs")
    scene = read(
        "zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs"
    )
    integration = read(
        "zircon_editor/tests/integration_contracts/workbench_animation_editor_shell.rs"
    )

    for source_name, source in {
        "animation projection": projection,
        "scene projection": scene,
        "integration contract": integration,
    }.items():
        assert OLD_ASSET not in source, f"{source_name} keeps deleted asset"

    assert SEQUENCE_ASSET in projection
    assert GRAPH_ASSET in projection
    assert "animation_sequence_pane_nodes" in scene
    assert "animation_graph_pane_nodes" in scene
    assert SEQUENCE_ASSET in integration
    assert GRAPH_ASSET in integration


def test_canonical_animation_host_assets_keep_distinct_mount_contracts() -> None:
    sequence = read("zircon_editor/assets/ui/editor/host/animation_sequence_body.zui")
    graph = read("zircon_editor/assets/ui/editor/host/animation_graph_body.zui")

    assert 'control_id = "AnimationSequencePaneBodyRoot"' in sequence
    assert 'slot_name = "animation_timeline_slot"' in sequence
    assert 'control_id = "AnimationGraphPaneBodyRoot"' in graph
    assert 'slot_name = "animation_graph_canvas_slot"' in graph


if __name__ == "__main__":
    for test in (
        test_animation_editor_projection_uses_canonical_host_assets,
        test_canonical_animation_host_assets_keep_distinct_mount_contracts,
    ):
        test()
    print("2 passed")
