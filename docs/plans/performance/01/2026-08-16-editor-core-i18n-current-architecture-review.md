# Editor core i18n current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-16.
- MVP priority: P0 only where locale changes feed the stable-tick notification projection; P1 for
  text-revision ownership and removal of the unused message-bus compatibility path. Catalog lookup
  itself is not a demonstrated P0 bottleneck at the current two-locale/54-key scale.
- Accounting: keep `zircon_editor/src/core/i18n/**` in `pending.md`. Do not add it to `review.md`
  before managed Cargo, allocation/lock counters and product F4 WPR acceptance pass.
- Code disposition: no Rust source changed. `mod.rs` and `tests.rs` contain pre-existing formatting
  changes; their bytes and owner were preserved. The session coordinator authorizes writes only under
  `docs/plans/performance`.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/i18n/**` | 7/7 | 1,085 | 10 | `1df6a8f93dd714466fc22d9aea91c0824b6c12aaa82f75326a31d86165252382` |

The fingerprint is SHA256 over normalized sorted path, NUL, raw file bytes, NUL. Every Rust file in
the folder was read in full. Production construction, settings hot apply, editor-message publication,
notification presentation and all non-test translation callers were traced. The embedded English and
Simplified Chinese bundles are 4,093/4,051 UTF-8 bytes and contain 54 translation keys each.

## Per-file acceptance record

| file | lines | verdict |
|---|---:|---|
| `catalog.rs` | 130 | Bundles parse once at construction and values are shared `Arc<str>`; stable lookup is bounded. English fallback constructs a fresh `EditorLocale` allocation per fallback query and an unknown key allocates its raw-key `Arc`; these are measurable changed-projection costs, not a reason for a second cache. |
| `error.rs` | 19 | Typed construction/locale errors only; no hot loop or hidden I/O. |
| `locale.rs` | 52 | Canonicalization is construction-time. `EditorLocale::english()` allocates a new `Arc<str>` on every call, including the catalog fallback path. |
| `macros.rs` | 6 | Thin `tr!` forwarding macro; it does not hide iteration, I/O or dispatch. |
| `mod.rs` | 18 | Ownership/export boundary is narrow. Existing formatting-only change preserved. |
| `service.rs` | 429 | Settings generations reject stale/no-op transitions; event backlog is hard-bounded and coalesces to a resync. A changed locale can synchronously drain the sink, and the service has no independent text revision for change-proportional consumers. |
| `tests.rs` | 431 | Ten tests cover parsing/fallback, settings generation ordering, FIFO concurrency, bounded slow-sink behavior and resync recovery. They do not measure allocations, lock wait/hold, translation scale or product-frame impact. Existing formatting changes preserved. |

## Architecture verdict

The current module has a sound MVP authority boundary and should not be replaced with an Unreal-sized
localization subsystem:

- two embedded TOML bundles are parsed once by `EditorI18nCatalog::embedded`, not on translation or
  every frame (`catalog.rs:8-55`);
- the active locale has one catalog-owned `RwLock`, and callers can capture a locale before resolving
  a compound row so one row cannot mix languages (`catalog.rs:58-95`);
- settings remain the sole preference authority. `SettingsSnapshot::generation` rejects stale and
  duplicate hot-apply transitions (`service.rs:133-189`);
- event delivery releases queue locks before calling foreign sinks, is capped at 32 events/64 logical
  locale bytes and coalesces overload into one latest-locale resync (`service.rs:226-305`);
- only 54 keys per locale exist today. Changing the stable catalog map solely because Unreal serves a
  much larger live table would be ungrounded. Measure lookup and allocation before selecting a hash,
  perfect-hash or generated-slot representation.

The structural gap is revision ownership. A locale value and a settings generation are not the same
contract as a monotonic text/display revision. Consumers currently either call translation whenever
they rebuild or listen to a generic serialized event. A retained consumer cannot cheaply prove that
its localized projection is current without rebuilding it.

## Current performance behavior

### Stable translation

`translate` takes the locale read lock, clones one small `Arc`, then performs active-locale and English
fallback `BTreeMap` lookups. Found display strings are shared through `Arc<str>` and their text bytes
are not copied (`catalog.rs:78-95`). With 54 keys and current production callers, this is not evidence
of a fatal algorithmic bottleneck.

The method still has two precise transient-allocation cases:

1. the fallback branch calls `EditorLocale::english()`, whose implementation is
   `Arc::from("en")`; a missing key in the active non-English bundle therefore allocates a locale
   owner before probing English (`catalog.rs:89-93`; `locale.rs:39-40`);
2. a key missing from both bundles returns `Arc::from(key)`, allocating on every query
   (`catalog.rs:94-95`).

These should be counted and removed within the revision-owned projection. Do not add an unbounded
missing-key cache. A retained English handle or direct English-bundle owner removes the first case;
unknown keys need bounded diagnostics/interning or changed-generation materialization only.

### Locale transition and dispatch

`apply_locale` holds the transition mutex while it validates settings generation, takes the catalog
locale write lock, clones the configured sink and enqueues the event. It releases the transition
mutex before dispatch, but the first caller synchronously drains the complete pending/resync sequence
and invokes the sink (`service.rs:153-189,268-305`). Locale changes are explicit and rare, so adding a
private worker would increase ownership and shutdown complexity without current evidence. Measure
sink wall time and move only genuinely blocking work to the existing bounded scheduling authority.

The configured production sink serializes a custom editor message for `EditorTopic::i18n`. Repository
search found production publication but no production subscriber; all registrations are builder
tests. Every accepted locale change therefore still builds JSON and executes bus fanout for a
compatibility path with no product consumer. The retained notification path reads `EditorI18nService`
directly and does not consume this message.

### Downstream amplification

Current non-test presentation calls are concentrated in
`core/notifications/presentation.rs:162-234`. Decision, toast and progress rows capture the locale and
resolve owned localized projections. The retained host currently invokes those functions during every
activity-notification synchronization, even on stable frames. Thus the P0 cost belongs to the unified
notification projection diagnosed by `PERF-MVP-596`, not to a per-lookup catalog rewrite.

The notification review already requires one immutable projection keyed by Decision, Toast, Progress
and locale generations. The i18n service must own and expose the locale/text revision used in that key;
Editor17 must consume it rather than inventing a notification-private locale counter. Settings
`PERF-MVP-591` remains responsible for publishing the affected locale slot once per accepted settings
generation, not polling settings every retained tick.

## Unreal primary-source evidence

- Unreal's localization manager stores shared display-string references in a text-id lookup table
  (`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/TextLocalizationManager.h:50-72`).
  This supports Zircon's shared immutable string bodies; it does not require copying Unreal's table
  scale or global singleton design.
- Unreal exposes a global text revision specifically so cached information can detect mismatch and be
  recached (`TextLocalizationManager.h:270-285`). It also retains local text-id revisions. The
  transferable contract is revision-checked localized projections, not stable-frame retranslating.
- `DirtyTextRevision` increments a non-zero revision under a write lock, clears local revisions, then
  broadcasts on the game thread or schedules the broadcast there
  (`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/TextLocalizationManager.cpp:1795-1817`).
  Zircon should similarly publish one accepted revision and apply UI changes on its UI authority,
  while preserving its existing settings-generation ordering.
- Unreal's own source warns that “language changed” and “new localization data available” are
  different invalidation causes, because indiscriminate font-cache flushing can refill the old glyph
  set and briefly display the wrong forms (`TextLocalizationManager.cpp:53-63`). Zircon must therefore
  distinguish at least `LocaleChanged` from future `BundleRevisionChanged`; one generic bus event must
  not become a global clear-everything signal.
- Unreal supports asynchronous localization-data loading, but Zircon's current 8 KiB of embedded data
  is already parsed once during construction. Introducing asynchronous bundle I/O now would solve a
  scale Zircon does not yet have.

## Required target architecture

1. `EditorI18nService` remains the sole localization authority and publishes a non-zero monotonic
   `TextRevision` plus a typed cause (`LocaleChanged`, future `BundleRevisionChanged`). The revision
   increments exactly once per accepted visible-text change; stale/no-op settings snapshots do not
   increment it.
2. Revision state exposes an O(1) compact token. Consumers retain `(revision, typed shared rows)` and
   return `NotModified` on a stable token. Notification `PERF-MVP-596` consumes this token in its
   unified projection; it does not maintain a second locale generation.
3. Locale preference stays in the single settings authority. `PERF-MVP-591` sends one affected-slot
   delta to i18n after unlock. Stable retained frames perform no settings snapshot load, i18n lock,
   translation or projection work.
4. Localized display strings remain shared. Retain the canonical English fallback owner so fallback
   lookup does not allocate; materialize missing raw keys only on a changed revision and bound missing-
   key diagnostics by entries and bytes.
5. Remove the `EditorTopic::i18n` JSON/fanout path if no production consumer is identified. If a
   plugin/runtime consumer is required, publish the same typed revision/cause through one bounded
   dispatcher and serialize only at that external compatibility boundary.
6. Locale switching stays synchronous until measurements identify blocking bundle/font work. Future
   bundle loading uses the shared job/I/O scheduler with cancellation, byte bounds and a revision-
   checked UI commit; it must not create an i18n-private worker or publish text before required glyph/
   font state is ready.

## Measurement plan

| matrix | required counters | acceptance |
|---|---|---|
| locale count 2/10/100; keys 54/1k/100k; found active/fallback/missing | lookup p50/p95, comparisons, locale/key allocations, returned text bytes, RSS | no per-query locale allocation; found strings share bodies; missing-key storage has entry+byte bounds; select map/slot algorithm from measured crossover |
| stable 1/10k translation requests and 1/1M retained ticks | locale lock wait/hold, token reads, translations, rows localized, projection builds/applies, allocations | after initial apply, stable notification/settings/i18n projection work=0; token check is O(1) |
| accepted/no-op/stale locale changes; 1/16 producers; slow/rejected sink | settings/i18n revisions, queue entries/bytes/age, sink wall, resyncs, JSON bytes, UI applies, stale rejects | accepted visible change increments once and applies at most once; no-op/stale increment=0; queue bounds/order/recovery preserved; unused JSON path=0 |
| future bundle refresh and locale change overlap | revision cause/order, parse/job wall, glyph/font readiness, stale commit rejects | no mixed-locale projection; stale bundle results never publish; locale and bundle invalidation causes remain distinct |
| product F4 before/after at 30/60/120 Hz | WPR CPU stacks, contention, allocations/RSS, context switches/package power; notification render parity | i18n and notification stages are separately attributable; stable-frame translation disappears; same-machine CPU/RSS/power improves without interaction or glyph regression |

RenderDoc is not the tool for catalog lookup, lock, allocation or event-fanout claims. After a current
editor binary launches, use it only to confirm that a locale change causes the expected text/glyph
resource update and no extra notification overdraw. Use WPR/xperf for CPU, contention, allocation,
context-switch and package-power evidence.

## Static gates executed

- Read all 7 Rust files in full at the recorded fingerprint and traced all current non-test callers,
  the settings subscriber, message sink, notification projection and Unreal sources above.
- `rustfmt --edition 2021 --check` is green for all 7 files. `git diff --check` is green; the only
  output is Git's existing LF-to-CRLF warning for the two foreign-formatted files. No Rust source was
  formatted by this review.
- All 13 explicitly routed repository paths exist. The documentation convention gate reports 0
  violations owned by these two records. Its global baseline remains red at 671 existing violations
  across 241 of 2,507 documents; this review does not claim or modify that unrelated debt.
- `python -m tools.session_coordinator --repo-root . --json plan audit` and the session heartbeat are
  green. The source fingerprint was recomputed after documentation edits and remains unchanged.
- Managed Cargo cannot run while `tools/build-editor.ps1:130` rejects approved D:/E:/F: target roots
  through its literal separator bug. See
  `failure-2026-08-15-build-editor-approved-root-separator.md`.
- WPR/xperf and RenderDoc dynamic acceptance remain pending because no launchable current-source
  editor binary exists. No latency, power or rendering improvement is claimed.
