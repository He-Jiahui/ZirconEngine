//! 向启动边界说明旧预检证据能否继续使用，或必须以新证据替换；调用端不能把先前路径或清单摘要当作永久授权。
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
