//! 引擎内部把 Kira 播放状态与源环境算法分开；DSP 和双二阶滤波当前仅由测试编译，不能据此推断实时输出已接入这些算法。
#[cfg(test)]
#[path = "dsp/tests/mod.rs"]
mod dsp;
#[cfg(test)]
#[path = "filter/tests/mod.rs"]
mod filter;
#[path = "hrtf/mod.rs"]
mod hrtf;
#[path = "math.rs"]
mod math;
#[path = "occlusion/mod.rs"]
mod occlusion;
#[path = "source_environment/mod.rs"]
mod source_environment;
#[path = "state/mod.rs"]
mod state;

pub(crate) use hrtf::{SoundHrtfRenderState, SoundHrtfRenderStateKey};
pub(crate) use occlusion::{occlusion_gain_for_query, SoundOcclusionQuery};
pub(crate) use state::{
    ActivePlayback, LoadedClip, SoundDynamicEventExecutor, SoundDynamicEventExecutorKey,
    SoundEngineState, SoundGraphSnapshot, SourceVoice,
};
