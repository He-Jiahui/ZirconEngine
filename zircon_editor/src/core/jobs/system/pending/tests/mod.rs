use std::any::Any;

use crate::core::jobs::JobContext;

use super::super::PendingTask;

pub(super) struct ReplaceablePendingTask;

impl PendingTask for ReplaceablePendingTask {
    fn run(self: Box<Self>, _context: JobContext) {}

    fn replace_with(&mut self, latest: Box<dyn PendingTask>) -> bool {
        latest.into_any().is::<Self>()
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

mod admission;
mod fairness;
