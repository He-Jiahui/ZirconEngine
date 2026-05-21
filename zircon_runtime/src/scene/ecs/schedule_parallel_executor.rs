use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::JobScheduler;

use super::ScheduleParallelBatch;

pub struct ScheduleParallelExecutor {
    scheduler: JobScheduler,
}

pub struct ScheduleParallelTaskRegistry<E> {
    tasks: HashMap<String, Box<dyn Fn() -> Result<(), E> + Send + Sync>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScheduleParallelExecutorError<E> {
    MissingTask { system_id: String },
    TaskFailed { system_id: String, error: E },
}

impl ScheduleParallelExecutor {
    pub fn new(scheduler: JobScheduler) -> Self {
        Self { scheduler }
    }

    pub fn scheduler(&self) -> &JobScheduler {
        &self.scheduler
    }

    pub fn run_batches<E>(
        &self,
        batches: &[ScheduleParallelBatch],
        registry: &ScheduleParallelTaskRegistry<E>,
    ) -> Result<(), ScheduleParallelExecutorError<E>>
    where
        E: Send,
    {
        for batch in batches {
            let tasks = registry.tasks_for_batch(batch)?;
            if tasks.len() == 1 {
                run_task(tasks[0].1, tasks[0].2)?;
                continue;
            }

            let results = Mutex::new(Vec::with_capacity(tasks.len()));
            self.scheduler.install(|| {
                rayon::scope(|scope| {
                    for (index, system_id, task) in tasks {
                        let results = &results;
                        scope.spawn(move |_| {
                            results.lock().expect("schedule task results lock").push((
                                index,
                                system_id.to_string(),
                                task(),
                            ));
                        });
                    }
                });
            });

            let mut results = results
                .into_inner()
                .expect("schedule task results lock should not be poisoned");
            results.sort_by_key(|(index, _, _)| *index);
            for (_, system_id, result) in results {
                if let Err(error) = result {
                    return Err(ScheduleParallelExecutorError::TaskFailed { system_id, error });
                }
            }
        }

        Ok(())
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

    fn tasks_for_batch<'registry>(
        &'registry self,
        batch: &'registry ScheduleParallelBatch,
    ) -> Result<
        Vec<(
            usize,
            &'registry str,
            &'registry (dyn Fn() -> Result<(), E> + Send + Sync),
        )>,
        ScheduleParallelExecutorError<E>,
    > {
        batch
            .system_ids()
            .iter()
            .enumerate()
            .map(|(index, system_id)| {
                self.tasks
                    .get(system_id)
                    .map(|task| (index, system_id.as_str(), task.as_ref()))
                    .ok_or_else(|| ScheduleParallelExecutorError::MissingTask {
                        system_id: system_id.clone(),
                    })
            })
            .collect()
    }
}

fn run_task<E>(
    system_id: &str,
    task: &(dyn Fn() -> Result<(), E> + Send + Sync),
) -> Result<(), ScheduleParallelExecutorError<E>> {
    task().map_err(|error| ScheduleParallelExecutorError::TaskFailed {
        system_id: system_id.to_string(),
        error,
    })
}
