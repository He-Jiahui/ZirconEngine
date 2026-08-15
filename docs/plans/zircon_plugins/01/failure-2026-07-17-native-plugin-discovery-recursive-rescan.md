---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-plugin-discovery-recursive-rescan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/collect_manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
tests:
  - unchanged plugin discovery generation-cache test
  - manifest add/change/remove invalidation test
  - symlink-junction cycle and depth-bound test
---

# Plugins01：native plugin discovery 重复递归扫描整棵 root

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP native plugin loader/discovery 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：discovery cache、watcher generation、产品 load manifest 与 live-host refresh 必须共用一套插件目录 authority，不能由 editor status 本地 memoize。

## 失败现象与复现证据

`NativePluginLoader::discover` 每次从 root 递归 `read_dir`，对每个 entry 调用 path metadata 并收集
全部 `plugin.toml`。扫描没有显式 symlink/junction cycle、max depth 或 canonical visited-root policy，
也没有 unchanged generation cache。原加载阶段还会 clone 整个 discovered candidate Vec（含 package
manifests）；该局部冗余复制已由性能审计 Session 用 `mem::take` 消除，但文件系统发现 authority 仍未收敛。

Editor export registration/status 与 live-host loading/refresh 均能调用 discovery。产品已有显式
`plugins/native_plugins.toml`，但通用递归路径仍可能在重复状态刷新中发生。

## 最低共享层根因

插件目录缺少“canonical root + discovery generation + manifest fingerprint”的共享 authority；扫描、
解析、加载选择与 editor 状态查询各自从文件系统重新发现事实。

## 架构修复验收

- 产品启动优先消费 export-time `native_plugins.toml`，不递归扫描任意 root。
- Editor/dev discovery 以 canonical root、file identity/visited set、symlink/junction policy和最大深度有界；
  watcher 或显式 refresh 增 generation，只重读新增/修改/删除 manifest。
- load 阶段继续保持已落地的 candidate ownership take/restore，不回退为深 clone discovered reports。
- unchanged 1k/10k tree refresh 的 enumerate/stat/read/parse count 为 0；单文件变更精确为 O(1)+受影响依赖，
  顺序与 duplicate-package diagnostics 保持确定。

## 禁止临时方案

- 不得只在 editor status 层缓存一个 `NativePluginLoadReport`，让 live host 继续独立重扫。
- 不得盲目并行执行动态库 entry callbacks；并行仅用于安全的 filesystem/stat/parse 阶段且要有预算。

## 修复结果与回传

当前状态：`open / current_source_incremental_refresh_implemented / managed_validation_pending`。

### 2026-07-22 current-source 实现

- 产品/export 路径继续以 `plugins/native_plugins.toml` 为唯一 authority；递归 discovery 仅服务 Editor/dev。
- `NativePluginLoader` 的全部调用点共享 process discovery authority。authority 以 lexical alias 命中 canonical root、
  每 root 独立 mutex 与 immutable generation snapshot 工作；稳定命中不再次 canonicalize/stat，最多保留 16 个
  least-recently-used roots。
- cold scan 使用 path-sorted breadth-first traversal、canonical visited set、16 层深度上限、symlink 不跟随与
  package manifest 后停止下钻；canonical junction cycle、root escape、depth limit 都产生确定诊断。
- 每 root 的 recursive watcher 只排队路径，下一次 read 在 per-root lock 内增量应用。显式
  `refresh_discovery_manifest` / `remove_discovered_path` 支持 editor watcher 或工具链确定性投递；单 manifest
  修改只 read/parse 一项，不重新 enumerate root。显式路径若来自 lexical root alias，会先按相对路径映射到
  authority 的 canonical root；删除后的路径也不依赖重新 canonicalize，因此不会被静默判为 root 外路径。
- duplicate package id 以 manifest path 排序选择首项，并在诊断中同时记录 first/duplicate path；load 阶段仍
  使用既有 `mem::take` ownership 流程，不回退为 report deep clone。
- 已新增 unchanged generation 零 enumerate/stat-read/parse delta、add/change/remove O(1)、并发首建单 generation、
  duplicate 顺序、depth bound、Unix symlink cycle，以及 ignored 1k/10k unchanged tree 证据。

Rust `1.94.1` scoped rustfmt、`git diff --check` 与 native public/lifecycle structure audit 已通过：root
re-export `0`、native namespace `68`、classification groups `5`、unclassified `0`、migration debt `0`、M4
`classified-and-clear`、risks `[]`。typed-error source guard 已同步到 generation authority 的 report boundary。
lexical-alias explicit refresh/remove 回归已写入，source contract 已完成 RED/GREEN；其 managed test 随下述
current-source focused batch 编译为同一受管 test binary。

### 2026-08-10 current-source re-audit

- `discover/authority.rs` 的 `refresh_manifest(root, _manifest_path)` 与
  `remove_path(root, _removed_path)` 均丢弃已通知路径并调用
  `project_root(root, true)`，后者提交 `RootScan`。因此当前 watcher mutation 仍是有界的整 root
  refresh，而不是本 handoff 所要求的单 manifest read/parse delta。
- `discover/tests.rs::manifest_notifications_refresh_the_same_authority_generation` 只断言报告内容和
  generation 递增；它没有 filesystem enumerate/read/parse 计数或通知路径断言，不能证明 O(1)
  mutation 合同。历史 managed jobs 仅可作为当时快照证据，不能覆盖本次 current-source re-audit。
- 本 failure 保持 `open`。后续修复必须以 authority 内唯一的 immutable manifest index 和合并通知批次为
  最低共享层，先补 one-path、burst、remove、failed-parse/last-good 和 overflow full-rescan 的 RED
  回归，再由受管 current-source gate 验证；不得以 editor 本地缓存或第二 watcher index 绕过。

### 2026-08-11 current-source batch-order correction

- authority、refresh work、immutable manifest index 和 source metrics 已在当前源码中构成单一的增量
  路径；单 manifest change/remove、failed parse last-good 和 root-external notification 的回归均保留在
  discovery owner 测试中。
- 对合并批次复审发现 `Refresh(child/plugin.toml)` 后接 `Remove(parent)` 时，旧归并会保留子刷新并使
  collector 尝试读取最终已删除的文件。`discovery_refresh/work.rs` 现在由后到的删除移除其目录内的全部
  先前动作；相反的 `Remove(parent)` 后 `Refresh(child)` 仍保留顺序，以支持同一批次中的目录重建。
  `later_parent_removal_discards_an_earlier_descendant_refresh` 固化该最低共享层回归。
- 受管归属转移 `ec1cc5b20b158cccb3deec51ba937fdcfe17a1363daec359b9f07959f3723b08` 将新增 work
  owner 纳入 Plugins01 r4；`rustfmt +1.94.1 --check` 和 scoped diff-check 通过。未运行直接 Cargo，且
  没有 current-source managed focused/broad 结果，因此本 failure 继续保持 `open`。
- 本轮复审另发现一条未归属于 `work.rs` 的正确性边界：全量扫描已将某 manifest 的解析失败写入
  published snapshot 后，同一路径修复并成功执行增量 refresh 时，`from_incremental_payload` 会无条件保留
  `base.collector_diagnostics` 并 append 新诊断，使已解决的解析错误永久出现在后续 report。现有
  `failed_incremental_parse_keeps_the_last_good_snapshot` 仅覆盖失败不发布的分支，不覆盖“失败初始扫描 -> 修复增量 refresh”。
  此最低层修复需要为 collector diagnostic 保留通知路径归属，以便从 immutable snapshot 中合法移除被修复路径的
  旧诊断，而不影响其他 package 的诊断。2026-08-11 coordinator transfer preview
  `d9767a4c50b75ba1c5058b4fc0ca1983c224ba490c4b5ee8ab261fefd5e2833f` 确认 `contract.rs` 与
  `discover/tests.rs` 仍由可执行的 Frameworks04 会话拥有，因此 Plugins01 未跨会话编辑该两个路径。此项不是当前
  `work.rs` 修复的完成证据，仍需 Frameworks04 的 TDD 修复和受管 current-source 验证。

Windows managed job `93f88e221e244b93b176afa90a07cdff` 在 coordinator-retained
`D:\cargo-targets` compatibility pool 完成 current-source core-min lib-test 编译并执行
`unchanged_discovery_reuses_generation_without_filesystem_work`：`1 passed / 0 failed / 4309 filtered`，exit `0`。
直接执行该 job 保留的
`zircon_runtime-d0d157582ce270bf.exe`（SHA-256
`0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）又通过
`explicit_changes_map_lexical_alias_paths_to_the_canonical_root`：
`1 passed / 0 failed / 4309 filtered`，使用 `--exact --test-threads=1 --nocapture`。add/change/remove focused
组随后暴露两个 Windows 回归：

- duplicate-package 测试仍用 lexical temp path 比较，而 authority 契约已返回 canonical
  manifest path（Windows 为 `\\?\` 前缀）；断言已改为比较 canonical expected path，不改变
  产品路径契约。
- watcher 对一次显式 refresh 同时投递 package-directory 与 manifest 事件，显式 API
  又重复 apply，导致预期 3 次的累计 read 实际为 6。`refresh_manifest` / `remove_path`
  现在先映射 canonical explicit path，再从同一 package subtree 的 watcher batch 排除重复事件；
  其他 package 事件仍正常应用。

上述修复已通过 scoped Rust `1.94.1` rustfmt 与 diff-check；源码改写后必须重编译复验。
reservation `691a10ea0c8a4dbda8bea9cb27a87a0f` 保留在 coordinator FIFO 中，在 add/change/remove
整组 GREEN 前不声明 current-source focused 通过。未受本次 explicit-change 改动影响的
ignored 1k/10k unchanged-tree benchmark 已在同一受管 binary 上单独通过
`1 passed / 0 failed / 4309 filtered`：1k cold/warm 为 `653.901 ms / 0.032 ms`，10k 为
`5959.253 ms / 0.014 ms`；两组 `enumerate/inspect/read/parse` warm delta 均为 `0`。broad
parity 与 failure return 仍待完成，因此本 failure 保持 `open`。

### 2026-07-22 warm report所有权增量证据

本轮继续逐文件审查discovery/load-manifest生产路径。generation authority已消除unchanged filesystem work，但`discover()`仍在root mutex内drain watcher并同步read/parse变化manifest，随后`DiscoverySnapshot::report()`为owned public结果深clone全部candidate/package manifest/diagnostics；Editor状态若调用完整discover而不是`discovery_generation()`，warm大目录仍有O(P+manifest bytes) allocation/clone和锁持有。

PERF-MVP-539已完成局部RED→GREEN止损：report精确reserve，duplicate索引借用snapshot plugin-id/path；export load manifest以owned pop/validate/push删除candidate深clone与shift remove，export root每operation只规范化一次。剩余验收增加：

- watcher callback只发布有界path/coalesced generation；filesystem read/TOML parse在Runtime11 single-flight candidate job锁外执行，root state短锁只校验base generation并commit immutable snapshot。
- status/generation consumer不得为只读轮询materialize owned report；需要完整report的loader只在changed generation取得一次Arc/owned handoff。
- unchanged 1k/10k manifests除filesystem delta为0外，report/candidate/manifest clone bytes、alloc和root lock hold也必须为0；单manifest change工作近affected bytes，失败保持last-good。
- current-source Cargo、alias/escape/duplicate/watcher burst、F0/F4 discovery trace与failure return完成前保持open。
