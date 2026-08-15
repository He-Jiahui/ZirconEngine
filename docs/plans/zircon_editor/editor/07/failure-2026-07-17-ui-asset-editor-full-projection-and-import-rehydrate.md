---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ui-asset-editor-full-projection-and-import-rehydrate
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/07
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/asset_editor/tree
  - zircon_editor/src/ui/asset_editor/binding
  - zircon_editor/src/ui/asset_editor/diagnostics
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_editor/src/ui/asset_editor/session/palette_state.rs
  - zircon_editor/src/ui/asset_editor/session/theme_state.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring.rs
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
  - dev/Fyrox/editor/src/asset/item.rs
  - dev/Fyrox/editor/src/plugins/absm/canvas.rs
  - dev/godot/editor/scene/canvas_item_editor_plugin.cpp
tests:
  - source typing and palette drag projection build-count stress
  - diamond/cyclic import read/parse-count regression
  - UI asset editor presentation byte/order parity
  - 1/100/10000 theme action generation and label projection stress
  - 1/100/10000 preview nodes and slot targets under 125/500/1000 Hz drag move
  - 1/100/10000 bindings, targets and payload leaves with depth 1/16/64
  - 1/100/10000 preview nodes, properties, overrides and expressions with depth 1/16/64
---

# Editor07：UI asset editor 每 mutation 全投影与 import graph 重水化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`ui/host/asset_editor_sessions` 19/19 Rust 文件逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：domain document generation、import dependency graph、presentation projection 与 editor gesture cadence 必须由 Editor07 UI asset editor session 统一拥有。

## 失败现象与复现证据

多数 binding/inspector/style/node/palette mutation 在 session 锁内更新后立即 `sync_ui_asset_editor_instance`，后者构造完整 reflection model并把 title/dirty/route 写回 view。source typing 还在每次 edit 后递归 `hydrate_ui_asset_editor_imports`，逐引用 resolve path、读盘、parse、project v2，再替换全部 resolved imports。palette pointer drag 原先即使状态未变也重建 projection，本轮已只在 changed 时同步；其余真实变更仍是全量。

import traversal 只在读盘/parse/insert之后检查 normalized visited id，diamond/cycle 会按重复边重新读取/解析同一物理文档后才停止递归；不同 fragment alias 又要求共享物理 parse但保留逻辑 reference rows，不能靠简单提前 return 修复。Slint repeater 对 row change 定点更新对应 instance或标 dirty，model 未 dirty 时直接返回；Zircon 需要相同的 document/property generation，而非每个 UI gesture 重建整份 presentation/import graph。

## 最低共享层根因

session mutation API 只返回 bool，不返回 dirty domains/changed nodes；reflection、preview、route、import graph 没有 generation cache或 physical-document parse cache，因此 host 只能统一 full sync/full hydrate。

## 架构修复验收

- mutation 返回 typed dirty domains/ids；source/property/palette/selection 只重建受影响 presentation rows，单帧相同 domain 合并一次。
- import graph 按 canonical physical asset id 缓存 parse/project结果，并保留 reference+fragment aliases；diamond/cycle 每 physical document 每 generation read/parse≤1。
- source typing 采用 parse/dependency debounce或增量 parser；磁盘 I/O 不在每 key event 主线程执行，最后 revision与 diagnostics 可观测。
- 1k typing/drag/selection stress 记录 build/read/parse/clone count与p95；save/undo/redo/conflict/preview/serialized route行为等价。
- theme helper/refactor/cascade typed actions按document/import/selection generation缓存并使用stable action id；stable pane/helper lookup action build与label format=0，可见列表之外label projection=0。
- source generation单次构建node→line range与line interval index；outline build近O(nodes+lines)，selection/line lookup不重建全outline，stable source lookup allocation/build=0。
- palette catalog与node/control identity index按document/registry/import generation发布；stable presentation/drag catalog build=0，unique-id候选不逐suffix全树扫描。
- preview generation发布canvas hit index与typed drag target artifact；pointer move不重建完整preview、不clone document试插，Grid/component候选与overlay按可见或chooser预算惰性物化，同帧move可合并但drop/cancel/manual chooser边沿保序。
- document/binding generation单次发布filtered diagnostics、payload tree与schema/preview artifact；stable inspector presentation不重跑full binding report或递归flatten/format，binding delta只更新受影响node/binding/path rows。
- document/mock generation编译subject/property index、typed expressions/dependency graph与shared mock/schema/state rows；stable presentation不重扫全document或复制nested Value，override delta只求值并patch依赖闭包。

## 性能结构调研（2026-08-09，待实施）

> 本节是优化前的架构与测量设计，不是实现完成、性能通过或 failure 关闭记录。尚未取得 current-source profile、p50/p95、RSS、GPU/CPU 功耗或跨引擎可比数据。

### 已确认的调用链与根因假设

- `EditorUiHost::ui_asset_editor_pane_presentation` 在持有 UI asset session 锁时调用 `UiAssetEditorSession::pane_presentation()`；后者在一个 pull 请求内重新构造 reflection、preview projection、palette drag overlay/candidates、source selection、mock/binding、slot/layout semantics、style、theme 与 command availability。
- `UiAssetEditorPanePresentation` 是包含大量 `String`/`Vec<String>` 的一次性 DTO。当前 source outline、palette catalog 与 preview hit index 已各自拥有局部 generation artifact，但稳定 pane 请求仍会跨越这些域重新物化输出。
- 静态盘点（不是 runtime profile）：当前 `pane_presentation` 为 757 行，含 23 个 `build_*` 调用、36 次 `.clone()`、4 次 `.position()`、39 次 `last_valid_document` 引用和 7 次 `format!`。这只证明单一 pull path 的算法/所有权密度，不能推出 CPU 时间、RSS、GPU 或功耗。
- 因此本 failure 的优先假设是“缺少 document/import/source/selection/preview/mock/style 等 typed dirty-domain 和 pane generation”，而不是任一 lookup 的常数项。必须用下列 profile 证伪或量化该假设后，才能继续优化实现。

### 参考架构与目标边界

- 主参考为 `dev/UnrealEngine`：`UAssetEditorSubsystem` 负责打开、聚焦、关闭与 editor-instance 生命周期；`FAssetEditorToolkit` 负责 toolkit、tab、command 与 extensibility 生命周期。它们不把每次 host 查询定义为重建全部 asset-editor 数据的边界。
- Rust 侧对照为 `dev/Fyrox/editor` 的 `asset`、`command`、`scene`、`ui_scene` 分层。Zircon 的落点仍是 `zircon_editor`：session 持有 authoring state，host 只负责 instance 生命周期与路由；runtime world、资产权威和渲染 extract 不迁入 editor DTO。
- 实施前必须先设计 `UiAssetEditorPresentationGeneration` 与按域不可变 artifact：document/import/source、selection/command availability、preview/drag、mock/binding、style/theme。mutation 产生 typed dirty domain/affected identity，host 消费 domain patch；不得以 host 侧全量 DTO cache、全局可变单例或旧路径兼容 shim 代替该边界。

### Windows 受管 profile 设计

1. 先在 coordinator 管理的 Windows validation copy 运行 `tools/ui-profile-capture.ps1`，输出与 Cargo target 必须位于批准的 `D:`、`E:` 或 `F:` 根目录，绝不写入 `C:` 或仓库 `target/`。本机已发现 `wpr`、`wpaexporter`、`xperf` 和 `dev/tracy/tracy-profiler.exe`，但尚未启动 profile；工具存在不是性能证据。
2. 对同一固定 UI asset 分别采集 `startup`、`idle_hover`、`click`、`drag`、`asset_refresh`；开启 `-RequireScenarioEvidence`，必要时分别使用 `-UseTracy` 与 `-UseWpr`，保留原始 trace、`ui_hotspots.json`、frame/draw/upload counters 和机器/adapter 元数据。
3. 先补齐可归因 instrumentation：每个 pane domain 的 build/patch/reuse、分配字节或可替代的 owned row/string count、clone count、session-lock hold time、source/import read/parse、visible-row/command count。现有 frame、redraw、draw、upload 数字不足以证明 pane CPU 根因。
4. 以 31 次采样报告 p50/p95 与最大值，按 1/100/1,000/10,000 document nodes、bindings、theme rules 和 source lines 分层；仅同机同驱动同 profile 可比较。功耗须有同一 Windows ETW/WPR 机器的基线与空闲扣除，取得前不得声称接近 Unreal 或其它引擎的能耗经验值。

### 进入实现的门槛

- 先冻结一个 source manifest 和产品交互脚本，得到上述原始 profile；若热点不在 pane assembly，不实现该 cache，而是将证据路由到实际最低 owner。
- 若假设成立，先以单个 domain 的 failing build/reuse/ordering test 验证 typed generation/patch contract，再从 framework boundary 向 leaf consumer 实现；不得一次性缓存完整 pane 或用延迟 paint 掩盖工作量。
- 改造后对同一脚本复测，并报告变化前后 p50/p95、分配/clone、lock hold、read/parse、GPU upload/draw 和 profile evidence。任何没有当前源指纹的历史截图或队列回执均不得作为结果。

## 禁止临时方案

- 不得只延迟 UI paint却继续每输入读盘/parse全图。
- 不得按 normalized path 简单跳过而丢失不同 fragment alias或 expected-kind诊断。
- 不得让后台 parse 旧 revision 覆盖新 source generation。

## 修复结果与回传

Partial implementation（2026-07-18）：Editor07 已落地 generation-scoped `UiAssetImportTraversal`。canonical physical source path 现在同时拥有成功/失败 read/parse/v2-project cache key、实际读取目标与 parser mode，physical expansion set 使 diamond/cycle 每物理文档每 generation 最多展开一次；logical `reference#fragment` rows 与逐 edge expected-kind 诊断继续保留。strict hydration 与 lossy refresh 已统一复用同一 traversal。TDD 静态合同 4/4、精确 rustfmt 与 scoped diff check 通过；Rust 行为测试已写但受共享 Cargo/source-bound 顺序门影响尚未运行。详见 [子计划记录](2026-07-18-ui-asset-import-physical-cache.md)。

增量证据（2026-07-22）：`zircon_editor/src/tests/host/ui_asset_editor_theme_tooling` 5/5 owner逐文件复核。batch helper生产路径确认为单次document clone、批量mutation和一次replay/apply，未发现per-item compile/save；但`theme_rule_helper_action(index)`与`pane_presentation()`都会重新执行helper/refactor/cascade scans、构建完整typed action Vec并格式化全部String labels，而测试只有2 tokens/1 rule。本轮先把helper/refactor单项lookup从`get(index).cloned()`改为`into_iter().nth(index)`，删除目标action字符串二次深clone，源码合同1/1、组合22/22、rustfmt/diff通过。仍需补1/100/10k tokens/rules/imports下的action builds、tree/rule scans、label bytes、clone/RSS/p95，并按上方generation-owned typed action cache验收；本failure保持open。

生产style增量证据（2026-07-22）：`zircon_editor/src/ui/asset_editor/style` 12/12逐文件复核。除单项action move-out外，本轮让structured inspector只读path/kind零临时Vec/String clone，让refactor presence不再构建labels或重复扫描，并以一次borrowed `(stylesheet.id, selector)` index替代imported/active-cascade action的逐rule全local-rule重扫。编辑器性能源合同25/25、rustfmt/diff通过。summary/details/compare/cascade/helper/refactor/matched-selector仍各自扫描与物化，必须由本failure的generation-owned document/style/action artifact统一解决；current-source Cargo和产品规模门仍pending。

生产source/palette增量证据（2026-07-22）：`zircon_editor/src/ui/asset_editor/{source,palette}` 9/9逐文件复核。source outline当前对每node重扫全部source lines，line→node查询又完整重建outline；palette stable presentation仍重建全catalog，unique node/control suffix逐候选全树查询。本轮仅删除reference conversion allowed-key `BTreeSet` clone，并让native/component slot occupancy借用mount名；组合性能源合同27/27、rustfmt/diff通过。上述generation index/cache与1/100/10k规模门仍open。

生产tree/drag增量证据（2026-07-22）：`zircon_editor/src/ui/asset_editor/tree` 8/8逐文件复核。`update_palette_drag_target`每move仍完整构建preview projection并反向线性扫canvas nodes；Grid/slot candidates与overlays无预算物化。修复前每个candidate分别clone整document试插，本轮改为仅验证selected plan一次，component复用已生成targets，并删除import map双lookup、mount名clone和target slot map clone；源码合同6/6、组合30/30、rustfmt/diff通过。参照Fyrox稳定handle路由/hit test与Godot typed drag state+redraw，最终generation hit/target artifact、frame coalescing、规模counter、current-source Cargo和F4 trace仍open。

生产diagnostics/binding增量证据（2026-07-22）：`zircon_editor/src/ui/asset_editor/{diagnostics,binding}` 9/9逐文件复核。diagnostic mappers未见独立热循环；binding schema presentation每次为单个selection运行全document binding report，并递归物化payload/schema/preview rows。本轮删除被立即覆盖的旧schema builder及dead helper，复用payload flatten与next map，单项suggestion move-out，table排序借用keys，并合并两处重复node DFS；源码合同4/4、组合34/34、rustfmt/diff通过。generation-shared diagnostics/schema artifact、1/100/10k规模counter、current-source Cargo和F4 trace仍open。

生产preview增量证据（2026-07-22）：`zircon_editor/src/ui/asset_editor/preview` 8/8逐文件复核。presentation/palette move仍全量构建preview projection；mock pane每次重建subjects/properties/nested suggestions/schema/state graph并扫描props/bindings、parse expressions、clone nested Values。本轮以单次borrowed control-id index替代逐command两次全树匹配，删除mock subject iter_nodes内二次DFS、override per-key DFS、reference/function二次parse、单项suggestion宽clone、table/root/sort/bool临时分配；源码合同8/8、组合42/42、rustfmt/diff通过。generation projection/mock/dependency artifact、规模counter、current-source Cargo、F4 trace与像素仍open。

Open state: `physical import generation cache 已实现；仍待 typed dirty-domain/delta projection、typing debounce 与后台 revision 安全、1k stress build/read/parse/clone/p95、save/undo/redo/conflict/preview/route 等价证据后再关闭`。

## 产出记录与时间

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-08 | Preview generation：palette drag compact hit index | 代码完成 / 二次审查 0/0/0 / failure 保持 open | `UiAssetPreviewHitIndex` 以 document/preview-host generation 缓存 canvas node id 与 frame，不携带标签、类型、选择态或展示字符串；`update_palette_drag_target` 只惰性读取该索引，已不再构建完整 `UiAssetPreviewProjection`，无 palette 选择时不建索引且 hover 不 clone entry。文档替换、预览快照重编译和预设尺寸切换均统一失效缓存；slot overlay 继续使用完整展示投影，因此绘制行为未改。新增 Rust 行为测试覆盖连续 hover 复用、快照重建、预设 resize 与文档替换后的失效重建；静态合同已由 red 转 green（2/2），`py_compile`、精确 `rustfmt --check` 与 scoped `git diff --check` 通过。两份独立复审均为 Critical/Important/Minor `0/0/0`。managed current-source Rust 验证及 1/100/10000 节点性能、typed drag target artifact 等其余验收尚未运行，不能关闭本 failure。 |
| 2026-08-10 | Source generation：immutable outline index | 代码完成 / 独立复审与 managed validation pending / failure 保持 open | source generation 现在一次扫描建立 node entry、node-id index 与 line-segment interval index；presentation 与 navigation 均借用 session-owned cache，稳定查询不重建 outline。direct block 的 malformed `[` 边界、tree wrapper 冻结、首 occurrence、空 range、完整 `node` path segment 及 parent wrapper byte boundary均有 Rust 回归命名覆盖；source build counter 只保留在 lifecycle/navigation 的实际构建分支。import/pane/preview/source 静态合同 13/13、专属 `rustfmt --check` 与 scoped `git diff --check` 通过；未运行 Cargo、无 current-source 规模/profile 数据，因而不得关闭本 failure。 |
