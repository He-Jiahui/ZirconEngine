use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_list_row_glyphs::{
    list_row_adornment_kind, ListRowAdornmentKind,
};

#[test]
fn list_row_adornment_kind_prefers_disabled_then_checked_then_navigation() {
    assert_eq!(
        list_row_adornment_kind(&TemplatePaneNodeData {
            disabled: true,
            selected: true,
            checked: true,
            ..TemplatePaneNodeData::default()
        }),
        ListRowAdornmentKind::DisabledDiamond
    );
    let mut loading_selected = TemplatePaneNodeData {
        selected: true,
        checked: true,
        ..TemplatePaneNodeData::default()
    };
    loading_selected.button_style.loading = true;
    assert_eq!(
        list_row_adornment_kind(&loading_selected),
        ListRowAdornmentKind::DisabledDiamond
    );
    assert_eq!(
        list_row_adornment_kind(&TemplatePaneNodeData {
            selected: true,
            ..TemplatePaneNodeData::default()
        }),
        ListRowAdornmentKind::Chevron
    );
    assert_eq!(
        list_row_adornment_kind(&TemplatePaneNodeData {
            checked: true,
            ..TemplatePaneNodeData::default()
        }),
        ListRowAdornmentKind::Check
    );
    assert_eq!(
        list_row_adornment_kind(&TemplatePaneNodeData::default()),
        ListRowAdornmentKind::Chevron
    );
}
