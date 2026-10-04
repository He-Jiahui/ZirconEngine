use super::budget::{validate_standalone_face_bytes_with_budget, FontSourceBudget};
use super::*;

#[test]
fn font_source_budget_rejects_woff2_declared_expansion_before_decoder_runs() {
    let budget = FontSourceBudget {
        max_source_bytes: 64,
        max_decoded_bytes: 32,
        ..FONT_SOURCE_BUDGET
    };
    let mut source = vec![0; 20];
    source[..4].copy_from_slice(b"wOF2");
    source[16..20].copy_from_slice(&33_u32.to_be_bytes());

    let error = decode_font_source_with_budget(source, budget).unwrap_err();

    assert!(matches!(
        error,
        FontSourceDecodeError::Budget(FontSourceBudgetError::Woff2DeclaredDecodedBytes {
            limit_bytes: 32,
            actual_bytes: 33,
        })
    ));
}

#[test]
fn font_source_budget_rejects_ttc_face_count_before_metadata_enumeration() {
    let budget = FontSourceBudget {
        max_faces: 1,
        ..FONT_SOURCE_BUDGET
    };
    let mut source = vec![0; 12];
    source[..4].copy_from_slice(b"ttcf");
    source[8..12].copy_from_slice(&2_u32.to_be_bytes());

    let error = validate_font_metadata_with_budget(&source, budget).unwrap_err();

    assert_eq!(
        error,
        FontSourceBudgetError::CollectionFaceCount {
            limit: 1,
            actual: 2,
        }
    );
}

#[test]
fn font_source_budget_rejects_excessive_table_directory_before_face_parse() {
    let budget = FontSourceBudget {
        max_tables_per_face: 1,
        ..FONT_SOURCE_BUDGET
    };
    let mut source = vec![0; 6];
    source[4..6].copy_from_slice(&2_u16.to_be_bytes());

    let error = validate_font_metadata_with_budget(&source, budget).unwrap_err();

    assert_eq!(
        error,
        FontSourceBudgetError::TableCount {
            face_index: 0,
            limit: 1,
            actual: 2,
        }
    );
}

#[test]
fn font_source_budget_rejects_excessive_fvar_axes_before_metadata_projection() {
    let budget = FontSourceBudget {
        max_variation_axes: 1,
        ..FONT_SOURCE_BUDGET
    };
    let fvar_offset = 28;
    let mut source = vec![0; fvar_offset + 16];
    source[4..6].copy_from_slice(&1_u16.to_be_bytes());
    source[12..16].copy_from_slice(b"fvar");
    source[20..24].copy_from_slice(&(fvar_offset as u32).to_be_bytes());
    source[24..28].copy_from_slice(&16_u32.to_be_bytes());
    source[fvar_offset + 8..fvar_offset + 10].copy_from_slice(&2_u16.to_be_bytes());

    let error = validate_font_metadata_with_budget(&source, budget).unwrap_err();

    assert_eq!(
        error,
        FontSourceBudgetError::VariationAxisCount {
            face_index: 0,
            limit: 1,
            actual: 2,
        }
    );
}

#[test]
fn font_source_budget_rejects_standalone_ttc_table_duplication_before_copy() {
    let budget = FontSourceBudget {
        max_standalone_face_bytes: 32,
        ..FONT_SOURCE_BUDGET
    };
    let mut source = vec![0; 64];
    source[..4].copy_from_slice(b"ttcf");
    source[8..12].copy_from_slice(&1_u32.to_be_bytes());
    source[12..16].copy_from_slice(&16_u32.to_be_bytes());
    source[20..22].copy_from_slice(&2_u16.to_be_bytes());
    source[28..32].copy_from_slice(b"name");
    source[36..40].copy_from_slice(&44_u32.to_be_bytes());
    source[40..44].copy_from_slice(&4_u32.to_be_bytes());
    source[44..48].copy_from_slice(b"name");
    source[52..56].copy_from_slice(&60_u32.to_be_bytes());
    source[56..60].copy_from_slice(&4_u32.to_be_bytes());

    let error = validate_standalone_face_bytes_with_budget(&source, 0, budget).unwrap_err();

    assert_eq!(
        error,
        FontSourceBudgetError::StandaloneFaceBytes {
            face_index: 0,
            limit_bytes: 32,
            actual_bytes: 48,
        }
    );
}
