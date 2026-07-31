use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Barrier, Condvar, Mutex};
use std::time::Duration;

use crate::plugin::PluginPackageManifest;

use super::super::super::NativePluginCandidate;
use super::super::{
    test_native_plugin_discovery_root, NativePluginDiscoveryInputIdentity,
    NativePluginDiscoveryRefreshBudget, NativePluginDiscoveryRefreshError,
    NativePluginDiscoveryRefreshInput, NativePluginDiscoveryRefreshSink,
    NativePluginDiscoveryRefreshTicket, NativePluginDiscoveryRoot,
    NativePluginDiscoveryTestCollector,
};

pub(super) fn test_budget() -> NativePluginDiscoveryRefreshBudget {
    NativePluginDiscoveryRefreshBudget {
        max_roots: 4,
        max_candidates: 4,
        max_diagnostics: 4,
        max_read_bytes: 1024,
        max_scratch_bytes: 1024,
        deadline: Duration::from_secs(1),
        max_terminal_observers: 4,
    }
}

pub(super) fn root(label: &str) -> NativePluginDiscoveryRoot {
    test_native_plugin_discovery_root(format!("C:/native-plugin-tests/{label}"))
}

pub(super) fn wait_for_terminal(ticket: &NativePluginDiscoveryRefreshTicket) {
    for _ in 0..100 {
        if ticket.is_complete() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("refresh ticket did not reach a terminal state");
}

pub(super) struct BlockingCollector {
    started: SyncSender<u64>,
}

impl BlockingCollector {
    pub(super) fn new(started: SyncSender<u64>) -> Self {
        Self { started }
    }
}

impl NativePluginDiscoveryTestCollector for BlockingCollector {
    fn collect(
        &self,
        request: &super::super::NativePluginDiscoveryRefreshRequest,
        sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
        self.started
            .send(request.generation())
            .expect("test receiver remains available");
        if request.generation() == 1 {
            while !request.is_cancelled() {
                std::thread::sleep(Duration::from_millis(1));
            }
            return Err(NativePluginDiscoveryRefreshError::cancelled());
        }
        publish_fixture(request, sink)
    }
}

pub(super) struct SequenceCollector {
    calls: Mutex<u64>,
}

impl SequenceCollector {
    pub(super) fn new() -> Self {
        Self {
            calls: Mutex::new(0),
        }
    }
}

impl NativePluginDiscoveryTestCollector for SequenceCollector {
    fn collect(
        &self,
        request: &super::super::NativePluginDiscoveryRefreshRequest,
        sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
        let mut calls = self.calls.lock().expect("test collector lock");
        *calls += 1;
        if *calls == 2 {
            return Err(NativePluginDiscoveryRefreshError::collector(
                "synthetic parse failure",
            ));
        }
        publish_fixture(request, sink)
    }
}

pub(super) struct BudgetFailureCollector {
    calls: Mutex<u64>,
}

impl BudgetFailureCollector {
    pub(super) fn new() -> Self {
        Self {
            calls: Mutex::new(0),
        }
    }
}

impl NativePluginDiscoveryTestCollector for BudgetFailureCollector {
    fn collect(
        &self,
        request: &super::super::NativePluginDiscoveryRefreshRequest,
        sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
        let mut calls = self.calls.lock().expect("test collector lock");
        *calls += 1;
        if *calls == 2 {
            for _ in 0..=request.budget().max_diagnostics {
                let reservation = sink.reserve_diagnostic(request)?;
                reservation.insert(sink, "budget".to_owned());
            }
        }
        publish_fixture(request, sink)
    }
}

pub(super) struct BarrierFailureCollector {
    started: SyncSender<u64>,
    barrier: Arc<Barrier>,
}

impl BarrierFailureCollector {
    pub(super) fn new(started: SyncSender<u64>, barrier: Arc<Barrier>) -> Self {
        Self { started, barrier }
    }
}

impl NativePluginDiscoveryTestCollector for BarrierFailureCollector {
    fn collect(
        &self,
        request: &super::super::NativePluginDiscoveryRefreshRequest,
        _sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
        self.started
            .send(request.generation())
            .expect("test receiver remains available");
        self.barrier.wait();
        Err(NativePluginDiscoveryRefreshError::collector(
            "coordinated synthetic failure",
        ))
    }
}

/// Holds distinct authority inputs until the test has observed their independent admission. This
/// makes the input-key isolation test deterministic: a root-only state key can launch just one
/// task, whereas the intended `(root, input)` key launches both onto its two-worker test pool.
pub(super) struct InputBarrierFailureCollector {
    started: SyncSender<NativePluginDiscoveryRefreshInput>,
    release: Arc<(Mutex<bool>, Condvar)>,
}

impl InputBarrierFailureCollector {
    pub(super) fn new(
        started: SyncSender<NativePluginDiscoveryRefreshInput>,
        release: Arc<(Mutex<bool>, Condvar)>,
    ) -> Self {
        Self { started, release }
    }

    pub(super) fn release(&self) {
        let (released, wake) = &*self.release;
        *released.lock().expect("test collector release lock") = true;
        wake.notify_all();
    }
}

impl NativePluginDiscoveryTestCollector for InputBarrierFailureCollector {
    fn collect(
        &self,
        request: &super::super::NativePluginDiscoveryRefreshRequest,
        _sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
        self.started
            .send(request.input().clone())
            .expect("test receiver remains available");
        let (released, wake) = &*self.release;
        let mut released = released.lock().expect("test collector release lock");
        while !*released {
            released = wake.wait(released).expect("test collector release wait");
        }
        let failure = match request.input() {
            NativePluginDiscoveryRefreshInput::RootScan => "root scan failure",
            NativePluginDiscoveryRefreshInput::LoadManifest { .. } => "load manifest failure",
        };
        Err(NativePluginDiscoveryRefreshError::collector(failure))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AdmissionProbeKind {
    Candidate,
    Diagnostic,
    ReadBytes,
    ScratchBytes,
}

pub(super) struct AdmissionProbeCollector {
    kind: AdmissionProbeKind,
    materialized_units: AtomicUsize,
}

impl AdmissionProbeCollector {
    pub(super) fn new(kind: AdmissionProbeKind) -> Self {
        Self {
            kind,
            materialized_units: AtomicUsize::new(0),
        }
    }

    pub(super) fn materialized_units(&self) -> usize {
        self.materialized_units.load(Ordering::SeqCst)
    }
}

impl NativePluginDiscoveryTestCollector for AdmissionProbeCollector {
    fn collect(
        &self,
        request: &super::super::NativePluginDiscoveryRefreshRequest,
        sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
        match self.kind {
            AdmissionProbeKind::Candidate => {
                for index in 0..=request.budget().max_candidates {
                    let reservation = sink.reserve_candidate(request)?;
                    self.materialized_units.fetch_add(1, Ordering::SeqCst);
                    reservation.insert(sink, candidate(index));
                }
            }
            AdmissionProbeKind::Diagnostic => {
                for index in 0..=request.budget().max_diagnostics {
                    let reservation = sink.reserve_diagnostic(request)?;
                    self.materialized_units.fetch_add(1, Ordering::SeqCst);
                    reservation.insert(sink, format!("diagnostic {index}"));
                }
            }
            AdmissionProbeKind::ReadBytes => {
                let _admission = sink.reserve_read_bytes(
                    request,
                    request.budget().max_read_bytes.saturating_add(1),
                )?;
                self.materialized_units.fetch_add(1, Ordering::SeqCst);
            }
            AdmissionProbeKind::ScratchBytes => {
                let _admission = sink.reserve_scratch_bytes(
                    request,
                    request.budget().max_scratch_bytes.saturating_add(1),
                )?;
                self.materialized_units.fetch_add(1, Ordering::SeqCst);
            }
        }
        Err(NativePluginDiscoveryRefreshError::collector(
            "admission probe unexpectedly exceeded its runtime-owned budget",
        ))
    }
}

fn publish_fixture(
    request: &super::super::NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
    let reservation = sink.reserve_diagnostic(request)?;
    reservation.insert(sink, format!("generation {}", request.generation()));
    sink.reserve_read_bytes(request, 16)?.commit(sink, 16)?;
    let _scratch_admission = sink.reserve_scratch_bytes(request, 8)?;
    NativePluginDiscoveryInputIdentity::new(format!("fixture-generation-{}", request.generation()))
}

fn candidate(index: usize) -> NativePluginCandidate {
    let plugin_id = format!("fixture-admission-{index}");
    NativePluginCandidate {
        plugin_id: plugin_id.clone(),
        package_manifest: PluginPackageManifest::new(&plugin_id, "Fixture admission candidate"),
        manifest_path: format!("{plugin_id}/plugin.toml").into(),
        library_path: format!("{plugin_id}/native/plugin.dll").into(),
    }
}
