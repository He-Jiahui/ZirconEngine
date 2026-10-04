use std::path::PathBuf;

use super::HubFocusBindingTarget;

#[test]
fn focus_binding_identity_requires_project_instance_and_generation_to_match() {
    let base = HubFocusBindingTarget::new(PathBuf::from("E:/Projects/Game"), "913-42".into(), 7);

    assert_eq!(
        base,
        HubFocusBindingTarget::new(PathBuf::from("E:/Projects/Game"), "913-42".into(), 7)
    );
    assert_ne!(
        base,
        HubFocusBindingTarget::new(PathBuf::from("E:/Projects/Game"), "913-42".into(), 8)
    );
}
