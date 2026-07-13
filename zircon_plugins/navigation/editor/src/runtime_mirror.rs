use zircon_runtime::core::framework::navigation::{NavAgentTickReport, NavigationAgentDebugState};

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationPieFrame {
    pub play_session_id: u64,
    pub sequence: u64,
    pub tick_report: NavAgentTickReport,
}

impl NavigationPieFrame {
    pub fn new(play_session_id: u64, sequence: u64, tick_report: NavAgentTickReport) -> Self {
        Self {
            play_session_id,
            sequence,
            tick_report,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationPieMirrorApply {
    Applied,
    WrongSession,
    Stale,
}

#[derive(Clone, Debug, Default)]
pub struct NavigationPieMirror {
    play_session_id: Option<u64>,
    sequence: Option<u64>,
    tick_report: Option<NavAgentTickReport>,
}

impl NavigationPieMirror {
    pub fn begin_session(&mut self, play_session_id: u64) {
        self.play_session_id = Some(play_session_id);
        self.sequence = None;
        self.tick_report = None;
    }

    pub fn end_session(&mut self, play_session_id: u64) -> bool {
        if self.play_session_id != Some(play_session_id) {
            return false;
        }
        *self = Self::default();
        true
    }

    pub fn apply_frame(&mut self, frame: NavigationPieFrame) -> NavigationPieMirrorApply {
        if self.play_session_id != Some(frame.play_session_id) {
            return NavigationPieMirrorApply::WrongSession;
        }
        if self
            .sequence
            .is_some_and(|sequence| frame.sequence <= sequence)
        {
            return NavigationPieMirrorApply::Stale;
        }
        self.sequence = Some(frame.sequence);
        self.tick_report = Some(frame.tick_report);
        NavigationPieMirrorApply::Applied
    }

    /// Consumes the shared `NavAgentTickReport` event registered by the runtime plugin.
    pub fn apply_tick_report(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        report: NavAgentTickReport,
    ) -> NavigationPieMirrorApply {
        self.apply_frame(NavigationPieFrame::new(play_session_id, sequence, report))
    }

    pub fn play_session_id(&self) -> Option<u64> {
        self.play_session_id
    }

    pub fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    pub fn tick_report(&self) -> Option<&NavAgentTickReport> {
        self.tick_report.as_ref()
    }

    pub fn agent(&self, entity: u64) -> Option<&NavigationAgentDebugState> {
        self.tick_report
            .as_ref()?
            .debug_agents
            .iter()
            .find(|agent| agent.entity == entity)
    }

    pub fn agents(&self) -> impl ExactSizeIterator<Item = &NavigationAgentDebugState> {
        self.tick_report
            .as_ref()
            .map(|report| report.debug_agents.as_slice())
            .unwrap_or_default()
            .iter()
    }
}
