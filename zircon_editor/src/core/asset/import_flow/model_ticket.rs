//! 在模型导入票据上保留运行时提交回执的类型边界，供宿主轮询或限时等待后完成目录交接；结果只能由一个消费端取走。
use std::fmt;
use std::path::PathBuf;
use std::time::Instant;

use zircon_runtime::asset::project::ProjectActivationIdentity;
use zircon_runtime::asset::ProjectImportReceipt;

use crate::core::jobs::{JobError, JobId, JobTicket};

pub struct EditorModelImportTicket {
    ticket: JobTicket<ProjectImportReceipt>,
    source_path: PathBuf,
    activation_identity: ProjectActivationIdentity,
}

impl EditorModelImportTicket {
    pub(super) fn new(
        ticket: JobTicket<ProjectImportReceipt>,
        source_path: PathBuf,
        activation_identity: ProjectActivationIdentity,
    ) -> Self {
        Self {
            ticket,
            source_path,
            activation_identity,
        }
    }

    pub fn project_activation_identity(&self) -> &ProjectActivationIdentity {
        &self.activation_identity
    }

    pub fn id(&self) -> JobId {
        self.ticket.id()
    }

    pub fn try_take(&self) -> Option<Result<ProjectImportReceipt, JobError>> {
        self.ticket.try_take()
    }

    pub fn wait_until(&self, deadline: Instant) -> Option<Result<ProjectImportReceipt, JobError>> {
        self.ticket.wait_until(deadline)
    }
}

impl fmt::Debug for EditorModelImportTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorModelImportTicket")
            .field("source_path", &self.source_path)
            .field("job_id", &self.id())
            .finish()
    }
}
