//! Parallel text work queues and worker-owned CPU tasks.

#[path = "completion_queue.rs"]
mod completion_queue;
#[path = "raster_pool.rs"]
pub(crate) mod raster_pool;
#[path = "shape_pool.rs"]
pub(crate) mod shape_pool;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
