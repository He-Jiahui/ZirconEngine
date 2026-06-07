use std::collections::HashMap;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
            let system_ids = batch.system_ids();
            if let [system_id] = system_ids {
                let task = registry.task_for_system(system_id)?;
                run_task(system_id, task)?;
                continue;
            }

            let tasks = registry.tasks_for_batch(system_ids)?;
            let results = self
                .scheduler
                .install(|| tasks.into_par_iter().map(|task| task()).collect::<Vec<_>>());
            for (index, result) in results.into_iter().enumerate() {
                if let Err(error) = result {
                    let system_id = system_ids
                        .get(index)
                        .expect("task result index must originate from batch order")
                        .clone();
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

    fn task_for_system<'registry>(
        &'registry self,
        system_id: &str,
    ) -> Result<
        &'registry (dyn Fn() -> Result<(), E> + Send + Sync),
        ScheduleParallelExecutorError<E>,
    > {
        self.tasks
            .get(system_id)
            .map(|task| task.as_ref())
            .ok_or_else(|| ScheduleParallelExecutorError::MissingTask {
                system_id: system_id.to_string(),
            })
    }

    fn tasks_for_batch<'registry>(
        &'registry self,
        system_ids: &'registry [String],
    ) -> Result<
        Vec<&'registry (dyn Fn() -> Result<(), E> + Send + Sync)>,
        ScheduleParallelExecutorError<E>,
    > {
        system_ids
            .iter()
            .map(|system_id| self.task_for_system(system_id))
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
