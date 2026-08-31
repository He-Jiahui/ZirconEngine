use super::TimelineSection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineSectionOverlapPolicy {
    Allow,
    Forbid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimelineSectionOverlapVerdict {
    Allowed,
    Rejected { existing_section_id: String },
}

impl TimelineSectionOverlapVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    pub fn is_rejected(&self) -> bool {
        !self.is_allowed()
    }
}

pub fn section_overlap_verdict(
    existing: &[TimelineSection],
    candidate: &TimelineSection,
    policy: TimelineSectionOverlapPolicy,
) -> TimelineSectionOverlapVerdict {
    if policy == TimelineSectionOverlapPolicy::Allow {
        return TimelineSectionOverlapVerdict::Allowed;
    }
    for section in existing {
        if section.id != candidate.id && ranges_overlap(section, candidate) {
            return TimelineSectionOverlapVerdict::Rejected {
                existing_section_id: section.id.clone(),
            };
        }
    }
    TimelineSectionOverlapVerdict::Allowed
}

fn ranges_overlap(left: &TimelineSection, right: &TimelineSection) -> bool {
    left.range.start < right.range.end && right.range.start < left.range.end
}
