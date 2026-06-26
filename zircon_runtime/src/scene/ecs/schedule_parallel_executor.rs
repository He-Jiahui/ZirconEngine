use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
};

use crate::core::{CoreHandle, JobHandle, JobScheduler};

use super::ScheduleParallelBatch;

pub const SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC: &str = "schedule.parallel_batches";
pub const SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC: &str = "schedule.serial_fallbacks";

pub struct ScheduleParallelExecutor {
    scheduler: JobScheduler,
    parallel_enabled: bool,
}

type ScheduleParallelTask<E> = Arc<dyn Fn() -> Result<(), E> + Send + Sync + 'static>;
type ScheduleParallelBatchResult<E> = Result<(), ScheduleParallelExecutorError<E>>;
type ScheduleParallelBatchSlot<E> = Arc<Mutex<Option<ScheduleParallelBatchResult<E>>>>;

pub struct ScheduleParallelTaskRegistry<E> {
    tasks: HashMap<String, ScheduleParallelTask<E>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScheduleParallelExecutionReport {
    parallel_batches: usize,
    serial_batches: usize,
    serial_fallbacks: usize,
    executed_systems: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScheduleParallelExecutorError<E> {
    MissingTask { system_id: String },
    TaskFailed { system_id: String, error: E },
}

impl ScheduleParallelExecutor {
    pub fn new(scheduler: JobScheduler) -> Self {
        Self {
            scheduler,
            parallel_enabled: true,
        }
    }

    pub fn scheduler(&self) -> &JobScheduler {
        &self.scheduler
    }

    pub fn parallel_enabled(&self) -> bool {
        self.parallel_enabled
    }

    pub fn with_parallel_enabled(mut self, enabled: bool) -> Self {
        self.parallel_enabled = enabled;
        self
    }

    pub fn run_batches<E>(
        &self,
        batches: &[ScheduleParallelBatch],
        registry: &ScheduleParallelTaskRegistry<E>,
    ) -> Result<(), ScheduleParallelExecutorError<E>>
    where
        E: Send + 'static,
    {
        self.run_batches_with_report(batches, registry).map(|_| ())
    }

    pub fn run_batches_with_report<E>(
        &self,
        batches: &[ScheduleParallelBatch],
        registry: &ScheduleParallelTaskRegistry<E>,
    ) -> Result<ScheduleParallelExecutionReport, ScheduleParallelExecutorError<E>>
    where
        E: Send + 'static,
    {
        let mut report = ScheduleParallelExecutionReport::default();
        let aborted = Arc::new(AtomicBool::new(false));
        let mut previous_batch = JobHandle::completed();
        let mut scheduled_batches: Vec<ScheduleParallelBatchSlot<E>> =
            Vec::with_capacity(batches.len());

        for batch in batches {
            let system_ids = batch.system_ids();
            if !self.parallel_enabled {
                report.record_serial_batch(system_ids.len(), system_ids.len() > 1);
            } else if let [_] = system_ids {
                report.record_serial_batch(1, false);
            } else {
                report.record_parallel_batch(system_ids.len());
            }

            let batch_result = Arc::new(Mutex::new(None));
            let batch_result_for_task = Arc::clone(&batch_result);
            let registry_for_task = registry.clone();
            let scheduler_for_task = self.scheduler.clone();
            let aborted_for_task = Arc::clone(&aborted);
            let batch_system_ids = system_ids.to_vec();
            let dependency = previous_batch.clone();
            let parallel_enabled = self.parallel_enabled;

            let batch_handle =
                self.scheduler
                    .schedule_after(std::slice::from_ref(&dependency), move || {
                        let result = if aborted_for_task.load(Ordering::Acquire) {
                            Ok(())
                        } else {
                            run_scheduled_batch(
                                &scheduler_for_task,
                                &batch_system_ids,
                                &registry_for_task,
                                parallel_enabled,
                            )
                        };

                        if result.is_err() {
                            aborted_for_task.store(true, Ordering::Release);
                        }

                        *lock_batch_result(&batch_result_for_task) = Some(result);
                    });
            previous_batch = batch_handle;
            scheduled_batches.push(batch_result);
        }

        previous_batch.wait();
        for batch_result in scheduled_batches {
            lock_batch_result(&batch_result).take().expect(
                "scheduled batch should publish a result before the tail handle completes",
            )?;
        }
        Ok(report)
    }
}

impl ScheduleParallelExecutionReport {
    fn record_parallel_batch(&mut self, system_count: usize) {
        self.parallel_batches += 1;
        self.executed_systems += system_count;
    }

    fn record_serial_batch(&mut self, system_count: usize, fallback: bool) {
        self.serial_batches += 1;
        self.executed_systems += system_count;
        if fallback {
            self.serial_fallbacks += 1;
        }
    }

    pub fn parallel_batches(&self) -> usize {
        self.parallel_batches
    }

    pub fn serial_batches(&self) -> usize {
        self.serial_batches
    }

    pub fn serial_fallbacks(&self) -> usize {
        self.serial_fallbacks
    }

    pub fn executed_systems(&self) -> usize {
        self.executed_systems
    }

    pub fn record_diagnostics(&self, core: &CoreHandle, frame_index: u64) {
        core.record_diagnostic(
            SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC,
            frame_index,
            self.parallel_batches as f64,
            Some("batch"),
            ["schedule", "parallel"],
        );
        core.record_diagnostic(
            SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC,
            frame_index,
            self.serial_fallbacks as f64,
            Some("batch"),
            ["schedule", "parallel"],
        );
    }
}

impl<E> Default for ScheduleParallelTaskRegistry<E> {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
}

impl<E> Clone for ScheduleParallelTaskRegistry<E> {
    fn clone(&self) -> Self {
        Self {
            tasks: self.tasks.clone(),
        }
    }
}

impl<E: 'static> ScheduleParallelTaskRegistry<E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        system_id: impl Into<String>,
        task: impl Fn() -> Result<(), E> + Send + Sync + 'static,
    ) -> Option<Arc<dyn Fn() -> Result<(), E> + Send + Sync + 'static>> {
        self.tasks.insert(system_id.into(), Arc::new(task))
    }

    pub fn contains(&self, system_id: &str) -> bool {
        self.tasks.contains_key(system_id)
    }

    fn task_for_system<'registry>(
        &'registry self,
        system_id: &str,
    ) -> Result<ScheduleParallelTask<E>, ScheduleParallelExecutorError<E>> {
        let Some(task) = self.tasks.get(system_id) else {
            return Err(ScheduleParallelExecutorError::MissingTask {
                system_id: system_id.to_string(),
            });
        };
        Ok(Arc::clone(task))
    }

    fn tasks_for_batch<'registry>(
        &'registry self,
        system_ids: &'registry [String],
    ) -> Result<Vec<ScheduleParallelTask<E>>, ScheduleParallelExecutorError<E>> {
        let mut tasks = Vec::with_capacity(system_ids.len());
        for system_id in system_ids {
            tasks.push(self.task_for_system(system_id)?);
        }
        Ok(tasks)
    }
}

fn run_scheduled_batch<E>(
    scheduler: &JobScheduler,
    system_ids: &[String],
    registry: &ScheduleParallelTaskRegistry<E>,
    parallel_enabled: bool,
) -> Result<(), ScheduleParallelExecutorError<E>>
where
    E: Send + 'static,
{
    if !parallel_enabled {
        return run_serial_batch(system_ids, registry);
    }

    if let [system_id] = system_ids {
        let task = registry.task_for_system(system_id)?;
        return run_task(system_id, &task);
    }

    if let [first_system_id, second_system_id] = system_ids {
        let first_task = registry.task_for_system(first_system_id)?;
        let second_task = registry.task_for_system(second_system_id)?;
        let (first_result, second_result) =
            scheduler.join(|| first_task.as_ref()(), || second_task.as_ref()());
        run_task_result(first_system_id, first_result)?;
        run_task_result(second_system_id, second_result)?;
        return Ok(());
    }

    if let [first_system_id, second_system_id, third_system_id] = system_ids {
        let first_task = registry.task_for_system(first_system_id)?;
        let second_task = registry.task_for_system(second_system_id)?;
        let third_task = registry.task_for_system(third_system_id)?;
        let nested_scheduler = scheduler.clone();
        let ((first_result, second_result), third_result) = scheduler.join(
            move || nested_scheduler.join(|| first_task.as_ref()(), || second_task.as_ref()()),
            || third_task.as_ref()(),
        );
        run_task_result(first_system_id, first_result)?;
        run_task_result(second_system_id, second_result)?;
        run_task_result(third_system_id, third_result)?;
        return Ok(());
    }

    if let [first_system_id, second_system_id, third_system_id, fourth_system_id] = system_ids {
        let first_task = registry.task_for_system(first_system_id)?;
        let second_task = registry.task_for_system(second_system_id)?;
        let third_task = registry.task_for_system(third_system_id)?;
        let fourth_task = registry.task_for_system(fourth_system_id)?;
        let left_scheduler = scheduler.clone();
        let right_scheduler = scheduler.clone();
        let ((first_result, second_result), (third_result, fourth_result)) = scheduler.join(
            move || left_scheduler.join(|| first_task.as_ref()(), || second_task.as_ref()()),
            move || right_scheduler.join(|| third_task.as_ref()(), || fourth_task.as_ref()()),
        );
        run_task_result(first_system_id, first_result)?;
        run_task_result(second_system_id, second_result)?;
        run_task_result(third_system_id, third_result)?;
        run_task_result(fourth_system_id, fourth_result)?;
        return Ok(());
    }

    if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id] =
        system_ids
    {
        let first_task = registry.task_for_system(first_system_id)?;
        let second_task = registry.task_for_system(second_system_id)?;
        let third_task = registry.task_for_system(third_system_id)?;
        let fourth_task = registry.task_for_system(fourth_system_id)?;
        let fifth_task = registry.task_for_system(fifth_system_id)?;
        let left_scheduler = scheduler.clone();
        let right_scheduler = scheduler.clone();
        let nested_scheduler = scheduler.clone();
        let ((first_result, second_result), ((third_result, fourth_result), fifth_result)) =
            scheduler.join(
                move || left_scheduler.join(|| first_task.as_ref()(), || second_task.as_ref()()),
                move || {
                    right_scheduler.join(
                        move || {
                            nested_scheduler
                                .join(|| third_task.as_ref()(), || fourth_task.as_ref()())
                        },
                        || fifth_task.as_ref()(),
                    )
                },
            );
        run_task_result(first_system_id, first_result)?;
        run_task_result(second_system_id, second_result)?;
        run_task_result(third_system_id, third_result)?;
        run_task_result(fourth_system_id, fourth_result)?;
        run_task_result(fifth_system_id, fifth_result)?;
        return Ok(());
    }

    if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id, sixth_system_id] =
        system_ids
    {
        let first_task = registry.task_for_system(first_system_id)?;
        let second_task = registry.task_for_system(second_system_id)?;
        let third_task = registry.task_for_system(third_system_id)?;
        let fourth_task = registry.task_for_system(fourth_system_id)?;
        let fifth_task = registry.task_for_system(fifth_system_id)?;
        let sixth_task = registry.task_for_system(sixth_system_id)?;
        let left_scheduler = scheduler.clone();
        let right_scheduler = scheduler.clone();
        let first_pair_scheduler = scheduler.clone();
        let second_pair_scheduler = scheduler.clone();
        let (
            ((first_result, second_result), (third_result, fourth_result)),
            (fifth_result, sixth_result),
        ) = scheduler.join(
            move || {
                left_scheduler.join(
                    move || {
                        first_pair_scheduler
                            .join(|| first_task.as_ref()(), || second_task.as_ref()())
                    },
                    move || {
                        second_pair_scheduler
                            .join(|| third_task.as_ref()(), || fourth_task.as_ref()())
                    },
                )
            },
            move || right_scheduler.join(|| fifth_task.as_ref()(), || sixth_task.as_ref()()),
        );
        run_task_result(first_system_id, first_result)?;
        run_task_result(second_system_id, second_result)?;
        run_task_result(third_system_id, third_result)?;
        run_task_result(fourth_system_id, fourth_result)?;
        run_task_result(fifth_system_id, fifth_result)?;
        run_task_result(sixth_system_id, sixth_result)?;
        return Ok(());
    }

    let tasks = registry.tasks_for_batch(system_ids)?;
    let results = run_parallel_tasks(scheduler, &tasks);
    for (system_id, result) in system_ids.iter().zip(results) {
        run_task_result(system_id, result)?;
    }
    Ok(())
}

fn run_parallel_tasks<E>(
    scheduler: &JobScheduler,
    tasks: &[ScheduleParallelTask<E>],
) -> Vec<Result<(), E>>
where
    E: Send + 'static,
{
    match tasks {
        [] => Vec::new(),
        [task] => vec![task.as_ref()()],
        _ => {
            let midpoint = tasks.len() / 2;
            let (left_tasks, right_tasks) = tasks.split_at(midpoint);
            let left_scheduler = scheduler.clone();
            let right_scheduler = scheduler.clone();
            let (mut left_results, right_results) = scheduler.join(
                move || run_parallel_tasks(&left_scheduler, left_tasks),
                move || run_parallel_tasks(&right_scheduler, right_tasks),
            );
            left_results.extend(right_results);
            left_results
        }
    }
}

fn run_serial_batch<E>(
    system_ids: &[String],
    registry: &ScheduleParallelTaskRegistry<E>,
) -> Result<(), ScheduleParallelExecutorError<E>>
where
    E: 'static,
{
    for system_id in system_ids {
        let task = registry.task_for_system(system_id)?;
        run_task(system_id, &task)?;
    }
    Ok(())
}

fn run_task<E>(
    system_id: &str,
    task: &ScheduleParallelTask<E>,
) -> Result<(), ScheduleParallelExecutorError<E>> {
    run_task_result(system_id, task.as_ref()())
}

fn run_task_result<E>(
    system_id: &str,
    result: Result<(), E>,
) -> Result<(), ScheduleParallelExecutorError<E>> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(ScheduleParallelExecutorError::TaskFailed {
            system_id: system_id.to_string(),
            error,
        }),
    }
}

fn lock_batch_result<E>(
    batch_result: &Mutex<Option<ScheduleParallelBatchResult<E>>>,
) -> MutexGuard<'_, Option<ScheduleParallelBatchResult<E>>> {
    batch_result
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    #[test]
    fn schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock() {
        let slot: Mutex<Option<ScheduleParallelBatchResult<&'static str>>> =
            Mutex::new(Some(Ok(())));

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = slot.lock().unwrap();
            panic!("poison schedule parallel executor batch result slot");
        }));

        let recovered = lock_batch_result::<&'static str>(&slot)
            .take()
            .expect("batch result should remain available after poison recovery");
        assert_eq!(recovered, Ok(()));

        *lock_batch_result(&slot) = Some(Err(ScheduleParallelExecutorError::MissingTask {
            system_id: "missing.task".to_string(),
        }));
        let recovered = lock_batch_result(&slot)
            .take()
            .expect("missing-task result should remain available");
        assert_eq!(
            recovered,
            Err(ScheduleParallelExecutorError::MissingTask {
                system_id: "missing.task".to_string()
            })
        );
    }
}
