---
title: Editor notification-center retained row-generation performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/**
priority: MVP-P0 editor activity and Play decision presentation
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate notification mutation and typed SListView item sources
---

# Goal

Make one changed notification generation produce one typed, shared retained row generation. Stable
frames must reuse that generation, and changed frames must not encode typed Decision/Toast/Progress
rows into pipe strings, copy them through generic values, parse them into owned entries, then retain
parallel label, structured-row and joined-text representations.

This report complements the 2026-08-15 core notification review. That report reviewed 25/25 files in
`core/notifications/**`; it did not review the eight current retained row-projection owners below.

## Reviewed source

- Rust files: 8/8
- lines: 659
- bytes: 22,670
- joined raw source-bytes SHA256:
  `079c4cb9945f567fea722112e3bcb6fc9fffa06678ed8e1465b3c108a72b1062`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `attributes.rs` | 68 | 1,887 | `27db6db14ab1f2613f6eef5859afdaab47af3ff39bc51aa637108c745f36f8b8` |
| `entries.rs` | 23 | 716 | `e37f3a20b385ed28b827bbeaa52311da2a5796c9d3f131568d1cec59ce333a79` |
| `entry.rs` | 26 | 667 | `2430794234e3843ad2beb6bf25bde9e0ac3a654aa9e4918c75b2d2d3faa5267f` |
| `metadata.rs` | 93 | 3,361 | `43948102b5543f835436f17c155b7207bf041bc14ea199dd828f25b4c76a44cf` |
| `mod.rs` | 18 | 535 | `8961f24da41202a7c05fe48cc50353940297124639d19a11f0efe1245808e9fe` |
| `options.rs` | 76 | 2,823 | `96ab724b73869a68e8fb69a50067eb8fe8a9a6e8873dd9cc6e9620e60525b06c` |
| `parse.rs` | 123 | 3,799 | `03ac4fc7bc9983d7f6942d1b4ebe8e27220f641dd419a1ed2baf09eda5402d9b` |
| `tests.rs` | 232 | 8,882 | `0cb1cba29b986b53c0af6068016964cfb995f7af0279b0964131fa726142840b` |

Supporting callers read in full or through the exact notification path:

- `ui/workbench_window_projection/{notification_cache,host_value_toml}.rs`
- `ui/workbench_window_projection.rs`
- `ui/pane_data_conversion/pane_component_projection/selection_options/mod.rs`
- `callback_dispatch/template_bridge/workbench/notifications.rs`
- `app/workbench_notifications.rs`

## Correct foundations to retain

1. `visible_limit` reaches `notification_entry_list_with_limit`; recursive array traversal stops as
   soon as the visible bound is reached. The existing test proves a 64-row source parses only two
   rows at limit two.
2. Workbench projection compares generation, unread/overflow, selected/focused and visible-limit
   receipts. A hit reuses `Rc<String>` and `ModelRc` labels/rows and excludes the large
   `notifications` property from host-value-to-TOML conversion.
3. Specialized notification options are projected once through the combined option-row function.
   Pane and workbench consumers do not independently invoke legacy and structured wrappers.
4. Empty and zero-generation inputs remain observable. Generation zero deliberately disables row
   reuse, so an unversioned producer cannot preserve stale notification rows.

## Structural findings

### P0: changed typed rows still make a generic string/TOML round trip

The bridge formats Decision, Progress and Toast data into up to 64 pipe-delimited strings. Retained
host properties copy those strings into `toml::Value`, the parser splits them back into owned entry
strings, and option projection constructs host rows. Generation caching avoids this work on a
stable projection hit, but it cannot remove the work already performed by the upstream stable-tick
snapshot/localization/encoding path documented on 2026-08-15.

For visible changed rows `W <= 64`, the retained stage is O(total encoded bytes + W). The algorithmic
bound is acceptable; the ownership path is not. A typed source generation should cross the bridge
without a presentation codec.

### P0: stable-frame work is owned upstream, not by this parser

`sync_activity_notifications` snapshots and localizes all three authorities before the bridge knows
whether anything changed. This module's generation cache is useful but too late to suppress those
locks, snapshots, sorting, localization and history formatting. The unified source-generation work
must remain routed to Editor17/Editor14/Editor04 and EditorUI08; duplicating another cache here would
hide rather than solve the architecture fault.

### P1: changed pipe rows perform avoidable per-row allocations

`NotificationProjectionEntry::new` clones the ID into the default title before parsing. All three
production history encoders write `title=...`, so that ID clone is immediately overwritten and
dropped for normal production rows. Hand-authored rows can omit a title, so the safe M1 is a lazy
fallback after parsing, with an explicit-title flag preserving the current empty-title behavior.

`normalized_tone` allocates a lowercase temporary string and then allocates the canonical tone
string. Case-insensitive comparisons can return one static canonical name and allocate only the
owned final DTO field. For `W` production rows, M1 removes `W` redundant default-title clones and up
to `W` lowercase temporaries. Table rows additionally avoid an owned intermediate tone lookup.

### P1: one visible generation retains three title representations

Every changed row clones its title into legacy `options`, moves another title into structured rows,
and the selection owner joins all labels into `options_text`. Generation hits share the completed
models, but the initial changed-generation build still retains all three. M2 must identify remaining
compatibility consumers and hard-cut legacy label/join owners only after typed row consumers migrate.

### P2: metadata still owns a selected-ID cache key

Workbench reads `selected_notification_id` into a new `String` before checking the generation cache.
This is one bounded allocation rather than a row-scale bottleneck. The typed generation receipt
should carry a shared/stable selected identity; a new ad hoc cache-key lifetime is not justified for
M1 without behavior measurements.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/NotificationManager.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/SlateAsyncTaskNotificationImpl.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`

Unreal requires direct notification creation on the game thread and queues foreign-thread
notifications for its manager tick. Progress uses direct start/update/cancel operations. Async task
notifications capture changed text for one deferred game-thread update, while state tick consumes an
optional pending state and updates the widget only when the state changes. `SListView` owns a typed
item source, generates only required widgets, and first tries to reuse a widget for an item.

The transferable invariant is changed typed state applied once on the UI thread and visible row
reuse. Zircon should preserve its stronger typed notification authorities and generation receipts;
it should not imitate Unreal's window implementation or invent Unreal numeric budgets.

## Target architecture

1. Editor17 publishes one immutable `ActivityNotificationProjection` with typed shared rows,
   Decision/Toast/Progress/locale revisions, selected identity, unread/overflow and next expiry.
2. EditorUI08 reads a compact revision before source snapshots and applies a changed generation at
   most once. Tick and dispatch dirtiness coalesce rather than synchronously rebuilding the same
   frame.
3. The retained bridge carries shared typed row handles. Generic pipe strings/TOML remain only at an
   explicitly measured compatibility boundary and are deleted after consumers migrate.
4. One retained row generation owns paint, hit testing, keyboard, accessibility and profiling.
   Focus/selection changes patch stable row identities rather than rebuilding all visible row DTOs.
5. Preserve visible-limit early stop and generation-zero stale-row protection throughout cutover.

## Instrumentation and acceptance

| Evidence | Required measurement |
| --- | --- |
| source and unified revisions | source changes, projection builds, retained applies and stale rejects |
| stable work | authority locks, snapshots, localized/formatted/parsed rows and bytes per frame |
| changed retained work | rows visited/materialized, default-title clones, tone temporaries, title copies/joins and allocations |
| interaction | selection/focus row patches, input-to-present p50/p95/p99 and accessibility parity |
| process | CPU stacks, allocation/RSS, contention, context switches and package power on one executable fingerprint |

Matrix: Decision 0/1/128, adapter 0/1/256, Toast 0/1/128, Progress 0/1/64; visible limits
0/1/16/64; 64 B/2 KiB/256 KiB encoded rows; stable, one-row change, publish/resolve/cancel, expiry,
progress update and locale change at 30/60/120 Hz with 1/16 producers.

Acceptance:

- after initial apply and before expiry, source projection/encoding/parsing work is zero;
- one accepted source revision builds and applies at most one unified generation;
- changed retained work is O(W + changed encoded bytes) during compatibility and O(W) after cutover;
- M1 redundant title clones and lowercase temporaries are zero without changing empty-title fallback;
- empty generation clears stale UI once; order, IDs, unread/tone/disabled, focus and selection match;
- WPR/power and optional RenderDoc draw/pixel parity use the same current-source executable and D/E/F
  artifacts only.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add source/revision/build/apply/row/byte/allocation counters and capture baseline. | complete matrix baseline |
| M1 | Make title fallback lazy and tone normalization allocation-free before final ownership. | focused RED-to-GREEN contract and Rust parity |
| M2 | Publish and consume one shared typed notification row generation. | pipe/TOML retained round trip = 0 |
| M3 | Coalesce tick/dispatch dirtiness and patch selected/focused rows by stable ID. | stable projection work = 0; apply <= 1/change |
| M4 | Remove legacy options/join owners and share rows across all consumers. | one row authority |
| M5 | Run managed scale, interaction, WPR/power and RenderDoc parity matrix. | quantified acceptance |

## M1 implementation result

M1 keeps parsing and row ownership unchanged while removing transient work before final ownership:

- pipe rows start with an empty title and clone the ID only when no title alias was present;
- an explicitly present empty title remains empty rather than falling back to the ID;
- tone aliases use ASCII-case-insensitive comparison and return a static canonical name;
- table tone lookup remains borrowed until the final canonical tone `String` is created.

For the production bridge, every encoded Decision/Progress/Toast history row contains `title=` and
`severity=`. For `W` visible changed pipe rows, structural transient work therefore changes by
`W` default-ID title clones -> 0 and `W` lowercase tone temporaries -> 0. A table row with a tone
also removes one owned source-tone intermediate plus the lowercase temporary. The required final
owned title/tone fields remain; M2 owns their replacement with shared typed row data.

Post-M1 scope:

- Rust files: 8/8
- lines: 702
- bytes: 23,972
- joined raw source-bytes SHA256:
  `a41cb033ff064d1e9a38affcdd1a86e5315d213125ed4cb5a36203e68212cd3a`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `attributes.rs` | 80 | 2,247 | `3e6a0f62fa5d06969acc613f456021cae8bf7f34e7a0f554aeec86b705d80f78` |
| `entries.rs` | 23 | 716 | `e37f3a20b385ed28b827bbeaa52311da2a5796c9d3f131568d1cec59ce333a79` |
| `entry.rs` | 26 | 670 | `3a99a282d7f7608d515414759c4d68c407225afb8a31e927ce4ba63e65e8096f` |
| `metadata.rs` | 93 | 3,361 | `43948102b5543f835436f17c155b7207bf041bc14ea199dd828f25b4c76a44cf` |
| `mod.rs` | 18 | 535 | `8961f24da41202a7c05fe48cc50353940297124639d19a11f0efe1245808e9fe` |
| `options.rs` | 76 | 2,823 | `96ab724b73869a68e8fb69a50067eb8fe8a9a6e8873dd9cc6e9620e60525b06c` |
| `parse.rs` | 135 | 4,057 | `ad107d301824ab6a97127cf1646a161b96163017eb0933152aaae924af80f811` |
| `tests.rs` | 251 | 9,563 | `d4f6cfa4d838dfee857f9ce818820102142833631ad3bae66c749c5351b0b6cd` |

Focused contract: `tools/tests/test_editor_notification_center_row_allocation_contract.py`, 44
lines, 1,744 bytes, SHA256
`3bd73f6d6517c0044f6dd0f83b5eea3b2aa9ad89415fec65688009879a8cb93d`.

## Validation state

- Full owner source review: passed, 8/8 Rust files.
- Supporting workbench cache, host-value conversion, bridge, app sync and Unreal sources: read.
- M1 focused contract: RED 3/3 before the change, GREEN 3/3 after the change and after `rustfmt`.
- Current owned performance-contract set: GREEN 38/38.
- `rustfmt --check` for 8/8 owner files and scoped `git diff --check`: passed.
- A Rust regression covers missing versus explicitly empty titles and uppercase tone aliases; it is
  present but not claimed passing until managed Cargo is executable.
- Managed Rust behavior tests and M0 plus M2-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; the focused command
  was rejected before Cargo launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
