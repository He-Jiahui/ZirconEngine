# Editor Recovery

`zircon_editor::core::recovery` owns the headless-safe autosave foundation. It
does not own source-file saves, transaction dirty state, project session locks,
job submission, or recovery UI.

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
the same project. Editor16 owns the required project session lock and startup
contract; its open handoff is
`docs/plans/zircon_editor/editor/16/failure-2026-07-23-project-session-lock-reuse-for-recovery.md`.
Until that lock is available, startup must not connect autosave to a real
multi-process project lifecycle. Editor17 M2.2 will add abnormal-exit detection
and the recovery Decision flow after the Editor16 lock and Editor03 journal
interfaces are available.
