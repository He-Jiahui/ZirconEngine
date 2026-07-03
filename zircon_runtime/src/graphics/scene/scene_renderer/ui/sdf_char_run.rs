// SDF still consumes Unicode scalar runs until full shaping-cluster output lands.
// Invisible format controls keep run indexes stable without occupying atlas slots.
pub(super) fn sdf_scalar_requires_atlas_slot(scalar: char) -> bool {
    !scalar.is_whitespace() && !sdf_scalar_is_invisible_format(scalar)
}

pub(super) fn sdf_scalar_is_invisible_format(scalar: char) -> bool {
    matches!(
        scalar,
        '\u{061C}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FEFF}'
            | '\u{E0100}'..='\u{E01EF}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_invisible_format_controls_from_sdf_slots() {
        for scalar in [
            '\u{061C}',
            '\u{200B}',
            '\u{200C}',
            '\u{200D}',
            '\u{200E}',
            '\u{202A}',
            '\u{2060}',
            '\u{FE0E}',
            '\u{FE0F}',
            '\u{FEFF}',
            '\u{E0100}',
        ] {
            assert!(sdf_scalar_is_invisible_format(scalar));
            assert!(!sdf_scalar_requires_atlas_slot(scalar));
        }
    }

    #[test]
    fn keeps_visible_scalars_as_sdf_slot_candidates() {
        for scalar in ['A', '中', '\u{2764}', '\u{1F469}'] {
            assert!(!sdf_scalar_is_invisible_format(scalar));
            assert!(sdf_scalar_requires_atlas_slot(scalar));
        }
    }

    #[test]
    fn keeps_whitespace_slotless_without_classifying_it_as_format_control() {
        for scalar in [' ', '\t', '\n'] {
            assert!(!sdf_scalar_is_invisible_format(scalar));
            assert!(!sdf_scalar_requires_atlas_slot(scalar));
        }
    }
}
