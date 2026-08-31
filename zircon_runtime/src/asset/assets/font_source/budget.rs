const KIBIBYTE: usize = 1024;
const MEBIBYTE: usize = KIBIBYTE * KIBIBYTE;

/// Crate-wide admission policy for untrusted font payloads.
///
/// The values bound the allocations and metadata fan-out that Zircon owns. They do not replace
/// format validation by `ttf-parser`, which remains responsible for malformed directory details.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FontSourceBudget {
    pub(crate) max_source_bytes: usize,
    pub(crate) max_decoded_bytes: usize,
    pub(crate) max_faces: u32,
    pub(crate) max_tables_per_face: u16,
    pub(crate) max_table_bytes: usize,
    pub(crate) max_standalone_face_bytes: usize,
    pub(crate) max_cmap_ranges: usize,
    pub(crate) max_variation_axes: u16,
    pub(crate) max_variation_instances: u16,
}

pub(super) const FONT_SOURCE_BUDGET: FontSourceBudget = FontSourceBudget {
    max_source_bytes: 64 * MEBIBYTE,
    max_decoded_bytes: 128 * MEBIBYTE,
    max_faces: 64,
    max_tables_per_face: 256,
    max_table_bytes: 64 * MEBIBYTE,
    max_standalone_face_bytes: 128 * MEBIBYTE,
    max_cmap_ranges: 65_536,
    max_variation_axes: 64,
    max_variation_instances: 256,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FontSourceBudgetError {
    #[error("font source is {actual_bytes} bytes, exceeding the {limit_bytes}-byte input budget")]
    SourceBytes { limit_bytes: u64, actual_bytes: u64 },
    #[error("WOFF2 source declares {actual_bytes} decoded bytes, exceeding the {limit_bytes}-byte budget")]
    Woff2DeclaredDecodedBytes { limit_bytes: u64, actual_bytes: u64 },
    #[error(
        "decoded font source is {actual_bytes} bytes, exceeding the {limit_bytes}-byte budget"
    )]
    DecodedBytes { limit_bytes: u64, actual_bytes: u64 },
    #[error("font collection has {actual} faces, exceeding the {limit} face budget")]
    CollectionFaceCount { limit: u32, actual: u32 },
    #[error("font face {face_index} has {actual} tables, exceeding the {limit} table budget")]
    TableCount {
        face_index: u32,
        limit: u16,
        actual: u16,
    },
    #[error("font face {face_index} table is {actual_bytes} bytes, exceeding the {limit_bytes}-byte budget")]
    TableBytes {
        face_index: u32,
        limit_bytes: u64,
        actual_bytes: u64,
    },
    #[error("materialized font face {face_index} is {actual_bytes} bytes, exceeding the {limit_bytes}-byte budget")]
    StandaloneFaceBytes {
        face_index: u32,
        limit_bytes: u64,
        actual_bytes: u64,
    },
    #[error("font face {face_index} has at least {observed_at_least} cmap ranges, exceeding the {limit} range budget")]
    CmapRangeCount {
        face_index: u32,
        limit: usize,
        observed_at_least: usize,
    },
    #[error(
        "font face {face_index} has {actual} variation axes, exceeding the {limit} axis budget"
    )]
    VariationAxisCount {
        face_index: u32,
        limit: u16,
        actual: u16,
    },
    #[error("font face {face_index} has {actual} variation instances, exceeding the {limit} instance budget")]
    VariationInstanceCount {
        face_index: u32,
        limit: u16,
        actual: u16,
    },
}

impl FontSourceBudgetError {
    pub(crate) fn face_index(self) -> Option<u32> {
        match self {
            Self::TableCount { face_index, .. }
            | Self::TableBytes { face_index, .. }
            | Self::StandaloneFaceBytes { face_index, .. }
            | Self::CmapRangeCount { face_index, .. }
            | Self::VariationAxisCount { face_index, .. }
            | Self::VariationInstanceCount { face_index, .. } => Some(face_index),
            Self::SourceBytes { .. }
            | Self::Woff2DeclaredDecodedBytes { .. }
            | Self::DecodedBytes { .. }
            | Self::CollectionFaceCount { .. } => None,
        }
    }
}

pub(super) fn validate_source_file_len(file_len: u64) -> Result<(), FontSourceBudgetError> {
    validate_source_len(file_len, FONT_SOURCE_BUDGET)
}

pub(super) fn max_cmap_ranges() -> usize {
    FONT_SOURCE_BUDGET.max_cmap_ranges
}

pub(super) fn validate_source_bytes(
    source: &[u8],
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    validate_source_len(source.len() as u64, budget)
}

pub(super) fn validate_woff2_declared_decoded_bytes(
    source: &[u8],
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    if !source.starts_with(b"wOF2") {
        return Ok(());
    }
    let Some(declared_bytes) = read_u32(source, 16) else {
        return Ok(());
    };
    if declared_bytes as usize > budget.max_decoded_bytes {
        return Err(FontSourceBudgetError::Woff2DeclaredDecodedBytes {
            limit_bytes: budget.max_decoded_bytes as u64,
            actual_bytes: u64::from(declared_bytes),
        });
    }
    Ok(())
}

pub(super) fn validate_decoded_bytes(
    bytes: &[u8],
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    if bytes.len() > budget.max_decoded_bytes {
        return Err(FontSourceBudgetError::DecodedBytes {
            limit_bytes: budget.max_decoded_bytes as u64,
            actual_bytes: bytes.len() as u64,
        });
    }
    Ok(())
}

pub(super) fn validate_font_metadata_with_budget(
    bytes: &[u8],
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    validate_decoded_bytes(bytes, budget)?;
    let face_offsets = face_offsets(bytes, budget)?;
    for (face_index, offset) in face_offsets.into_iter().enumerate() {
        validate_face_directory(bytes, face_index as u32, offset, budget)?;
    }
    Ok(())
}

pub(super) fn validate_standalone_face_bytes_with_budget(
    bytes: &[u8],
    face_index: u32,
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    let face_offsets = face_offsets(bytes, budget)?;
    let Some(face_offset) = face_offsets.get(face_index as usize).copied() else {
        return Ok(());
    };
    let Some(table_count) = read_u16(bytes, face_offset + 4) else {
        return Ok(());
    };
    let mut materialized_bytes = 12_usize.saturating_add(usize::from(table_count) * 16);
    for table_index in 0..usize::from(table_count) {
        let record_offset = face_offset + 12 + table_index * 16;
        let Some(table_len) = read_u32(bytes, record_offset + 12) else {
            return Ok(());
        };
        materialized_bytes = materialized_bytes
            .checked_add(3)
            .map(|value| value & !3)
            .and_then(|value| value.checked_add(table_len as usize))
            .ok_or(FontSourceBudgetError::StandaloneFaceBytes {
                face_index,
                limit_bytes: budget.max_standalone_face_bytes as u64,
                actual_bytes: u64::MAX,
            })?;
        if materialized_bytes > budget.max_standalone_face_bytes {
            return Err(FontSourceBudgetError::StandaloneFaceBytes {
                face_index,
                limit_bytes: budget.max_standalone_face_bytes as u64,
                actual_bytes: materialized_bytes as u64,
            });
        }
    }
    Ok(())
}

fn validate_source_len(
    actual_bytes: u64,
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    if actual_bytes > budget.max_source_bytes as u64 {
        return Err(FontSourceBudgetError::SourceBytes {
            limit_bytes: budget.max_source_bytes as u64,
            actual_bytes,
        });
    }
    Ok(())
}

fn face_offsets(
    bytes: &[u8],
    budget: FontSourceBudget,
) -> Result<Vec<usize>, FontSourceBudgetError> {
    if !bytes.starts_with(b"ttcf") {
        return Ok(vec![0]);
    }
    let Some(face_count) = read_u32(bytes, 8) else {
        return Ok(Vec::new());
    };
    if face_count > budget.max_faces {
        return Err(FontSourceBudgetError::CollectionFaceCount {
            limit: budget.max_faces,
            actual: face_count,
        });
    }

    let mut offsets = Vec::with_capacity(face_count as usize);
    for face_index in 0..face_count {
        let Some(offset) = read_u32(bytes, 12 + face_index as usize * 4) else {
            return Ok(Vec::new());
        };
        offsets.push(offset as usize);
    }
    Ok(offsets)
}

fn validate_face_directory(
    bytes: &[u8],
    face_index: u32,
    face_offset: usize,
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    let Some(table_count) = read_u16(bytes, face_offset + 4) else {
        return Ok(());
    };
    if table_count > budget.max_tables_per_face {
        return Err(FontSourceBudgetError::TableCount {
            face_index,
            limit: budget.max_tables_per_face,
            actual: table_count,
        });
    }

    for table_index in 0..usize::from(table_count) {
        let record_offset = face_offset + 12 + table_index * 16;
        let Some(tag) = bytes.get(record_offset..record_offset + 4) else {
            return Ok(());
        };
        let Some(table_offset) = read_u32(bytes, record_offset + 8) else {
            return Ok(());
        };
        let Some(table_len) = read_u32(bytes, record_offset + 12) else {
            return Ok(());
        };
        if table_len as usize > budget.max_table_bytes {
            return Err(FontSourceBudgetError::TableBytes {
                face_index,
                limit_bytes: budget.max_table_bytes as u64,
                actual_bytes: u64::from(table_len),
            });
        }
        let table_offset = table_offset as usize;
        let table_len = table_len as usize;
        let Some(table_end) = table_offset.checked_add(table_len) else {
            return Ok(());
        };
        let Some(table) = bytes.get(table_offset..table_end) else {
            return Ok(());
        };
        if tag == b"fvar" {
            validate_fvar_counts(table, face_index, budget)?;
        }
    }
    Ok(())
}

fn validate_fvar_counts(
    fvar: &[u8],
    face_index: u32,
    budget: FontSourceBudget,
) -> Result<(), FontSourceBudgetError> {
    let (Some(axis_count), Some(instance_count)) = (read_u16(fvar, 8), read_u16(fvar, 12)) else {
        return Ok(());
    };
    if axis_count > budget.max_variation_axes {
        return Err(FontSourceBudgetError::VariationAxisCount {
            face_index,
            limit: budget.max_variation_axes,
            actual: axis_count,
        });
    }
    if instance_count > budget.max_variation_instances {
        return Err(FontSourceBudgetError::VariationInstanceCount {
            face_index,
            limit: budget.max_variation_instances,
            actual: instance_count,
        });
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
