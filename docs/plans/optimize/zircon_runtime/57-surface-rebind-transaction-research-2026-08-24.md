# Runtime57 surface rebind transaction research

Status: research complete; implementation validation pending. This is not a
milestone acceptance record and does not update PLH-P1-059 status.

## Scope

PLH-P1-059 requires a resize rebind to prepare the replacement surface and
extent, fence submissions using the old surface, publish the replacement as
one render-framework state transition, then retire the old generation. ABI
qualified surface generations remain PLH-P1-060 work and are out of scope.

## Current-source observations

- `RuntimeRenderBridge::ensure_viewport` destroyed and recreated a viewport
  whenever its requested size changed. Calling the existing bind path after a
  resize therefore discarded the old surface before replacement could succeed.
- The WGPU binding path created a new `ViewportSurface` and immediately
  overwrote the record, with no submission fence and without updating the
  viewport descriptor extent atomically with that replacement.
- Destroying a viewport released frame histories. A same-handle rebind must
  explicitly invalidate resolution-dependent histories, capture state and
  temporal runtime state instead of retaining them across the new extent.
- Async readback callbacks retain an `Arc` to their capture mailbox. Clearing
  one shared mailbox in place lets a late callback write an old generation
  into the replacement viewport; the replacement transaction must install a
  new mailbox generation instead.

## Reference and design decision

The local Unreal source keeps `FSlateViewportInfo` as the viewport owner during
`ResizeViewport`: it suspends resize-sensitive rendering, waits for cleanup,
flushes render commands, updates the extent, then calls `RHIResizeViewport`.
Its separate `ReleaseRHI` path blocks for GPU idle before releasing the RHI
resource. `WindowsApplication` also removes a native window on `WM_DESTROY`,
which is distinct from the resize path.

Zircon has a raw-handle WGPU surface rather than Unreal's in-place RHI
viewport, so this implementation follows the ownership and ordering model,
not a line-for-line API translation. It does not introduce an ABI generation
field prematurely:

1. Prepare and configure the new native surface while the current one remains
   published.
2. Drain the render submission scheduler.
3. Under the render-framework operation/state locks, publish the new surface
   and descriptor extent, invalidate extent-dependent runtime state, and
   detach the old capture mailbox generation.
4. Release retired histories through the renderer after record publication.
5. Commit the bridge cached size only after the runtime bind succeeds.

The bridge uses a dedicated surface-rebind viewport lookup, so ordinary
headless/capture resize behavior keeps its existing create/destroy policy.

`finish_submission()` is a Zircon submission-scheduler drain, not a GPU
timeline-completion fence. No shared Runtime90-style completion contract is
available at this boundary yet, so this slice must not claim GPU-idle safety or
use a global device wait that would inflate resize latency and power. WGPU
resource lifetime retains queued GPU work; a qualified GPU timeline point is
follow-up work owned by Runtime90 rather than an ad hoc Runtime57 API.

## Profiling and validation plan

- The transaction is instrumented as
  `runtime.render_framework.replace_viewport_surface`.
- A managed Windows run must collect resize count, transaction p50/p95/p99,
  submission-fence wait, retained-history release count, GPU memory before and
  after repeated resize, and CPU package power alongside the existing native
  presenter metrics.
- The acceptance run must include repeated resize during synchronous and
  pipelined submission, a late readback callback, and a failed replacement.
  It must show that the prior surface remains published on prepare failure,
  no stale capture reaches the new mailbox, and no viewport handle churn
  occurs on a successful rebind.

No profiler samples or Cargo results are recorded here because the coordinator
did not return a terminal validation receipt for the current source snapshot.
