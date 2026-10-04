use super::super::{
    SoundError, SoundHrtfProfileDescriptor, SoundImpulseResponseId,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
};

/// 声学资源与空间卷积的运行时登记入口；HRTF 和脉冲响应身份在声源配置中引用。
pub trait SoundAcousticsManager {
    fn set_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
        samples: Vec<f32>,
    ) -> Result<(), SoundError>;
    fn remove_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError>;
    fn load_hrtf_profile(&self, profile: SoundHrtfProfileDescriptor) -> Result<(), SoundError>;
    fn remove_hrtf_profile(&self, profile_id: &str) -> Result<(), SoundError>;
    fn hrtf_profiles(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError>;
    fn set_ray_tracing_convolution_status(
        &self,
        status: SoundRayTracingConvolutionStatus,
    ) -> Result<(), SoundError>;
    fn submit_ray_traced_impulse_response(
        &self,
        descriptor: SoundRayTracedImpulseResponseDescriptor,
    ) -> Result<(), SoundError>;
    fn ray_traced_impulse_responses(
        &self,
    ) -> Result<Vec<SoundRayTracedImpulseResponseDescriptor>, SoundError>;
    fn clear_ray_traced_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError>;
}
