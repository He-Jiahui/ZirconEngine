use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::super::candidate_from_manifest::append_candidate_from_manifest_path;
use super::super::collect_manifests::{
    traverse_plugin_manifests, NativePluginManifestTraversalDiagnostic,
    NativePluginManifestTraversalError, NativePluginManifestTraversalVisitor,
};
use super::super::discovery_refresh::{
    is_native_plugin_discovery_io_lane, native_plugin_discovery_refresh_service,
    native_plugin_discovery_root, NativePluginDiscoveryInputIdentity,
    NativePluginDiscoveryRefreshBudget, NativePluginDiscoveryRefreshError,
    NativePluginDiscoveryRefreshInput, NativePluginDiscoveryRefreshRequest,
    NativePluginDiscoveryRefreshService, NativePluginDiscoveryRefreshSink,
    NativePluginDiscoveryRefreshTerminal, NativePluginDiscoveryRefreshTicket,
    NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
};
use super::super::NativePluginLoadReport;

static DISCOVERY_AUTHORITY: OnceLock<NativePluginDiscoveryAuthority> = OnceLock::new();
const MAX_ROOT_IDENTITIES: usize = 32;

pub(in crate::plugin::native_plugin_loader) fn discovery_authority(
) -> &'static NativePluginDiscoveryAuthority {
    DISCOVERY_AUTHORITY.get_or_init(NativePluginDiscoveryAuthority::default)
}

/// Construction proof consumed by the refresh module. Its private field makes the production
/// factory unavailable to sibling loader modules even though the service implementation lives
/// below `discovery_refresh`.
pub(in crate::plugin::native_plugin_loader) struct NativePluginDiscoveryAuthorityCapability(());

#[cfg(test)]
impl NativePluginDiscoveryAuthorityCapability {
    pub(in crate::plugin::native_plugin_loader) fn for_test() -> Self {
        Self(())
    }
}

/// The only owner of native-discovery work and publication. It never stores a second discovery
/// snapshot: synchronous callers project the refresh service's published last-good snapshot.
pub(in crate::plugin::native_plugin_loader) struct NativePluginDiscoveryAuthority {
    refresh: NativePluginDiscoveryRefreshService,
    in_flight: Mutex<BTreeMap<AuthorityRefreshKey, InFlightRefresh>>,
    root_identities: Mutex<BTreeMap<PathBuf, NativePluginDiscoveryRoot>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AuthorityRefreshKey {
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
}

impl AuthorityRefreshKey {
    fn new(root: NativePluginDiscoveryRoot, input: NativePluginDiscoveryRefreshInput) -> Self {
        Self { root, input }
    }
}

struct InFlightRefresh {
    ticket: NativePluginDiscoveryRefreshTicket,
    forced: bool,
}

impl Default for NativePluginDiscoveryAuthority {
    fn default() -> Self {
        Self {
            refresh: native_plugin_discovery_refresh_service(
                NativePluginDiscoveryAuthorityCapability(()),
                NativePluginDiscoveryRefreshBudget::default(),
            ),
            in_flight: Mutex::new(BTreeMap::new()),
            root_identities: Mutex::new(BTreeMap::new()),
        }
    }
}

impl NativePluginDiscoveryAuthority {
    pub(super) fn discover(&self, root: &Path) -> NativePluginLoadReport {
        self.project_root(root, false)
    }

    pub(in crate::plugin::native_plugin_loader) fn discover_load_manifest(
        &self,
        export_root: &Path,
    ) -> NativePluginLoadReport {
        // Collection runs asynchronously. Bind a relative export root before the ticket is
        // scheduled so a later process working-directory change cannot redirect the selection.
        let export_root = lexical_root_path(export_root);
        let load_manifest_path =
            super::super::discover_load_manifest::native_plugin_load_manifest_path(&export_root);
        self.project_input(
            &load_manifest_path,
            NativePluginDiscoveryRefreshInput::load_manifest(export_root),
            // Explicit export selection has no watcher notification surface. Preserve the
            // facade's previous read-current-manifest behavior by refreshing this distinct
            // selection root on each call, while still coalescing concurrent callers here.
            true,
        )
    }

    pub(super) fn refresh_manifest(
        &self,
        root: &Path,
        _manifest_path: &Path,
    ) -> NativePluginLoadReport {
        // A notification has no private incremental cache. The filesystem mutation must already
        // be visible; this schedules the authority's bounded full-root refresh.
        self.project_root(root, true)
    }

    pub(super) fn remove_path(&self, root: &Path, _removed_path: &Path) -> NativePluginLoadReport {
        // Removal is likewise a full-root notification. A path that still exists is rediscovered.
        self.project_root(root, true)
    }

    pub(super) fn generation(&self, root: &Path) -> Option<u64> {
        let root = self.cached_root_identity(root)?;
        self.refresh
            .snapshot(&root)
            .filter(|snapshot| snapshot.input() == &NativePluginDiscoveryRefreshInput::RootScan)
            .map(|snapshot| snapshot.generation())
    }

    fn project_root(&self, path: &Path, force_refresh: bool) -> NativePluginLoadReport {
        self.project_input(
            path,
            NativePluginDiscoveryRefreshInput::root_scan(),
            force_refresh,
        )
    }

    fn project_input(
        &self,
        path: &Path,
        input: NativePluginDiscoveryRefreshInput,
        force_refresh: bool,
    ) -> NativePluginLoadReport {
        if is_native_plugin_discovery_io_lane() {
            return self.report_without_wait(path, &input);
        }
        let root = self.root_identity(path);
        self.project_refresh(root, input, force_refresh)
    }

    fn project_refresh(
        &self,
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        force_refresh: bool,
    ) -> NativePluginLoadReport {
        if !force_refresh {
            if let Some(snapshot) = self.refresh.snapshot_for(&root, &input) {
                return self.report_from_snapshot(snapshot);
            }
        }

        let mut force_refresh = force_refresh;
        loop {
            let ticket = self.ticket_for(&root, &input, force_refresh);
            // A notification may supersede an active generation exactly once. If it loses to a
            // later notification while waiting, reuse that winner instead of submitting another
            // generation and turning latest-wins into an authority-level livelock.
            force_refresh = false;
            let terminal = ticket.wait_terminal();
            self.clear_terminal_ticket(&root, &input, &ticket);

            if let Some(snapshot) = self.refresh.snapshot_for(&root, &input) {
                let mut report = self.report_from_snapshot(snapshot);
                if let Some(failure) = self.refresh.last_failure_for(&root, &input) {
                    report.diagnostics.push(format!(
                        "native plugin discovery refresh generation {} failed after the published snapshot: {failure}",
                        ticket.generation()
                    ));
                }
                return report;
            }

            if matches!(
                &terminal,
                NativePluginDiscoveryRefreshTerminal::Superseded { .. }
            ) {
                // Another notification won while this caller was waiting. Reuse that authority
                // ticket instead of scheduling a competing generation.
                continue;
            }

            return self.failure_report(&root, &input, ticket.generation(), terminal);
        }
    }

    fn ticket_for(
        &self,
        root: &NativePluginDiscoveryRoot,
        input: &NativePluginDiscoveryRefreshInput,
        force_refresh: bool,
    ) -> NativePluginDiscoveryRefreshTicket {
        let mut in_flight = lock_recover(&self.in_flight);
        let key = AuthorityRefreshKey::new(root.clone(), input.clone());
        if let Some((existing_ticket, existing_forced)) = in_flight
            .get(&key)
            .map(|existing| (existing.ticket.clone(), existing.forced))
        {
            if !existing_ticket.is_complete() {
                if !force_refresh || existing_forced {
                    return existing_ticket;
                }

                // The first notification after ordinary discovery creates one latest-wins
                // successor. Later notifications merge into that successor instead of making a
                // cancellation loop.
                let ticket = self.refresh.submit_with_input(root.clone(), input.clone());
                in_flight.insert(
                    key.clone(),
                    InFlightRefresh {
                        ticket: ticket.clone(),
                        forced: true,
                    },
                );
                return ticket;
            }
            in_flight.remove(&key);
        }
        let ticket = self.refresh.submit_with_input(root.clone(), input.clone());
        in_flight.insert(
            key,
            InFlightRefresh {
                ticket: ticket.clone(),
                forced: force_refresh,
            },
        );
        ticket
    }

    fn clear_terminal_ticket(
        &self,
        root: &NativePluginDiscoveryRoot,
        input: &NativePluginDiscoveryRefreshInput,
        completed: &NativePluginDiscoveryRefreshTicket,
    ) {
        let mut in_flight = lock_recover(&self.in_flight);
        let key = AuthorityRefreshKey::new(root.clone(), input.clone());
        if in_flight.get(&key).is_some_and(|current| {
            current.ticket.generation() == completed.generation() && current.ticket.is_complete()
        }) {
            in_flight.remove(&key);
        }
    }

    fn report_without_wait(
        &self,
        path: &Path,
        input: &NativePluginDiscoveryRefreshInput,
    ) -> NativePluginLoadReport {
        let Some(root) = self.cached_root_identity(path) else {
            return NativePluginLoadReport::diagnostic_only(
                "native plugin discovery cannot synchronously establish a root from its collector I/O lane",
            );
        };
        if let Some(snapshot) = self.refresh.snapshot_for(&root, input) {
            return self.report_from_snapshot(snapshot);
        }
        NativePluginLoadReport::diagnostic_only(
            "native plugin discovery cannot synchronously wait from its collector I/O lane",
        )
    }

    fn root_identity(&self, path: &Path) -> NativePluginDiscoveryRoot {
        let lexical_path = lexical_root_path(path);
        if let Some(root) = lock_recover(&self.root_identities)
            .get(&lexical_path)
            .cloned()
        {
            return root;
        }

        // Canonicalization can stat the filesystem. Keep cache readers, especially collector-I/O
        // re-entry, independent of that slow path and reconcile an equivalent concurrent miss
        // only after the filesystem work has completed.
        let root = native_plugin_discovery_root(&lexical_path);
        let mut root_identities = lock_recover(&self.root_identities);
        if let Some(existing) = root_identities.get(&lexical_path) {
            return existing.clone();
        }
        if root_identities.len() >= MAX_ROOT_IDENTITIES {
            if let Some(evicted) = root_identities.keys().next().cloned() {
                root_identities.remove(&evicted);
            }
        }
        root_identities.insert(lexical_path, root.clone());
        root
    }

    fn cached_root_identity(&self, path: &Path) -> Option<NativePluginDiscoveryRoot> {
        let lexical_path = lexical_root_path(path);
        lock_recover(&self.root_identities)
            .get(&lexical_path)
            .cloned()
    }

    fn report_from_snapshot(
        &self,
        snapshot: std::sync::Arc<NativePluginDiscoverySnapshot>,
    ) -> NativePluginLoadReport {
        NativePluginLoadReport {
            discovered: snapshot.candidates().to_vec(),
            diagnostics: snapshot.diagnostics().to_vec(),
            ..NativePluginLoadReport::default()
        }
    }

    fn failure_report(
        &self,
        root: &NativePluginDiscoveryRoot,
        input: &NativePluginDiscoveryRefreshInput,
        generation: u64,
        terminal: NativePluginDiscoveryRefreshTerminal,
    ) -> NativePluginLoadReport {
        let diagnostic = match terminal {
            NativePluginDiscoveryRefreshTerminal::DeadlineExceeded => format!(
                "native plugin discovery refresh generation {generation} exceeded its deadline before publication"
            ),
            NativePluginDiscoveryRefreshTerminal::Cancelled => format!(
                "native plugin discovery refresh generation {generation} was cancelled before publication"
            ),
            NativePluginDiscoveryRefreshTerminal::Shutdown => format!(
                "native plugin discovery refresh generation {generation} was stopped during shutdown"
            ),
            NativePluginDiscoveryRefreshTerminal::Rejected { reason } => format!(
                "native plugin discovery refresh generation {generation} was rejected: {reason}"
            ),
            NativePluginDiscoveryRefreshTerminal::Failed(error) => format!(
                "native plugin discovery refresh generation {generation} did not publish: {error}"
            ),
            NativePluginDiscoveryRefreshTerminal::Published(_) | NativePluginDiscoveryRefreshTerminal::Superseded { .. } => self
                .refresh
                .last_failure_for(root, input)
                .map(|failure| {
                    format!(
                        "native plugin discovery refresh generation {generation} did not publish: {failure}"
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "native plugin discovery refresh generation {generation} completed without a published snapshot"
                    )
                }),
        };
        NativePluginLoadReport::diagnostic_only(diagnostic)
    }
}

/// Production collection is fixed to the native authority. Only the service calls this function;
/// test fixtures use the cfg(test) constructor rather than an injectable production collector.
pub(in crate::plugin::native_plugin_loader) fn collect_refresh(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
    match request.input() {
        NativePluginDiscoveryRefreshInput::RootScan => collect_root_scan(request, sink),
        NativePluginDiscoveryRefreshInput::LoadManifest { export_root } => {
            super::super::discover_load_manifest::collect_load_manifest(request, sink, export_root)
        }
    }
}

fn collect_root_scan(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
    let mut visitor = MeteredDiscoveryVisitor { request, sink };
    let root = request.root().as_path();
    visitor.checkpoint()?;
    if !root.is_dir() {
        visitor
            .emit_diagnostic(|| format!("native plugin root does not exist: {}", root.display()))?;
        drop(visitor);
        return input_identity(request, sink, "missing-root", 0, 0);
    }

    let traversal = match traverse_plugin_manifests(root, &mut visitor) {
        Ok(traversal) => traversal,
        Err(NativePluginManifestTraversalError::Collection(error)) => {
            visitor.emit_diagnostic(|| error.to_string())?;
            drop(visitor);
            return input_identity(request, sink, "collection-error", 0, 0);
        }
        Err(NativePluginManifestTraversalError::Visitor(error)) => return Err(error),
    };
    visitor.checkpoint()?;
    drop(visitor);
    input_identity(
        request,
        sink,
        "scan",
        traversal.enumerated_directories,
        traversal.inspected_entries,
    )
}

pub(in crate::plugin::native_plugin_loader) fn input_identity(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    kind: &str,
    enumerated_directories: u64,
    inspected_entries: u64,
) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
    let required_bytes = request
        .root()
        .as_path()
        .as_os_str()
        .len()
        .saturating_add(kind.len())
        .saturating_add(96) as u64;
    let _admission = sink.reserve_scratch_bytes(request, required_bytes)?;
    NativePluginDiscoveryInputIdentity::new(format!(
        "native-plugin-authority:{}:{kind}:directories={enumerated_directories};entries={inspected_entries}",
        request.root().as_path().display(),
    ))
}

struct MeteredDiscoveryVisitor<'a> {
    request: &'a NativePluginDiscoveryRefreshRequest,
    sink: &'a mut NativePluginDiscoveryRefreshSink,
}

impl MeteredDiscoveryVisitor<'_> {
    fn checkpoint(&mut self) -> Result<(), NativePluginDiscoveryRefreshError> {
        self.request.check_active()
    }

    fn emit_diagnostic(
        &mut self,
        build: impl FnOnce() -> String,
    ) -> Result<(), NativePluginDiscoveryRefreshError> {
        let reservation = self.sink.reserve_diagnostic(self.request)?;
        reservation.insert(self.sink, build());
        Ok(())
    }
}

impl NativePluginManifestTraversalVisitor for MeteredDiscoveryVisitor<'_> {
    type Error = NativePluginDiscoveryRefreshError;

    fn checkpoint(&mut self) -> Result<(), Self::Error> {
        MeteredDiscoveryVisitor::checkpoint(self)
    }

    fn reserve_scratch(&mut self, total_bytes: u64) -> Result<(), Self::Error> {
        self.checkpoint()?;
        let _admission = self.sink.reserve_scratch_bytes(self.request, total_bytes)?;
        Ok(())
    }

    fn manifest(&mut self, manifest_path: PathBuf) -> Result<(), Self::Error> {
        self.checkpoint()?;
        match append_candidate_from_manifest_path(self.request, self.sink, manifest_path) {
            Ok(()) => Ok(()),
            Err(error @ NativePluginDiscoveryRefreshError::BudgetExceeded { .. })
            | Err(error @ NativePluginDiscoveryRefreshError::Cancelled)
            | Err(error @ NativePluginDiscoveryRefreshError::DeadlineExceeded) => Err(error),
            Err(error) => self.emit_diagnostic(|| error.to_string()),
        }
    }

    fn diagnostic(
        &mut self,
        build: impl FnOnce() -> NativePluginManifestTraversalDiagnostic,
    ) -> Result<(), Self::Error> {
        self.checkpoint()?;
        self.emit_diagnostic(|| build().into_message())
    }
}

fn lexical_root_path(path: &Path) -> PathBuf {
    // Bind a relative spelling to the current working directory before it enters the bounded
    // cache. Reusing `plugins` after a working-directory change must not project the old root.
    // This obtains process state only; canonicalization remains on the cold authority path.
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
