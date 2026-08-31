# Runtime Rich-Text Provider Snapshot And Revoke Design

Date: 2026-08-30
Status: `architecture_review_complete / implementation_not_started / managed_validation_pending`
Owner: Text07 / RRT-P1-010, RRT-P1-014, RRT-P1-016

## Scope

This record defines the provider ownership and unload contract required before Zircon exposes rich
decorators or emoji providers to project/runtime plugins. It does not add registration-count magic
constants, a text-local plugin identity, a global provider registry, or a second lifecycle authority.
The current parser-local mutable registration API remains a construction/test path until the owner
chain below is implemented.

## Current-source findings

- Production rich compilation is now owned by
  `UiSurface -> UiTextMeasureCache -> SharedTextLayoutSession -> RichTextParser`; independent Surface
  sessions do not share compiled residency.
- `RichTextParser::register_decorator` and `register_emoji_shortcode` require `&mut self`, while a
  retained session stores `Arc<RichTextParser>`. Repository call tracing finds registration only in
  tests. Product hot reload/unload therefore has no usable registration path today.
- A successful current registration now advances a checked non-reusing generation and retires that
  parser's compiled cache. Failed registration is transactional. This removes stale residency but is
  not an unregister or plugin-unload fence.
- Request-local `RichParseBudget` authorizes one parse. Provider count, provider-name bytes, registry
  retained bytes, and plugin ownership are retained-service concerns and must not be added to that
  request budget.
- Core already owns the required unload primitive: service admission closes, `ServiceCallGuard`
  tracks in-flight calls, a condition variable drains them with an optional timeout, then the module
  service is invalidated. Runtime plugin bridge providers already transition at a frame boundary.
  A text-only active-call counter would duplicate this authority and still could not prove that a
  native module is unloadable.

## Unreal reference decision

The primary local reference is Unreal's
`URichTextBlock -> FRichTextLayoutMarshaller -> ITextDecorator` chain:

- `URichTextBlock::RebuildWidget` creates decorator instances, passes shared decorator references to
  one marshaller, and installs that marshaller on one retained rich-text widget.
- `FRichTextLayoutMarshaller` owns parser/writer/decorator references. `SetDecorators` replaces the
  retained set, while `SRichTextBlock::SetDecorators` immediately dirties content and layout.
- `URichTextBlock::ReleaseSlateResources` resets the widget/style owner and empties decorator
  instances. Provider objects are not kept in a process-global parser registry.

Zircon should preserve this owner/rebuild model. Its exact-tag `HashMap` remains valid because Unreal's
linear scan serves the broader arbitrary `ITextDecorator::Supports` predicate contract.

## Target ownership

```text
Project runtime / plugin catalog
  -> immutable RichTextProviderCatalogSnapshot
      -> UiSurface construction or frame-boundary replacement
          -> SharedTextLayoutSession
              -> RichTextParser provider snapshot + compiled cache owner
                  -> compile-local provider call lease
```

The upper runtime/plugin layer qualifies providers with its canonical project, runtime module, and
provider package identities. Text receives one opaque, validated owner descriptor; it must not invent
parallel string IDs or depend upward on UI Surface identity.

The snapshot owns one exact-tag index and one emoji index. Compilation clones one snapshot `Arc`, so
it never holds a registry write lock while invoking provider code. A callback may run on a text worker,
must be `Send + Sync`, receives only neutral parse/decorate data, and may not access UI/GPU state or
reenter registry mutation.

## Publication and revoke protocol

1. Build a candidate catalog off the render/layout path. Validate owner qualification, normalized
   keys, duplicates/reserved names, provider-count and retained-byte policy, then allocate the next
   non-reusing generation before mutation.
2. At a Surface frame boundary, publish the immutable snapshot and invalidate rich derived layout.
   The parser cache admits only the published generation; a completion from a retired snapshot may
   return to its caller but may not reinsert old-generation residency.
3. Deactivation first closes Core provider-call admission. Publish a catalog without that owner's
   entries to every affected Surface, then clear/retire derived compiled residency.
4. Drain compile-local provider call leases through the existing Core module/service timeout. Only a
   successful drain permits plugin cleanup or native-library unload. Timeout returns a typed module
   lifecycle block; it must not force-drop code still executing.
5. Existing `Arc<CompiledRichText>` values remain valid because compiled artifacts contain neutral
   text/style/link/inline metadata rather than provider trait objects. The revoke fence covers active
   provider execution, not immutable artifact readers.

## Admission and telemetry

Introduce a retained `RichTextProviderBudget`, separate from `RichParseBudget`, only after the
registry profile establishes defaults. It must account for custom decorator count, emoji count,
normalized key/replacement bytes, estimated registry residency, and per-owner shares. Registration
failure reports the qualified owner and one low-cardinality reason without copying complete provider
metadata into frame diagnostics.

Thresholds require an E-drive release profile at 1/64/1,024 providers and representative project
packages. No production numeric default is authorized by this architecture record alone.

## Required validation

- Correctness: duplicate/reserved/over-budget transactions, generation exhaustion, activate/reload/
  deactivate, two independent projects and Surfaces, blocked callback during revoke, no callback
  admitted after close, timeout leaves the plugin loaded, and old completion cannot regain residency.
- Concurrency: 1/2/8 compile workers racing publication and revoke; every waiter receives one typed
  terminal result and no registry lock is held across provider code.
- Performance: cold publication plus warm compile at 1/64/1,024 providers, allocation count, retained
  bytes, RSS, contention and package power. Exact-tag lookup must remain independent of unrelated
  provider count.
- Product: current-source Windows WGPU text/layout frame and reviewed PNG under
  `docs/tests/runtime/text`; no strategy-only screenshot is accepted.

## Milestones

| Milestone | State | Deliverable |
|---|---|---|
| M0 architecture and call-map review | complete | Unreal owner mapping, Zircon Surface chain, Core call-guard/drain reuse decision |
| M1 retained provider policy owner | pending | qualified owner descriptor, measured provider budget, typed registration outcome |
| M2 immutable parser snapshot | pending | lock-free callback read path, generation-qualified cache admission and retirement |
| M3 Surface publication | pending | construction injection, frame-boundary replacement and rich-layout invalidation |
| M4 module revoke integration | pending | Core admission close, lease drain/timeout, plugin unload gate |
| M5 managed acceptance | pending | Cargo tests, concurrency/profile/RSS/power, WGPU frame and reviewed PNG |

## 2026-08-30 execution audit

The repository Runtime Interface convergence audit was run before implementation. The full aggregate
audit exceeded both a 65-second and a 244-second read-only tool window; both spawned audit processes
were explicitly terminated. The task-relevant inventory/module/plugin-lifecycle sub-audit then completed
in 109.1 seconds and reported:

- production `stub_module_descriptor` usage is zero and `zircon_runtime` has a real `TextModule` owner;
- the plugin runtime gap list is empty, but Runtime06 remains `in_progress`;
- the plugin-surface lifecycle audit reports drift in Runtime06 refinement, native namespace re-export
  count/classification, unclassified namespace symbols, and `zircon_app` native-plugin call-site counts;
- `zircon_runtime` is still classified `needs-refactor` because of large production owners.

Therefore M1-M4 are not admitted as a Text-local implementation slice in this worktree. M1 also remains
blocked by this design's own no-default rule until the E-drive 1/64/1,024-provider profile exists. A
Text-private call counter, unload fence, plugin identity, numeric provider budget, or Surface lifecycle
facade would duplicate an unsettled Runtime06/Core authority. The legal next implementation remains an
upstream-qualified catalog plus Core admission/drain integration after those owners converge; this Text
goal continues with unrelated non-validation work instead of treating the audit as an acceptance queue.

Current status remains `architecture_review_complete / implementation_not_started /
runtime06_owner_convergence_and_provider_profile_pending / managed_validation_pending`.

## Non-goals for the next slice

- Do not put provider limits into request-local `RichParseBudget`.
- Do not make a process-global `RuntimeRichTextService` or thread-local parser.
- Do not hold `RwLock`/`Mutex` registry guards while invoking a decorator.
- Do not claim revoke completion from cache clear or `Arc<CompiledRichText>` retirement.
- Do not implement cancellation by abandoning a thread that may still execute plugin code.
