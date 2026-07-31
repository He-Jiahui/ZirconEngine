# Tool scheduler

`core/tools/` is the editor-side authority for tools that require exclusive access to shared interaction
resources. It does not execute tools. It decides which typed tool owns a resource and which tools are waiting.

## Identity and resources

`ToolId` is a validated, cheaply cloned identifier. Empty values, values longer than 128 bytes, and characters
outside the ASCII alphanumeric, dot, dash, and underscore set are rejected before the id can enter scheduler
state.

`ExclusiveResource` currently defines three independent lanes:

- `ViewportInput` for direct viewport pointer and keyboard ownership;
- `ModalSurface` for modal dialogs and wizards;
- `SceneModeSlot` for the active editor scene mode.

Holding one resource does not implicitly hold another. A tool that needs multiple resources must acquire each
one explicitly and release all acquired resources when its activation is cancelled or shut down.

## Bounded FIFO contract

Every resource has at most one holder and one FIFO queue. The queue capacity is set when `ToolScheduler` is
created; the default is 64 pending tools per resource. A request observes one of these typed outcomes:

- `Acquired` or `AlreadyHeld` when the caller owns the resource;
- `Queued { position }` or `AlreadyQueued { position }` while another tool owns it;
- `Denied { holder, QueueFull }` when the bounded queue cannot accept another distinct tool.

Repeated acquire calls from the holder or an already queued tool are idempotent. They do not append duplicate
entries and do not emit duplicate lifecycle events. A non-holder release is side-effect free.

Releasing the holder emits `Deactivated` first, pops the FIFO head, and then emits `Activated` for the successor.
`withdraw` removes only the caller's pending request. `release_all` applies both operations across the three
resources and is the required shutdown/cancellation cleanup path.

`ToolScheduler` is deliberately not cloneable. The mounted service must keep one scheduler owner and expose a
handle to that owner if shared access is required; copying holder and queue state would create two conflicting
arbitration authorities.

## Event delivery

The scheduler intentionally stores no event history. Each mutation returns a `ToolScheduleReport` containing
the typed outcome and the finite lifecycle events caused by that mutation. The integration adapter must publish
those events synchronously to the Editor02 bounded message bus before it exposes the resulting UI state.
The report is `must_use` so accidentally discarding the publication obligation produces a compiler warning.

This keeps scheduling state bounded and prevents repeated denied requests from creating an unbounded internal
event queue. Consumers that need replay or retention must use the existing Editor02 delivery policy instead of
adding scheduler-local storage.

## Integration boundary

The folder owner is source-complete but is not yet mounted in `core/mod.rs`. The existing facade currently
contains uncommitted Editor02/13 module mounts and cannot legally be shared by a second immutable manifest.
Mounting, Editor05 scene-mode consumption, Editor15 export-wizard consumption, Editor02 bus publication, Cargo
validation, and managed commit remain successor work. No caller should duplicate this scheduler while waiting
for that integration.
