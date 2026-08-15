use super::*;

mod assertions;
mod base_scene;
mod editor_panes;
mod workbench_panes;

#[test]
fn host_scene_projection_converts_host_owned_panes_to_host_contract_panes() {
    let mut scene = base_scene::host_scene();
    editor_panes::populate(&mut scene);
    workbench_panes::populate(&mut scene);

    let projected = to_host_contract_host_scene_data(&scene);
    assert_eq!(projected.page_chrome.overflow_widest_title_width_px, 123.0);
    assertions::assert_host_contract_scene(&projected);
}
