---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-arranged-index-and-stage-invalidation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/render/clipping.rs
  - zircon_runtime/src/ui/surface/render/agent_chat.rs
  - zircon_runtime/src/ui/surface/render/semantic_components/shared.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/arranged_authority.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/command/clip_tests.rs
  - zircon_runtime/src/ui/surface/interaction_gate.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/report.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/authored_geometry.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/patch_publication.rs
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetHeap.h
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - 10k-node arranged node-slot-ancestor probe counter
  - one-percent dirty stage visited-node test
  - stable-generation zero-arranged-hit-render rebuild test
  - cargo +1.94.1 test -p zircon_runtime_interface --lib --locked -- ui::surface::render::command::clip_tests:: --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked -- ui::tests::surface_frame_authority::arranged_authority:: --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --features profiling -- ui::tests::surface_dirty_domains::patch_publication:: --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --features profiling -- ui::tests::surface_dirty_domains::patch_publication::render_patch_records_the_owned_elapsed_counter --exact --ignored --nocapture --test-threads=1
---

# Runtime UI arranged无索引且dirty下游全量重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface core当前批21文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 联动责任：EditorUI01消费indexed focus/hit path；EditorUI08拥有published frame generation。
- 交接原因：arranged/layout/slot index与stage invalidation由EditorUI02定稿。

## 失败现象与复现证据

PERF-MVP-277/281：arranged build重复祖先与全slots走查，`UiArrangedTree::get`线性；incremental layout后仍全量arranged/hit/render。局部止损已把dirty flags/count合并为一次树扫描，但主要stage工作未变。

## 最低共享层根因

arranged artifact缺node/slot dense index与inherited effective-state cache，dirty transaction也没有携带changed nodes/ranges跨越layout→arranged→hit→render边界。

## 架构修复验收

- node id→dense index、parent/child/slot直接查询；一次DFS计算clip/visibility/input/disabled，draw order独立。
- changed ranges按layout boundary增量patch arranged/hit/render，stable generation所有stage visits=0。
- 1/100/1k/10k nodes记录node/slot/ancestor probes、visited/reused/rebuilt/damage和CPU p95；单叶/1% dirty不随N全扫。
- z/paint/canvas/clip/focus/hit/像素与serde/Cargo通过。

## 禁止临时方案

- 不得只给某个consumer加私有HashMap，留下arranged owner与其他consumer继续线性查找。
- 不得把全量stage移动到worker后无限积压；changed set、budget和frame publish必须闭环。

## 修复结果与回传

Open state: `等待EditorUI02回传generation-owned arranged index和跨stage changed-range证据`。

## 2026-09-27 changed-stage publication and reporting repair

Stable primary `failure-roll-01a084c8-editorui02-dirty-authority-r3` continues this
lifecycle separately from `ui-surface-dirty-full-tree-scans`. Its existing Session
baseline remains `6b4bc86089cb4464f850136c8079e90cb6513ebc` / epoch 611.
Audited transfer `e8a39d0ab62e4c52b345c6ea5d39b9a2`, fingerprint
`f62b09d9c0ac293c1e445c8bbc3faccc63fedb6c30abfe7121ea68d77552e2d6`,
preserves the current source from archived, cancelled or unattributed owners.
No live lease was displaced. Historical tickets remain with their original owners.

Current production inspection found a downstream consistency defect: successful
local arranged/render patches set only `*_patched`, while the frame publication
call sites passed only `*_rebuilt`. Publication consequently discarded render
patch ranges and reused stale arranged/render domains. Stage reports and elapsed
counters also treated these local operations as skipped. This was established by
the writer and direct consumer code; no current dynamic failure is claimed.

The existing public `*_rebuilt` fields describe stage updates, including local
updates. Existing production and test consumers already use that contract.
`*_patched` identifies the successful local route and distinguishes it from a
full rebuild. Incremental producers retain that stage-update meaning, and their
two publication calls consume shared arranged/render update predicates. Hit
publication retains its actual change/full-rebuild signal: a successful no-op
hit patch alone does not advance the hit domain. Pipeline/profiling execution
gates include successful patches; the complete hit-grid rebuild counter excludes
local routes. The authored-geometry producer also fills the patch-route fields.

UI08 continues to own frame publication. Its current and Gitbase interfaces
already accept domain-change flags plus exact render node IDs, so no UI08
production source is edited for this repair. Unreal's `SlateInvalidationRoot.cpp`
reports repaint work from its fast path as well as its slow path; Zircon keeps
that distinction between performed work and the selected update route.

The mounted `patch_publication` regression module covers actual opacity changes
in a 130-button surface, published payload equality, unchanged old-frame content,
render-domain generation, and shared untouched command segments. It also checks
publication of changed arranged input state, successful patch-stage counters,
idle skipped stages. Profiling counter ownership is tested in a separate ignored
test that must be explicitly executed with `--exact --ignored --test-threads=1`.
An ordinary parallel test run must not open this global capture; the isolated
managed invocation must execute exactly one target test and verify its counter's
presence/value, including a legitimate zero-microsecond sample.
The regression must execute before it becomes acceptance evidence. It does not
replace the original 1/100/1k/10k probe, one-percent dirty, p95, damage, pixel,
serde or upward gates.

Managed dynamic submission remains held by the
[external archive diagnostics handoff](../../../zircon_tooling/session_coordinator/01/failure-2026-09-27-external-worktree-archive-error-diagnostics.md).
This repair has no accepted Cargo ticket. Its frozen source, original scale
acceptance, the named independent closeout review, canonical return, managed
commit and notification remain required. Status stays `open`.

## 2026-09-27 empty clip propagation follow-up

The arranged owner used `Option<UiFrame>` for both "no clip" and an empty
intersection. A disjoint leaf/parent pair therefore lost its empty clip and a
later ancestor or frame fallback restored visibility. The actual render path
also discarded an empty clip in semantic child commands, AgentChat text and
text field content. The repair keeps `Some(zero-area frame)` through the
arranged calculation and private render intersections; only an original
`None` remains unbounded. It does not change the public `UiFrame::intersection`
contract, which intentionally reports no positive overlap as `None`. The
interface paint conversion also keeps zero-area scissors unchanged before
pixel snapping so fractional coordinates cannot reopen one pixel of content.

The new surface-frame regressions cover two and three clip owners, touching
edges, no-clip behavior, the local authored-geometry patch, retained old frame,
full rebuild parity, hit authority and paint scissors for TreeView, AgentChat,
ChatComposer and InputField child text commands. An interface command test
checks zero-area clip conversion at four DPI scales and keeps positive and
absent clip behavior. They are source-mounted but
not yet executed by managed Cargo; rustfmt and source review alone do not
constitute dynamic acceptance. The nine production/test paths preserve
their current preimages recorded in the exact ownership transfer previews
`1313f0fc74f57483e3f54ef30f0cae001f85e31c7334b55f1592a6ac800554de`,
`169567e69a4846352dbda31977c41884afbfc41f8ed139f2cd2051e3eb58bb86`,
`b9a60adc60a9a36ad8453ac3296c1d197d54cd088abe62df5a852463181f10a4`,
`61ae1d58bfd584f3e14a571813e356c9319995615f3adc45a327ff93a3f35b4a`
and `73ebe04cd3e21d9b5d07affd0e11a7d2868b6e05910817767b5370300832d0b3`.
Several render files already contained untracked or modified work; transfer
records current content, while closeout must isolate this repair's exact
changes without absorbing unrelated preexisting work.

This clipping correction is a dependent correctness slice of the open
arranged-index/stage failure. The original 10k scale, one-percent dirty,
node/slot/ancestor probes, CPU p95, real pixel, complete Cargo and upward
acceptance gates remain pending. It creates no `fixed-*` return or closeout.
