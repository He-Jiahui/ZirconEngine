---
related_code:
  - zircon_editor/src/ui/preview_scene/mod.rs
  - zircon_editor/src/ui/preview_scene/preview_scene.rs
  - zircon_editor/src/ui/preview_scene/preview_subject.rs
  - zircon_editor/src/ui/preview_scene/tests.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/ui/animation_editor/session.rs
implementation_files:
  - zircon_editor/src/ui/preview_scene/mod.rs
  - zircon_editor/src/ui/preview_scene/preview_scene.rs
  - zircon_editor/src/ui/preview_scene/preview_subject.rs
  - zircon_editor/src/ui/preview_scene/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/ui/preview_scene/tests.rs
  - cargo test -p zircon_editor --lib ui::preview_scene::tests --locked --jobs 1
doc_type: module-detail
---

# Preview Scene Framework

## Purpose

`zircon_editor::ui::preview_scene` is the editor-side lifecycle contract for a runtime-backed
secondary preview session. Animation clips, graphs, state machines, and montages can share one
`PreviewScene` through `Rc<RefCell<...>>`, matching Persona's shared preview-scene ownership while
keeping the actual runtime world and rendering authority outside the editor UI module.

The module is not a Play-In-Editor controller. `PlaySessionController` continues to own PIE state,
gateways, edit protection, and production play backends. The preview backend is the sole adapter
point to the future Editor04 secondary-session implementation.

## Lifecycle

`PreviewScene::open` creates exactly one backend secondary session. `close` is idempotent and
destroys it once. If backend destruction fails, the session handle is restored so the caller can
retry; `Drop` performs best-effort cleanup for an abandoned scene. The backend is also responsible
for exposing its typed `Error`; `PreviewSceneError<BackendError>::Backend` preserves that source
instead of flattening it into an editor-local string.

The framework forwards subject updates, playback updates, `invalidate_views`, and `focus_views` to
the active session only. After close, calls return `PreviewSceneError::Closed` rather than creating
a replacement session implicitly.

## Subject And Playback

`PreviewSubject` describes a primary asset, optional additional assets, optional animation asset,
and parameter overrides. Asset locators are intentionally opaque strings here: URI resolution,
asset loading, mesh creation, animation evaluation, and GPU objects belong to the runtime-backed
backend. The builder deduplicates additional asset locators and stores parameter overrides in a
stable map.

`PreviewPlayback` carries playing, looping, rate, and time. Non-finite rate/time values are rejected
before calling the backend. The preview scene treats playback as a session command, not as a local
timer, so editor widgets cannot advance a second clock independently of the runtime session.

## Scope

This M2.2 contract does not itself implement a runtime secondary session, a preview viewport,
animation graph evaluation, frame scheduling, or UI widget painting. Editor04 must provide the
backend that maps a real secondary runtime session into `PreviewSceneBackend`; animation-family
toolkits then receive the same shared scene instance. No direct dependency on `PlaySessionController`
or a legacy preview implementation is permitted.
