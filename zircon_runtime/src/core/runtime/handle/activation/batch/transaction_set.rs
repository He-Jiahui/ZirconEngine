use crate::core::{CoreError, LifecycleState};

use super::super::super::super::state::{ModuleLifecycleCommand, ModuleLifecycleTransitionToken};
use super::super::super::CoreHandle;

// 批量激活已占有的转换令牌必须全部完成；出错或 unwind 时也要唤醒等待者。
pub(in super::super) struct LifecycleTransactionSet<'a> {
    handle: &'a CoreHandle,
    module_name: String,
    tokens: Vec<LifecycleTransitionReceipt>,
}

struct LifecycleTransitionReceipt {
    module_name: String,
    token: ModuleLifecycleTransitionToken,
    preexisting_running: bool,
    result: Option<Result<(), CoreError>>,
}

impl<'a> LifecycleTransactionSet<'a> {
    pub(in super::super) fn new(
        handle: &'a CoreHandle,
        capacity: usize,
        module_name: impl Into<String>,
    ) -> Self {
        Self {
            handle,
            module_name: module_name.into(),
            tokens: Vec::with_capacity(capacity),
        }
    }

    pub(in super::super) fn push(
        &mut self,
        module_name: &str,
        token: ModuleLifecycleTransitionToken,
        preexisting_running: bool,
    ) {
        self.tokens.push(LifecycleTransitionReceipt {
            module_name: module_name.to_owned(),
            token,
            preexisting_running,
            result: None,
        });
    }

    pub(in super::super) fn complete_module(
        &mut self,
        module_name: &str,
        result: Result<(), CoreError>,
    ) {
        if let Some(receipt) = self
            .tokens
            .iter_mut()
            .find(|item| item.module_name == module_name)
        {
            receipt.result = Some(result);
        }
    }

    // A later dependent failure preserves only a provider that was already Running
    // when this reservation was taken; a newly-owned member is part of the failed
    // transaction even if its callback published Running before the failure.
    pub(in super::super) fn complete_running_modules_with_error(&mut self, error: &CoreError) {
        let modules = self.handle.lock_modules();
        for receipt in &mut self.tokens {
            if receipt.result.is_some() {
                continue;
            }
            receipt.result = Some(
                if receipt.preexisting_running
                    && modules
                        .get(receipt.module_name.as_str())
                        .is_some_and(|entry| entry.lifecycle == LifecycleState::Running)
                {
                    Ok(())
                } else {
                    Err(error.clone())
                },
            );
        }
    }

    pub(in super::super) fn finish(
        mut self,
        result: Result<(), CoreError>,
    ) -> Result<(), CoreError> {
        if let Err(error) = &result {
            self.complete_running_modules_with_error(error);
            for receipt in &mut self.tokens {
                if !receipt.preexisting_running
                    && receipt
                        .result
                        .as_ref()
                        .is_some_and(|receipt_result| receipt_result.is_ok())
                {
                    receipt.result = Some(Err(error.clone()));
                }
            }
        }
        for receipt in &mut self.tokens {
            if receipt.result.is_none() {
                receipt.result = Some(result.clone());
            }
        }
        self.complete_all();
        result
    }

    fn complete_all(&mut self) {
        let receipts = std::mem::take(&mut self.tokens);
        for receipt in receipts.into_iter().rev() {
            let result = receipt.result.unwrap_or_else(|| {
                Err(CoreError::ModuleLifecycleCallbackPanicked {
                    module: self.module_name.clone(),
                    command: ModuleLifecycleCommand::Activate.as_str(),
                })
            });
            self.handle
                .complete_module_lifecycle_transition(&receipt.token, result);
        }
    }
}

impl Drop for LifecycleTransactionSet<'_> {
    fn drop(&mut self) {
        if self.tokens.is_empty() {
            return;
        }
        for receipt in &mut self.tokens {
            if receipt.result.is_none() {
                receipt.result = Some(Err(CoreError::ModuleLifecycleCallbackPanicked {
                    module: self.module_name.clone(),
                    command: ModuleLifecycleCommand::Activate.as_str(),
                }));
            }
        }
        self.complete_all();
    }
}
