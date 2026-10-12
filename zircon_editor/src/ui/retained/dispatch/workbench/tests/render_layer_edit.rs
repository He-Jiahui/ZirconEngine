use super::*;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn render_layer_mask_parser_accepts_decimal_hex_and_binary() {
    assert_eq!(parse_render_layer_mask("17").unwrap(), 17);
    assert_eq!(parse_render_layer_mask("0x11").unwrap(), 17);
    assert_eq!(parse_render_layer_mask("0b10001").unwrap(), 17);
    assert!(parse_render_layer_mask("-1").is_err());
    assert!(parse_render_layer_mask("4294967296").is_err());
}

#[test]
fn render_layer_commit_uses_the_reflected_unsigned_field() {
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let binding = bridge
        .render_layer_mask_commit_binding(
            RENDER_LAYER_MASK_CONTROL,
            RENDER_LAYER_MASK_COMMIT,
            "0x20",
        )
        .unwrap()
        .expect("render layer commit should resolve");
    let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
        panic!("render layer commit must dispatch an inspector field batch");
    };
    assert_eq!(changes[0].field_id, RENDER_LAYER_MASK_FIELD);
    assert_eq!(changes[0].value, UiBindingValue::Unsigned(32));
}

#[test]
fn render_layer_edit_updates_only_the_retained_draft_value() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    assert_eq!(
        bridge
            .edit_inspector_render_layer_mask(
                RENDER_LAYER_MASK_CONTROL,
                RENDER_LAYER_MASK_EDIT,
                " 0x20 ",
            )
            .unwrap(),
        Some(true)
    );
    assert_eq!(
        bridge
            .control_string(RENDER_LAYER_MASK_CONTROL, "value")
            .as_deref(),
        Some("0x20")
    );
}
