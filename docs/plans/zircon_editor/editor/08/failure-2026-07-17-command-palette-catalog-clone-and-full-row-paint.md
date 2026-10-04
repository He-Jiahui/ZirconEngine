---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: command-palette-catalog-clone-and-full-row-paint
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/commands/eval_snapshot_handle.rs
  - zircon_editor/src/core/commands/palette.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/tests/commands/descriptor_when.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette/commands.rs
  - tools/tests/test_editor08_command_palette_query_contract.py
  - tools/tests/test_editorui06_command_palette_paged_keyboard_contract.py
---

# Command palette catalog clone and full-row paint

> 深页键盘导航不是本记录中查询目录的剩余算法补丁；runtime 组件缺少分页窗口语义出口，已交接到 [EditorUI06：CommandPalette 分页键盘导航契约](../../editor_ui/06/failure-2026-07-18-command-palette-paged-keyboard-navigation.md)。该交接禁止恢复完整目录 UI 投影。

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_command_palette*` 39/39 个 Rust 文件及已审查 command registry/open-state 入口聚焦回查
- 修复责任计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 交接原因：command catalog generation、enabled evaluation、typed search index/result 与 entry ownership 属于 Editor08；EditorUI08 只负责 visible-row consumption/paint。

## 失败现象与复现证据

Palette open先收集完整entry Vec，再完整转换为commands UiValue，并再次clone全部id生成filtered_commands。Painter随后对全部structured rows执行row_data，leaf才clip。大catalog和query burst因此可能同时放大catalog owned bytes与offscreen paint work。

## 最低共享层根因

Command registry没有发布可共享的immutable catalog generation和typed query result；UI边界以多个完整owned DTO表示同一catalog，也没有可直接消费的visible/top-K结果预算。

## 架构修复验收

- Stable catalog重复open不深clone完整entries；catalog generation只在descriptor/when依赖变化时更新。
- Query使用typed index/result并有明确top-K/visible预算；1,000 keystrokes报告visited/comparisons/allocations与input p95。
- EditorUI08只clone/visit visible+overscan handles；offscreen row_data/text/build为零。
- Enabled/when、selection/focus/commit、search/empty、row detail、ordering和pixels等价。

## 禁止临时方案

- 不得在painter建立第二份command catalog或不受registry generation约束的cache。
- 不得保留commands与filtered ids两份完整owned catalog再仅优化其中一份。
- 不得以截断结果静默改变keyboard selection/commit语义；top-K/virtualization必须保留完整可检索集合。

## 修复结果与回传

Resolving failure（2026-07-18）：Editor08 已交付 generation-owned shared catalog、typed paged query、
权威 `QueryChanged` route/binding/host intercept 与 bounded bridge update；旧全量 entry/value API
扫描为 0，open/query edit 均收敛为 8 visible + 4 overscan，1,000-query 测试锁定 retained
handles 不超过窗口预算。EditorUI08 painter 也已从全行 `row_data` 改为 clip-derived visible +
1-row overscan borrowed access。EditorUI06/Editor08 现已接通 typed 深页 keyboard window request、
stale offset/generation 拒绝和 bounded host requery。当前仍保持 `resolving_failure`：受管 current-source
Cargo、disabled/commit 产品门、像素等价、1,000 输入 p95 与独立 review 尚未完成；Coordinator01 immutable
full-input snapshot failure 仍阻断有效验收。不得把本次源码阶段记录改名为 `fixed-*` 或提前
回传来源计划。

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-07-18 | generation-owned query、bounded paint 与深页 keyboard source closure | resolving_failure / 源码完成，待受管验收 | catalog 查询、visible+overscan paint 与 typed keyboard window request 已形成闭环，host 不恢复全量 UI catalog，并对 stale offset/generation 无副作用拒绝。Python contract 3/3、Workbench ZUI TOML、focused rustfmt、scoped diff check 通过；Cargo、1,000 输入 p95、像素/产品交互与独立 review 仍开放。 |
| 2026-08-13 | immutable postings index、共享 context、单遍 fuzzy 与 registry 短锁硬切 | resolving_failure / 实现与二审修复完成，待受管验收 | Catalog generation 现持有按规范化字节构建的 256 postings 与 descriptor-aligned enablement slots；查询以最稀有字节无损收窄候选，并在一次 document byte pass 中同时保持 exact substring 255 分和既有 greedy subsequence 排序。`CommandEvalSnapshotHandle` 按语义代际发布共享 `Arc<CommandEvalCtx>`，palette 三入口不再逐键深拷贝 capability strings。Registry 的 query facade 已删除，retained host 只在 mutex 内取得 catalog `Arc`，匹配、when/MRU 排序和 12 行 UI 投影均在锁外。Rust 回归覆盖 1,000 catalog 的 selective visits、窗口/总命中、exact/subsequence、重复字节 postings 与 shared context Arc 代际；Python contract 先捕获旧路径的 5 个 RED finding，最终 Editor08 4/4 + EditorUI06 3/3 GREEN。独立二审的 1 个 Important 为 EditorUI06 guard 仍引用旧 facade，已前向改为 catalog Arc/query 并保留 stale generation 检查；其余算法/锁/shared snapshot 无 finding。旧 facade 扫描 0，focused rustfmt/diff-check 通过。受管 current-source Cargo、1,000 input p95、pixel/disabled/commit 产品证据仍待 coordinator receipt，因此不改名 fixed、不提前回传。 |

2026-07-22逐文件复核：generation-owned catalog和bounded window继续成立，因此旧“open时全量clone”根因不恢复；remaining query每次仍两遍扫描全部search documents，enabled行每遍按String id回查registry BTreeMap，window UiValue再clone字段。failure保持`resolving_failure`，PERF-MVP-211验收补descriptor slot/enablement index、增量候选或等价top-K+count证据，不能只以retained handles≤window关闭。

2026-09-02 静态合同同步：host 的 stale-query fence 已从隐式 `String`/`&str` 比较写法收敛为
`query.as_str() != request_query`，行为与代际拒绝顺序不变；旧 Python guard 因仍匹配
`query != request_query` 产生 1 个假 RED。本次只同步 guard 到显式借用表达式，不改生产源码；
受管 current-source Cargo、1,000 input p95、pixel/disabled/commit 产品证据和 fixed return 仍开放。

### 2026-09-18 static ticket terminal result

The managed command-palette static ticket `f99abb14cc314fff982c423174cc7399`
reached terminal `passed` with job `d51bf22e0b8a46a1a58c44267f88f542`, run
`f99abb14cc314fff982c423174cc7399`, and exit code `0`; all 3 Python contract
tests passed. This confirms the typed paged-keyboard/stale-generation guard only;
current-source Cargo, product interaction/p95/pixel evidence, review, and fixed
return/closeout remain pending. The original failure lifecycle stays `open` and
the static receipt is not reused as whole-feature acceptance. The external
`E:/Git/zr_vm` dirty prerequisite and the Coordinator01 immutable full-input
snapshot blocker remain recorded without absorbing foreign changes.

The receipt was completed after a stale receipt-only Session was reconciled by
the coordinator; no production source was edited in this continuation. The
canonical ticket evidence remains the original immutable source snapshot and
its terminal event, with no duplicate validation request.

### 2026-09-21 immutable current-baseline validation request

Successor Session `failure-roll-01a084c8-editor08-command-palette-r3` sealed
owned plan/failure inputs under source manifest
`5a09eab798d51819dbbb879613afbde30832d7ff41b058aa966ff898b8e3fc5b` and
submitted static contract ticket `b0d815439d7e46329eb7b1a52acb46c2`.
The command is prepared to verify the shared generation-owned catalog, typed
postings/top-window query, short-lock `Arc` extraction, stale query/generation
fences, clip-derived borrowed row paint, and both Python command-palette
contracts. Production paths with newer foreign-attributed optimization overlays
were deliberately not absorbed into this Session; their current hashes and
semantics are handled by the independent review.

The ticket remains `queued` and has not executed. Coordinator admission records
`validation_dependency_failed`, with the direct command-palette blocker
`command-palette-paged-keyboard-navigation` under EditorUI06 and additional
transitive open plan dependencies. This receipt is therefore neither a pass nor
dynamic acceptance. The previously passed ticket `f99abb14cc314fff982c423174cc7399`
still covers only its older paged-keyboard/stale-generation Python snapshot.
Current-source Cargo, disabled/commit interaction, 1,000-input p95, pixel
equivalence, independent C/I/M review, fixed return, and closeout remain open;
`E:/Git/zr_vm` is also still dirty.

### 2026-09-21 independent current-source review

Reviewer Session `review-editor08-command-palette-r3` audited the current tree
at HEAD `6b4bc86089cb4464f850136c8079e90cb6513ebc` and reported
`Critical=0`, `Important=2`, `Moderate=0`. The locally read-only Python guards
were 7/7, but the findings demonstrate why those textual guards are not feature
acceptance:

- `Important 1`: `palette.rs::search_document` indexes id, label, source,
  category, and keywords but omits `EditorCommandPaletteEntry.shortcut`.
  The runtime component matching contract includes shortcut, so a shortcut-only
  query such as `Ctrl+Shift+P` can disappear on the typed postings path. The
  document also uses ASCII-only lowercasing while the query uses Unicode
  lowercasing; search/order parity needs shortcut-only and non-ASCII case tests.
- `Important 2`: a window request serializes only current offset, target offset,
  focus, and query. The handler reads the bridge's catalog generation at
  processing time rather than consuming the generation that produced the
  request. Although the runtime reducer writes `window_request_generation`, the
  native producer does not carry it and no consumer qualifies the request with
  it. A delayed old request with the same query/offset can therefore be applied
  after a newer catalog generation.

The reviewed SHA-256 values include `palette.rs`
`b0aadad016d58c20dd94abaf7a066c07adaf84eeb92fd9215d198dd6ffb50e45`,
`command_palette_actions.rs`
`11f0e9332852a006506295ba9b84b8b843b2b2e9224b9b911629e977609fb260`,
native window-request producer
`06ca6b6d892dd044e53d779d0be1ae38bef009eb0ec906924a5e49b2b75300f5`,
and runtime reducer
`c70d3dbe4cb35a58e326505626b17ea5b77cea0b42b1859606947f8ab2a78ceb`.
Shared catalog ownership, short registry locking, bounded typed handles/full
match count, and visible-plus-overscan borrowed painting otherwise remain
intact. The two findings require precise Editor08 and EditorUI06/native-keyboard
ownership reconciliation before edits. Review is not green, and all dynamic,
performance, pixel, return, and closeout gates remain open.


### 2026-10-04 independent shortcut lower-layer result (UTC)

The current workflow follows the authorized independent Windows validation
policy. The historical coordinator requests above are preserved; none was
replayed, altered, or promoted to acceptance. This lifecycle remains `open`.

Root installed only the missing shortcut search-document delta and its new
catalog regression file. The preimage of `palette.rs` was
`92db70fa00cbee06efe96529a52882ffc32bd2859144d35f9c56eb55af14f7e6`;
three insertions total 243 bytes, and removing them exactly recovers that
preimage. All original business bytes and seven comment lines (540 bytes)
remain; Root does not adopt the unknown preexisting business delta. The
Unicode lowercasing path was already present in this preimage and is retained.
A nonempty shortcut is indexed after a field boundary; an empty shortcut
preserves the previous document bytes. Catalog ownership and query budgets
are unchanged.

| Current installed input | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/commands/palette.rs` | `685763c093dc95dfd7fb0db30d5568794302591970a4712fc3cb90052ee2b146` |
| `zircon_editor/src/core/commands/palette/shortcut_search_tests.rs` | `c10cc0102f23e447362cd5a777c1a367b67c24786d658e40b4966956ddea9c04` |

- [Exact installation](../../../../../.codex/tmp/failure-roll-20261004-editor08-shortcut-root-install-r1/terminal.json) binds both postimages, the preserved preimage, and unchanged scoped shared-index entries. No Git write or commit occurred.
- [Actual Windows Rust helper execution](../../../../../.codex/tmp/failure-roll-20261004-editor08-document-actual-preparation-r1/actual-terminal.json) used the exact production entry and search-document body with the same three regression tests. The original body compiled and produced the expected two failing shortcut assertions plus one passing empty-shortcut control; the candidate compiled and passed all three. This is standalone standard-library helper evidence, not an Editor package, catalog, or UI test.
- [Corrected helper evidence review](../../../../../.codex/tmp/failure-roll-20261004-editor08-shortcut-corrected-actual-independent-review-r2/review.json) and [formatted candidate review](../../../../../.codex/tmp/failure-roll-20261004-editor08-shortcut-formatted-reception-r3/review.json) are C/I/M 0/0/0. Formatting preserved the exact tested helper body. The old erroneous eight-line comment count is retained as history; the reviewed correction records the actual seven lines/540 bytes.
- The [original Python receipt](../../../../../.codex/tmp/failure-roll-20261004-editor08-original-python7-r1/terminal.json) recorded child exit zero but mistakenly parsed zero tests from CRLF output. The [result reception correction](../../../../../.codex/tmp/failure-roll-20261004-editor08-original-python7-result-reconciliation-r2/terminal.json) binds the unchanged raw log and all seven original test IDs: 7 passed, 0 failed, 0 skipped. The original receipt was not overwritten and no test was rerun. All eleven declared Main/copy inputs matched before and after execution.
- The [independent installed-source/result reception](../../../../../.codex/tmp/failure-roll-20261004-editor08-original-python7-independent-reception-r2/review.json) is C/I/M 0/0/0. These seven Python tests are source contract assertions; they do not dynamically execute Rust catalog, runtime, or host behavior.

The three new real catalog regressions are present but have not been
typechecked or run. The second Important finding, request-origin catalog
generation, remains unaccepted; its coherent Editor08/EditorUI06/native and
ScenePicker consumer candidate is still private at this record. The real
current-source Cargo batch, original catalog/window and direct-consumer
regressions, disabled/commit behavior, 1,000-input visited/comparison/allocation
and p95 measurements, visible-plus-overscan/offscreen-zero checks, and pixel
equivalence remain open. The latest D/E/F storage sample is below the
repository's 35 GiB compilation admission threshold, so no further compiler
was launched on that sample. No failure return, `fixed-*`, commit, WeCom, push,
coordinator service, or whole-MVP acceptance is claimed.
