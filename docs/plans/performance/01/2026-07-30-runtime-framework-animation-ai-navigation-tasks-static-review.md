---
related_code:
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/ai
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/core/framework/audio
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/framework/tasks
  - zircon_runtime/src/core/runtime/tasks
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_app/src/task_pool_plugin.rs
  - dev/godot/scene/3d/navigation/navigation_agent_3d.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Pipe.h
tests:
  - framework Rust source rustfmt check passed for 84 files
  - current-source Cargo and product-scale traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime framework animation、AI、navigation 与 tasks 静态审查（2026-07-30）

## 范围与当前源边界

本轮逐文件读取当前源 94/94 个 Rust 文件、8,951 行、30 条 `#[test]`：framework animation 37/37（3,072 行、5 tests）、AI 9/9（1,026 行、0 tests）、navigation 19/19（1,645 行、9 tests）、audio 3/3（331 行、2 tests）、time 6/6（307 行、0 tests）、tasks 10/10（254 行、0 tests）以及 runtime tasks 10/10（2,316 行、14 tests）。当前路径排序后以 `path=SHA256` 再聚合的完整指纹依次为 `fcc8783f3b367508bc763d338049c1167b635db1a9d315e79acb19279b7d47fd`、`728b27d860e1e391c40adc0d8c61874b072a07155eb5dc692493a8f957ebe268`、`ba3990ef1c8ebd41a5e82ba011db0e081f371e03a59a0dfc89c06df4e7639a3f`、`5accee45310dfbf9e3eb016557628dfd0e970661b70639d8ea962091287354dc`、`482e58a20a98f9a87eccd727107af6935ed40a8be23d831f23afca980931efa7`、`d30d7566d4bf355005cbde0986888bb9a7b27240d2794335e3f864eea7cd6796`、`e8851f1544ad45859248869232320e4a7a5c2a0e99def1bdd72608e7c25abef3`。

animation、navigation 与 framework tasks 当前 clean；AI、audio、time 和 runtime tasks 含其他 owner 的共享未提交改动，本轮只读并按当前内容取证，未修改任何 Rust 文件。framework 84 个文件的 `rustfmt --check --edition 2021` 通过；runtime tasks 的同一检查只报告现有 `mod.rs`、`report.rs`、`timer.rs` import 排序差异，因此不能把整组格式门记为 GREEN。

## 已确认瓶颈

### PERF-MVP-581：animation 资源写入整包复制与兼容解码多遍扫描

`asset/binary.rs::encode_binary_asset` 要求 `T: Clone` 并把 `payload.clone()` 放进 document；clip/state-machine 等调用方已先从公开资产构造一份 owned binary DTO，随后又在通用 writer 深拷贝一次。大 clip/graph/state-machine 写入可同时持有 source、binary DTO、document clone 与最终 bytes。解码先尝试完整 document，再回退 header+payload stream；v1 或 v3/v2/v1 fallback 对同一 bytes 重复调用该双路径，失败或旧资产可被多次完整遍历。

计划让 current writer 借用 payload 并直接写最终输出 owner，在 header 中冻结可单次分派的 payload schema revision；current reader只做一次 header 与一次 payload decode，旧格式迁移走有 byte/node/depth 上限的单一 bounded owner。不能用扩大缓存掩盖重复遍历，也不能在编辑器保存线程保留整资产副本。

### PERF-MVP-582：animation 热投影重复克隆和字符串匹配

`AnimationSequenceAsset::{track_paths,target_track_paths}` 先克隆每个完整 track，再只取 property path，并逐 track 克隆 entity/target String；`AnimationRuntimeStatus::sanitized_snapshot` 先深 clone 全 snapshot，又逐 player 调用会再次 clone 的 `sanitized_snapshot`；avatar mask 每次 target 判定分别线性扫描 included/excluded String 并重复 `rsplit('/')`。`AnimationManager` 的 graph/state/status API还以 owned parameter map、clip Vec和完整 runtime status作为常规返回边界。

Plugins04 应把 track target、mask、graph traversal与parameter slot编译为 generation-owned dense artifact；运行状态在发布时一次 sanitize并共享 immutable snapshot，稳定 generation 的 editor/debug consumer只拿 handle或delta。此任务复用 PERF-MVP-329/440 的 compiled binding/evaluator owner，不允许在 framework 再建第二套私有缓存。

### PERF-MVP-583：navigation gizmo 每次重建三角形和重复共享边

`NavMeshAsset::debug_triangles()` 先为全部 triangle 物化 DTO Vec；`NavigationGizmoSnapshot::to_scene_gizmo_overlay()` 再为每个三角形追加三条独立 line command，邻接三角形共享边被重复输出，off-mesh link 还追加 line 与 pick shape。接口没有 generation、visible tile/selection范围或 command/byte budget，稳定 navmesh 的编辑器 overlay 也可能重复 O(T) 构建并产生约 3T commands。

Plugins05 M6 应在 navmesh generation 更新时建立 persistent indexed unique-edge geometry，隐藏 overlay 不做工作，显示时只投影可见/选中 tile并服从 command/byte/LOD预算。Godot `NavigationAgent3D` 以 `debug_path_dirty` 只在路径或调试配置变化时重建持久 mesh，可作为 owner/失效语义参考。

### PERF-MVP-584：AI contract 强制 owned 全量 runtime/debug 快照

framework AI trait 的 behavior tree/schema/blackboard/runtime snapshot 查询均返回 owned Vec或完整 snapshot；tick request又拥有整份 blackboard/perception，debug frame拥有 report、tree、blackboard、perception和多组 String。该目录当前 0 个测试，单看中立 DTO 不能证明每帧一定发生复制，但 Plugins06 M5 若直接把这些接口用作 editor mirror，会把 agent/node/key/stimulus规模线性复制到运行线程。

Plugins06 应保留 owned DTO作为显式序列化/抓取边界，生产 tick与调试流改用 compiled tree/blackboard generation handle、changed slot/node event和有 entry/bytes/age预算的镜像；只有显式 full snapshot 才物化全量 DTO，并测 observer stall。此项为 P2，不能先于 runtime/editor MVP 热区抢占实现队列。

### PERF-MVP-585：任务终态与 timer 回调在完成线程无预算串行执行

`JobState::publish_terminal` 在完成任务的线程同步遍历全部 dependency continuations和terminal observers；continuation可立即让下游 handle terminal并再次进入 `publish_terminal`，深依赖链存在递归栈增长，宽 fan-out或慢 observer会独占完成 worker。`TaskTimer` 虽把注册数限制为512，但所有同 deadline callback仍在唯一 timer thread逐项直接执行，单个慢 callback会延迟后续所有生命周期 deadline。

Runtime11 应把 ready continuation/observer以trampoline或明确 affinity 投递到有 count/time/age预算的统一 lane，保持依赖、panic containment和exactly-once顺序；timer线程只移动到期ticket，不执行未知时长工作。验收扩展现有64层正确性测试到1/100/10k chain/fan-out与0/1/100 ms observer/callback，记录递归深度、completion-thread wall、ready age和deadline lateness。

## 未立独立任务的目录

framework audio 只做小型channel-layout值校验，time为O(1) Duration算术，framework tasks为显式pool/budget DTO；本轮未发现需要脱离既有 PERF-MVP-317/Runtime11 的独立热点。runtime三池在低核机器因每池 `min_threads=1` 产生有意oversubscription，与Bevy参考实现一致；是否调整必须先用WPR证明上下文切换/吞吐损失，并同时报告预算逻辑核与实际worker总数，不能仅凭线程数直接改策略。

## 动态验收缺口

1. animation：1 KiB/64 MiB资产、1/1k/100k tracks/nodes/bones与current/v1/v2/v3，记录完整decode passes、clone/copy bytes、peak owners、mask/path visits和save/load p95。
2. navigation：1/1k/1M triangles、0/50/100% shared edges、hidden/selected/stable generation，记录triangle DTO、unique edges、commands/bytes、build count和editor frame p95。
3. AI：1/100/10k agents/nodes/blackboard keys/stimuli及observer stall 0/60s，记录snapshot owners、clone bytes、event entries/bytes/age/drop和runtime tick p95。
4. tasks：1/100/10k chain/fan-out、same-deadline callbacks和1/2/N workers，记录stack depth、worker monopolization、timer lateness、diagnostic RMW与WPR context switches。

本轮未创建或运行 Cargo reservation，也未运行RenderDoc：这些目录没有GPU提交路径，RenderDoc不能替代CPU/内存/线程动态门。当前只具静态与rustfmt证据，全部仍留在`pending.md`，不进入`review.md`。
