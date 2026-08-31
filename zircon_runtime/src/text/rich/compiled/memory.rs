use std::mem::size_of;
use std::sync::Arc;

use crate::text::{InlineObjectRef, LinkRef, OpenTypeFeature};

use super::{CompiledRichText, RichTableCellProjectionIndex, RichTextDependency};

pub(super) fn calculate_estimated_bytes(compiled: &CompiledRichText) -> usize {
    let run_metadata_bytes = compiled
        .parsed
        .runs
        .iter()
        .map(|run| {
            run.style
                .family
                .as_ref()
                .map_or(0, |family| family.0.capacity())
                + run.style.features.as_ref().map_or(0, |features| {
                    features.capacity() * size_of::<OpenTypeFeature>()
                })
                + run.link.as_ref().map_or(0, LinkRef::retained_heap_bytes)
                + match run.inline.as_ref() {
                    Some(InlineObjectRef::Image {
                        alternative_text,
                        tooltip,
                        ..
                    }) => {
                        alternative_text.as_ref().map_or(0, String::capacity)
                            + tooltip.as_ref().map_or(0, String::capacity)
                    }
                    Some(InlineObjectRef::Icon {
                        alternative_text, ..
                    }) => alternative_text.as_ref().map_or(0, String::capacity),
                    Some(InlineObjectRef::Widget { .. }) | None => 0,
                }
        })
        .sum::<usize>();
    let semantic_text_bytes = if Arc::ptr_eq(&compiled.semantic_text, &compiled.parsed.text) {
        0
    } else {
        compiled.semantic_text.len()
    };
    let table_bytes = compiled
        .parsed
        .tables
        .iter()
        .map(|table| {
            table.columns.capacity() * size_of::<crate::text::RichTableColumn>()
                + table.cells.capacity() * size_of::<crate::text::RichTableCell>()
        })
        .sum::<usize>();
    size_of::<CompiledRichText>()
        .saturating_add(compiled.source_markup.len())
        .saturating_add(compiled.parsed.text.len())
        .saturating_add(semantic_text_bytes)
        .saturating_add(compiled.parsed.runs.capacity() * size_of::<crate::text::StyledRun>())
        .saturating_add(
            compiled.parsed.paragraphs.capacity()
                * size_of::<((u32, u32), crate::text::ParagraphOverride)>(),
        )
        .saturating_add(compiled.parsed.tables.capacity() * size_of::<crate::text::RichTable>())
        .saturating_add(
            compiled.parsed.authoring_diagnostics.capacity()
                * size_of::<crate::text::RichTextAuthoringDiagnostic>(),
        )
        .saturating_add(compiled.inline_run_indices.len() * size_of::<u32>())
        .saturating_add(compiled.link_run_indices.len() * size_of::<u32>())
        .saturating_add(compiled.dependencies.len() * size_of::<RichTextDependency>())
        .saturating_add(
            compiled.table_cell_projection_indices.len()
                * size_of::<RichTableCellProjectionIndex>(),
        )
        .saturating_add(compiled.cell_run_indices.len() * size_of::<u32>())
        .saturating_add(compiled.cell_paragraph_indices.len() * size_of::<u32>())
        .saturating_add(compiled.cell_nested_table_indices.len() * size_of::<u32>())
        .saturating_add(run_metadata_bytes)
        .saturating_add(table_bytes)
}
