---
related_code:
  - zircon_editor/src/core/i18n
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/notifications/presentation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/TextLocalizationManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/TextLocalizationManager.cpp
---

# Protected plan routing: editor core i18n

## Reason for routing

`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`, `review.md`, `pending.md` and
the owner plans are protected/foreign dirty in this session. This record routes the completed
current-source review without overwriting their owners. Evidence source:
`2026-08-16-editor-core-i18n-current-architecture-review.md`.

## Requested Performance01 corrections

Do not add a new independent P0 catalog-rewrite task. Correct the existing tasks as follows.

### PERF-MVP-596

- EditorI18nService, not the notification authority, owns the monotonic text revision and typed
  invalidation cause.
- The unified activity notification projection consumes that revision together with Decision, Toast
  and Progress generations. A locale change builds/applies one localized generation; stable frames
  perform zero translation, fallback allocation, formatting or row reconstruction.
- Preserve captured-locale consistency and empty-generation clearing. Remove the per-fallback
  `EditorLocale::english()` allocation and bound missing-key materialization/diagnostics within the
  changed-generation projection.
- Distinguish `LocaleChanged` from future `BundleRevisionChanged`; do not use one generic event as a
  global font/text/cache clear signal.

### PERF-MVP-591

- The settings authority remains the sole locale-preference owner and publishes one affected locale
  slot after unlock. Accepted settings generation is a trigger; i18n publishes the visible-text
  revision only when the canonical locale actually changes.
- Stable retained frames do no settings snapshot read or i18n synchronization. No-op and stale
  settings generations increment neither locale nor text revision.
- Keep current stale-snapshot ordering tests and add exact revision-count acceptance.

### Message-bus compatibility path

Repository search found `EditorTopic::i18n` production publication but subscriber registrations only
inside builder tests. Editor17 must remove the JSON/editor-message fanout if no product/plugin consumer
is identified. If an external consumer is required, serialize the same typed revision/cause only at
that boundary and retain existing bounded resync semantics. Do not preserve an unused internal bus
path solely for tests.

## Requested owner-plan updates

### Editor17

Own one EditorI18n text-revision contract, typed invalidation cause, retained English fallback owner
and bounded missing-key diagnostics. Integrate with the one settings affected-slot dispatcher and one
unified notification projection. Do not create an i18n-private thread pool, notification-private
locale generation or second localization authority.

### EditorUI08

Retain the last applied text/notification revision and update visible localized rows at most once per
accepted generation. Future font/glyph invalidation must use the typed cause and revision-checked UI
commit so async data cannot briefly publish the wrong language's display/glyph combination.

## Requested protected index state

- `pending.md`: add or replace one concise module row for `zircon_editor/src/core/i18n/**`,
  `static_complete / dynamic_pending`, 7/7 files, 1,085 lines, 10 tests and the current-review link.
- `review.md`: do not add the module. Managed Cargo, allocation/lock/scale counters, product F4 WPR,
  same-machine CPU/RSS/package-power and locale-change RenderDoc glyph/render parity are absent.

## Milestone and notification state

This is a static review/routing record, not an accepted performance milestone. No git commit or WeCom
notification is due. Commit and quantified WeCom notification occur only after current-source dynamic
evidence closes the acceptance matrix and the protected indexes are reconciled by their owner.

