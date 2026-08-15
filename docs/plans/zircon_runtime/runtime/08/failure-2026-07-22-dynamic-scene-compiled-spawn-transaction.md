---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-compiled-spawn-transaction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/value
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/world
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene --locked --jobs 1 -- --nocapture --test-threads=1
  - preview/apply parity, failure atomicity and large reflected scene fixtures
---

# Runtime08：dynamic scene compiled spawn transaction交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime dynamic scene非session基础35/35逐Rust文件审查，PERF-MVP-472
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有entity/component/type registry、archetype与World mutation transaction；Runtime04消费prepared ticket。
- 生命周期键：`dynamic-scene-compiled-spawn-transaction`

## 失败现象与复现证据

Prepared spawn只做scene schema自检；target remap/compatibility/parent、adapter/field resolution仍在主线程。preview物化remapped values后apply全部重做。spawn先注册types/insert records，再逐field写component/resource，adapter和field metadata按component clone、field lookup O(F²)，失败可留下partial World。capture反向按entity×全TypeRegistry扫描。

## 最低共享层根因

没有绑定`{scene content, target world generation, schema generation}`的compiled mutation plan；preview report与apply各自执行解释器。Reflection暴露String field dispatch而非dense accessor，World也没有一次提交最终signatures/rows/resources的事务入口。

## 架构修复验收

- prepare构建target-generation compiled plan：dense entity remap、resolved type/adapter/field slots、remapped values、final component signatures、resource writes和preview summary共享同一authority。
- preview只借用plan summary；apply验证generation token后按budget一次commit affected rows/resources并单次发布world/query/derived generation。
- failure/cancel丢弃未发布plan，World types/entities/components/resources零partial mutation；generation mismatch显式recompile或stale error。
- reflected multi-field component一次构造/validate/write，不逐field clone整组件或线性找metadata；capture按actual storage/type generation遍历。
- entities/types/fields 1/1k/100k记录probes/clone/moves/main-worker wall：compile≤1、field O(1)、per-field whole component write=0、failure authority不变。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止preview cache和apply cache两份compiled truth；必须共享同一generation plan。
- 禁止先mutate再以补偿命令回滚；publish前authority不可见。
- 禁止缓存raw adapter引用跨schema generation而无token/ownership。

## 修复结果与回传

Open state: `前向修复中`; no pass is claimed.

- 已完成：`CompiledSceneSpawn` 现在唯一拥有 target World generation、component schema catalog generation、entity remap、预写入 `NodeRecord`、已解析 component/resource adapter、dense field slot 与 remapped value；preview 与 apply 消费同一 plan，apply 在任何 type/record/component/resource 写入前拒绝 stale target 或 schema。反射 resource/component write 先在只含 schema、affected rows 与 staged affected resources 的隔离 World 预飞，target entity/component storage 不会被 clone 或修改；`PreparedDynamicSceneSpawn::{spawn_into,stage_into,stage_into_level}` 与 Level asset-reload ticket 已硬切到该 compiled transaction，worker ticket 不再持有 target `World`。
- 已完成：`NodeRecord` batch 先整体预验证，再一次发布 records、World generation 与 lifecycle；不会在已发布的 batch 中暴露半完成 record 集。
- 已完成（实现与静态复审）：预检 World 现在在全部 adapter 写入成功后提取 `PreflightedSceneMutation`，其中包含 descriptor-owned component rows、plugin JSON 和仅限计划资源的 owned rows；component descriptor 在目标 registry 重新解析，storage row 在目标 slot 重键并以 target tick 发布。`World::commit_preflighted_dynamic_scene` 在 mutation 前完成 descriptor/storage preflight，随后一次发布 identity、final archetype、资源、World generation 和 lifecycle；`PreparedDynamicSceneSpawn` 与 `LevelSystem` 已硬切为消费该 artifact，target 不再 replay component/resource adapter。新回归覆盖一次资源 adapter 写入和跨 registry slot 重键；bundle override 指标也区分 staged allocations 与 canonical publications。
- 验证状态：当前 source-bound snapshot 已由 coordinator receipt 持久化；受管 Windows CPU reservation 因 coordinator `database is locked` 被拒绝，未生成 reservation/job，故没有 Cargo pass/fail 可报告。`rustfmt --check`、scoped `git diff --check`、旧 replay/fallback symbol 扫描和独立二次源码审查已完成；二次审查对当前 hard-cut 快照确认 P0/P1 为零，bundle metric 覆盖补齐 explicit override 的 staged/canonical 差异。当前 handoff 保持 `open`，下一步是对 snapshot-matching source 重新申请声明的 managed `zircon_runtime` dynamic-scene lib test，并补齐 1/1k/100k probes 后才可返回 `fixed-*`。

### 2026-08-10 结构性性能复审与实施计划

#### 当前源码证据

- `CompiledSceneSpawn` 在 `transaction.rs` 同时持有 remap、records、resolved component/resource writes 与 preview。
  `apply_compiled_scene_spawn` 为了在 isolated preflight World 执行所有 fallible adapter writes，传入
  `plan.clone()`；成功后 `PreflightedSceneMutation` 又继续保留原完整 plan 和已经 materialize 的
  component/resource rows。带 plugin JSON 或 reflected field vectors 的 scene 在这段窗口有两份
  compiled payload，且 commit 还会 clone remap 后才消费 records。
- 这不是缓存命中问题，也不能通过缩短 adapter 生命周期或在 target replay 来修复：preflight 与 target
  必须各自保持正确职责。根因是 preflight 成功之后没有把 prepare-only payload 转换为 commit-only
  artifact，导致不再需要的 adapter、JSON、field vector、preview 和 remap 副本仍跨越 publication boundary。

#### 参考实现与裁决

- Unreal 主参考：`Engine/Source/Runtime/Engine/Private/Actor.cpp` 的
  `AActor::PostSpawnInitialize` 明确把基础初始化和最终 `FinishSpawning` 分开；deferred path 不调用
  `FinishSpawning`，而该函数以 `bHasFinishedSpawning` 保证最终 publication 只发生一次并通知 World。
  Zircon 不复制 Actor API，但采用相同的 prepare -> one final publish ownership boundary。
- Bevy 交叉参考：`bevy_ecs/src/world/mod.rs::World::spawn_batch` 明确区分批量 owned bundle publication
  与逐 entity spawn/insert；`bevy_ecs/src/bundle/tests.rs` 还对 add/insert/remove hook order 建立回归。
  Zircon 的 artifact 必须同样只在 preflight 成功后进入一次 canonical World commit，不能把 rows
  重新解释为 adapter calls。
- 裁决：不新增 public API、第二个 cache、type-name special case 或 compatibility path。新增私有
  `PreparedSceneSpawnCommit`，其唯一字段为 target-generation token、schema-catalog token、owned remap
  与 owned records；它不能包含 component/resource writes 或 preview。

#### 实施和量化门

1. `CompiledSceneSpawn::into_prepared_commit` 在 resource rows transfer 完成后消费整个 plan；extract
   artifact 改持 compact commit input，立即 drop prepare-only payload。
2. commit 只校验 compact token、安装已经验证的 descriptor、消费 records/rows，并直接返回 move 出的
   remap，删除 `remap.clone()`。所有 target write 的顺序、single generation publish、lifecycle staging
   和 typed stale errors 保持不变。
3. 增加 artifact-release counters（released component/resource write batches 与 materialized
   component/dynamic/resource rows），并继续记录现有 compile entities、dense write batches、
   `preflight_staged_bytes` 和 transaction spans。计时只在受管 `--profile profiling --features profiling`
   run 中解读；不能以未校准的 wall-clock threshold 充当 CI 断言。
4. 回归覆盖：existing write-once resource adapter 与 cross-registry rekey 保持；新增 compact artifact
   behavior gate 证明 commit 不重放 adapter 并返回原 owned remap。规模 fixture 覆盖 1、1k，100k 保持
   explicit ignored managed performance probe；采集 counter/spans、peak RSS 与一次性 publication metrics。
   接受条件是 compile/apply compile count=1、field lookup stays dense O(1)、target adapter replay=0、
   failure authority unchanged，并在相同机器/feature/profile 下报告 p50/p95 与 RSS，而非臆测功耗等价。

该设计仍是 `implementation_planned / failure_open`：没有 managed Cargo、profile export 或性能数值时，
不得将静态分析写成 acceptance。

### 2026-08-10：注册表预检增量裁决

#### 新发现的最低层成本

`World::commit_preflighted_dynamic_scene` 目前为保留失败原子性而 clone 整个
`ComponentRegistry`，再逐 row 调用 `resolve_transferred_descriptor`。该 registry 包含全部
descriptor、Rust/动态 type-id maps 与 table-column layouts；因此一个只涉及一个 row 的 compiled spawn
仍然与 target 已注册组件总数线性相关。这直接违反 PERF-MVP-472 的 affected-row / affected-type budget，
且不能由上层缩小 `CompiledSceneSpawn` 生命周期解决。

`ComponentStorage::preflight_transferred_row` 已证明 row 预检只检查目标 slot 的已有 storage/type
约束，并不需要 descriptor 已经写入 live registry；为新的 projected component id 预检是安全的。现有
full-registry clone 的唯一必要职责是“同一 transaction 中重复 descriptor 的 local id 复用、兼容性
比对及新增 descriptor/table layout 的延迟发布”，不是读取整个 registry 的业务需求。

#### 实施裁决

1. 在 `ComponentRegistry` 内引入私有的预检 descriptor delta：持有 base descriptor length、仅本
   transaction 新增的 descriptors/layouts，以及按 Rust `TypeId` / dynamic type path 的 affected-key
   index。解析时先查询 base registry，再查询 delta；重复 transfer 必须得到同一 projected id，冲突
   descriptor 必须在 preflight 返回当前 `ComponentTypeMismatch` 路径。
2. `World::commit_preflighted_dynamic_scene` 以该 delta 预检全部 row，并只在 records/rows/resources
   均已验证、下一 entity id 可用之后一次性追加 delta；不能 clone/replace whole registry，也不能把
   descriptor import 变成独立的提前 mutation。该 publish 没有 fallible work，保留现有 lifecycle 与
   world-generation 顺序。
3. 为 delta 增加 `affected descriptor imports` / `reused descriptor resolves` probes，并以 registry
   unit test 覆盖 table layout、重复 transfer、冲突 transfer 与 preflight error 不改变 base registry。
   dynamic-scene 回归要同时覆盖跨 registry slot rekey 与真实 compiled spawn 的一次发布。

#### Schema projection 边界

`dynamic_scene_preflight_world` 也复制 component-type/type registry 与 VM type sets。这是第二个
潜在的 catalogue-scale 成本，但当前不能用 `World::empty()` 直接替换：plugin component 的验证会依赖
target 已声明 type，VM backing 还依赖保留的 registration/set 语义。先补“scene 未携带 descriptor、
target 已注册 descriptor”及 VM descriptor 的等价回归，再决定采用 affected-schema projection 或将
scene document 收紧为 self-describing hard contract。无该证据不得把 schema clone 误删，也不得宣称
它已经满足 100k acceptance。

现有 `dynamic_scene_world_mutation_preserves_scene_error_source` 已锁定 metadata-only type-path conflict
在隔离预检中返回 `WorldMutation(DuplicateTypePath)` 且 target 不变。后续缩小 schema projection 时必须
复现该错误源和多 descriptor 的全量预检；不能把它粗暴提前改为新的 `ComponentTypeDescriptorConflict`，
也不得以补偿 rollback 处理。

#### 当前实施状态

- 已实现：`ComponentRegistry` 不再为 compiled dynamic-scene transaction clone/replace whole registry。
  `PreflightedTransferredDescriptorImports` 只记录 affected descriptor/layout 与 Rust/dynamic source
  identity；row 全部预检成功后，`World::commit_preflighted_dynamic_scene` 才一次 publish delta。回归覆盖
  table layout 的 deferred import、同一 transaction 的 projected-id reuse，以及冲突 transfer 不改变
  base registry。
- 已实现：`PreflightedSceneMutation` 已从完整 `CompiledSceneSpawn` 收束为 `PreparedSceneSpawnCommit`，
  只保留 target/schema tokens、owned remap 和 records。component/resource adapters、field values、preview
  与 resource write batches 在 extract 后释放；commit move 出 remap，不再 clone。新增 released/materialized
  artifact 与 descriptor import/reuse profiling counters。
- 已完成静态门：受影响 Rust 文件已 `rustfmt +1.94.1`，scoped `git diff --check` 通过；production source
  已无 `mutation.plan`、direct transferred-descriptor resolver 或 dynamic-scene commit registry clone。
- 受管验证：首个 Windows CPU reservation `302a38471d0c46a083b7327c2262018c` 在 failure record
  更新导致其 source manifest 失效后，已由 owner 在未绑定 job 前正常释放（request
  `0aaefe5159ef4fd8920c159b60e953de`）。新的 profiling counter test 已补入；随后必须基于最终七个
  owner 文件的新 manifest 重新入 FIFO，依次运行 default registry/dynamic-scene 与
  `--profile profiling --features profiling` counter test。当前没有 terminal Cargo 结果，故本记录仍为
  `failure_open`，不把静态或 reservation 结果写成 test green。

### 2026-08-10：二次审查前向修复

独立二次审查在 compact artifact 初版中确认了两项必须在 Cargo 前修复的问题，均已采用前向 hard cut
处理，未回滚已集成的 transaction 路径。

1. **P0 resource lost update**：预检复制了目标 resource，随后 `get_resource_mut` / `insert_resource`
   只推进 change tick 而不推进 `world_generation`。原 commit 只检查 schema/world generation，故预检与
   commit 之间的资源写可能被 transferred row 覆盖。`CompiledSceneSpawnTarget` 现同时绑定
   `expected_target_change_tick`；每次 publication 重新比较该 token，并以
   `DynamicSceneError::TargetChangeTickChanged` 拒绝任何预检后的写入。它是保守的 world-wide mutation
   fence，保证不会以仅按 resource type 的局部比较漏掉间接写入。回归
   `compiled_spawn_commit_rejects_a_resource_change_after_preflight` 覆盖两个不同 resource 值：commit
   返回 typed stale error，后写值与 target records 均保持不变。
2. **P1 caller-scene mismatch**：初版 commit 接收外部 `&DynamicScene` 并在 publication 前重新安装其
   descriptors，因此调用者理论上可用 scene A 预检、用 scene B 提交。`CompiledSceneSpawn` 现在在 compile
   时 only clone affected `component_types`，`PreparedSceneSpawnCommit` 取得其所有权；capture、validate、
   extract、commit 均只消费 compiled plan/artifact。`StagedDynamicSceneSpawn` 已不再持有 `DynamicScene`，
   `LevelSystem` 和 `DynamicScene` 的 commit 入口也删除 receiver scene 参数。回归
   `prepared_commit_uses_the_preflighted_component_descriptors` 在 extraction 后篡改 caller scene，仍证明
   target 只安装 artifact 内原 descriptor。

为了让 preflight 同样不再读取 caller scene，`stage_compiled_resource_writes_bounded` 直接消费 compile 时
解析的 resource adapters/type paths；旧 document-bound staging helper 仅保留给未编译的 legacy snapshot
utility，不能进入 compiled transaction。新增目标 token 与 descriptor list 都只按 affected scene
content 增长，不恢复全 registry clone 或 adapter replay。

当前静态门（`rustfmt +1.94.1 --check`、scoped `git diff --check`、旧 receiver/call-site source scan）已通过。
二次审查复核与 final source-manifest 的 managed Windows Cargo 尚未有 terminal receipt；因此 lifecycle
仍为 `failure_open`，不得以本段或单元测试代码声称 green。

该 final snapshot 的 focused Windows `--lib`
`compiled_spawn_commit_rejects_a_resource_change_after_preflight` reservation 申请返回 coordinator
`database is locked`；未创建 reservation/job/run，因而它是协调器维护证据，不是 Rust RED 或验证终态。按
receipt-driven 规则，不复用或通用重试该请求；待协调器恢复后必须以当时 source manifest 新建受管申请。

### 2026-08-10：二次审查 remove-resource 补正

复审随后指出 `World::remove_resource` 是该 mutation fence 的遗漏路径：它直接从 `ResourceStore` 删除值，
但此前不推进 `ChangeTick` 或 world generation。于是 preflight 后删除 resource 仍可能让 commit 的 token
检查通过，并把旧 transferred row 插回 target。该问题归属于 shared typed World API，而不是 dynamic-scene
call-site；已在 `remove_resource` 成功移除后推进 `mutation_change_tick()`。缺失资源仍是无 tick 的 no-op，
避免无 mutation 时错误地使 compiled plan 失效。

`compiled_spawn_commit_rejects_a_resource_removal_after_preflight` 现在覆盖该链路：resource 被 stage、
preflight/materialize，随后在 target remove；commit 必须返回 `TargetChangeTickChanged`、不发布 entity，且
resource 保持 absent。该 test 和此前 mutable-resource test 都尚未取得 managed Cargo terminal evidence。

### 2026-08-10：补正后二次独立复审

只读复审以当前 hard-cut source 为准，结论为 P0=0、P1=0：成功 `remove_resource` 已推进
`mutation_change_tick`，artifact 在 publication 前比较该 token；descriptor list 由 compiled plan move 入
opaque commit artifact，commit 只接收 `World + mutation`，且 staged task 不再保留 caller `DynamicScene`。
复审建议的 staged removal 覆盖已补为
`staged_compiled_spawn_rejects_a_resource_removal_before_commit`：它走
`PreparedDynamicSceneSpawn::stage_into -> StagedDynamicSceneSpawn::commit_into`，仍要求 typed stale error、
零新 entity 和 absent resource。plugin descriptor 行矩阵也已补为
`prepared_commit_keeps_plugin_rows_bound_to_its_preflighted_descriptor`：它在 preflight 后替换 caller scene
的同 ID descriptor，仍要求 artifact descriptor、entity remap 和 plugin JSON row 一起发布。下一次 managed
validation 必须覆盖三条 stale-resource 回归与该 plugin-row fixture；任何通过结论仍以 coordinator 的 terminal
job/run receipt 为准。

最终独立只读复审已覆盖这两条补充矩阵以及 shared `remove_resource` token 语义，结论仍为 P0=0、P1=0。
该结论不替代 managed validation，也不将当前 `failure_open` 升为 fixed。

### 2026-08-10：受影响 schema 投影与复审补正

`dynamic_scene_preflight_world` 已从完整 clone target `ComponentTypeRegistry`、`TypeRegistry` 和两套 VM
type-path set，硬切为按 compiled mutation 的 affected type paths 点投影。输入先经 target `TypeRegistry`
归一化 canonical full path 并用 `BTreeSet` 去重；随后仅 point lookup/clone 对应 component descriptor、runtime
registration、VM catalog membership 与 VM dynamic membership。target entity/component rows、resources、callbacks
与 runtime queues 仍不进入隔离 World。复杂度因此由 target catalog size 收束为 affected type count 的
`O(K log N + K log K)`，没有引入第二 registry owner、compatibility fallback 或全 catalog iterator。

独立只读复审先发现两个 P1，均已前向修复：

1. 稀疏投影不能把“target component catalog 非空”误变成 permissive empty-catalog。capture 现在在投影前保留
   strict catalog invariant：plugin type 必须已由 target 注册，或由本次 compiled scene descriptor 明确声明；
   否则仍返回 `WorldMutation(UnregisteredDynamicComponentType)`，不创建预检 entity，也不发布 target mutation。
2. plugin write 在 compile 时通过 target `TypeRegistry` 将唯一 short alias 归一化为 canonical path；descriptor、
   runtime registration、VM membership、preflight write 与最终 dynamic JSON row 因而使用同一 identity。
   ambiguous/unknown alias 保持原 fail-closed 或 empty-catalog 宽松边界，不增加猜测式解析。

对应回归现已覆盖：target-only VM descriptor 且 scene 不携带 descriptor、short alias compiled write、unselected
target descriptor 不进入 preflight、strict catalog 拒绝未知 plugin type，以及 strict target catalog 接受 scene
自带 descriptor。direct projection 还以 `sync_vm_types` 建立 VM catalog ownership，并同时输入 short/full path；
若 canonical 去重退化会在重复 descriptor 注册时立即失败。profiling 专项测试要求
`affected_type_paths`、projected component descriptors、runtime registrations、VM catalog paths 和 VM dynamic paths
五个 counter 的值均精确为 `1`，防止 counter 仅存在但重新统计 full catalog。

当前 `rustfmt +1.94.1 --check` 与 scoped `git diff --check` 通过；生产路径扫描未发现本次投影重新引入
component/type registry full clone。补正后的最终独立只读复核为 P0=0、P1=0、P2=0：canonical 去重、strict
与 scene-declared catalog 语义、VM catalog/dynamic ownership，以及五项 exact counter 均无遗留审查项。
managed default/profiling Cargo 仍待 coordinator terminal receipt；因此本 handoff 继续保持 `failure_open`，
不以静态检查、测试代码或审查结论声称 green/fixed。

### 2026-08-10：最终源码闭包与受管验证收据

- Runtime08 已通过协调器 ownership transfer 接收 57 个现存的 component storage、resource store、
  archetype、query-order、World 与回归测试路径（fingerprint
  `eef4b0fac017e34b27024812b2ff8278a0d96b429636e54668d18f7b0ef9f696`），并另行接收 4 个
  迁移后的 persistence 测试文件（fingerprint
  `d50d81d48a1a825f2f7706a9e552d2387918250c617b64a0eeb31f65696aae9c`）。已删除的旧
  `scene/spawn.rs` 通过 live lease 记录为 deletion attribution；Runtime09 持有的
  `world/render.rs` 保持外部集成依赖，Runtime08 未抢占或修改其归属。
- 最终 default-feature Windows `--lib` focused reservation 已持久化：request
  `c34be11ba70e41219c0a147166e48dcc`，command fingerprint
  `16e0c7205bd617d58d800153cd905bde25dab1bbe10d94aca473a7fddc58159e`，source manifest
  fingerprint `a660ab36aa424024ae21280f34afd12b7709e82b690ba65364588161c27adbcb`。精确命令为
  `cargo +1.94.1 test -p zircon_runtime --lib dynamic_scene::scene::spawn::transaction::tests --locked --jobs 1 -- --nocapture --test-threads=1`，
  target 位于 `E:\cargo-targets\runtime08-compiled-spawn-r1`。
- receipt 时 reservation 为 `pending` 且未绑定 job。按连续里程碑规则，queued/running validation 不阻塞
  Session 实现和二次审查；在 coordinator wakeup 提供 terminal job/run 前，本 failure 继续保持 `open`，
  不把 reservation、静态审查或测试源码写成 Cargo green。

### 2026-08-13：live publication 原子边界硬切

- compiled artifact 现在同时绑定 target world、World generation/change tick、component type catalog、resource schema 和 `ComponentRegistry` generation。preflight 后新增或替换 component descriptor 会返回 typed stale error，不进入 target publication。
- commit 不再在 live World 上逐项安装 descriptor 后继续执行可失败解析。`preflight_dynamic_scene_publication` 以一个 `PreflightedTransferredDescriptorImports` delta 统一预留 target-local component IDs，校验 component catalog、runtime `TypeRegistry` registration、plugin full path、row storage type、resource row 和 next entity id；同 batch descriptor 可复用同一未发布 local ID，不创建第二 registry owner。
- `publish_preflighted_dynamic_scene` 是 preflight artifact 的唯一 live publication owner：先发布 descriptor import delta，再发布 component/runtime catalog、identity、resource、dynamic JSON 和 final archetype rows，最后统一 lifecycle/generation。该函数不返回 `SceneResult` 且无 `?`，因此 authority 首次变化之后不存在 recoverable error 分支；旧 `commit_preflighted_dynamic_scene -> SceneResult<()>` 已硬切删除。
- 新增 source/behavior guard 覆盖 unpublished dynamic ID reservation、stale component-registry token，以及 commit 函数的 preflight-then-infallible publication 形状。精确 `rustfmt +1.94.1` 与 scoped `git diff --check` 已通过。一次受管 focused 提交在 admission 前返回 `cargo_reuse_pool_busy`，没有创建本切片 reservation/job/run；这是共享验证基础设施状态，不是 Rust RED 或 GREEN。本 failure 继续保持 `open`，等待 coordinator wakeup 后的 current-source terminal evidence。
