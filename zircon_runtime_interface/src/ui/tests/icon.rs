use super::{builtin_icon_supported, UiBuiltinIcon, UiBuiltinIconReference, UiIconSize};

#[test]
fn builtin_icon_ids_normalize_case_and_whitespace() {
    assert!(builtin_icon_supported(" info "));
    assert!(builtin_icon_supported("arrow-up@s"));
    assert!(builtin_icon_supported("info@xl"));
    assert!(builtin_icon_supported("CHECK-CIRCLE"));
    assert!(!builtin_icon_supported("project-custom-icon"));
    assert_eq!(
        UiBuiltinIcon::parse("package"),
        Some(UiBuiltinIcon::Package)
    );
}

#[test]
fn icon_reference_retains_its_semantic_size_override() {
    let reference = UiBuiltinIconReference::parse("arrow-up@l").unwrap();
    assert_eq!(reference.icon(), UiBuiltinIcon::ArrowUp);
    assert_eq!(reference.resolved_size(16.0), UiIconSize::Large);
    assert_eq!(
        UiBuiltinIconReference::parse("arrow-up")
            .unwrap()
            .resolved_size(23.0),
        UiIconSize::Large
    );
}

#[test]
fn icon_sizes_converge_to_four_semantic_tiers() {
    assert_eq!(UiIconSize::parse("s"), Some(UiIconSize::Small));
    assert_eq!(UiIconSize::parse("medium"), Some(UiIconSize::Medium));
    assert_eq!(UiIconSize::parse("l"), Some(UiIconSize::Large));
    assert_eq!(UiIconSize::parse("xl"), Some(UiIconSize::ExtraLarge));
    assert_eq!(UiIconSize::Small.logical_extent(), 16.0);
    assert_eq!(UiIconSize::Medium.logical_extent(), 20.0);
    assert_eq!(UiIconSize::Large.logical_extent(), 24.0);
    assert_eq!(UiIconSize::ExtraLarge.logical_extent(), 32.0);
    assert_eq!(UiIconSize::parse("18"), None);
}
