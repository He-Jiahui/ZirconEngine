use super::HybridGiRuntimeUpdate;
use super::{HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput};

pub trait HybridGiRuntimeState: Send + Sync {
    fn prepare_frame(
        &mut self,
        input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput;

    fn update_after_render(&mut self, feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate;
}
