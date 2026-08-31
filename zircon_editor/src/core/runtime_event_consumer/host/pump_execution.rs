use std::time::Instant;

use super::super::{
    EditorRuntimeEventConsumerCallbackPhase, EditorRuntimeEventConsumerDeliveryDisposition,
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerQuarantineReason,
    EditorRuntimeEventPumpBudget, EditorRuntimeEventPumpReport,
};
use super::execution_support::{
    invoke_consumer_callback, p95_duration, validate_delivery, PumpExecutionGuard,
};
use super::pending::PendingDeliveryBatchRestoreGuard;
use super::{ActiveConsumerIdentity, EditorRuntimeEventConsumerHost};

impl EditorRuntimeEventConsumerHost {
    pub fn pump(&self) -> Result<usize, EditorRuntimeEventConsumerError> {
        Ok(self
            .pump_with_budget(EditorRuntimeEventPumpBudget::default())?
            .applied())
    }

    pub fn pump_with_budget(
        &self,
        budget: EditorRuntimeEventPumpBudget,
    ) -> Result<EditorRuntimeEventPumpReport, EditorRuntimeEventConsumerError> {
        let Some(_pump_guard) = PumpExecutionGuard::enter(&self.execution_state) else {
            return Ok(EditorRuntimeEventPumpReport::default());
        };
        let play_session_id = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwrap_or_default();
        if play_session_id == 0 {
            self.store_pump_report(EditorRuntimeEventPumpReport::default());
            return Ok(EditorRuntimeEventPumpReport::default());
        }
        let snapshots = self.snapshot_active_consumers();
        let started = Instant::now();
        let mut report = EditorRuntimeEventPumpReport::default();
        let mut runtime_drain_samples = Vec::with_capacity(snapshots.len());
        let mut decode_samples = Vec::with_capacity(snapshots.len());
        let mut visited_consumer_count = 0;
        let mut first_error = None;

        for snapshot in &snapshots {
            if report.applied() >= budget.max_events() || started.elapsed() >= budget.max_elapsed()
            {
                break;
            }
            visited_consumer_count += 1;
            if self.gateway.current_lease().identity() != snapshot.origin.identity() {
                report.record_stale_consumer();
                if let Err(error) = self.retire_active_consumer(
                    &ActiveConsumerIdentity {
                        consumer_id: snapshot.consumer_id.clone(),
                        subscription: snapshot.subscription.clone(),
                        generation: snapshot.generation,
                    },
                    play_session_id,
                ) {
                    first_error.get_or_insert(error);
                }
                continue;
            }
            if !snapshot.has_pending {
                let page = match snapshot
                    .origin
                    .gateway()
                    .drain_plugin_events(snapshot.subscription.raw())
                {
                    Ok(page) => page,
                    Err(message) => {
                        first_error.get_or_insert(EditorRuntimeEventConsumerError::Gateway {
                            consumer_id: snapshot.consumer_id.clone(),
                            message: message.to_string(),
                        });
                        continue;
                    }
                };
                if self.gateway.current_lease().identity() != snapshot.origin.identity() {
                    report.record_stale_consumer();
                    if let Err(error) = self.retire_active_consumer(
                        &ActiveConsumerIdentity {
                            consumer_id: snapshot.consumer_id.clone(),
                            subscription: snapshot.subscription.clone(),
                            generation: snapshot.generation,
                        },
                        play_session_id,
                    ) {
                        first_error.get_or_insert(error);
                    }
                    continue;
                }
                let page_encoded_bytes = page.encoded_bytes();
                let runtime_remaining_deliveries = page.runtime_remaining_deliveries();
                let runtime_oldest_pending_age_millis = page.runtime_oldest_pending_age_millis();
                report.record_drained_page(
                    page.deliveries().len(),
                    page_encoded_bytes,
                    page.runtime_drain_elapsed(),
                    page.decode_elapsed(),
                );
                runtime_drain_samples.push(page.runtime_drain_elapsed());
                decode_samples.push(page.decode_elapsed());
                report.record_dropped(self.append_drained_deliveries(
                    snapshot,
                    page.into_deliveries(),
                    page_encoded_bytes,
                    runtime_remaining_deliveries,
                    runtime_oldest_pending_age_millis,
                ));
            }

            let Some(batch) = self.take_pending_batch(snapshot) else {
                continue;
            };
            let mut pending = PendingDeliveryBatchRestoreGuard::new(self, snapshot, batch);
            let mut applied_for_consumer = 0;
            while report.applied() < budget.max_events()
                && applied_for_consumer < budget.max_events_per_consumer()
                && started.elapsed() < budget.max_elapsed()
            {
                let last_sequence = pending.batch().last_sequence();
                let Some(delivery) = pending.batch_mut().begin_current() else {
                    break;
                };
                if let Err(error) = validate_delivery(
                    snapshot,
                    snapshot.origin.session_handle().raw(),
                    last_sequence,
                    delivery.delivery(),
                ) {
                    let dropped = pending.batch_mut().complete_current(
                        EditorRuntimeEventConsumerDeliveryDisposition::DropWithReason,
                    );
                    self.release_pending_bytes(dropped.retained_bytes_upper_bound());
                    report.record_dropped(1);
                    first_error.get_or_insert(error);
                    break;
                }

                let sequence = delivery.delivery().sequence;
                let callback_started = Instant::now();
                let apply_result = invoke_consumer_callback(
                    &snapshot.consumer_id,
                    EditorRuntimeEventConsumerCallbackPhase::Consume,
                    Some(sequence),
                    || {
                        snapshot.registration.consume(
                            play_session_id,
                            sequence,
                            delivery.delivery().payload.as_ref(),
                        )
                    },
                );
                let callback_elapsed = callback_started.elapsed();
                match apply_result {
                    Ok(Ok(())) => {}
                    Ok(Err(source)) => {
                        let dropped = pending.batch_mut().complete_current(
                            EditorRuntimeEventConsumerDeliveryDisposition::DropWithReason,
                        );
                        self.release_pending_bytes(dropped.retained_bytes_upper_bound());
                        report.record_dropped(1);
                        let error = EditorRuntimeEventConsumerError::Payload {
                            consumer_id: snapshot.consumer_id.clone(),
                            source,
                        };
                        if let Some(reason) = self.record_callback_health(
                            snapshot,
                            true,
                            callback_elapsed,
                            budget.slow_callback_threshold(),
                        ) {
                            self.quarantine_consumer(&snapshot.consumer_id, reason);
                            let (discarded_tail, discarded_tail_bytes) = pending.discard();
                            self.release_pending_bytes(discarded_tail_bytes);
                            let cleanup_error = self
                                .retire_active_consumer(
                                    &ActiveConsumerIdentity {
                                        consumer_id: snapshot.consumer_id.clone(),
                                        subscription: snapshot.subscription.clone(),
                                        generation: snapshot.generation,
                                    },
                                    play_session_id,
                                )
                                .err();
                            report.record_dropped(discarded_tail);
                            first_error.get_or_insert(cleanup_error.map_or(error, |cleanup| {
                                EditorRuntimeEventConsumerError::with_cleanup(
                                    "quarantine runtime event consumer",
                                    error,
                                    cleanup,
                                )
                            }));
                        } else {
                            first_error.get_or_insert(error);
                        }
                        break;
                    }
                    Err(error) => {
                        self.record_callback_fault(
                            &snapshot.consumer_id,
                            play_session_id,
                            EditorRuntimeEventConsumerCallbackPhase::Consume,
                            Some(delivery.delivery()),
                            None,
                        );
                        let poisoned = pending.batch_mut().complete_current(
                            EditorRuntimeEventConsumerDeliveryDisposition::Poison,
                        );
                        self.release_pending_bytes(poisoned.retained_bytes_upper_bound());
                        self.quarantine_consumer(
                            &snapshot.consumer_id,
                            EditorRuntimeEventConsumerQuarantineReason::CallbackPanicked,
                        );
                        let (discarded_tail, discarded_tail_bytes) = pending.discard();
                        self.release_pending_bytes(discarded_tail_bytes);
                        let cleanup_error = self
                            .retire_active_consumer(
                                &ActiveConsumerIdentity {
                                    consumer_id: snapshot.consumer_id.clone(),
                                    subscription: snapshot.subscription.clone(),
                                    generation: snapshot.generation,
                                },
                                play_session_id,
                            )
                            .err();
                        report.record_dropped(1 + discarded_tail);
                        first_error.get_or_insert(cleanup_error.map_or(error, |cleanup| {
                            EditorRuntimeEventConsumerError::with_cleanup(
                                "quarantine runtime event consumer",
                                error,
                                cleanup,
                            )
                        }));
                        break;
                    }
                }

                report.record_applied(callback_elapsed, budget.slow_callback_threshold());
                applied_for_consumer += 1;
                let applied = pending
                    .batch_mut()
                    .complete_current(EditorRuntimeEventConsumerDeliveryDisposition::Applied);
                self.release_pending_bytes(applied.retained_bytes_upper_bound());
                pending.batch_mut().set_last_sequence(sequence);
                if let Some(reason) = self.record_callback_health(
                    snapshot,
                    false,
                    callback_elapsed,
                    budget.slow_callback_threshold(),
                ) {
                    self.quarantine_consumer(&snapshot.consumer_id, reason);
                    let (discarded_tail, discarded_tail_bytes) = pending.discard();
                    self.release_pending_bytes(discarded_tail_bytes);
                    let cleanup_error = self
                        .retire_active_consumer(
                            &ActiveConsumerIdentity {
                                consumer_id: snapshot.consumer_id.clone(),
                                subscription: snapshot.subscription.clone(),
                                generation: snapshot.generation,
                            },
                            play_session_id,
                        )
                        .err();
                    report.record_dropped(discarded_tail);
                    if let Some(cleanup) = cleanup_error {
                        first_error.get_or_insert(cleanup);
                    }
                    break;
                }
            }
            let committed = pending.restore();
            debug_assert!(committed, "lifecycle owner changed during an active pump");
        }
        report.set_drain_percentiles(
            p95_duration(&mut runtime_drain_samples),
            p95_duration(&mut decode_samples),
        );
        self.advance_round_robin_start(&snapshots, visited_consumer_count);
        self.finish_pump_report(&mut report);
        first_error.map_or(Ok(report), Err)
    }
}
