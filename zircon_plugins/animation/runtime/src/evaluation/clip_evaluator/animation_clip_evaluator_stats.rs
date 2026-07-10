#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationClipEvaluatorStats {
    pub skeleton_compile_count: u64,
    pub clip_compile_count: u64,
    pub clip_cache_hit_count: u64,
    pub pose_pool_miss_count: u64,
    pub skeleton_eviction_count: u64,
    pub clip_eviction_count: u64,
    pub cached_skeleton_count: usize,
    pub cached_clip_count: usize,
}
