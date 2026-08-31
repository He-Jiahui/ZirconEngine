use super::{
    CommandList, CommandListCommand, DeviceGeneration, DeviceId, DiagnosticPassQueryScope,
    DiagnosticQueryPlan, RenderQueueClass, RhiError,
};

/// Immutable, device-generation-qualified command collection for one logical
/// queue submission.
///
/// Command lists keep their own render/compute pass scopes. The packet only
/// establishes the queue timeline and resource-retirement boundary shared by
/// those lists; it never exposes backend command buffers or queue objects.
pub struct RhiSubmissionPacket {
    device_id: DeviceId,
    generation: DeviceGeneration,
    queue_class: RenderQueueClass,
    command_lists: Vec<Box<dyn CommandList>>,
    diagnostic_query_plan: Option<DiagnosticQueryPlan>,
}

impl RhiSubmissionPacket {
    pub fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        queue_class: RenderQueueClass,
        command_lists: Vec<Box<dyn CommandList>>,
    ) -> Result<Self, RhiError> {
        if command_lists.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        for command_list in &command_lists {
            if command_list.queue_class() != queue_class {
                return Err(RhiError::SubmissionPacketQueueMismatch {
                    packet_queue: queue_class,
                    command_queue: command_list.queue_class(),
                });
            }
        }
        let packet = Self {
            device_id,
            generation,
            queue_class,
            command_lists,
            diagnostic_query_plan: None,
        };
        packet.validate_diagnostic_scopes()?;
        Ok(packet)
    }

    /// Creates a packet and attaches its diagnostic plan atomically, so scope
    /// validation cannot observe an intermediate no-plan packet.
    pub fn new_with_diagnostic_query_plan(
        device_id: DeviceId,
        generation: DeviceGeneration,
        queue_class: RenderQueueClass,
        command_lists: Vec<Box<dyn CommandList>>,
        diagnostic_query_plan: DiagnosticQueryPlan,
    ) -> Result<Self, RhiError> {
        if diagnostic_query_plan.frame_index().is_none() {
            return Err(RhiError::DiagnosticQueryFrameIndexRequired);
        }
        let packet = Self {
            device_id,
            generation,
            queue_class,
            command_lists,
            diagnostic_query_plan: Some(diagnostic_query_plan),
        };
        packet.validate_shape()?;
        packet.validate_diagnostic_scopes()?;
        Ok(packet)
    }

    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub const fn queue_class(&self) -> RenderQueueClass {
        self.queue_class
    }

    pub fn command_list_count(&self) -> usize {
        self.command_lists.len()
    }

    pub fn command_lists(&self) -> &[Box<dyn CommandList>] {
        &self.command_lists
    }

    /// Attaches the bounded, frame-qualified query plan that owns every
    /// diagnostics scope recorded by this packet.
    pub fn with_diagnostic_query_plan(
        mut self,
        diagnostic_query_plan: DiagnosticQueryPlan,
    ) -> Result<Self, RhiError> {
        if diagnostic_query_plan.frame_index().is_none() {
            return Err(RhiError::DiagnosticQueryFrameIndexRequired);
        }
        self.diagnostic_query_plan = Some(diagnostic_query_plan);
        self.validate_diagnostic_scopes()?;
        Ok(self)
    }

    pub fn diagnostic_query_plan(&self) -> Option<&DiagnosticQueryPlan> {
        self.diagnostic_query_plan.as_ref()
    }

    pub fn into_command_lists(self) -> Vec<Box<dyn CommandList>> {
        self.command_lists
    }

    fn validate_diagnostic_scopes(&self) -> Result<(), RhiError> {
        let scopes = self
            .command_lists
            .iter()
            .flat_map(|command_list| command_list.recorded_commands())
            .filter_map(command_scope)
            .collect::<Vec<_>>();
        if scopes.iter().any(|scope| scope.is_empty()) {
            return Err(RhiError::EmptyDiagnosticPassScope);
        }
        match self.diagnostic_query_plan.as_ref() {
            Some(plan) => plan
                .validate_submission_scopes(&scopes)
                .map_err(RhiError::from),
            None if scopes.is_empty() => Ok(()),
            None => Err(RhiError::DiagnosticQueryPlanRequired),
        }
    }

    fn validate_shape(&self) -> Result<(), RhiError> {
        if self.command_lists.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        for command_list in &self.command_lists {
            if command_list.queue_class() != self.queue_class {
                return Err(RhiError::SubmissionPacketQueueMismatch {
                    packet_queue: self.queue_class,
                    command_queue: command_list.queue_class(),
                });
            }
        }
        Ok(())
    }
}

fn command_scope(command: &CommandListCommand) -> Option<DiagnosticPassQueryScope> {
    match command {
        CommandListCommand::BeginRenderPassWithDiagnostics {
            diagnostic_scope, ..
        }
        | CommandListCommand::BeginComputePassWithDiagnostics {
            diagnostic_scope, ..
        } => Some(*diagnostic_scope),
        _ => None,
    }
}
