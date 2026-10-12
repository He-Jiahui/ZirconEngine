use crate::core::jobs::JobError;

use super::super::{ExportWizardJobEvent, ExportWizardJobSnapshot};

pub struct ExportWizardJobCompletion {
    pub events: Vec<ExportWizardJobEvent>,
    pub result: Result<ExportWizardJobSnapshot, JobError>,
}
