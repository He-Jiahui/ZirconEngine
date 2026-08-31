---
title: Editor workbench context menu provider generation performance review
date: 2026-08-23
module: zircon_editor retained-host workbench_context_menu
priority: MVP-P1 editor secondary-input and plugin menu projection
status: source_reviewed_m0_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine ToolMenus registered hierarchy and context-bound on-demand generation
---

# Goal

Keep secondary-click classification bounded while replacing string-heuristic provider ownership and
encoded menu rows with a typed, pre-registered context-menu projection suitable for editor plugins.
Opening one menu may materialize that menu, but must not discover plugins, rescan the UI tree, perform
repeated independent property mutations, or rebuild unrelated presentation state on the input path.

## Reviewed source

- owner Rust files: 6/6
- lines: 227
- bytes: 7,814
- source-only SHA256 over lexicographically sorted owner files:
  `99a42d29cd907771b01ec5b4a98eaf98945aa930ec41757e62f93bc96cb7f3de`
- post-M0 owner files/lines/bytes/SHA256: 6 / 248 / 8,549 /
  `bbd88e7c395db5b9d926a73c6acb13b40c831864628fb07da031eca3536bb512`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `workbench_context_menu.rs` | 1/1 | 10 | 224 |
| `workbench_context_menu_tests.rs` | 1/1 | 60 | 2,192 |
| `workbench_context_menu/**` | 4/4 | 157 | 5,398 |

All owner files were read in full. Secondary-button dispatch, request data, host callback, workbench
template bridge open/close projection and UI tests were inspected as direct consumers. Unreal
ToolMenus generation and Content Browser context-menu sources were read directly. The 2026-07-17
combined record covered this owner at 203 lines; this record supersedes that stale coverage.

## Correct foundations to retain

1. Workbench secondary dispatch starts from an already resolved `TemplateNodePointerHit`; this owner
   does not rescan presentation or synchronously discover plugins.
2. Popup option/menu rows are rejected before provider construction, preventing nested context menus.
3. Provider classification is a fixed number of prefix/identity checks and menu construction is
   on-demand, appropriate for a low-frequency secondary-click path.
4. The request carries exact pointer anchor and bounded hit-frame damage; tests preserve scene target
   URI and nested-popup rejection.

## Structural findings

### P0: provider authority is inferred from presentation strings

Scene, module and generic providers are selected from control/action string prefixes. This couples
input behavior to naming conventions and cannot express typed plugin contributions, ordering,
permissions, selection cardinality or owner lifetime. Adding more modules will grow a central branch
chain and repeated prefix work, while collisions can silently route to the wrong provider.

M1 adds a startup/plugin-lifecycle `WorkbenchContextMenuRegistry` keyed by typed hit target kind and
stable owner id. The input path selects pre-registered provider descriptors from the resolved hit;
it never loads modules or discovers extensions at click time.

### P0: bridge opening performs fragmented state mutation and tree discovery

The direct consumer `callback_dispatch/template_bridge/workbench/context_menu.rs` converts every menu
string into owned `UiValue::String`, performs roughly eleven independent control-property mutations,
linearly searches surface nodes for the context-menu control, allocates a descendant-id vector,
performs four boolean mutations for every descendant, then refreshes the surface. Close repeats the
lookup/traversal pattern. This dominates the small provider's CPU and allocation cost.

That consumer is outside this six-file owner and is already routed to the popup/binding generation
plan. M2 requires one typed context-menu state patch, a retained control/subtree handle and one scoped
generation refresh. No per-property public mutation loop or per-open tree lookup survives.

### P1: menu rows are encoded strings rather than typed actions

Menu labels, separators, icons and danger flags are packed into strings such as
`Delete|danger,icon=trash`, then later split by paint/projection consumers. Every request allocates a
new vector and shared strings. The current three tiny static menus make this low frequency, but the
format is the wrong plugin contract and prevents prevalidated command identity, enable predicates and
lazy sections.

M1 registers immutable typed menu/section/action descriptors and materializes only the active menu's
resolved rows. Dynamic enable/visibility functions receive a compact context snapshot and have
per-provider time/allocation budgets.

### P1: target URI construction uses an avoidable intermediate allocation

`path_segment` collects a normalized `String`, trims it and copies it again with `to_string`; the
provider then formats a second final URI. M0 now writes the normalized segment directly into one
capacity-reserved final URI buffer and resolves target value once, preserving leading/trailing
hyphen, ASCII case, whitespace and punctuation semantics. This is a local deterministic improvement,
not the structural acceptance.

### P1: no provider or bridge cost evidence exists

There are no counters for classification probes, provider lookup, contributed sections/actions,
descriptor materialization, URI/menu allocations, property mutations, surface lookup/subtree visits,
refresh reason or end-to-end open latency. M1/M2 add these before plugin-scale acceptance.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenus.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/MenuStack.cpp`

Unreal's `UToolMenus::GenerateWidget` takes a registered menu name and `FToolMenuContext`, generates
the named menu/hierarchy, then builds the widget (`ToolMenus.cpp:2888-2908`). Submenus inherit the
generated parent's context and execute a registered construction delegate only for that submenu
(`ToolMenus.cpp:1485-1522`). Content Browser list/tile/column views bind the same retained
`OnGetContextMenuContent` provider; opening validates state, snapshots selected items and invokes the
bound item-context-menu delegate (`SAssetView.cpp:3046-3078, 3178-3188, 5727-5738`). The resulting
menu is then owned by Slate's retained menu stack.

The transferable rule is registered typed menu ownership plus a compact invocation context and
on-demand active-menu generation. It is not a mandate to copy UObject/Slate widget machinery.

## Target architecture

1. Plugins register immutable typed menu descriptors at load/unload time under stable menu/section/
   owner ids; the registry publishes an immutable generation for lock-free reads.
2. `TemplateNodePointerHit` carries a typed context-target kind/provider key instead of requiring
   control/action prefix inference.
3. Secondary click snapshots only target identity, selection context, anchor and registry generation;
   provider predicates are bounded and instrumented.
4. One typed `WorkbenchContextMenuStatePatch` carries resolved rows and target metadata into the
   retained template surface.
5. The bridge retains the context-menu control/subtree handle and applies one scoped patch/refresh.
6. Popup/menu-stack ownership, keyboard, dismiss, hit and paint share the resulting generation.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| provider lookup/probes | O(1) typed lookup; no control/action prefix chain after cutover |
| registered owners/sections/actions | unload removes one owner without global rediscovery |
| predicate CPU/allocations | budgeted per active provider; no plugin load/discovery on click |
| URI/menu descriptor allocations | M0 one URI allocation; M1 only active dynamic rows materialize |
| property mutations/control lookup/subtree visits | one scoped patch; zero per-open tree lookup after M2 |
| refresh reason/changed nodes | context-menu subtree only |
| open CPU p50/p95/p99 | slope independent of unrelated UI nodes and inactive plugins |
| correctness | scene/module/generic/plugin, popup rejection, anchor, actions and visuals match |

Matrix: UI nodes `1/100/1K/10K`; registered plugins `0/1/10/100`; contributions `1/10/100/1K`;
selected targets `0/1/100/1K`; provider `scene/module/generic/plugin`; URI `ASCII/whitespace/punctuation/
empty`; update `open/close/reopen/plugin-unload`; scale `1x/1.5x/2x/4K`.

WPR owns CPU, allocation, context-switch and power evidence. RenderDoc is used only after a current-
source GPU presenter is launchable and only for popup draw/resource/pixel parity after the bridge
cutover. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Write normalized target URI directly into one final buffer. | applied; static contract GREEN, managed Rust/dynamic pending |
| M1 | Add typed immutable provider/action registry and resolved hit provider key with telemetry. | no string-prefix authority or click-time plugin discovery |
| M2 | Apply one typed context-menu surface patch using retained control/subtree handles. | no per-open tree scan or fragmented mutation loop |
| M3 | Converge popup generation and run plugin/UI/WPR/power/GPU parity matrices. | quantified acceptance and milestone closeout |

## Validation state

- Owner source review: passed, 6/6 current Rust files.
- Secondary dispatch, request, host callback, template bridge and UI test consumers: read and mapped.
- Unreal ToolMenus, Content Browser provider and retained menu stack sources: read and mapped.
- M0 static performance contract moved RED 0/2 to GREEN 2/2. Together with keyboard, dismiss,
  popup-binding, hit-index and presentation-generation contracts, the focused set passes 16/16.
- A Rust regression records normalization equivalence for boundary hyphens, whitespace, ignored
  punctuation and ASCII case; execution remains pending with managed Rust validation.
- The changed Rust files pass independent `rustfmt --check`; scoped `git diff --check` passes with
  line-ending warnings only.
- Managed Rust tests, current-source launch, WPR and RenderDoc remain pending because the managed
  Cargo Session is terminal `archived` with `cargo_session_not_executable`. No raw Cargo bypass is
  allowed.
- M0 dynamic acceptance and M1-M3 remain pending; this owner stays out of `review.md` until dynamic
  acceptance.
