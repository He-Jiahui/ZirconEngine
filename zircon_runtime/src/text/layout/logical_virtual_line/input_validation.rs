use crate::text::TextRange;

use super::LogicalVirtualFragmentRole;

pub(super) fn logical_virtual_sequence_input_is_valid(
    text: &str,
    logical_ranges: &[TextRange],
    source_ranges: &[TextRange],
    style_owner_source_ranges: &[Option<TextRange>],
    replaced_source_ranges: &[Option<TextRange>],
    external_clusters: &[bool],
    virtual_roles: &[Option<LogicalVirtualFragmentRole>],
) -> bool {
    logical_ranges.len() == source_ranges.len()
        && source_ranges.len() == style_owner_source_ranges.len()
        && source_ranges.len() == replaced_source_ranges.len()
        && source_ranges.len() == external_clusters.len()
        && source_ranges.len() == virtual_roles.len()
        && source_ranges.iter().all(|range| range.start <= range.end)
        && style_owner_source_ranges
            .iter()
            .flatten()
            .all(|range| range.start < range.end)
        && replaced_source_ranges
            .iter()
            .flatten()
            .all(|range| range.start < range.end)
        && source_ranges
            .iter()
            .zip(external_clusters)
            .all(|(range, external)| !*external || range.start < range.end)
        && source_ranges
            .iter()
            .zip(virtual_roles)
            .all(|(range, role)| role.is_none() || range.start == range.end)
        && virtual_roles
            .iter()
            .zip(style_owner_source_ranges)
            .all(|(role, style_owner)| role.is_none() || style_owner.is_some())
        && virtual_roles
            .iter()
            .zip(replaced_source_ranges)
            .all(|(role, replaced)| {
                !matches!(role, Some(LogicalVirtualFragmentRole::DiscretionaryHyphen))
                    || replaced.is_some()
            })
        && logical_ranges
            .iter()
            .zip(virtual_roles)
            .all(|(logical_range, role)| {
                let grapheme = text.get(logical_range.start..logical_range.end);
                (!matches!(role, Some(LogicalVirtualFragmentRole::Ellipsis))
                    || grapheme == Some("\u{2026}"))
                    && (!matches!(role, Some(LogicalVirtualFragmentRole::DiscretionaryHyphen))
                        || grapheme == Some("-"))
            })
        && (source_ranges.iter().any(|range| range.start == range.end)
            || external_clusters.iter().any(|external| *external))
}
