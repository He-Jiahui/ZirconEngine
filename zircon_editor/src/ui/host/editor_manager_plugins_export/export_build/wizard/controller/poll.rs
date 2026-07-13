use crate::core::jobs::JobError;

use super::super::{ExportWizardJobEvent, ExportWizardJobSnapshot};

pub(in super::super) enum ExportWizardJobPoll {
    Pending,
    Completed {
        events: Vec<ExportWizardJobEvent>,
        snapshot: ExportWizardJobSnapshot,
    },
    Failed {
        events: Vec<ExportWizardJobEvent>,
        error: JobError,
    },
}
