---
title: Editor command palette shared-query-window performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{command_palette/**,selection_options/**}
priority: MVP-P0 editor command search and keyboard navigation
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate SListView visible-row generation and reuse
---

# Goal

Carry the immutable command catalog and bounded query-window handles directly into one retained
command-row generation. Query, MRU, focus and selection changes must patch or replace only the bounded
window; host conversion must not serialize those rows to generic values and parse them once or twice
again to produce parallel legacy and structured options.

## Reviewed source

- owner files: command attributes/IDs/parser/entry/index/options/tests and generic selection option,
  search, tree and structured projection
- Rust files: 15/15
- current pre-M1 lines: 616
- current pre-M1 bytes: 20,147
- joined pre-M1 source-bytes SHA256:
  `37ee9f1a870faf0347ab5eae312f41a9daf1d137f6fc6222d9149581d402e37b`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `selection_options/mod.rs` | 57 | 2,047 | `71a2c5158439851c1821aaaa46dc73c1bd168212078a5a06116d605271ff387d` |
| `selection_options/model.rs` | 12 | 515 | `2c31aa8d7d4964744c5dd98252a6d2d4639ac74e3649e315d5bf9a97225c04cb` |
| `selection_options/options.rs` | 9 | 298 | `71c4360aefc291baac04256c612e473e988682b3ff1b48ccd255bc01943ba301` |
| `selection_options/search.rs` | 10 | 295 | `04af4940a86d0a9a5dd276efcc6a0c28e8e9008dd43c2f1a5eeae2523fe52142` |
| `selection_options/selection.rs` | 23 | 731 | `6a781c26ec9abf8db47f07d352a15390a9a1e9f1b01f06464032c16226c38805` |
| `selection_options/structured.rs` | 11 | 380 | `77517e825b65058e85c61494c1f8c5f71fdf0390e86d317ff3bcd8495c71ae1a` |
| `selection_options/tree.rs` | 24 | 666 | `3d9999f5b4d72b8a350d233a80b1963cee0ee902298ec21cec81650d870ece16` |
| `command_palette/mod.rs` | 14 | 288 | `442a31ca297a58ec5878829340dfd1f7187a432c84bf63383e44b122a988f745` |
| `command_palette/attributes.rs` | 52 | 1,400 | `8c7e7094865e0ed3297803e7d467b3eef86eabf6a1602325491fd48048deef3c` |
| `command_palette/entries.rs` | 37 | 1,175 | `4b8a77275129dced4130a78c9710a56324a68dee755a3a670775cc9481613032` |
| `command_palette/entry.rs` | 41 | 1,184 | `b85ce7bdfa7b0881828336f7226d428f509387656bbe923d7645d321ca56ac1d` |
| `command_palette/ids.rs` | 30 | 859 | `0974a721464e63c42622b73d0fd332f10d73be59237ede3b660ea2bd603cd24a` |
| `command_palette/options.rs` | 68 | 2,634 | `e0897b1e9017420f4685d9afb7e51b00a41a4b956fd0bbc41b6722c91b26d527` |
| `command_palette/parse.rs` | 69 | 2,317 | `64b8c2809d8fa8bb10237de683ab99f0a593989560d297de8f412b8f143dcd41` |
| `command_palette/tests.rs` | 159 | 5,358 | `306ec641e655eb84bda98e430648c8daccb6d8926a0a21d1511e6d13fb69b04e` |

All fifteen files were read in full. Production ownership was followed through the immutable command
catalog/query window, command-palette state bridge, workbench host projection and keyboard window
consumer. These related files are not counted in the 15/15 owner total.

## Existing foundations to retain

`EditorCommandPaletteCatalog` owns immutable entries, ID indices, search documents/postings and
enablement. Query uses postings, a bounded top-candidate heap and returns lightweight handles into an
`Arc` catalog generation. The UI requests only the visible row count plus overscan and publishes
catalog generation, total matches and offset. Selection option projection already calls one combined
specialized row projector instead of separately projecting command strings and structured rows.
These are correct foundations and should become the presentation authority.

## Structural findings

### P0: a bounded typed query window is serialized and parsed back into owned rows

The app converts the query window into a `UiValue::Array` of complete command tables and separately
builds another array of command ID strings. The template bridge stores both as generic control
properties. Final projection recursively parses commands back into owned `CommandProjectionEntry`
values, parses filtered IDs, builds an ID index and clones matching entries into filtered order.

The source window is already the exact ordered row generation. Publish it as a shared typed artifact
with query/catalog/MRU/offset receipts and let host rows reference catalog entries by handle. Remove
the full commands plus filtered-ID round trip.

### P0: one workbench host path parses the same command rows twice

`workbench_window_projection` calls `projected_command_palette_options` and then
`projected_command_palette_structured_options`. Both wrappers call the complete combined row
projector, so commands, filtered IDs, ID index, selection/recent/query state and structured rows are
rebuilt twice. The pane component path already uses the combined function once, proving the safe M1
shape. Expose that combined result to workbench projection and consume it once.

### P1: one projection retains three label representations

After entries are parsed, the projector clones every label into legacy `options`, moves another label
into each structured row and joins all labels into `options_text`. The wide host ABI therefore keeps
legacy string rows, structured rows and a comma-separated aggregate. The command palette paint/input
path should consume typed structured rows only; accessibility and diagnostics should derive from the
same shared labels without aggregate reconstruction.

### P1: transient selection state rebuilds the complete bounded window

Focused index, selected ID, recent set and query are interpreted while constructing every structured
row. W is currently bounded (default 12 plus overscan), so this is not a catalog-scale algorithm
failure, but focus-only movement still reparses and recreates all W entries. Store row identity and
catalog data once; focus/selection/MRU changes patch flags by row ID.

### P2: ASCII query matching is allocation-free and bounded

Fallback query matching uses byte windows and ASCII-insensitive comparison without lowercase
allocation. Preserve it for compatibility rows. The primary producer already performs ranked fuzzy
matching and marks filtered rows; do not add a second fuzzy matcher in presentation.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SSearchBox.h`

Unreal keeps search input as a dedicated widget and its list view maps stable data items to retained
row widgets. `SListView` begins at the scroll offset, generates until the visible area is filled,
reuses an existing widget and releases unseen rows (`SListView.h:978-1067`, `1524-1690`). List refresh
is coalesced by a pending invalidation flag (`STableViewBase.cpp:1393-1406`).

The transferable invariant is a typed query/list model plus retained visible rows. It is not encoding
the same result window into two generic arrays and reparsing it in presentation.

## Target architecture

1. Publish `CommandPaletteRowGeneration` containing an `Arc<EditorCommandPaletteCatalog>`, ranked
   handles, offset/total and query/catalog/MRU receipts.
2. Make bridge, workbench and pane projection share that artifact. Remove generic full-command and
   filtered-ID property arrays after compatibility consumers migrate.
3. Retain typed row identity/label/description/enablement. Focus, selection, recent and hover are
   narrow row-state patches keyed by command ID.
4. Keep one visible window shared by paint, hit, keyboard, accessibility and profiling.
5. Remove legacy options and comma-joined text from the command palette host ABI; share labels with
   accessibility/diagnostics instead.
6. Preserve the catalog query algorithm and bounded window. Do not introduce a presenter-side search
   index or private worker pool.

Complexity targets:

- stable query window projection: O(1), zero parse/index/row construction;
- query/window change: source query cost plus O(W) row handles, no generic serialization;
- focus/selection/MRU change: O(1) lookup plus changed row flags;
- paint/hit/keyboard/accessibility: O(W), one window authority;
- duplicate label/option representations: zero.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| query window handles and producer metrics | preserve current bounded behavior |
| UiValue/TOML command/ID bytes encoded and parsed | target = 0 |
| presenter parse/index/entry/label/join counts | stable = 0; target compatibility = 0 |
| row flags rebuilt vs patched | focus/selection = changed rows |
| visible visits by paint/hit/keyboard/a11y | one W-sized window |
| key-to-paint and event-to-paint latency | report median/p95/max |

Matrix: catalog entries 0/1/1,000/10,000/100,000; query bytes 0/1/32/256; result windows 0/1/12/64;
offset first/middle/end; duplicate IDs, missing compatibility IDs, MRU 0/32; stable projections and
query/focus/selection/MRU/catalog changes. Capture query metrics, encoded/parsed bytes, indices,
rows/labels/joins, allocations, CPU, latency, RSS and package energy on one source/executable
fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source pixel/draw parity; it cannot validate query-window ownership or parse cost.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add encode/parse/index/row/label/join and row-patch counters; capture baseline. | catalog/query/window scale evidence |
| M1 | Make workbench host projection consume the existing combined command row result once. | focused RED-to-GREEN contract and parity |
| M2 | Publish shared typed command row generations from catalog handles. | generic command/ID round trip = 0 |
| M3 | Patch focus/selection/MRU/hover by row identity and remove legacy labels/join. | changed row flags only |
| M4 | Share one window across paint, hit, keyboard, accessibility and profiling. | one row authority |
| M5 | Run managed scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## M1 implementation result

The workbench projection now consumes `projected_command_palette_option_rows` once and shares its
legacy labels and structured rows. The two single-result wrappers remain private to the command
palette module for the existing focused behavior tests and are no longer exported to production
workbench consumers.

Per command-palette mount on the workbench path, structural calls change as follows:

| Work | Before | After | Change |
| --- | ---: | ---: | ---: |
| combined command-row projection | 2 | 1 | -50% |
| command entry parse/materialization | 2 | 1 | -50% |
| recent-command ID set construction | 2 | 1 | -50% |
| selected/focused/query metadata reads | 2 sets | 1 set | -50% |

This is a bounded-window M1 reduction, not the target architecture. Generic command tables and ID
arrays are still serialized by the app and parsed by the retained projection; M2 owns removal of
that round trip.

| M1 file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `pane_data_conversion/pane_component_projection/mod.rs` | 51 | 1,570 | `c5f266cfdd0fb6b05e13ef81b740e8e4baa906ec56d902ce5cb40090bb73b021` |
| `pane_data_conversion/mod.rs` | 75 | 3,289 | `ac9f63b40590397581b0212618e3ea135cf618dc292c55faba9cb47825336ca8` |
| `workbench_window_projection.rs` | 569 | 26,154 | `da0d06ec44dfb1e17635b0fff3f1b43e42f317ecbc2d9f0fd441b465bb522c0a` |
| `tools/tests/test_editor_command_palette_projection_performance_contract.py` | 31 | 1,176 | `8a611e5d49ac164d9d3a7fff571d06f196a8e8b28208e9dcfc49879df041c0db` |

## Validation state

- Full owner source review: passed, 15/15 Rust files.
- Catalog/query window, state bridge, workbench host projection, keyboard consumer and Unreal
  references: read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change and after `rustfmt`.
- Current owned performance-contract set: GREEN 35/35.
- `git diff --check` for the M1 source and contract: passed.
- Managed Rust behavior tests and M0 plus M2-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; the focused command
  was rejected before Cargo launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
