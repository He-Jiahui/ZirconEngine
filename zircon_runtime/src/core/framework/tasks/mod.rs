//! Backend-neutral task execution contracts for framework consumers.

#[path = "parallel_slice_executor.rs"]
mod parallel_slice_executor;

pub use parallel_slice_executor::ParallelSliceExecutor;
