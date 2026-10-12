use super::*;
use crate::core::play::PlayInstanceId;

#[test]
fn retiring_repeated_play_routes_keeps_selection_storage_bounded() {
    let mut context = CoreEditContext::default();

    for raw in 1..=256 {
        let world_domain = WorldDomain::Play(PlayInstanceId::for_test(raw));
        context
            .selections
            .insert(world_domain, SelectionSnapshot::fixture_value(raw, raw));

        context
            .retire_world_route(world_domain)
            .expect("an inactive play route should retire");
        assert_eq!(context.selections.len(), 1);
        assert!(context.selections.contains_key(&WorldDomain::Edit));
    }
}

#[test]
fn retiring_the_active_world_route_is_rejected() {
    let mut context = CoreEditContext::default();
    let world_domain = WorldDomain::Play(PlayInstanceId::for_test(1));
    context
        .selections
        .insert(world_domain, SelectionSnapshot::fixture_value(1, 1));
    context.active_route = Some(EditWorldRoute::logical(world_domain));

    assert!(matches!(
        context.retire_world_route(world_domain),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    assert!(context.selections.contains_key(&world_domain));
}

use std::sync::atomic::{AtomicUsize, Ordering};

use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{DefaultLevelManager, World};
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::gateway::EditorRuntimeGateway;

const CARDINALITY_DISPATCH_ERROR: &str = "cardinality gateway returned a dispatch error";

struct CallbackCardinalityGateway {
    level: LevelSystem,
    callback_count: usize,
    dispatch_fails: bool,
    dispatch_calls: Arc<AtomicUsize>,
    callback_attempts: Arc<AtomicUsize>,
}

impl EditorRuntimeGateway for CallbackCardinalityGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::detached()
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        self.dispatch_calls.fetch_add(1, Ordering::SeqCst);
        self.level.with_world(read);
        Ok(())
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.dispatch_calls.fetch_add(1, Ordering::SeqCst);
        self.level.with_world_mut(|world| {
            for _ in 0..self.callback_count {
                self.callback_attempts.fetch_add(1, Ordering::SeqCst);
                write(world);
            }
        });
        if self.dispatch_fails {
            Err(GatewayError::Protocol {
                message: CARDINALITY_DISPATCH_ERROR.to_owned(),
            })
        } else {
            Ok(())
        }
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

fn context_gateway_diagnostic(error: &EditCommandError) -> &GatewayError {
    let EditCommandError::ExternalEffect { source } = error else {
        panic!("the actual gateway diagnostic must retain its error type");
    };
    source
        .downcast_ref::<GatewayError>()
        .expect("a boxed GatewayError must survive the Context boundary")
}

#[test]
fn scene_mutation_retains_callback_result_and_ordered_gateway_diagnostics() {
    for callback_count in [0, 1, 2] {
        for dispatch_fails in [false, true] {
            let level = DefaultLevelManager::default().create_default_level();
            let initial_node_count = level.with_world(|world| world.nodes().len());
            let dispatch_calls = Arc::new(AtomicUsize::new(0));
            let callback_attempts = Arc::new(AtomicUsize::new(0));
            let user_writes = Arc::new(AtomicUsize::new(0));
            let gateway = EditorRuntimeGatewayHandle::detached();
            gateway
                .replace(Arc::new(CallbackCardinalityGateway {
                    level: level.clone(),
                    callback_count,
                    dispatch_fails,
                    dispatch_calls: dispatch_calls.clone(),
                    callback_attempts: callback_attempts.clone(),
                }))
                .unwrap();
            let context = CoreEditContext::new(gateway);

            let outcome = context.with_scene_mut(|scene| {
                user_writes.fetch_add(1, Ordering::SeqCst);
                scene.spawn_node(NodeKind::Cube)
            });
            assert_eq!(dispatch_calls.load(Ordering::SeqCst), 1);
            assert_eq!(callback_attempts.load(Ordering::SeqCst), callback_count);
            assert_eq!(
                user_writes.load(Ordering::SeqCst),
                usize::from(callback_count != 0)
            );
            assert_eq!(
                level.with_world(|world| world.nodes().len()),
                initial_node_count + usize::from(callback_count != 0)
            );

            if callback_count == 0 {
                let error = match outcome {
                    Err(error) => error,
                    Ok(_) => panic!("an unexecuted callback must not produce an outcome"),
                };
                if dispatch_fails {
                    assert_eq!(
                        context_gateway_diagnostic(&error),
                        &GatewayError::Protocol {
                            message: CARDINALITY_DISPATCH_ERROR.to_owned(),
                        }
                    );
                } else {
                    assert!(matches!(
                        error,
                        EditCommandError::TargetMissing { target }
                            if target == "active editor scene"
                    ));
                }
                continue;
            }

            let (result, diagnostics) = outcome.unwrap().into_parts();
            let node = result.expect("the one legal scene write should succeed");
            assert!(level.with_world(|world| world.node_record(node).is_some()));
            assert_eq!(
                diagnostics.len(),
                usize::from(callback_count > 1) + usize::from(dispatch_fails)
            );
            if diagnostics.is_empty() {
                assert_eq!(diagnostics.capacity(), 0);
            }
            if callback_count > 1 {
                assert_eq!(
                    context_gateway_diagnostic(&diagnostics[0]),
                    &GatewayError::Protocol {
                        message: "borrowed world callback was invoked more than once".to_owned(),
                    }
                );
            }
            if dispatch_fails {
                assert_eq!(
                    context_gateway_diagnostic(&diagnostics[usize::from(callback_count > 1)]),
                    &GatewayError::Protocol {
                        message: CARDINALITY_DISPATCH_ERROR.to_owned(),
                    }
                );
            }
        }
    }
}

#[test]
fn scene_mutation_retains_ordinary_callback_failure_with_both_gateway_diagnostics() {
    let level = DefaultLevelManager::default().create_default_level();
    let dispatch_calls = Arc::new(AtomicUsize::new(0));
    let callback_attempts = Arc::new(AtomicUsize::new(0));
    let user_writes = Arc::new(AtomicUsize::new(0));
    let gateway = EditorRuntimeGatewayHandle::detached();
    gateway
        .replace(Arc::new(CallbackCardinalityGateway {
            level,
            callback_count: 2,
            dispatch_fails: true,
            dispatch_calls: dispatch_calls.clone(),
            callback_attempts: callback_attempts.clone(),
        }))
        .unwrap();
    let context = CoreEditContext::new(gateway);

    let (result, diagnostics) = context
        .with_scene_mut(|_| {
            user_writes.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(EditCommandError::TargetMissing {
                target: "ordinary callback target".to_owned(),
            })
        })
        .unwrap()
        .into_parts();

    assert!(matches!(
        result,
        Err(EditCommandError::TargetMissing { target }) if target == "ordinary callback target"
    ));
    assert_eq!(dispatch_calls.load(Ordering::SeqCst), 1);
    assert_eq!(callback_attempts.load(Ordering::SeqCst), 2);
    assert_eq!(user_writes.load(Ordering::SeqCst), 1);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        context_gateway_diagnostic(&diagnostics[0]),
        &GatewayError::Protocol {
            message: "borrowed world callback was invoked more than once".to_owned(),
        }
    );
    assert_eq!(
        context_gateway_diagnostic(&diagnostics[1]),
        &GatewayError::Protocol {
            message: CARDINALITY_DISPATCH_ERROR.to_owned(),
        }
    );
}
