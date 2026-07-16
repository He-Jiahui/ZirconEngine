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
        self.opportunities.iter().fold(
            ClusterLineBreakFlags::default(),
            |mut flags, opportunity| {
                match opportunity.kind {
                    LineBreakKind::Soft if opportunity.byte_index == visual_end => {
                        flags.soft_break = true;
                    }
                    LineBreakKind::Mandatory
                        if opportunity.byte_index > visual_start
                            && opportunity.byte_index <= visual_end =>
                    {
                        flags.mandatory_break = true;
                    }
                    _ => {}
                }
                flags
            },
        )
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
