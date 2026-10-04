use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn pane_backgrounds_project_viewport_and_empty_roles_from_the_current_theme() {
    let mut palette = PALETTE;
    palette.shell_background = [3, 5, 7, 255];
    palette.surface_inset = [11, 13, 17, 255];

    assert_eq!(pane_background_color("Scene", palette), [3, 5, 7, 255]);
    assert_eq!(pane_background_color("Game", palette), [3, 5, 7, 255]);
    assert_eq!(
        pane_background_color("Hierarchy", palette),
        [11, 13, 17, 255]
    );
}
