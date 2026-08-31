mod backend;
mod composition;
mod direct;

#[cfg(test)]
mod tests;

pub(super) use backend::{HorizontalBackendRun, shape_horizontal_run};
pub(super) use composition::{
    HorizontalDirectShapeAttempt, HorizontalPartialShape, compose_horizontal_partial,
};
pub(super) use direct::shape_horizontal_request;
