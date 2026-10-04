use super::resource_kind_name;
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn resource_kind_name_includes_font() {
    assert_eq!(resource_kind_name(ResourceKind::Font), "Font");
}
