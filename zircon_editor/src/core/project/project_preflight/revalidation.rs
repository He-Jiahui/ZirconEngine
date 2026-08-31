use zircon_runtime_interface::project::ProjectManifestDigest;

use super::ProjectPreflightReceipt;

/// Result of re-reading a preflighted manifest immediately before session admission.
///
/// A replacement is an expected concurrent outcome, not a parsing error. The caller must discard
/// its previous policy decision and evaluate `observed` again before it can acquire a writer lease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectPreflightRevalidation {
    Unchanged {
        current: ProjectPreflightReceipt,
    },
    Changed {
        expected: ProjectManifestDigest,
        observed: ProjectPreflightReceipt,
    },
}
