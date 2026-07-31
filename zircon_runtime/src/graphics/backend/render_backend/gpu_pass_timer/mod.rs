mod gpu_pass_timer;

pub(crate) use gpu_pass_timer::{
    DEFAULT_GPU_TIMER_MAX_PASSES, GPU_TIMESTAMP_REQUIRED_FEATURES, GpuPassTimer,
    GpuPassTimestampScope, GpuPassTiming, GpuTimerFrameResult,
};
