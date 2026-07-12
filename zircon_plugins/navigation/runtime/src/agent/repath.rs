use zircon_runtime::scene::ecs::Resource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavRepathBudget {
    pub max_queries_per_frame: usize,
    pub queries_used: usize,
}

impl NavRepathBudget {
    pub fn new(max_queries_per_frame: usize) -> Self {
        Self {
            max_queries_per_frame,
            queries_used: 0,
        }
    }

    pub(super) fn begin_frame(&mut self) {
        self.queries_used = 0;
    }

    pub(super) fn try_consume(&mut self) -> bool {
        if self.queries_used >= self.max_queries_per_frame {
            return false;
        }
        self.queries_used += 1;
        true
    }
}

impl Default for NavRepathBudget {
    fn default() -> Self {
        Self::new(32)
    }
}

impl Resource for NavRepathBudget {}
