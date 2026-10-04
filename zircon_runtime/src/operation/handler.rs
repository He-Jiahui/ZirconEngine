use std::any::Any;

use super::{RuntimeOperationContext, RuntimeOperationHandlerError};

pub struct RuntimeOperationSnapshot {
    payload: serde_json::Value,
    owner_state: Option<Box<dyn Any + Send>>,
    owner_bytes: usize,
}

impl RuntimeOperationSnapshot {
    pub fn new(payload: serde_json::Value) -> Self {
        Self {
            payload,
            owner_state: None,
            owner_bytes: 0,
        }
    }
    pub fn with_owner_state(
        payload: serde_json::Value,
        owner_state: Box<dyn Any + Send>,
        owner_bytes: usize,
    ) -> Self {
        Self {
            payload,
            owner_state: Some(owner_state),
            owner_bytes,
        }
    }
    pub fn into_parts(self) -> (serde_json::Value, Option<Box<dyn Any + Send>>, usize) {
        (self.payload, self.owner_state, self.owner_bytes)
    }
    pub fn owner_bytes(&self) -> usize {
        self.owner_bytes
    }
}

pub struct RuntimeOperationApply {
    command: serde_json::Value,
    owner_state: Option<Box<dyn Any + Send>>,
}

impl RuntimeOperationApply {
    pub fn new(command: serde_json::Value, owner_state: Option<Box<dyn Any + Send>>) -> Self {
        Self {
            command,
            owner_state,
        }
    }
    pub fn into_parts(self) -> (serde_json::Value, Option<Box<dyn Any + Send>>) {
        (self.command, self.owner_state)
    }
}

/// Worker-owned command and result produced before an owner-thread mutation.
pub struct RuntimeOperationPrepared {
    command: serde_json::Value,
    result: serde_json::Value,
    owner_state: Option<Box<dyn Any + Send>>,
    owner_bytes: usize,
}

impl RuntimeOperationPrepared {
    pub fn new(command: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            command,
            result,
            owner_state: None,
            owner_bytes: 0,
        }
    }
    pub fn with_owner_state(
        command: serde_json::Value,
        result: serde_json::Value,
        owner_state: Box<dyn Any + Send>,
        owner_bytes: usize,
    ) -> Self {
        Self {
            command,
            result,
            owner_state: Some(owner_state),
            owner_bytes,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        serde_json::Value,
        serde_json::Value,
        Option<Box<dyn Any + Send>>,
        usize,
    ) {
        (
            self.command,
            self.result,
            self.owner_state,
            self.owner_bytes,
        )
    }
}

pub trait RuntimeOperationHandler: Send + Sync {
    /// Captures immutable, owned input on the runtime owner thread.
    fn snapshot(
        &self,
        context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError>;

    fn snapshot_owned(
        &self,
        context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<RuntimeOperationSnapshot, RuntimeOperationHandlerError> {
        self.snapshot(context, payload)
            .map(RuntimeOperationSnapshot::new)
    }

    /// Produces an owned apply command and its bounded terminal result off-thread.
    fn prepare(
        &self,
        snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError>;

    fn prepare_owned(
        &self,
        snapshot: RuntimeOperationSnapshot,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        let (payload, _, _) = snapshot.into_parts();
        self.prepare(payload)
    }

    /// Commits a previously prepared command on the runtime owner thread.
    fn apply(
        &self,
        context: RuntimeOperationContext<'_>,
        command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError>;

    fn apply_owned(
        &self,
        context: RuntimeOperationContext<'_>,
        apply: RuntimeOperationApply,
    ) -> Result<(), RuntimeOperationHandlerError> {
        let (command, _) = apply.into_parts();
        self.apply(context, command)
    }
}
