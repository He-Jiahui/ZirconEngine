use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

const LEGACY_BODY_SIZE: f32 = 10.0;
const LEGACY_CAPTION_SIZE: f32 = 8.5;
const LEGACY_TITLE_SIZE: f32 = 14.0;
const LEGACY_SIZE_EPSILON: f32 = 0.001;

/// Upgrades the original point-as-pixel defaults without overwriting custom sizes.
pub(super) fn migrate_legacy_workbench_typography(tokens: &mut EditorTypographyTokens) {
    migrate_if_legacy(
        &mut tokens.body_size,
        LEGACY_BODY_SIZE,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE,
    );
    migrate_if_legacy(
        &mut tokens.caption_size,
        LEGACY_CAPTION_SIZE,
        EditorTypographyTokens::WORKBENCH_CAPTION_SIZE,
    );
    migrate_if_legacy(
        &mut tokens.title_size,
        LEGACY_TITLE_SIZE,
        EditorTypographyTokens::WORKBENCH_TITLE_SIZE,
    );
}

fn migrate_if_legacy(value: &mut f32, legacy: f32, current: f32) {
    if (*value - legacy).abs() < LEGACY_SIZE_EPSILON {
        *value = current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_typography_migration_preserves_custom_sizes() {
        let mut typography = EditorTypographyTokens::workbench_default();
        typography.body_size = 15.0;
        typography.caption_size = LEGACY_CAPTION_SIZE;
        typography.title_size = 21.0;

        migrate_legacy_workbench_typography(&mut typography);

        assert_eq!(typography.body_size, 15.0);
        assert_eq!(
            typography.caption_size,
            EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
        );
        assert_eq!(typography.title_size, 21.0);
    }
}
