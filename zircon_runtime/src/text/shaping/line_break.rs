use unicode_linebreak::{linebreaks, BreakOpportunity};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ClusterLineBreakFlags {
    pub soft_break: bool,
    pub mandatory_break: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineBreakKind {
    Soft,
    Mandatory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LineBreakOpportunity {
    byte_index: usize,
    kind: LineBreakKind,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LineBreakOpportunityMap {
    opportunities: Vec<LineBreakOpportunity>,
}

impl LineBreakOpportunityMap {
    pub(crate) fn new(text: &str) -> Self {
        let opportunities = linebreaks(text)
            .filter_map(|(byte_index, opportunity)| match opportunity {
                BreakOpportunity::Allowed => Some(LineBreakOpportunity {
                    byte_index,
                    kind: LineBreakKind::Soft,
                }),
                BreakOpportunity::Mandatory if is_content_mandatory_break(text, byte_index) => {
                    Some(LineBreakOpportunity {
                        byte_index,
                        kind: LineBreakKind::Mandatory,
                    })
                }
                BreakOpportunity::Mandatory => None,
            })
            .collect();

        Self { opportunities }
    }

    pub(crate) fn flags_for_cluster(
        &self,
        visual_start: usize,
        visual_end: usize,
    ) -> ClusterLineBreakFlags {
        if visual_start > visual_end {
            return ClusterLineBreakFlags::default();
        }
        let first = self
            .opportunities
            .partition_point(|opportunity| opportunity.byte_index < visual_start);
        let end = self
            .opportunities
            .partition_point(|opportunity| opportunity.byte_index <= visual_end);
        let mut flags = ClusterLineBreakFlags::default();
        for opportunity in &self.opportunities[first..end] {
            match opportunity.kind {
                LineBreakKind::Soft if opportunity.byte_index == visual_end => {
                    flags.soft_break = true;
                }
                LineBreakKind::Mandatory if opportunity.byte_index > visual_start => {
                    flags.mandatory_break = true;
                }
                _ => {}
            }
        }
        flags
    }
}

fn is_content_mandatory_break(text: &str, byte_index: usize) -> bool {
    let preceding_text = text.get(..byte_index.min(text.len())).unwrap_or_default();
    preceding_text.ends_with('\n')
        || preceding_text.ends_with('\r')
        || preceding_text.ends_with('\u{000b}')
        || preceding_text.ends_with('\u{000c}')
        || preceding_text.ends_with('\u{0085}')
        || preceding_text.ends_with('\u{2028}')
        || preceding_text.ends_with('\u{2029}')
}

#[cfg(test)]
mod tests {
    use super::{
        ClusterLineBreakFlags, LineBreakKind, LineBreakOpportunity, LineBreakOpportunityMap,
    };

    #[test]
    fn cluster_flags_only_visit_the_cluster_opportunity_window() {
        let map = LineBreakOpportunityMap {
            opportunities: vec![
                LineBreakOpportunity {
                    byte_index: 2,
                    kind: LineBreakKind::Soft,
                },
                LineBreakOpportunity {
                    byte_index: 4,
                    kind: LineBreakKind::Mandatory,
                },
                LineBreakOpportunity {
                    byte_index: 8,
                    kind: LineBreakKind::Soft,
                },
            ],
        };

        assert_eq!(
            map.flags_for_cluster(0, 2),
            ClusterLineBreakFlags {
                soft_break: true,
                mandatory_break: false,
            }
        );
        assert_eq!(
            map.flags_for_cluster(2, 4),
            ClusterLineBreakFlags {
                soft_break: false,
                mandatory_break: true,
            }
        );
        assert_eq!(
            map.flags_for_cluster(5, 8),
            ClusterLineBreakFlags {
                soft_break: true,
                mandatory_break: false,
            }
        );
    }

    #[test]
    fn cluster_flags_do_not_restore_a_full_opportunity_fold() {
        let source = include_str!("line_break.rs");
        let compact = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        assert!(!compact.contains(concat!("self.opportunities.iter()", ".fold")));
    }
}
