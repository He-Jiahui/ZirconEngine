use std::sync::Arc;

use super::super::super::super::*;
use crate::core::{CoreError, CoreResult, InitLevel, ModuleContext, ModuleLifecycle};

#[derive(Debug)]
struct ReentrantBatchLifecycle;

impl ModuleLifecycle for ReentrantBatchLifecycle {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        let core = context
            .core
            .upgrade()
            .expect("module callback must retain a live runtime handle");
        let error = core
            .activate_registered_modules()
            .expect_err("same-thread batch reentry must be rejected");
        assert!(matches!(
            error,
            CoreError::ModuleLifecycleCommandReentrant { module, command }
                if module == context.module_name.as_str() && command == "activate"
        ));
        Ok(())
    }
}

#[test]
fn batch_acquire_error_completes_tokens_owned_before_the_reentrant_module() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(
            ModuleDescriptor::new("BatchLeasePrefix", "batch token acquired before reentry")
                .with_init_level(InitLevel::Kernel),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("BatchLeaseReentrantOwner", "batch reentry owner")
                .with_init_level(InitLevel::Post)
                .with_lifecycle(Arc::new(ReentrantBatchLifecycle)),
        )
        .unwrap();

    runtime.activate_module("BatchLeaseReentrantOwner").unwrap();
    runtime.activate_module("BatchLeasePrefix").unwrap();
}
