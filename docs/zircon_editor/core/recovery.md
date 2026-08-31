# Editor Recovery

`zircon_editor::core::recovery` owns the headless-safe autosave foundation,
residual session-lock record, and recovery decision contract. It does not own
source-file saves, transaction dirty state, project liveness arbitration, job
submission, or recovery UI.

## Dirty Input And Scheduling

`AutosaveDocumentState::from_history_dirty(document, HistoryDirtyState)` is the
only production construction path for a scheduled document. `HistoryDirtyState`
is the immutable result of Editor03's `saved_top` history query; document-to-
history routing remains with the editor manager. Recovery stores no mutable
dirty registry and never changes a history save token.

`AutosaveScheduler` defaults to a 300-second interval. At or after the due
time, `plan` returns the sorted set of dirty documents and immediately enters
single-flight state. Additional polls return no plan until one terminal method
is called:

- `mark_submission_failed()` releases a plan whose Editor14 job was not
  admitted, allowing an immediate retry.
- `mark_finished(at)` handles either a successful or failed admitted write,
  releases single-flight, and starts the next interval from `at`.

The recovery core only creates an `AutosaveJobPolicy`. Its job specification is
always `JobCategory::Misc`, `JobPriority::Background`, and the save mutex group
provided by the save owner. Editor14 remains the only job queue owner and must
submit and report the terminal task.

`AutosaveJobAdapter` reserves the selected batch through that shared admission
owner before it materializes document requests. A snapshot source must declare
its project-relative `AutosaveSourcePath` before admission, so a capture failure
cannot lose its document/source identity. Completion polling is bounded by an
explicit ticket count; the default is `DEFAULT_AUTOSAVE_COMPLETION_BUDGET`
(64). A poll reports cumulative succeeded/failed counts, the remaining ticket
count, immutable generation health counters, and only the document-bound
terminal outcomes newly observed by that poll. Each outcome retains its stage,
retryability, bounded error chain, and any still-usable snapshot path. Pending
tickets rotate through one retained `VecDeque`; the terminal poll advances the
scheduler exactly once and resets only the batch counters for the next interval.

## Snapshot Store

`AutosaveStore` writes snapshots only below:

```text
<project-root>/.zircon/autosave/<document-id>/<sequence>.<extension>
```

`<project-root>` is the physical operation identity selected by the shared project-path resolver, so directory aliases do not create a second autosave tree.

Document identifiers and extensions accept only ASCII letters, digits, `_`, and
`-`; zero sequences and traversal-shaped values are rejected. Every snapshot
also receives an `AutosaveSourcePath`: a non-empty UTF-8 path made only of
project-relative normal components. It is recovery metadata, never an
authority to open, copy, rewrite, or mark the source file saved. A snapshot is
written to a unique temporary file in the destination directory, flushed, then
renamed into place. Non-Windows targets also synchronize the parent directory.

Each document/sequence write first claims a hidden per-sequence marker with an
atomic create-new operation. The marker is shared across independent
`AutosaveStore` instances and processes, so a document/sequence pair remains
unavailable while its write or retention rotation is in flight. Existing
snapshots with the same numeric sequence are rejected even when the requested
extension differs. A payload `N.<extension>` becomes recoverable only after its
no-replace `N.snapshot.json` commit record is published. That record carries the
source content digest captured with the dirty generation, the explicit journal
availability state, the autosave schema id/version, and the payload BLAKE3
checksum. The store keeps the latest three committed numeric sequences per
document. If rotation fails after a new snapshot is durable, it returns
`AutosaveError::RotationAfterWrite` with the persisted path so callers must
allocate a new sequence rather than retry the same one.

An interrupted atomic write can leave a hidden temporary file or its
per-sequence marker. Both are ignored by snapshot discovery and sequence
rotation. A retained marker prevents only reuse of its numeric sequence; the
runtime does not delete it by inference, because project-session admission owns
safe interrupted-process repair.

## Recovery Catalog

Each document autosave directory has one immutable `recovery.json` v2 that
binds the document id to a validated, normalized `AutosaveSourcePath` and its
BLAKE3 path identity. The first mapping is published with no-replace semantics;
a concurrent or later write must read the same mapping, and a different source
path is rejected. This prevents a later snapshot from silently changing the
document restored by an earlier snapshot.

`AutosaveStore::recovery_catalog()` enumerates only committed metadata/payload
pairs and returns `AutosaveRecoveryCatalogReport { candidates, diagnostics }`.
Once the catalog root is enumerable, malformed document directories, metadata,
or checksums are quarantined per entry so they cannot suppress valid documents.
The catalog streams the current source digest and classifies each snapshot as
`SourceMissing`, `SnapshotAheadOfSource`, `SourceDiverged`, or
`SnapshotAlreadyCommitted`; it does not use modification times. The last state
is not offered for recovery. A missing source remains an explicit candidate
state rather than a guessed restore action.

## Terminal Autosave Diagnostics

Every worker terminal result is appended as an independent no-replace JSON
record below `<project-root>/.zircon/autosave/diagnostics/`; records retain the
document/source outcome but never authorize source overwrite. Retention keeps
the newest 128 records. `AutosaveDiagnosticStore::load()` isolates one damaged
record and still returns the rest for Welcome or Hub; `document_folder()`
provides the per-document autosave folder for an explicit host open-folder
action. Normal worker persistence stays off the UI poll path. If a job is
cancelled before its worker begins, the active/retired project lifecycle writes
that one fallback record before discarding the adapter; a failed write retains
the retired adapter for a later poll retry.

## Final Autosave Shutdown

`EditorAutosaveService::shutdown_with_final_autosave` is the only editor
shutdown path for autosave. The retained host first captures document-bound
requests from the current dirty-toolkit projection; snapshot serialization
remains deferred to the recovery worker. The service fences interval admission,
drains any already-admitted work while the shared Editor14 job system remains
live, then uses a final scheduler window that bypasses the periodic deadline.
It still observes the shared entry and byte admission limits, repeats bounded
windows until every requested document reaches a final terminal outcome or the
deadline expires, and never starts a second in-flight window.

Only outcomes from that final pass determine whether normal editor shutdown may
release the project session guard. A request left without a final terminal
result at the deadline becomes a document-bound, retryable lifecycle
cancellation and is persisted through the normal fallback diagnostic path. A
diagnostic persistence error, final failure, or unfinished shared job keeps the
normal project-close path from removing its OS-backed admission lease. This
recovery boundary neither writes authoritative source files nor bypasses the
shared job-system shutdown owner.

If final requests arrive without an active project adapter, the service returns
an explicit retryable lifecycle cancellation without starting a worker. It
cannot derive a diagnostic-store root in that invalid lifecycle state, so the
host must treat the outcome as a failed close and preserve the project guard
rather than silently dropping the request.

## Project Session Boundary

An autosave sequence reservation cannot authorize multiple editor processes to
write the same project. `SessionGuard` persists the one project session record
at:

```text
<project-root>/.zircon/session.lock
```

`<project-root>` is the physical operation identity selected by the shared project-path resolver, so directory aliases address the same persisted session lock and ownership lease.

Each record carries the process id, a process-instance identity, and the latest
heartbeat. Initial acquisition publishes a fully flushed staging record without
overwriting an existing lock. Heartbeat and residual takeover stage and flush a
new record before atomically replacing the current file, so a failed write
leaves the selected record readable. `release` and `refresh_heartbeat` both
compare the exact persisted record before acting and report `OwnershipLost`
when that comparison observes another owner. The lifecycle owner serializes
cross-process ownership transitions through the guard's OS lease: Windows
holds a canonical-project `Global\` named mutex for the guard lifetime, while
Unix holds a nonblocking advisory lock on the existing `.zircon` directory. Neither
implementation adds a second persisted lock path. This persistence primitive
is not a liveness detector or a content-CAS protocol for arbitrary writers.

`SessionGuard::durability()` reports whether the latest create, replacement, or
removal was published, and whether a post-publication directory sync was
uncertain. `PublishedWithDurabilityUncertainty` means the record mutation was
already live but directory synchronization failed; the guard remains
authoritative and must not be discarded as if no mutation occurred. Windows
creates through `hard_link`, replaces through `ReplaceFileW`, and removes the
record directly; this layer has no parent-directory fsync equivalent for any
of those Windows publication paths, so successful mutations intentionally
report this state.

The editor manager removes a session record only through its explicit,
successful project-close transaction. Dropping the manager never calls
`SessionGuard::release()`: an early startup failure, final-autosave failure, or
release I/O failure must leave a residual record after its OS lease is dropped,
so the next startup enters the explicit recovery/admission path instead of
silently treating the project as cleanly closed.

`SessionGuard::replace_residual_at` is deliberately not a liveness detector.
The project lifecycle owner must first determine that the inspected record is
not a live editor, then pass that exact record for takeover. This keeps PID and
process-lifetime policy at the unique project-activation boundary rather than
duplicating it in autosave or UI code. The unresolved lifecycle handoff remains
`docs/plans/zircon_editor/editor/16/failure-2026-07-23-project-session-lock-reuse-for-recovery.md`;
Editor17 must not add a second lock path or bypass the prepared-project
installation boundary.

## Restore Decisions

`RestoreFlow::detect` returns `NoRecoveryNeeded` only for a missing lock. A
residual without a newer autosave returns `ResidualTakeoverRequired` and still
exposes the exact record for lifecycle takeover. A residual with candidates
returns `RecoveryRequired`; candidates are unique per document.
`RestoreFlow::plan` requires exactly one explicit `RestoreAutosave`,
`DiscardAutosave`, or `OpenComparison` choice for every offered document, and
rejects any choice when no document was offered. The returned `RestorePlan`
describes choices only; it does not read, overwrite, or delete source files.
Project activation and the notification presentation owner remain responsible
for wiring this plan to the startup lifecycle.
