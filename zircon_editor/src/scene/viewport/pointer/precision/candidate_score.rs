//! 候选评分分开屏幕接近度和投影深度，Runtime 适配层按类别、像素容差和深度排序，不可把二者混成几何距离。

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::scene::viewport::pointer) struct CandidateScore {
    pub(in crate::scene::viewport::pointer) score: f32,
    pub(in crate::scene::viewport::pointer) depth: f32,
}
