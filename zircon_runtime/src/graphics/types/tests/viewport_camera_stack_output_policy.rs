use super::ViewportCameraStackOutputPolicy;

#[test]
fn final_target_output_owner_is_stack_terminal_not_viewport_terminal() {
    let intermediate = ViewportCameraStackOutputPolicy::new(false, false);
    let texture_stack_terminal = ViewportCameraStackOutputPolicy::new(true, false);
    let viewport_terminal = ViewportCameraStackOutputPolicy::new(true, true);
    let viewport_single = ViewportCameraStackOutputPolicy::stack_terminal();

    assert!(!intermediate.owns_final_target_output());
    assert!(!intermediate.owns_viewport_submission());
    assert!(!intermediate.owns_shared_viewport_products());
    assert!(texture_stack_terminal.owns_final_target_output());
    assert!(!texture_stack_terminal.owns_viewport_submission());
    assert!(!texture_stack_terminal.owns_shared_viewport_products());
    assert!(viewport_terminal.owns_final_target_output());
    assert!(viewport_terminal.owns_viewport_submission());
    assert!(viewport_terminal.owns_shared_viewport_products());
    assert!(!viewport_terminal.starts_viewport_submission());
    assert!(viewport_single.starts_viewport_submission());
}
