---
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/context
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-authoring-transaction-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-context-service-composition-current-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/ScopedTransaction.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorEngine.cpp
  - dev/Fyrox/editor/src/command/mod.rs
doc_type: implementation-evidence
status: static_current_revalidated_dynamic_blocked_structural_cutover_required
---

# Editor Editing与Context currentness复核（2026-08-23）

## 当前冻结与结论

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/editing/**` | 29/29 | 4,846 | 158,609 | 13 | `6da41a2effdd340640359473521a145e8d644fd442763577d8955f6218ab15b4` |
| `zircon_editor/src/core/context/**` | 5/5 | 1,384 | 49,997 | 14 | `7b2b71e697e45c7ff2a67ba8d4c189221b5a09c44025d86bb9ea8605664db20d` |

- 34/34当前Rust文件和生产调用锚点已逐文件复核，并与8月16日基线、`7a20f921b`及当前
  worktree差异对账。
- Context旧的构造不匹配已经关闭：builder创建共享`Arc<EditorLogService>`，从同一
  `EditorJobSystem`构造autosave，并按新签名发布完整`EditorContext`。
- 当前并发改动把tool topic从每次`publish_events`解析一次改为服务构造时解析一次；稳定操作
  的topic parse为`1 -> 0`，但每事件topic/event clone和同步bus fanout不变。本轮保留且不认领该改动。
- 当前并发改动还把delete undo从完整`Vec<NodeRecord>`克隆切到Runtime move-only
  `DetachedEntityBatch`，并让selection失败走`Applied`回滚；该方向保留，但新增camera不变量阻塞见下文。
- `editing/paths.rs`的并发变化只是typed error和4项测试；同步canonicalize/is-file及extension
  lowercase仍在import admission，本轮不把它登记为性能收益。
- 主要P0仍未改变：非原子多步scene mutation、全局operation槽、无deadline scope等待、
  selection-only exclusive rollback和count-only history。当前没有可运行的current-source editor，
  因此本模块继续pending，不声明latency、CPU、RSS、power或算法最优。

## Editing 29/29逐文件复核

| 文件 | current-source结果 |
|---|---|
| `authoring_world.rs` | Gateway边界正确；callback仍让调用者在一次lease内执行任意宽工作，缺immutable generation与batch receipt。 |
| `command.rs` | Delete已用move-only detached batch消除完整subtree record clone；P0仍在parent/name/transform的3个独立fallible mutator，且新capture会允许一个subtree删除全部多个camera。 |
| `context.rs` | transaction mutex外执行world callback值得保留；原子性仍由每个command自行保证，owner没有all-or-none batch合同。 |
| `engine/command.rs` | typed effect/error/merge面紧凑；`Applied/Unchanged`由command猜测，不能表达partial commit。 |
| `engine/events.rs` | compact event且锁外发布；应继续升级为exact changed-range/generation receipt。 |
| `engine/history.rs` | `VecDeque`按128条目有界、detail分页；没有byte/resident/age预算，journal按transaction线性查找。 |
| `engine/journal.rs` | journal不是normal-edit eager path；显式projection仍拥有完整selection vectors、command JSON和strings。 |
| `engine/mod.rs` | 导出与默认容量；128 entry不是内存上限。 |
| `engine/routing.rs` | typed history route为常数匹配，无独立热点。 |
| `engine/transaction.rs` | 已收束为facade；底层仍共享一个context、mutex、condvar和process-wide operation marker。 |
| `engine/transaction/dirty_batch.rs` | 4,096 generation journal/cursor有界；显式delta的`BTreeSet`可接受，禁止frame polling。 |
| `engine/transaction/engine_state.rs` | 一个state集中context、histories、active scopes、operation group和唯一operation槽；阻断独立document preparation/query。 |
| `engine/transaction/exclusive_transition.rs` | P0：closure可修改任意typed context，失败只restore selection，不能证明world/project状态回滚。 |
| `engine/transaction/lifecycle.rs` | callbacks/finalize/event避开state lock；failed command仅在effect=`Applied`时revert，无法修复partial `apply_node_state`。 |
| `engine/transaction/operation_gate.rs` | admission立即返回busy；`wait_for_operation`无deadline，且一个槽串行化所有history/control-plane操作。 |
| `engine/transaction/operation_group.rs` | gesture merge方向正确；flush/cleanup仍进入全局槽，apply/undo/redo仍逐command进入world。 |
| `engine/transaction/replay.rs` | replay锁外执行context mutation；journal在state lock内线性定位并构造owned projection，read-like API会先flush group。 |
| `engine/transaction/save_token.rs` | compact generation compare-and-mark应保留，并并入authoring commit receipt。 |
| `engine/transaction/scope.rs` | P0：cancel、commit、commit-after-apply、Drop共4处busy循环；timed wait/deadline为0，`!Send` scope可卡主线程。 |
| `intent.rs` | 小型typed declaration；shell锁内工作应止于intent与expected generation。 |
| `mod.rs` | 模块挂载，无独立热点。 |
| `operation/command.rs` | boxed command bridge继承wide payload和per-command replay；目标应是typed prepared batch。 |
| `operation/error.rs` | typed conversion，无独立热点。 |
| `operation/factory.rs` | construction factory，无独立hot path。 |
| `operation/mod.rs` | 导出，无独立工作。 |
| `operation/pending_edit_retention.rs` | lossless/latest/bounded policy声明合理；history自身仍缺bytes/age/resident admission。 |
| `operation/registration.rs` | 一次freeze metadata/factory/policy；construction-time string ownership不属于edit热路。 |
| `paths.rs` | 当前typed error保留diagnostic；extension lowercase分配和同步filesystem query仅允许在explicit import admission，不得进入frame path。 |
| `selection.rs` | `Arc`共享payload避免旧JSON/full-copy；selection snapshot不能冒充通用world rollback。 |

## Context 5/5逐文件复核

| 文件 | current-source结果 |
|---|---|
| `builder.rs` | 旧log/autosave构造阻塞已关闭；F0仍是未分阶段计量的同步assembly，log/i18n JSON和bus fanout在producer path，resync dropped/delivered membership为`O(D*R)`。 |
| `builder/quota_startup_tests.rs` | 覆盖settings job limits与多context，不覆盖stage receipt、forced rollback、唯一owner identity及F0 allocation/I/O预算。 |
| `editor_context.rs` | typed aggregate和borrowed/shared accessors正确；应保持单一service owner，禁止退化为反射service locator。 |
| `mod.rs` | 导出shell，无独立热点。 |
| `tool_scheduler.rs` | scheduler mutex在publish前释放且queue有界；topic parse稳态`1 -> 0`，逐事件clone和同步fanout仍需PERF-MVP-019 owner处理。 |

## 并发M0复核：move-only delete方向正确，但camera门退化

新`DeleteNodeCommand`只保留`Option<DetachedEntityBatch>`，Runtime detach/restore移动exact erased rows、
component ticks、dynamic values和observers，不再在capture时构造完整`Vec<NodeRecord>`。Runtime静态fixture也
要求full-world clone bytes和node-record clone bytes为0；这些Rust测试尚未在本会话执行，不能作为动态数据。

当前capture条件是`camera_count == 1 && subtree_contains(active_camera)`。当一个父subtree同时包含场景中
两个或更多全部camera时，该条件为false，detach成功后Runtime会把active camera设为
`first_stable_camera_entity().unwrap_or(0)`，从而留下0 camera/active=0。现有multiple-camera测试删除的是
多个独立root，第二个command在前一个apply后才看到count=1，未覆盖单一父subtree。

不得恢复`subtree_records()`来计数。Runtime08应提供只读`DetachedEntityBatchPreflight`或generation ticket，
包含affected entity/camera counts和normalized roots；Editor05在commit前验证remaining camera>=1，随后同一
generation执行move-only detach。新增“父节点含全部2+ cameras”的行为测试，拒绝后world/selection/history/
generation digest不变。失败记录：
`failure-2026-08-23-editor-delete-subtree-all-cameras-invariant.md`。

## 结构性瓶颈与算法判定

### P0：mutation不是一个可证明的原子算法

当前`apply_node_state`顺序调用parent、name、transform三个mutation。三步中任一步可失败，owner没有
prepared inverse或generation swap，错误又统一映射到`Unchanged`。这不是优化一两个clone能解决的热点，
而是事务算法错误。目标必须是：

`ImmutableAuthoringGeneration -> PreparedEditBatch(forward + inverse) -> one validated commit -> exact receipt`

prepare阶段解析目标、层级环、schema、权限和changed fields；commit只复验generation并一次安装全部delta。
拒绝、stale和fault必须使提交前后完整world digest一致。Undo/redo走同一batch commit路径。

### P0：一个全局operation槽把控制面与authoring互相阻塞

静态shape为：operation slot **1**、scope busy-wait branches **4**、timed/deadline wait **0**。
正确方向不是让多个线程同时写同一个world，而是immutable generation上并行prepare，按document/history lane
有界排队，主线程/authoring owner只做短commit；status/journal/inspection读取共享generation，不进入mutation槽。
`Drop`只释放reservation并记录terminal cleanup，不允许等待、I/O、plugin callback或world mutation。

### P0：history admission不随真实成本缩放

当前byte-budget字段 **0**。一个deleted subtree或large reflected value可占满RSS但只算1条记录。history必须同时
约束entries、owned bytes、resident resources和age；大tombstone由immutable artifact owner持有。eviction只访问
被淘汰记录，finalize在owner lock外完成。journal需先取得immutable record handle，再在lock外按bytes/deadline
编码；transaction lookup不应每次扫描整个deque。

### P0：Context启动与事件fanout缺少阶段归因

构造类型正确不等于F0性能正确。Context assembly仍把settings I/O、quota、11类service construction、wiring和
publication合在同步路径。目标是显式`Definition/CoreOwners/ServiceOwners/Wiring/Publish/Activate` receipt；只把
独立阻塞I/O交给Runtime11，并带dependency、deadline和cancel。消息topic缓存应保留，但真正的steady-state
成本是JSON materialization、subscriber fanout、backpressure集合比较和event clone，不应继续微调parse。

## Unreal主参考与适配边界

- `ScopedTransaction.h:10-36`提供lexical Begin/End与显式Cancel。Zircon保留scope ownership，但析构不得
  隐藏无界等待。
- `TransBuffer.h:22-37,68-121`同时维护UndoBuffer、ActiveCount和`MaxMemory`，按`DataSize`淘汰旧事务，
  直接支持entry+byte双预算；不支持照搬UE全局对象模型。
- `EditorTransaction.cpp:178-350,701-742,814-900,1091`以object/range record保存可翻转状态，按确定顺序
  save/apply/swap并计算DataSize。Zircon应采用prepare/inverse/exact range与batch顺序，不复制UObject序列化。
- `EditorTransaction.cpp:1289-1459`显式管理Begin/End/Cancel；其同步modal/global路径不是Zircon主线程等待依据。
- `Fyrox/editor/src/command/mod.rs:195-210`group正序execute、逆序revert/finalize，`:215-280`按条目容量
  清理。它佐证确定顺序，但无fallible atomicity和byte budget，不能作为当前算法已正确的证据。
- `LaunchEngineLoop.cpp`和`EditorEngine.cpp`把启动分成有名、可计量的依赖阶段。Zircon应采用阶段receipt，
  不复制UE globals或“每服务一个线程”。

## Hard-cut计划目标

1. Editor03/05定义`PreparedEditBatch`、field-specific forward/inverse delta、expected authoring generation和
   one-commit receipt；删除partial-effect猜测与任意mutable-context closure。
2. Editor03把全局operation/Condvar scope改为document/history lane admission；prepare可调度，commit短且唯一；
   public completion立即返回typed busy/stale/ticket，Drop wait=0。
3. Editor03增加entries+bytes+resident+age history admission、indexed immutable record handle和lock外journal。
4. Editor01/Runtime07发布immutable authoring generation及exact hierarchy/transform/reflection/selection/render delta；
   stable frame world lease、scene visit和plugin reflection均为0。
5. Editor00/01/17把Context assembly拆成可失败、可回滚、有stage receipt的依赖图，只发布完整context。
6. Editor02/08把message construction、zero-target fast path、bounded fanout/backpressure receipt集中到唯一bus；
   tool scheduler只发布compact lifecycle fact，不创建第二queue/executor。
7. Editor14/Runtime11只调度prepare、serialization和独立I/O；不得把同步wait包装为worker后再让UI等待。
8. EditorUI08从commit generation消费affected delta；一个edit最多产生一个inspection/render generation和一次提交。

## 验收矩阵

| gate | matrix | 必须满足 |
|---|---|---|
| atomicity | parent/name/transform/reflection/N-command每阶段fault，batch `1/128/10K` | rejected/stale/fault前后world/selection/history/dirty/render digest完全相同；partial effect=0 |
| waits/locks | edit、undo、redo、project swap；1/128/100K nodes | UI no-deadline wait=0、Drop wait=0、authoring commit lease=1/batch、stable reads不进入operation槽 |
| retention | payload `16B/4KiB/1MiB/256MiB`，history `1/128` | entries/bytes/resident/age全有界；eviction `O(evicted)`；serialization不持engine owner lock |
| startup | no-project/MVP project，workers `1/4/16`，每stage forced failure | owner各1个、无partial publication、stage wall/CPU/alloc/I/O/messages/worker starts完整 |
| fanout | subscribers `0/1/100/10K`，events `0/10/10K` | zero-target不构造delivery；bytes/count/age有界；stable tool topic parse=0 |
| product | cold/warm F0、idle/edit/undo/save/close，至少31个可比样本 | WPR CPU/waits/locks/CSwitch/File I/O、allocator/RSS、package power报告p50/p95/p99/CI/effect size |
| render | authoring commit到首个呈现帧 | CPU generation与RenderDoc frame相关；一次edit无重复extract/submit，GPU parity无回归 |

RenderDoc只能验收最后一行的draw/pass/GPU与像素一致性，不能证明transaction、history、startup或power问题。

## 当前验证回执

- scope/read/fingerprint：GREEN，Editing 29/29（4,846 lines、13 tests）、Context 5/5。
- source-shape：operation slots `1`、busy wait branches `4`、timed waits `0`、history byte fields `0`、
  node mutators `3`、journal linear find `1`。
- Python source contracts：22项中16通过、6失败；Tool scheduler相关9/9通过。6项失败都位于旧
  `test_editor03_scene_transaction_hardcut_contract.py`的路径/字符串/结构断言，需owner按当前入口和typed
  error合同更新，不能记为Rust产品行为通过。
- rustfmt：未被并发修改的33/33文件GREEN；并发`editing/paths.rs`仅有import排序RED，本轮未改写。
- scoped `git diff --check`：GREEN。
- docs convention：本轮新增文档owned violations `0`；仓库全局为3,129 documents、275 affected、
  801 existing violations，故全局门仍RED。
- Rust/Cargo/WPR/allocator/power/RenderDoc：未执行；当前managed Windows会话不可执行且无current-source
  editor executable。禁止用旧binary采样或伪造性能数据。
- 受保护`review.md`、`pending.md`和编号计划未修改；动态门通过前不提交里程碑、不发送企微完成通知。
