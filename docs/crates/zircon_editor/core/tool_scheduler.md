# Tool scheduler

`core/tools/` is the editor-side authority for tools that need exclusive shared resources. It
does not execute plugin code while holding the authority lock. It owns admission, exact claim
identity, ordered lifecycle transitions, shutdown cleanup, and tool input capture.

## Identity and claims

`ToolDefinitionId` identifies a registered tool kind. `ToolInstanceId` combines that definition,
a non-zero `ToolOwnerGeneration`, and a non-zero process-local ordinal to identify one live tool
instance. Its qualified form is `definition@generation.ordinal`. Definition ids use the validated
ASCII alphanumeric, dot, dash, and underscore alphabet; the length limit reserves both numeric
fields and separators so every qualified instance id remains at most 128 bytes.

`ToolSchedulerService` owns generation issuance. Generation `1` is the protected built-in owner;
extension generations are monotonically issued and tracked in the same authority lock. The active
generation registry has an independent default ceiling of 1,024 entries. Instance allocation,
resource admission, and input-capture admission all reject a stale or unregistered generation, so
a deserialized `ToolInstanceId` cannot bypass owner revocation. Authority snapshots carry the
bounded active-generation set at the same revision as claim state, and a closed authority reports
no active owners.

Every admission reserves both a `ToolRequestId` and its future `ToolLeaseId`. A queued caller keeps
the returned `ToolRequestHandle` and may withdraw only that request id. An active caller keeps the
returned `ToolLeaseHandle` and may release only that lease id. A stale request or lease cannot
withdraw or release a newer claim, and one instance may have at most one queued or active claim.
There are no string-owner, resource-only release, set-specific, or `release_all` compatibility
paths.

`ToolResourceKey { kind, scope, channel }` is the exclusive-resource identity. Resource kind and
optional channel ids are validated, bounded identifiers. `ToolScope` is one of `Editor`,
`Project`, `Document`, `Window`, or `Viewport`; scope ids are typed rather than reconstructed from
display strings at the scheduler boundary. Built-in kinds enforce their scope when a key is
constructed or deserialized:

- `editor.viewport-input` uses `Viewport { viewport_id }`;
- `editor.modal-surface` uses `Window { window_id }`;
- `editor.scene-mode-slot` uses `Viewport { viewport_id }`.

Two viewports therefore do not contend merely because they both need viewport input, while tools
for the same viewport still serialize. Modal work conflicts within its native window and does not
globally block unrelated windows. Extension kinds may use any explicit scope and optional channel;
key construction alone does not register them.

The authority owns a compiled `ToolResourceCatalog`. Its three built-in registrations are present
at revision zero. Extension registrations require an active owner generation and declare a
canonical non-empty supported-scope set plus `Forbidden`, `Optional`, or `Required` channel policy.
Admission validates every key against that catalog before reserving a request or revision. The
catalog is bounded to 256 total kinds and 64 kinds per owner generation; duplicate, unregistered,
unsupported-scope, and invalid-channel claims fail with typed errors and zero scheduler mutation.
Non-built-in owners cannot register the reserved `editor.` resource namespace. Catalog entries are
included in revision-qualified snapshots in stable kind order.

A non-empty, canonically sorted `ToolResourceSet` is the only admission unit. `acquire` grants or
queues the whole set without partial ownership. Scheduler snapshots enumerate resource keys that
currently have ownership or queue state; there is no fixed global resource-lane table.

## Bounded fairness

Single-resource queues have an independent default ceiling of 64 entries per resource. Atomic
multi-resource requests use a separate global FIFO with a default ceiling of 64. The set-queue
head reserves only resources it overlaps, so it cannot block unrelated single-resource work.
Overlapping singles yield to that head. A release or withdrawal promotes the runnable set prefix
to a fixpoint and then promotes eligible singles in deterministic resource order.

Repeated acquire calls for the same canonical claim return the existing handle without duplicate
events. Queue-full and conflicting reclaims fail closed with typed denial details. Release and
withdraw reports include every lease activated by the resulting promotion.

## Input capture

Input capture is distinct from resource admission. `ToolInputScope` names a window and surface;
`ToolInputSource` adds optional user/device/pointer identity plus a pointer source, keyboard source,
or device source. Pointer identity remains optional because pointerless mouse metadata must not be
invented. A capture request binds that source to an active `ToolLeaseHandle`, the generation
derived from that lease's `ToolInstanceId`, the exact leased `ToolResourceKey`, and an explicit
priority. Capture callers cannot supply a second generation that disagrees with the lease. The
scheduler rejects stale leases and resources outside the active lease before capture state changes.

One source has at most one capture holder. Same-owner requests are idempotent, lower or equal
priority competitors are denied, and a higher priority request emits `Ended(Stolen)` before
`Started` for the replacement. Active captures are bounded to 64 and included in scheduler
snapshots. Capture end requires the exact capture id and owner generation. Explicit cancellation
retains the distinct `Aborted` terminal disposition. Window focus loss ends captures across every
surface in that exact window while preserving captures in other windows. Lease release emits
capture `OwnerLost` before tool `Deactivated`, and authority close emits capture `Shutdown` before
deactivating tools.

Owner-generation revoke is one revisioned authority transaction. It first blocks subsequent
admission by removing the generation from the active set, force-ends all matching captures with
`OwnerLost`, removes all matching active leases, and withdraws all matching queued requests. If the
owner registered resource kinds, claims from other owners that depend on any of those kinds are
removed in the same pass. The authority then promotes only surviving claims to a fixpoint, removes
the catalog entries, publishes `ResourceKindsRevoked`, and finally publishes
`OwnerGenerationRevoked`. A repeated stale revoke is an observable no-op; the built-in generation
cannot be revoked independently of authority close.

Runtime UI node capture remains the host-routing mechanism. It must bridge pointer metadata into
this editor authority; it is not a replacement for tool capture ownership.

## Authority and observation

`ToolSchedulerService` owns the editor-wide scheduler behind one mutex. Its state is
`Open -> Quiescing -> Draining -> Closed`, with `Faulted` as fail-stop recovery state. Quiescing
rejects new definitions, claims, and captures while still allowing exact cleanup. Close drains all
captures, leases, and requests without promotion. Mutex poison records a revisioned fault instead
of continuing with potentially invalid authority state.

Each mutation produces one `ToolScheduleReport` and one ordered `ToolTransitionBatch`. The service
commits the batch, snapshot revision, bounded outbox, and bounded 256-batch journal while holding
the authority lock, then dispatches after unlocking. Consumers use `ToolTransitionCursor` to read
contiguous batches; an old cursor beyond the retained journal receives an atomic snapshot resync.
Delivery health reports unobserved, dropped, backpressured, and failed deliveries without making
the message bus a second state owner.

## Integration boundary

`EditorContextBuilder` mounts the single `ToolSchedulerService`. Scene viewport mode activation
claims viewport input and the scene-mode slot for its exact `ViewInstanceId`. The export wizard
claims only the modal surface of its exact `UiWindowId`; an existing session rejects reuse from a
different window. Both retain exact lease/request handles from the shared service. New products
must use the same service and must not create a host-local scheduler, infer ownership from UI
booleans, or call tool/plugin callbacks while the scheduler mutex is held.

The capture lifecycle follows Unreal's `InteractiveToolsFramework` `InputRouter`: behavior sources
register with one authority, captures have explicit owners, focus loss force-terminates the affected
input side, and teardown terminates outstanding captures. Zircon keeps window/surface input-source
identity separate from resource scope so native routing metadata does not become scheduler
ownership. The retained UI metadata bridge still needs its own topology review before it can feed
this authority and is not claimed complete here.

Plugin registration now issues one `ToolOwnerGeneration` after contribution, command, scene-mode,
overlay-provider, and runtime-consumer candidates have all validated. It returns a host-issued
`EditorContributionHandle` containing the exact owner id, `ContributionTicket`, and owner generation;
external callers cannot construct a replacement. `OwnedContribution` retains that same handle, and
host revocation accepts only an exact handle match. A delayed teardown from an older plugin load is
therefore an idempotent no-op after a same-id plugin has reloaded instead of revoking the new
generation. Plugins request `ToolInstanceId` allocation through the host with that handle; the host
checks the exact live handle under the contribution lifecycle gate and supplies the generation to
the scheduler internally. Raw owner generations are not part of the external allocation contract.
Revocation retires the ticket's views and runtime consumers, then executes capture-first scheduler
revocation before publishing the remaining Store/router projection. Built-in contributions use the
protected built-in generation and are never allocated an extension generation. A rejected candidate
therefore does not leave a live generation behind, and a stale plugin generation cannot allocate or
acquire resources after ticket retirement.

Extension resource kinds use one declaration pipeline. `SerializedEditorContribution` owns the
versioned `zircon.editor.tool-resource-kind/1` DTO, and the plugin SDK builder canonicalizes its
non-empty scope set before publication. Materialization converts that DTO into an ownerless
`ToolResourceKindDeclaration` retained by `EditorExtensionRegistry` and `ContributionBatch`.
`ContributionStore` is the single source-namespace admission owner: a plugin may declare only
`plugin.<package-id>.*` resource kinds, while a built-in contribution carrying an extension kind is
rejected without publishing a Store generation. The host does not implement a second namespace
loop.

After every Store, command, scene-mode, overlay-provider, and runtime-consumer candidate has
validated, the host submits the complete declaration set to `register_owner_generation`. The tool
authority binds every declaration to the tentative generation in a cloned catalog and publishes
the owner plus all kinds in one revision. Duplicate, reserved-namespace, and capacity failures do
not consume a generation, change the catalog, or advance the revision. Later host allocation and
revocation require the exact `EditorContributionHandle`, so resource declarations, tool instances,
claims, and teardown share the same generation identity.

Typed native-library provenance, callback lease quiescence, and the final DLL unload transaction
remain separate gates. Generic contribution revocation must not be treated as proof that native
code is safe to unload until those gates are connected.

Resource admission and capture mutation are control-plane operations, not per-pointer-event
dispatch. This architecture change has static ownership and contention tests but no runtime
performance claim; any later optimization must first profile the integrated routing path and record
the measured bottleneck, workload, and comparison baseline in the optimization plan.
