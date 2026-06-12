use std::collections::HashMap;

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::core::{CoreHandle, JobScheduler};

use super::ScheduleParallelBatch;

pub const SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC: &str = "schedule.parallel_batches";
pub const SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC: &str = "schedule.serial_fallbacks";

pub struct ScheduleParallelExecutor {
    scheduler: JobScheduler,
    parallel_enabled: bool,
}

pub struct ScheduleParallelTaskRegistry<E> {
    tasks: HashMap<String, Box<dyn Fn() -> Result<(), E> + Send + Sync>>,
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
        E: Send,
    {
        self.run_batches_with_report(batches, registry).map(|_| ())
    }

    pub fn run_batches_with_report<E>(
        &self,
        batches: &[ScheduleParallelBatch],
        registry: &ScheduleParallelTaskRegistry<E>,
    ) -> Result<ScheduleParallelExecutionReport, ScheduleParallelExecutorError<E>>
    where
        E: Send,
    {
        let mut report = ScheduleParallelExecutionReport::default();
        for batch in batches {
            let system_ids = batch.system_ids();
            if !self.parallel_enabled {
                report.record_serial_batch(system_ids.len(), system_ids.len() > 1);
                run_serial_batch(system_ids, registry)?;
                continue;
            }

            if let [system_id] = system_ids {
                report.record_serial_batch(1, false);
                let task = registry.task_for_system(system_id)?;
                run_task(system_id, task)?;
                continue;
            }

            if let [first_system_id, second_system_id] = system_ids {
                report.record_parallel_batch(2);
                let first_task = registry.task_for_system(first_system_id)?;
                let second_task = registry.task_for_system(second_system_id)?;
                let (first_result, second_result) = self
                    .scheduler
                    .install(|| rayon::join(first_task, second_task));
                run_task_result(first_system_id, first_result)?;
                run_task_result(second_system_id, second_result)?;
                continue;
            }

            if let [first_system_id, second_system_id, third_system_id] = system_ids {
                report.record_parallel_batch(3);
                let first_task = registry.task_for_system(first_system_id)?;
                let second_task = registry.task_for_system(second_system_id)?;
                let third_task = registry.task_for_system(third_system_id)?;
                let ((first_result, second_result), third_result) = self
                    .scheduler
                    .install(|| rayon::join(|| rayon::join(first_task, second_task), third_task));
                run_task_result(first_system_id, first_result)?;
                run_task_result(second_system_id, second_result)?;
                run_task_result(third_system_id, third_result)?;
                continue;
            }

            if let [first_system_id, second_system_id, third_system_id, fourth_system_id] =
                system_ids
            {
                report.record_parallel_batch(4);
                let first_task = registry.task_for_system(first_system_id)?;
                let second_task = registry.task_for_system(second_system_id)?;
                let third_task = registry.task_for_system(third_system_id)?;
                let fourth_task = registry.task_for_system(fourth_system_id)?;
                let ((first_result, second_result), (third_result, fourth_result)) =
                    self.scheduler.install(|| {
                        rayon::join(
                            || rayon::join(first_task, second_task),
                            || rayon::join(third_task, fourth_task),
                        )
                    });
                run_task_result(first_system_id, first_result)?;
                run_task_result(second_system_id, second_result)?;
                run_task_result(third_system_id, third_result)?;
                run_task_result(fourth_system_id, fourth_result)?;
                continue;
            }

            if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id] =
                system_ids
            {
                report.record_parallel_batch(5);
                let first_task = registry.task_for_system(first_system_id)?;
                let second_task = registry.task_for_system(second_system_id)?;
                let third_task = registry.task_for_system(third_system_id)?;
                let fourth_task = registry.task_for_system(fourth_system_id)?;
                let fifth_task = registry.task_for_system(fifth_system_id)?;
                let ((first_result, second_result), ((third_result, fourth_result), fifth_result)) =
                    self.scheduler.install(|| {
                        rayon::join(
                            || rayon::join(first_task, second_task),
                            || rayon::join(|| rayon::join(third_task, fourth_task), fifth_task),
                        )
                    });
                run_task_result(first_system_id, first_result)?;
                run_task_result(second_system_id, second_result)?;
                run_task_result(third_system_id, third_result)?;
                run_task_result(fourth_system_id, fourth_result)?;
                run_task_result(fifth_system_id, fifth_result)?;
                continue;
            }

            if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id, sixth_system_id] =
                system_ids
            {
                report.record_parallel_batch(6);
                let first_task = registry.task_for_system(first_system_id)?;
                let second_task = registry.task_for_system(second_system_id)?;
                let third_task = registry.task_for_system(third_system_id)?;
                let fourth_task = registry.task_for_system(fourth_system_id)?;
                let fifth_task = registry.task_for_system(fifth_system_id)?;
                let sixth_task = registry.task_for_system(sixth_system_id)?;
                let (
                    ((first_result, second_result), (third_result, fourth_result)),
                    (fifth_result, sixth_result),
                ) = self.scheduler.install(|| {
                    rayon::join(
                        || {
                            rayon::join(
                                || rayon::join(first_task, second_task),
                                || rayon::join(third_task, fourth_task),
                            )
                        },
                        || rayon::join(fifth_task, sixth_task),
                    )
                });
                run_task_result(first_system_id, first_result)?;
                run_task_result(second_system_id, second_result)?;
                run_task_result(third_system_id, third_result)?;
                run_task_result(fourth_system_id, fourth_result)?;
                run_task_result(fifth_system_id, fifth_result)?;
                run_task_result(sixth_system_id, sixth_result)?;
                continue;
            }

            report.record_parallel_batch(system_ids.len());
            let tasks = registry.tasks_for_batch(system_ids)?;
            let mut results = Vec::with_capacity(tasks.len());
            self.scheduler.install(|| {
                tasks
                    .into_par_iter()
                    .map(|task| task())
                    .collect_into_vec(&mut results);
            });
            for (system_id, result) in system_ids.iter().zip(results) {
                run_task_result(system_id, result)?;
            }
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

impl<E> ScheduleParallelTaskRegistry<E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        system_id: impl Into<String>,
        task: impl Fn() -> Result<(), E> + Send + Sync + 'static,
    ) -> Option<Box<dyn Fn() -> Result<(), E> + Send + Sync>> {
        self.tasks.insert(system_id.into(), Box::new(task))
    }

    pub fn contains(&self, system_id: &str) -> bool {
        self.tasks.contains_key(system_id)
    }

    fn task_for_system<'registry>(
        &'registry self,
        system_id: &str,
    ) -> Result<
        &'registry (dyn Fn() -> Result<(), E> + Send + Sync),
        ScheduleParallelExecutorError<E>,
    > {
        let Some(task) = self.tasks.get(system_id) else {
            return Err(ScheduleParallelExecutorError::MissingTask {
                system_id: system_id.to_string(),
            });
        };
        Ok(task.as_ref())
    }

    fn tasks_for_batch<'registry>(
        &'registry self,
        system_ids: &'registry [String],
    ) -> Result<
        Vec<&'registry (dyn Fn() -> Result<(), E> + Send + Sync)>,
        ScheduleParallelExecutorError<E>,
    > {
        let mut tasks = Vec::with_capacity(system_ids.len());
        for system_id in system_ids {
            tasks.push(self.task_for_system(system_id)?);
        }
        Ok(tasks)
    }
}

fn run_serial_batch<E>(
    system_ids: &[String],
    registry: &ScheduleParallelTaskRegistry<E>,
) -> Result<(), ScheduleParallelExecutorError<E>> {
    for system_id in system_ids {
        let task = registry.task_for_system(system_id)?;
        run_task(system_id, task)?;
    }
    Ok(())
}

fn run_task<E>(
    system_id: &str,
    task: &(dyn Fn() -> Result<(), E> + Send + Sync),
) -> Result<(), ScheduleParallelExecutorError<E>> {
    run_task_result(system_id, task())
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
