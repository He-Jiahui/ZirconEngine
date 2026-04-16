#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::editing::viewport::pointer) struct CandidateScore {
    pub(in crate::editing::viewport::pointer) priority: u8,
    pub(in crate::editing::viewport::pointer) score: f32,
    pub(in crate::editing::viewport::pointer) depth: f32,
}
