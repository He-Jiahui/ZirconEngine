//! The Unreal toolbar keeps core commands explicit and routes auxiliary controls by intent.
const UNREAL_TOOLBAR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/toolbar.zui");

fn node_source(node_id: &str) -> &str {
    let marker = format!("[nodes.{node_id}]");
    UNREAL_TOOLBAR_ASSET
        .split_once(&marker)
        .unwrap_or_else(|| panic!("Unreal toolbar must declare node `{node_id}`"))
        .1
        .split("\n[nodes.")
        .next()
        .unwrap_or_default()
}

#[test]
fn unreal_toolbar_keeps_save_play_stop_and_mode_routes_explicit() {
    let save = node_source("save");
    assert!(save.contains("control_id = \"WorkbenchToolbarSave\""));
    assert!(save.contains("file.project.save"));

    let play = node_source("play");
    assert!(play.contains("control_id = \"WorkbenchRunPlay\""));
    assert!(play.contains("runtime.play_mode.enter"));

    let stop = node_source("stop");
    assert!(stop.contains("control_id = \"WorkbenchRunStop\""));
    assert!(stop.contains("runtime.play_mode.exit"));

    let mode = node_source("play_options");
    assert!(mode.contains("control_id = \"WorkbenchRunMode\""));
    assert!(mode.contains("workbench.run.open_mode_menu"));
}
