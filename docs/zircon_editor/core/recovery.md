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

## Snapshot Store

`AutosaveStore` writes snapshots only below:

```text
<project-root>/.zircon/autosave/<document-id>/<sequence>.<extension>
```

Document identifiers and extensions accept only ASCII letters, digits, `_`, and
`-`; zero sequences and traversal-shaped values are rejected. A snapshot is
written to a unique temporary file in the destination directory, flushed, then
renamed into place. Non-Windows targets also synchronize the parent directory.
No autosave API accepts a source path or copies, rewrites, or marks an
authoritative source file saved.

Each `AutosaveStore` clone shares one in-process sequence reservation set. A
document/sequence pair is unavailable while its write or retention rotation is
in flight; existing snapshots with the same numeric sequence are rejected even
when the requested extension differs. The store keeps the latest three numeric
sequences per document. If rotation fails after a new snapshot is durable, it
returns `AutosaveError::RotationAfterWrite` with the persisted path so callers
must allocate a new sequence rather than retry the same one.

An interrupted atomic write can leave a hidden temporary file. Such files are
ignored by snapshot discovery and sequence rotation; there is no persistent
autosave reservation or claim file to recover.

## Project Session Boundary

The in-process reservation cannot authorize multiple editor processes to write
the same project. `SessionGuard` persists the one project session record at:

```text
<project-root>/.zircon/session.lock
```

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
