use crate::core::framework::render::{RenderHybridGiPreparedFrame, RenderPluginRendererOutputs};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HybridGiRuntimePrepareOutput {
    evictable_probe_ids: Vec<u32>,
    renderer_outputs: RenderPluginRendererOutputs,
    prepared_frame: Option<RenderHybridGiPreparedFrame>,
}

impl HybridGiRuntimePrepareOutput {
    pub fn new(evictable_probe_ids: Vec<u32>) -> Self {
        Self {
            evictable_probe_ids,
            renderer_outputs: RenderPluginRendererOutputs::default(),
            prepared_frame: None,
        }
    }

    pub fn with_renderer_outputs(mut self, renderer_outputs: RenderPluginRendererOutputs) -> Self {
        self.renderer_outputs = renderer_outputs;
        self
    }

    pub fn with_prepared_frame(
        mut self,
        prepared_frame: Option<RenderHybridGiPreparedFrame>,
    ) -> Self {
        self.prepared_frame = prepared_frame;
        self
    }

    pub fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probe_ids
    }

    pub fn renderer_outputs(&self) -> &RenderPluginRendererOutputs {
        &self.renderer_outputs
    }

    pub fn prepared_frame(&self) -> Option<&RenderHybridGiPreparedFrame> {
        self.prepared_frame.as_ref()
    }

    pub fn into_evictable_probe_ids(self) -> Vec<u32> {
        self.evictable_probe_ids
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<u32>,
        RenderPluginRendererOutputs,
        Option<RenderHybridGiPreparedFrame>,
    ) {
        (
            self.evictable_probe_ids,
            self.renderer_outputs,
            self.prepared_frame,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderHybridGiPreparedFrame, RenderHybridGiPreparedProbe, RenderHybridGiReadbackOutputs,
    };

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

        let (evictable_probe_ids, renderer_outputs, prepared_frame) = output.into_parts();
        assert_eq!(evictable_probe_ids, vec![7]);
        assert_eq!(renderer_outputs.hybrid_gi.completed_probe_ids, vec![11]);
        assert!(prepared_frame.is_none());
    }

    #[test]
    fn prepare_output_carries_neutral_hybrid_gi_prepared_frame() {
        let prepared_frame = RenderHybridGiPreparedFrame {
            resident_probes: vec![RenderHybridGiPreparedProbe {
                probe_id: 9,
                slot: 1,
                stable_instance_key: 0,
                source_mask: crate::core::framework::render::HYBRID_GI_SOURCE_FULL_DYNAMIC,
                dynamic_weight_q8: u8::MAX,
                ray_budget: 64,
                irradiance_rgb: [4, 5, 6],
            }],
            ..RenderHybridGiPreparedFrame::default()
        };
        let output =
            HybridGiRuntimePrepareOutput::new(Vec::new()).with_prepared_frame(Some(prepared_frame));

        assert_eq!(
            output.prepared_frame().unwrap().resident_probes[0].probe_id,
            9
        );

        let (_, _, prepared_frame) = output.into_parts();
        assert_eq!(prepared_frame.unwrap().resident_probes[0].slot, 1);
    }
}
