use crate::editing::viewport::pointer::precision::CandidateScore;

pub(in crate::editing::viewport::pointer) fn better_score(
    candidate: CandidateScore,
    current: Option<CandidateScore>,
) -> bool {
    let Some(current) = current else {
        return true;
    };
    if candidate.priority != current.priority {
        return candidate.priority < current.priority;
    }
    if (candidate.score - current.score).abs() > 0.5 {
        return candidate.score < current.score;
    }
    candidate.depth < current.depth
}
