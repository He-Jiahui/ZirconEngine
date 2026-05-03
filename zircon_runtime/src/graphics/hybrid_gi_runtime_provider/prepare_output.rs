use crate::core::framework::render::RenderPluginRendererOutputs;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HybridGiRuntimePrepareOutput {
    evictable_probe_ids: Vec<u32>,
    renderer_outputs: RenderPluginRendererOutputs,
}

impl HybridGiRuntimePrepareOutput {
    pub fn new(evictable_probe_ids: Vec<u32>) -> Self {
        Self {
            evictable_probe_ids,
            renderer_outputs: RenderPluginRendererOutputs::default(),
        }
    }

    pub fn with_renderer_outputs(mut self, renderer_outputs: RenderPluginRendererOutputs) -> Self {
        self.renderer_outputs = renderer_outputs;
        self
    }

    pub fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probe_ids
    }

    pub fn renderer_outputs(&self) -> &RenderPluginRendererOutputs {
        &self.renderer_outputs
    }

    pub fn into_evictable_probe_ids(self) -> Vec<u32> {
        self.evictable_probe_ids
    }

    pub(crate) fn into_parts(self) -> (Vec<u32>, RenderPluginRendererOutputs) {
        (self.evictable_probe_ids, self.renderer_outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::RenderHybridGiReadbackOutputs;

    #[test]
    fn prepare_output_carries_neutral_hybrid_gi_renderer_outputs() {
        let output = HybridGiRuntimePrepareOutput::new(vec![7]).with_renderer_outputs(
            RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![11],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        assert_eq!(output.evictable_probe_ids(), &[7]);
        assert_eq!(
            output.renderer_outputs().hybrid_gi.completed_probe_ids,
            vec![11]
        );

        let (evictable_probe_ids, renderer_outputs) = output.into_parts();
        assert_eq!(evictable_probe_ids, vec![7]);
        assert_eq!(renderer_outputs.hybrid_gi.completed_probe_ids, vec![11]);
    }
}
