use super::*;

#[test]
fn host_palette_projects_from_central_tokens() {
    let tokens = EditorDesignTokens::workbench_dark();

    assert_eq!(project_host_palette(&tokens), DEFAULT_HOST_PALETTE);
    assert_eq!(DEFAULT_HOST_PALETTE.border, tokens.palette.border.to_u8());
    assert_eq!(
        DEFAULT_HOST_PALETTE.text,
        tokens.palette.text_primary.to_u8()
    );
    assert_eq!(
        DEFAULT_HOST_PALETTE.text_muted,
        tokens.palette.text_secondary.to_u8()
    );
    assert_eq!(DEFAULT_HOST_PALETTE.error, tokens.palette.error.to_u8());
}

#[test]
fn changing_central_accent_moves_projected_accent_roles() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.accent = UiRgbaColor::from_u8(9, 180, 220, 255);
    tokens.palette.focus_ring = tokens.palette.accent;

    let projected = project_host_palette(&tokens);

    assert_eq!(projected.accent, [9, 180, 220, 255]);
    assert_eq!(projected.focus_ring, [9, 180, 220, 255]);
}
