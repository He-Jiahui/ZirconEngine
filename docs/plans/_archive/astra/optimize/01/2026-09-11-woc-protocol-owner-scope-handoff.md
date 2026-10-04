---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: WOC protocol admission and WOS schema identity
session: astra-woc-protocol-audit-20260911
---

# WOC protocol invariant owner-scope handoff

## Finding

This read-only audit evaluated two candidate Zircon-side slices:

1. admit only the active WOS schema identity (with an explicit migration for
   older identities); and
2. require every decoded command to pass one bounded typed payload-validation
   gate before queueing, replay, bot execution, or VM dispatch, with rejection
   leaving queues and simulation state unchanged.

Neither invariant has a safe single-file owner in the current checkout. The
WOS identity is shared by the authored Zr writer/reader, the ignored native VM
codec adapter, and runtime tests. The command invariant is shared by protocol
decode/validation, the payload registry and payload modules, server ingress,
and the client/replay/bot consumers. A local helper or constant change would
leave other paths on a different schema/admission contract.

## Evidence and ownership boundary

- The canonical plan records both open contracts: WOS writer/reader identity
  drift at `docs/plans/astra/optimize/01-review-and-repair.md:87` and command
  admission drift at `:88`. The detailed protocol finding remains
  `PROTOCOL-P0-002` (one bypass-proof typed decode -> validate -> admit path),
  with related identity and authority findings in the same protocol review.
- The tracked protocol and host files are already modified by other work:
  `examples/woc/native/crates/woc_protocol/src/lib.rs`,
  `src/payload.rs`, `src/movement_input.rs`, the server
  `apps/woc_server/src/fixed_tick_driver.rs`/`src/main.rs`, and the native
  runtime/client/server manifests and tests. This audit did not alter or
  stage any of them.
- Eleven production/test protocol files are ignored by the repository rule
  `.gitignore:139` (`examples/*`), including
  `challenge_payload.rs`, `corpse_harvest_*`, `emote_payload.rs`,
  `inventory_move_payload.rs`, `save_loadout_payload.rs`,
  `telemetry_payload.rs`, `town_focus_payload.rs`,
  `weapon_skin_{contract,payload}.rs`, and
  `tests/command_admission.rs`. They are visible in the working tree but are
  not clone-reproducible source ownership.
- `woc_protocol/src/payload.rs` currently decodes a `Command`, calls
  `Command::validate`, and returns a raw `Command`; server ingress separately
  calls `command.validate()` in `FixedServerTickDriver::enqueue_commands`
  before appending to its pending queue. That is useful evidence for the
  invariant, but it is not a unique typed admission boundary and does not
  prove replay/client/bot/VM paths use the same gate.
- `zircon_plugins/zr_vm_language/runtime` is also dirty, with modified VM
  adapter files and staged state-encoding/WOC tests. Its backend feature uses
  path dependencies on `../../../../zr_vm/...`; the external checkout is
  dirty and its checkpoint/budget work is not an immutable receipt. No
  external VM file was read for mutation or changed here.
- `examples/woc/native/Cargo.lock` exists, but the current client manifest
  requires the `zircon_runtime` graph that the nested lock does not yet carry;
  the existing runtime contract note records managed lock regeneration as
  required. A reproducible native receipt therefore cannot be produced by
  this source-only slice.
- Coordinator inspection found an active waiting-validation client
  presentation session (`astra-woc-client-presentation-20260911`) covering
  client/app and ignored VM-adapter paths. No live exact lease for the
  protocol payload/admission set was discovered. The client/runtime overlap
  is enough to require an explicit owner handoff before changing shared
  ingress or codec files. This audit acquired no coordinator lease.

## Why this is blocked

Changing `PROTOCOL_VERSION`, a WOS admission constant, or a validator in
`payload.rs` alone could create a mixed generation: ignored payload modules
would still export the old registry, server ingress could retain a separate
check, and VM/replay/client consumers could accept a different identity. The
candidate WOS path also crosses the VM adapter whose real symbols and lock
graph require the external dirty dependency. Without a claimed, tracked
source set and a real dependency receipt, static source inspection cannot
prove that invalid input is rejected before every state-mutating path.

## Dependency-ready implementation order

1. A protocol owner should claim the complete exact scope (tracked protocol,
   all ignored payload/contract modules and admission tests, server ingress,
   and each replay/client/bot consumer) and make the production source
   reproducible through an explicit manifest/ignore decision. Coordinate the
   shared client/runtime paths with the active presentation owner.
2. Define one protocol identity table. Implement a typed
   `DecodedCommand -> ValidatedCommand -> AdmittedCommand` boundary that all
   ingress, replay, bot, and VM dispatch paths consume; reject before queue or
   VM mutation and preserve duplicate/ordering/budget checks in that owner.
3. In the same or a separately claimed schema lane, define WOS writer/reader
   identity and an explicit older-version migration. Keep the outer WOC wire
   version distinct from the inner WOS identity.
4. The VM owner must publish an immutable external `zr_vm` commit/patch and
   dependency lock receipt, then regenerate the managed native lock graph.
   Only after that receipt exists can the adapter prove checkpoint,
   rollback, bounded-call, and state-codec behavior.
5. Run the managed Windows focused protocol/codec/admission tests, native
   client/server tests, real ZrVM round-trip and next-tick equivalence checks,
   and clean-clone source/lock reproduction. No Cargo/native command was run
   for this handoff.

## Status

This record is intentionally `blocked_owner_scope`; it is neither an
implementation nor an acceptance claim. It is the only file written by this
audit. No WOC protocol, Zr language, or external VM source was modified, no
commit was created, and no coordinator record lease was acquired (therefore
there was no lease to release). The next owner must establish attribution and
leases before implementing either invariant.
