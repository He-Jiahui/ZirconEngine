use super::super::pending_mesh_draw::PendingMeshDraw;

pub(in super::super) enum PendingMeshDrawRemainder {
    All(Vec<PendingMeshDraw>),
    Residual(Vec<(usize, PendingMeshDraw)>),
}

impl PendingMeshDrawRemainder {
    pub(in super::super) fn all(pending_draws: Vec<PendingMeshDraw>) -> Self {
        Self::All(pending_draws)
    }
}

pub(in super::super) enum PendingMeshDrawRemainderIntoIter {
    All(std::iter::Enumerate<std::vec::IntoIter<PendingMeshDraw>>),
    Residual(std::vec::IntoIter<(usize, PendingMeshDraw)>),
}

impl Iterator for PendingMeshDrawRemainderIntoIter {
    type Item = (usize, PendingMeshDraw);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::All(draws) => draws.next(),
            Self::Residual(draws) => draws.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::All(draws) => draws.size_hint(),
            Self::Residual(draws) => draws.size_hint(),
        }
    }
}

impl IntoIterator for PendingMeshDrawRemainder {
    type Item = (usize, PendingMeshDraw);
    type IntoIter = PendingMeshDrawRemainderIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::All(draws) => {
                PendingMeshDrawRemainderIntoIter::All(draws.into_iter().enumerate())
            }
            Self::Residual(draws) => PendingMeshDrawRemainderIntoIter::Residual(draws.into_iter()),
        }
    }
}
