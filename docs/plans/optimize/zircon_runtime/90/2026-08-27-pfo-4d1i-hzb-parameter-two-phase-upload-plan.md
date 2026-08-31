# PFO-4d1i HZB Parameter Two-Phase Upload Plan

## Status

- Date: 2026-08-27
- Scope: HZB occlusion parameter workspace, RDG pass-local upload, and post-admission state commit
- Current status: `source_implemented_static_checks_passed_dynamic_validation_pending`
- Evidence boundary: source and call-graph review only; no Cargo, WGPU, RenderDoc, timing, or power result is claimed.

## Observed Structural Failure

`HzbOcclusionParamsWorkspace::prepare` currently performs three operations under one mutex:

1. materialize or reuse a persistent per-workspace uniform buffer;
2. call `queue.write_buffer` when `args_count` changes;
3. immediately publish `args_count` and `initialized = true`.

That publication was only valid while the write happened immediately. PFO-4d1e delays graph uploads until all graph stages succeed and the frame owner admits one merged batch. A mechanical conversion that updates the workspace during graph recording would make failure non-retryable: a later graph or backend failure would drop the upload while the workspace incorrectly reported the new value as resident.

The current `created_buffer_count = !initialized` calculation is also not a buffer-create metric. After a failed upload it reports the already-existing buffer as newly created on the next frame.

## Architecture Decision

HZB parameter updates use an explicit three-stage owner protocol:

1. `prepare`: create/reuse the persistent buffer, compare against committed `args_count`, and return an immutable upload plus a generation-qualified commit token when bytes are required. Preparation does not mutate committed state.
2. `append`: the HZB graph executor appends uploads to its pass-local `WgpuBufferUploadBatch`. Serial and parallel graph recording carry commit tokens beside the corresponding ordered pass result; no shared commit mutex is introduced.
3. `commit`: only after the merged frame upload batch is accepted and its `FrameBufferUpload` ticket is retained does the outer compiled-frame owner return tokens to `HzbOcclusionParamsWorkspace`. A token commits only when its workspace id and buffer revision still match.

Graph failure, backend admission failure, or ledger failure before this point leaves committed state unchanged, so the next frame produces the upload again. Buffer allocation may remain cached because it is CPU-owned materialization, but it is counted as created only in the call that inserted the entry.

## Data Model

- Workspace entry: persistent `Arc<wgpu::Buffer>`, monotonically assigned buffer revision, and `Option<u32>` committed args count.
- Prepared update: cloned buffer owner, optional `WgpuBufferUpload`, prepare stats, and optional `(workspace_id, buffer_revision, args_count)` commit token.
- Graph result: ordered `Vec<HzbOcclusionParamsCommit>` moved with pass-local uploads.
- Frame commit: one linear pass over accepted HZB tokens. Complexity is `O(dispatched phases)` and does not scan the workspace map.

Repeated tokens for the same workspace are legal but the last token in graph order wins only after every corresponding upload has been accepted. Current HZB phase ownership is expected to produce one workspace id per indirect execution; source tests will lock that no global workspace scan or commit-time buffer clone is added.

## Failure And Concurrency Contract

- The workspace mutex covers only one map lookup/materialization or one token commit.
- No guard crosses command recording, graph execution, backend admission, or submission.
- Parallel graph workers return owned tokens; the stage owner merges them in the same topology/bucket order as command buffers and uploads.
- A stale token cannot publish into a replaced entry because revision equality is required.
- An unchanged committed `args_count` emits no upload and no commit token.
- A failed prepared upload remains retryable without falsely incrementing buffer-create metrics.

## Validation Contract

Static/source checks must prove:

1. HZB production code contains no `queue.write_buffer` and HZB execute no longer accepts a queue;
2. `prepare` does not mutate committed args state;
3. graph pass/stage results carry owned HZB commit tokens without a shared mutex;
4. compiled rendering commits HZB tokens after upload admission and ledger retention, before scene submission;
5. unchanged state returns an empty upload batch and no token;
6. create stats reflect map insertion, not initialization state.

Managed Cargo tests, real WGPU output, screenshots, RenderDoc capture, p50/p95/p99 CPU/GPU timing, memory traffic, and power validation remain later acceptance gates.

## Current Source Result

- The workspace now stores a revision and `Option<u32>` committed args count. Preparation creates the persistent buffer only on map insertion and never mutates committed args state.
- Changed or uninitialized entries return one 32-byte immutable upload and one revision-qualified token; unchanged entries return neither.
- HZB execute no longer receives a queue. Its pass context appends uploads and owns tokens; recorded pass and stage owners move tokens in the same order as upload batches without a shared mutex.
- The compiled frame drains tokens only after graph success and commits them after `FrameBufferUpload` backend admission and ledger retention, before scene submission.
- Exact touched Rust files pass `rustfmt --edition 2021 --config skip_children=true`; scoped `git diff --check` passes with only existing LF/CRLF notices. Static ordering checks pass.
- The current scene-renderer production-candidate inventory, excluding test-only paths, changes from 13 writes in 11 files to 12 writes in 10 files.
- Cargo, real WGPU, PNG, RenderDoc, timing, memory-traffic, and power acceptance remain pending.
