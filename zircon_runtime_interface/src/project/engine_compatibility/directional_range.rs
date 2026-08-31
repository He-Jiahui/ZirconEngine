use std::cmp::Ordering;

use semver::{Comparator, Op, Version, VersionReq};

use super::ProjectEngineCompatibilityDisposition;

#[derive(Clone, Debug)]
struct VersionBound {
    version: Version,
    inclusive: bool,
}

#[derive(Clone, Debug, Default)]
struct VersionRange {
    minimum: Option<VersionBound>,
    maximum: Option<VersionBound>,
}

impl VersionRange {
    fn intersect(self, other: Self) -> Self {
        Self {
            minimum: stricter_minimum(self.minimum, other.minimum),
            maximum: stricter_maximum(self.maximum, other.maximum),
        }
    }

    fn is_empty(&self) -> bool {
        let (Some(minimum), Some(maximum)) = (&self.minimum, &self.maximum) else {
            return false;
        };
        match minimum.version.cmp(&maximum.version) {
            Ordering::Less => false,
            Ordering::Greater => true,
            Ordering::Equal => !(minimum.inclusive && maximum.inclusive),
        }
    }

    fn directional_disposition(&self, running: &Version) -> ProjectEngineCompatibilityDisposition {
        if self.is_empty() {
            return ProjectEngineCompatibilityDisposition::Incompatible;
        }
        if self.minimum.as_ref().is_some_and(|minimum| {
            running < &minimum.version || (running == &minimum.version && !minimum.inclusive)
        }) {
            return ProjectEngineCompatibilityDisposition::ProjectRequiresNewerEngine;
        }
        if self.maximum.as_ref().is_some_and(|maximum| {
            running > &maximum.version || (running == &maximum.version && !maximum.inclusive)
        }) {
            return ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine;
        }
        ProjectEngineCompatibilityDisposition::Incompatible
    }
}

/// Returns a direction only when the entire requirement can be reduced to a stable-version
/// interval. Prerelease ranges intentionally stay indeterminate because Cargo's prerelease
/// admission rules cannot be represented by ordinary inclusive bounds.
pub(super) fn classify_incompatible_requirement(
    requirement: &VersionReq,
    running: &Version,
) -> ProjectEngineCompatibilityDisposition {
    if !running.pre.is_empty()
        || requirement
            .comparators
            .iter()
            .any(|comparator| !comparator.pre.is_empty())
    {
        return ProjectEngineCompatibilityDisposition::Incompatible;
    }

    let Some(range) = requirement
        .comparators
        .iter()
        .map(comparator_range)
        .try_fold(VersionRange::default(), |current, next| {
            next.map(|next| current.intersect(next))
        })
    else {
        return ProjectEngineCompatibilityDisposition::Incompatible;
    };
    range.directional_disposition(running)
}

fn comparator_range(comparator: &Comparator) -> Option<VersionRange> {
    let base = comparator_version(comparator);
    match comparator.op {
        Op::Exact if comparator.patch.is_some() => Some(exact_range(base)),
        Op::Exact | Op::Wildcard => prefix_range(&base, comparator.minor.is_some()),
        Op::Greater => Some(VersionRange {
            minimum: Some(VersionBound {
                version: greater_lower_bound(&base, comparator.minor, comparator.patch)?,
                inclusive: comparator.patch.is_none(),
            }),
            maximum: None,
        }),
        Op::GreaterEq => Some(VersionRange {
            minimum: Some(VersionBound {
                version: base,
                inclusive: true,
            }),
            maximum: None,
        }),
        Op::Less => Some(VersionRange {
            minimum: None,
            maximum: Some(VersionBound {
                version: base,
                inclusive: false,
            }),
        }),
        Op::LessEq => Some(VersionRange {
            minimum: None,
            maximum: Some(VersionBound {
                version: less_equal_upper_bound(&base, comparator.minor, comparator.patch)?,
                inclusive: comparator.minor.is_some() && comparator.patch.is_some(),
            }),
        }),
        Op::Tilde => Some(VersionRange {
            minimum: Some(VersionBound {
                version: base.clone(),
                inclusive: true,
            }),
            maximum: Some(VersionBound {
                version: if comparator.minor.is_some() {
                    next_minor(&base)?
                } else {
                    next_major(&base)?
                },
                inclusive: false,
            }),
        }),
        Op::Caret => Some(caret_range(&base, comparator.minor, comparator.patch)?),
        _ => None,
    }
}

fn comparator_version(comparator: &Comparator) -> Version {
    let mut version = Version::new(
        comparator.major,
        comparator.minor.unwrap_or_default(),
        comparator.patch.unwrap_or_default(),
    );
    version.pre = comparator.pre.clone();
    version
}

fn exact_range(version: Version) -> VersionRange {
    VersionRange {
        minimum: Some(VersionBound {
            version: version.clone(),
            inclusive: true,
        }),
        maximum: Some(VersionBound {
            version,
            inclusive: true,
        }),
    }
}

fn prefix_range(base: &Version, has_minor: bool) -> Option<VersionRange> {
    Some(VersionRange {
        minimum: Some(VersionBound {
            version: base.clone(),
            inclusive: true,
        }),
        maximum: Some(VersionBound {
            version: if has_minor {
                next_minor(base)?
            } else {
                next_major(base)?
            },
            inclusive: false,
        }),
    })
}

fn caret_range(base: &Version, minor: Option<u64>, patch: Option<u64>) -> Option<VersionRange> {
    let upper = match (base.major, minor, patch) {
        (_, None, _) => next_major(base)?,
        (1.., Some(_), _) => next_major(base)?,
        (0, Some(minor), _) if minor > 0 => next_minor(base)?,
        (0, Some(_), Some(_)) => next_patch(base)?,
        (0, Some(_), None) => next_minor(base)?,
    };
    Some(VersionRange {
        minimum: Some(VersionBound {
            version: base.clone(),
            inclusive: true,
        }),
        maximum: Some(VersionBound {
            version: upper,
            inclusive: false,
        }),
    })
}

fn greater_lower_bound(base: &Version, minor: Option<u64>, patch: Option<u64>) -> Option<Version> {
    match (minor, patch) {
        (None, _) => next_major(base),
        (Some(_), None) => next_minor(base),
        (Some(_), Some(_)) => Some(base.clone()),
    }
}

fn less_equal_upper_bound(
    base: &Version,
    minor: Option<u64>,
    patch: Option<u64>,
) -> Option<Version> {
    match (minor, patch) {
        (None, _) => next_major(base),
        (Some(_), None) => next_minor(base),
        (Some(_), Some(_)) => Some(base.clone()),
    }
}

fn next_major(version: &Version) -> Option<Version> {
    version
        .major
        .checked_add(1)
        .map(|major| Version::new(major, 0, 0))
}

fn next_minor(version: &Version) -> Option<Version> {
    version
        .minor
        .checked_add(1)
        .map(|minor| Version::new(version.major, minor, 0))
}

fn next_patch(version: &Version) -> Option<Version> {
    version
        .patch
        .checked_add(1)
        .map(|patch| Version::new(version.major, version.minor, patch))
}

fn stricter_minimum(
    current: Option<VersionBound>,
    candidate: Option<VersionBound>,
) -> Option<VersionBound> {
    match (current, candidate) {
        (None, other) | (other, None) => other,
        (Some(current), Some(candidate)) => match current.version.cmp(&candidate.version) {
            Ordering::Less => Some(candidate),
            Ordering::Greater => Some(current),
            Ordering::Equal => Some(VersionBound {
                version: current.version,
                inclusive: current.inclusive && candidate.inclusive,
            }),
        },
    }
}

fn stricter_maximum(
    current: Option<VersionBound>,
    candidate: Option<VersionBound>,
) -> Option<VersionBound> {
    match (current, candidate) {
        (None, other) | (other, None) => other,
        (Some(current), Some(candidate)) => match current.version.cmp(&candidate.version) {
            Ordering::Less => Some(current),
            Ordering::Greater => Some(candidate),
            Ordering::Equal => Some(VersionBound {
                version: current.version,
                inclusive: current.inclusive && candidate.inclusive,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use semver::{Version, VersionReq};

    use super::{caret_range, classify_incompatible_requirement};
    use crate::project::engine_compatibility::ProjectEngineCompatibilityDisposition;

    fn classify(requirement: &str, running: &str) -> ProjectEngineCompatibilityDisposition {
        let requirement = VersionReq::parse(requirement).expect("test requirement must parse");
        let running = Version::parse(running).expect("test engine version must parse");
        classify_incompatible_requirement(&requirement, &running)
    }

    #[test]
    fn caret_with_nonzero_major_has_a_next_major_upper_bound() {
        assert_eq!(
            classify("^1.2", "2.0.0"),
            ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
        );
    }

    #[test]
    fn caret_with_omitted_minor_still_has_a_next_major_upper_bound() {
        assert_eq!(
            classify("^1", "2.0.0"),
            ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
        );
    }

    #[test]
    fn caret_with_zero_major_and_nonzero_minor_has_a_next_minor_upper_bound() {
        assert_eq!(
            classify("^0.2", "0.3.0"),
            ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
        );
    }

    #[test]
    fn caret_with_zero_major_minor_and_patch_has_a_next_patch_upper_bound() {
        assert_eq!(
            classify("^0.0.3", "0.0.4"),
            ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
        );
    }

    #[test]
    fn caret_with_zero_major_minor_and_omitted_patch_has_a_next_minor_upper_bound() {
        assert_eq!(
            classify("^0.0", "0.1.0"),
            ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
        );
    }

    #[test]
    fn caret_with_maximum_major_is_indeterminate_when_its_upper_bound_overflows() {
        let base = Version::new(u64::MAX, 2, 0);

        assert!(caret_range(&base, Some(2), None).is_none());
    }

    #[test]
    fn caret_with_maximum_zero_major_minor_is_indeterminate_on_overflow() {
        let base = Version::new(0, u64::MAX, 0);

        assert!(caret_range(&base, Some(u64::MAX), None).is_none());
    }

    #[test]
    fn caret_with_maximum_zero_major_patch_is_indeterminate_on_overflow() {
        let base = Version::new(0, 0, u64::MAX);

        assert!(caret_range(&base, Some(0), Some(u64::MAX)).is_none());
    }
}
