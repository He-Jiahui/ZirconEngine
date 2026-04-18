#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::scene::viewport::pointer) struct CandidateScore {
    pub(in crate::scene::viewport::pointer) priority: u8,
    pub(in crate::scene::viewport::pointer) score: f32,
    pub(in crate::scene::viewport::pointer) depth: f32,
}
