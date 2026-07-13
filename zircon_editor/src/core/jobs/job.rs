use super::{JobContext, JobError};

pub trait EditorJob: Send + 'static {
    type Output: Send + 'static;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError>;
}
