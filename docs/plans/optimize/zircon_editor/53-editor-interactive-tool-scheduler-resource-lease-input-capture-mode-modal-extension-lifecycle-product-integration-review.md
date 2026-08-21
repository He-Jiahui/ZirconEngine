---
related_code:
  - zircon_editor/src/core/tools
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/editor_message/message/tool.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
  - zircon_editor/src/core/editor_message/retention.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/scene/modes/scene_mode_activation.rs
  - zircon_editor/src/scene/modes/scene_mode_stack.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_apply_command.rs
  - zircon_editor/src/ui/host/editor_scene_mode_lifecycle.rs
tests:
  - zircon_editor/src/core/tools/tests.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InteractiveToolManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InputRouter.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InputRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InteractiveToolsContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolsContext.cpp
  - dev/Fyrox/editor/src/interaction/mod.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/plugins/editor_plugin.cpp
  - dev/godot/editor/plugins/editor_plugin_list.h
  - dev/godot/editor/plugins/editor_plugin_list.cpp
  - dev/bevy/crates/bevy_picking/src/lib.rs
  - dev/bevy/crates/bevy_picking/src/input.rs
  - dev/bevy/crates/bevy_picking/src/pointer.rs
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Utilities/GenericEditorTool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/LightPlacementTool.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 53 · Editor Interactive Tool Scheduler / Resource Lease / Input Capture / Scene Mode / Modal / Extension Lifecycle 产品集成工程化差距

## 1. 结论

Zircon Editor并非完全没有交互工具基础。`ToolScheduler`已经具备规范化非空资源集合、单资源FIFO、集合原子申请、撤队、`release_all`、typed outcome与typed lifecycle event；`SceneModeStack`也有base/overlay、enter/exit、input consumed/pass-through、update、overlay build与shutdown。现有43项聚焦测试能够证明若干局部行为，而不是只有空接口。

但是这两套基础目前彼此分离，也没有形成产品工具系统。全量production caller反查只找到`EditorContextBuilder`构造`ToolSchedulerService`；没有任何production代码调用`context.tools()`、`acquire*`、`release*`或订阅`editor.tool`。Viewport输入、Scene Mode切换、Gizmo drag、Modal与Export均绕过租约服务。`ExclusiveResource::{ViewportInput, ModalSurface, SceneModeSlot}`因此只是三个未被产品消费的枚举值，不能证明编辑器具备冲突仲裁。

调度器本体也存在会破坏authority的硬错误：同一个`ToolId`用不同集合再次成功申请时，`active_sets.insert()`覆盖旧集合，但旧资源holder仍保留，使旧集合的`release_set`失效并允许按single resource部分拆除集合占用；服务又在释放scheduler mutex后逐条发布事件，另一线程可先完成反向状态变更和发布，使观察者收到与最终authority相反的顺序，同时所有dispatch failure/backpressure均被丢弃。把现有服务直接接入产品会把潜伏缺陷升级为输入锁死、模式冲突和插件卸载泄漏。

本报告登记 **3项P0、48项P1、12项P2与36个资格门**。Editor53唯一拥有交互工具type/instance、resource lease、input capture、activation/deactivation、terminal disposition、extension unload cleanup与真实产品接入；Editor03继续拥有Scene/Prefab/Selection/Gizmo业务语义，Editor09拥有后台job/export operation，Editor48拥有message bus传输与retention，Editor50拥有extension contribution/reload总生命周期。

## 2. 审查边界、currentness与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 测试 | 证据等级 | 本轮检查重点 |
|---|---:|---:|---|---|
| 聚焦Zircon源码 | 20 / 4,910 / 170,594 | 43个`#[test]`、0 ignore | E3 | scheduler、context service、event、Scene Mode、Viewport产品调用链逐文件检查 |
| `core/tools/**`核心 | 4 / 1,336 / 41,588 | 14个`#[test]` | E3 | identity、资源集合、队列、晋升、释放与事件不变量 |
| 产品反查 | 全量`zircon_editor/src` | 0个真实acquire/release caller，0个`editor.tool` subscriber | E3 | 构造到消费、输入到模式、关闭到清理的可达性 |
| 参考源码 | 19 / 9,893 / 369,905 | Unreal/Fyrox/Godot/Bevy/Unity Graphics | E2/E3 | tool instance、capture、focus、shutdown、plugin removal与阶段排序 |

20份聚焦源码按normalized relative path排序，写入`path + NUL + raw bytes + NUL`后取SHA-256，fingerprint为`c77effa7d049d8d16f5b49174332c4dce48b31206934223aaddd09cf4026644c`。19份参考源码fingerprint为`1aeb239eee2e721c561b2f9b00f199306274f5ad8403660443778236249c3871`。冻结Git基线为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

`scene_viewport_controller_accessors.rs`与`editor_state_viewport.rs`存在非本轮产生的在途修改，本轮按working tree内容审查并保守设置`source_recheck_required: true`。实施前必须重取20文件fingerprint、全量caller矩阵及两份在途文件，不得回退共享工作树。

### 2.2 产品可达性证据

| 事实 | 全量production反查 | 工程结论 |
|---|---:|---|
| `ToolSchedulerService::new` | 1处，`core/context/builder.rs` | 每个EditorContext都构造服务 |
| `context.tools()` / `.tools()` | 0处 | 没有产品consumer |
| `acquire*` / `release*` / `withdraw*` | 仅service与tests | 没有产品租约生命周期 |
| `TOPIC_TOOL` / `editor.tool` | 定义与re-export；0 subscriber | lifecycle event没有产品观察者 |
| `ModalSurface` | scheduler、context import与tests | 真实modal没有owner/caller |
| `SceneModeSlot` | scheduler与tests | SceneMode activation/push/pop直接改`SceneModeStack` |
| `ViewportInput`资源 | scheduler与tests | 真实Viewport event直接进入controller/input stack |
| Export/build tool ID | 测试fixture | export wizard/job没有scheduler接入 |

产品输入链是`EditorViewportEvent -> ViewportCommand -> WorkbenchShellStateData -> EditorState::apply_viewport_command -> SceneViewportController::handle_input -> SceneModeStack`。这一链有输入消费与Gizmo transaction，但没有tool instance、lease、capture owner或失焦清理。静态证据足以证明scheduler不是产品authority；本轮没有运行Cargo、GUI、并发模型检查、插件reload、焦点丢失、真实Modal/Export或性能基准。

## 3. 必须保留的工程基础

1. 保留`ToolResourceSet`非空、去重、稳定排序和反序列化重验证的不变量。
2. 保留typed acquire/release/withdraw outcome与显式denial，不退化为bool或字符串。
3. 保留单资源FIFO、集合请求不部分占有及non-owner release拒绝的现有测试语义。
4. 保留EditorContext持有唯一工具authority的方向，但把它升级为可配置、可观测、可关闭的lifecycle service。
5. 保留`SceneModeStack`真实实例、enter/exit、overlay、pass-through checkpoint与shutdown，它应成为首个工具产品adapter，而不是被删除。
6. 保留Viewport把transform preview交给Editor transaction层提交的边界，不让工具直接绕过undo/redo修改Scene。
7. 保留typed `ToolMessage`入口，但事件必须来自原子transition log/outbox并具备revision与resync。
8. 保留现有单元测试，新增model/property/concurrency/product tests；不能用“重写”删除已验证行为。

## 4. 参考源码给出的结构约束

| 参考 | 本轮源码事实 | 对Zircon的约束 | 不照搬的部分 |
|---|---|---|---|
| Unreal Interactive Tools Framework | manager注册builder并创建真实active tool；Setup后注册input behaviors；停用先从InputRouter注销source，再按Accept/Cancel/Completed调用Shutdown；unregister与manager shutdown清理active tool | definition、instance、input capture、terminal disposition、transaction与反注册必须闭环；清理顺序要能承受modal/reentrant shutdown | 不复制UObject、left/right side或具体transaction API |
| Unreal InputRouter | capture request有source/owner与priority；稳定排序；支持mouse/keyboard/hover、capture stealing、force end和focus loss termination | `ViewportInput`不能只是全局枚举占位，必须有pointer/window/source/capture generation和强制终止 | 不要求照搬其数组与事件类型 |
| Fyrox Editor | 每个scene持有InteractionModeContainer和current mode；activate/deactivate、mouse/key/hotkey/UI/update/on_drop均走真实mode instance；插件mode可增删并同步 | Zircon现有SceneModeStack应接入per-session tool authority，输入、drop和scene close都必须到达同一实例 | Fyrox本身不是多资源租约的完整范本 |
| Godot EditorPluginList | 插件列表真实转发2D/3D GUI input与draw，聚合PASS/CUSTOM/STOP；插件可add/remove并有visible/edit/clear生命周期 | extension贡献的交互入口必须可到达、可撤销，并保留明确消费结果 | Godot列表转发不等同公平租约调度器 |
| Bevy Picking/Input Focus | PointerId区分mouse/touch/custom；PointerMap维护identity；PickingSystems显式排序输入/backend/hover；focused entity despawn时清焦点 | 输入阶段、pointer identity、focus owner与owner失效清理必须是可测试合同 | 不把ECS schedule当Editor工具ABI |
| Unity Graphics包内工具 | `EditorTool`使用OnActivated/OnWillBeDeactivated/OnToolGUI；LightPlacement保存并恢复SceneView状态，Escape恢复前一工具并消费事件 | 即使包级工具也必须保存/恢复外部状态、支持退出与前一工具语义 | 本地Graphics镜像不含完整Unity Editor，不能外推全局ToolManager实现 |

共同规律不是“类更多”，而是用户交互由真实实例拥有，激活与输入捕获同代，终止有明确语义，owner消失必然释放资源。Zircon可以采用更紧凑的数据布局和更低开销的调度，但不能省略这些可观察合同来宣称性能优势。

## 5. P0 阻断项

### ED53-P0-01 · Scheduler被构造但没有任何产品consumer，三个exclusive resource全部是装饰性合同

`EditorContextBuilder`创建`ToolSchedulerService`后，没有production调用`tools()`。Scene Mode、Viewport、Gizmo、Modal、Export和extension unload均不申请或释放租约，`editor.tool`也没有subscriber。当前UI可以同时开启理论上互斥的路径，而scheduler测试仍全绿；因此“存在服务”与“产品受仲裁”是两套事实。

**必须重构：** 先建立`InteractiveToolAuthority`与产品admission；SceneMode/Viewport作为首批adapter，Modal/Export按各自operation owner接入。未接入的功能不得标记为scheduler-protected，也不得用测试ID证明产品完成。

### ED53-P0-02 · 同一ToolId可覆盖active set，使旧set release失效并破坏集合原子性

`acquire_set()`只把“同一tool + 同一set”视为AlreadyHeld。若同一`ToolId`已持有集合A，又申请互不冲突的集合B，`activate_set()`先给B写holder，再执行`active_sets.insert(tool, B)`覆盖A；A的resource state仍以该tool为holder。随后`release_set(tool, A)`因map中只剩B而返回NotHeld，`release(tool, A_resource)`又因map中B不含A_resource而可能单独释放，但这已经破坏“集合不可部分释放”的不变量；若集合有交叠，还会出现更复杂的错误归属。现有测试没有覆盖第二集合申请。

**必须重构：** identity分为`ToolDefinitionId`与唯一`ToolInstanceId`，所有持有由不可伪造`ToolLeaseId`索引；一个instance的active lease必须是显式多lease集合或严格单lease状态，申请阶段验证不变量，transition commit后运行debug/model invariant，禁止覆盖式insert。

### ED53-P0-03 · State commit、解锁与逐条event publish不是同一因果序，观察者可看到与authority相反的最终状态

每个service方法在scheduler mutex内先改变authority，随后解锁，再循环`bus.publish()`。线程A完成Acquired并解锁后，线程B可以Release并先发布Deactivated，线程A再发布Activated；bus sequence会把Activated放在最后，而实际holder已经为空。`publish_events()`还忽略dispatch error、drop与backpressure，零subscriber也被静默接受。`#[must_use]`只约束调用者不丢report，无法实现注释所声称的“publish before exposing new state”。

**必须重构：** 每次变更生成单调`ToolTransitionRevision`和原子transition record；authority与outbox在同一commit中写入，单一dispatcher按revision发布batch。query/snapshot返回revision，subscriber支持gap detection/resync；delivery失败进入typed health/fault状态，不能回滚已公开authority或静默丢失。

## 6. P1 工程差距

### 6.1 Identity、owner与resource model

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-01 | `ToolId`同时冒充definition、instance、request和lease身份 | 分离qualified definition ID、instance ID、request ID与lease ID |
| ED53-P1-02 | ID没有package/plugin owner与owner generation | 绑定extension owner、generation、session与capability lease |
| ED53-P1-03 | ID没有project/document/window/viewport scope | 引入typed `ToolScope`，拒绝跨scope释放与冲突 |
| ED53-P1-04 | 调用者只凭可复制字符串即可release/withdraw别人的同名工具 | release只接受不可伪造lease/request handle并校验generation |
| ED53-P1-05 | 同一tool可同时进入多个single queue、set queue和holder，没有聚合状态机 | 每个instance建立唯一状态与显式子lease集合，非法transition typed fail |
| ED53-P1-06 | 三项固定enum不能表达插件资源、多个viewport、pointer或document | `ResourceKey { kind, scope, channel }`使用注册表/typed namespace并有owner撤销 |

### 6.2 Arbitration、公平性与活性

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-07 | 任意非空set queue会阻止所有空闲资源的新single acquire | 只阻止与保留集合冲突的key，建立冲突图/claim plan |
| ED53-P1-08 | set head等待另一资源时，single release会把刚空闲资源交给旧single queue | 定义集合保留、aging与公平策略，证明set request最终可运行 |
| ED53-P1-09 | 每次release/withdraw最多晋升一个set，其他已满足请求可无事件地滞留 | transition后执行有界fixpoint promotion并批量生成结果 |
| ED53-P1-10 | `release_set`在set head仍不可用时不晋升释放资源上的single queue | 统一single/set选择器，禁止不同入口产生相反公平语义 |
| ED53-P1-11 | 初始queue position发布后不会随前项撤销/晋升更新 | 提供request snapshot/revision或position-changed事件，禁止把旧位置当承诺 |
| ED53-P1-12 | `max_queue_per_resource`同时限制每资源single queue和全局set queue，名称与预算域不符 | 拆分per-key/global/per-owner entries与bytes预算，并由settings/qualification约束 |

### 6.3 Definition、instance与activation lifecycle

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-13 | 没有tool definition、builder/factory或capability metadata | 建立compiled `ToolDefinition` catalog与owner generation |
| ED53-P1-14 | Acquired只代表holder写入，不代表真实tool instance已Setup | 引入Prepare/Create/Setup/Activate阶段与失败补偿 |
| ED53-P1-15 | 没有CanActivate、selection/target/context eligibility | admission snapshot包含target、mode、permission、capability与conflict reason |
| ED53-P1-16 | Deactivated没有Accept/Cancel/Completed/Aborted/OwnerLost语义 | 定义terminal disposition并让tool执行相应commit/rollback |
| ED53-P1-17 | 没有previous persistent tool、temporary overlay或resume policy | 建立tool stack/return token和可验证恢复策略 |
| ED53-P1-18 | `release_all`无人绑定scene close、window close、plugin unload、shutdown | owner teardown必须quiesce、cancel、release并取得terminal receipt |

### 6.4 Input capture与focus

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-19 | `ViewportInput`资源只有全局holder，没有pointer/window/device identity | capture key包含viewport、window、PointerId、device/channel |
| ED53-P1-20 | 没有hit/capture request、priority或稳定仲裁 | InputRouter收集request并按明确稳定规则选owner |
| ED53-P1-21 | 没有capture stealing、cooperative handoff或拒绝原因 | 定义priority/preemption policy与old-owner force-end receipt |
| ED53-P1-22 | mouse、touch、pen、keyboard、hover与text focus没有分型 | 分离pointer capture、keyboard focus、hover source与IME/text owner |
| ED53-P1-23 | window focus loss、pointer cancel与owner despawn不会终止capture | 平台/host失效事件必须同步cancel并清理generation |
| ED53-P1-24 | scheduler不知道输入是否Consumed/PassThrough及effects是否提交 | router输出typed routing result，和SceneMode effect/transaction receipt关联 |

### 6.5 Scene Mode、Modal、Export与transaction接入

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-25 | `activate_scene_mode`直接replace base，不申请`SceneModeSlot` | SceneMode adapter以instance/lease激活，失败保持旧mode与capture |
| ED53-P1-26 | host的overlay push/pop直接操作stack | overlay定义共享/独占资源和stack policy，由authority提交 |
| ED53-P1-27 | Viewport event直接进入controller，未校验capture owner | host在发布command前按published routing snapshot分发 |
| ED53-P1-28 | `ModalSurface`没有任何真实modal owner或scope | dialog/sheet/picker建立window-scoped modal lease与nested policy |
| ED53-P1-29 | Export/build只在测试中出现tool ID，真实job/wizard无关联 | UI tool lease只拥有交互surface，后台operation由Editor09 ticket继续执行并独立取消 |
| ED53-P1-30 | tool lifecycle与Editor transaction没有start/commit/cancel bracket | activation receipt绑定transaction policy，terminal disposition驱动提交或回滚 |

### 6.6 Event、snapshot与诊断

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-31 | event没有transition revision或authority epoch | 所有event携带epoch/revision且按batch原子排序 |
| ED53-P1-32 | event没有request/instance/lease identity | 消费者可将Queued、Activated与Terminal严格关联 |
| ED53-P1-33 | event没有scope、owner generation、cause与terminal disposition | 形成可诊断且可审计的完整transition envelope |
| ED53-P1-34 | set release可能生成多事件但逐条publish，可被其他变更穿插 | 一次transition只发布一个ordered batch/envelope |
| ED53-P1-35 | service忽略bus dispatch report | delivery failure/backpressure进入health state、metrics和resync queue |
| ED53-P1-36 | 只有single holder query，没有set queue、active set或完整snapshot | 提供immutable bounded snapshot、cursor与gap/resync合同 |

### 6.7 Concurrency、failure与performance

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-37 | 所有scope共享一个`Mutex<ToolScheduler>` | 先以正确性为准，再按scope shard；跨key申请用ordered prepare/commit |
| ED53-P1-38 | poison后无条件`into_inner()`继续服务 | 验证不变量、进入faulted/rebuild或fail-stop，记录cause与receipt |
| ED53-P1-39 | 没有deadline、aging、cancellation token或awaitable wakeup | request handle支持cancel/deadline/terminal wait，调度使用单调clock |
| ED53-P1-40 | 每次事件克隆ToolId/resource set并重复parse topic | 编译topic、batch transition，量化clone/allocation后再优化布局 |
| ED53-P1-41 | 没有queue depth、wait time、hold time、preemption、leak指标 | 按resource/scope/owner输出有界metrics与structured diagnostics |
| ED53-P1-42 | 没有shutdown状态，关闭期间仍可acquire | Open -> Quiescing -> Draining -> Closed状态机拒绝新请求并排空旧lease |

### 6.8 Contract、version与qualification

| ID | 差距 | 需要的重构 |
|---|---|---|
| ED53-P1-43 | serialized `ToolLifecycleEvent`/resource enum没有schema/version | 定义qualified schema、compatibility、unknown variant与migration policy |
| ED53-P1-44 | outcome、event与query没有同一operation receipt | 统一request receipt，记录admission、transition和delivery终态 |
| ED53-P1-45 | 测试没有property/model check，无法系统发现map/holder不一致 | 建立reference model和随机operation序列，逐步检查所有不变量 |
| ED53-P1-46 | 没有双线程publish reordering、poison、backpressure与shutdown race测试 | 使用barrier/fault sink构造确定性并发回归 |
| ED53-P1-47 | 没有SceneMode/Gizmo/Modal/Export/plugin unload产品测试 | 以真实host/controller/context链验证租约而非直接测scheduler |
| ED53-P1-48 | 没有1K/10K request、multi-viewport或长时公平性基准 | 建立correctness-bound workload，报告p50/p95/p99 wait、CPU、alloc和内存 |

## 7. P2 完整性差距

| ID | 差距 | 建议 |
|---|---|---|
| ED53-P2-01 | `ExclusiveResource::ALL`是私有手写数组 | 由compiled resource catalog生成稳定迭代与诊断名 |
| ED53-P2-02 | `ToolResourceSetError`只有Empty，缺少owner/scope/schema上下文 | 使用构建阶段validation report而非扩展一个扁平字符串 |
| ED53-P2-03 | wrong-set release统一返回NotHeld | 返回typed lease mismatch，不泄露其他owner的敏感细节 |
| ED53-P2-04 | denial把全局set head报告成single resource holder | 分离ConflictHolder、Reservation与Capacity denial |
| ED53-P2-05 | 没有human-readable resource/tool diagnostics label | metadata使用LocalizationKey，identity保持稳定非本地化 |
| ED53-P2-06 | 没有queued request listing的分页/过滤 | 调试视图使用bounded cursor query，不暴露内部容器 |
| ED53-P2-07 | 没有工具状态历史的有界保留 | 保存短期transition ring并支持导出source-bound trace |
| ED53-P2-08 | `AlreadyHeld`/`AlreadyQueued`没有返回canonical lease/request handle | 幂等调用返回同一handle与revision |
| ED53-P2-09 | queue capacity为裸`usize`构造参数 | 接入typed settings、hard ceiling与启动诊断 |
| ED53-P2-10 | 没有resource conflict graph可视化来源 | 从compiled catalog导出diagnostic graph，禁止UI维护第二份表 |
| ED53-P2-11 | 没有accessibility/keyboard tool switching资格 | tool switcher与escape/cancel路径纳入focus和a11y测试 |
| ED53-P2-12 | 没有性能比较workload identity | receipt记录BuildSet、场景、工具目录、输入trace和硬件环境 |

## 8. 目标架构

### 8.1 核心类型

```text
ToolDefinitionId + OwnerGeneration
             |
             v
CompiledToolDefinition --factory--> ToolInstanceId
             |                          |
             |                    ToolRequestId
             |                          |
             v                          v
       ResourceClaimPlan --------> ToolLeaseId
                                      |
                                      v
                          InputCapture / Mode / Modal
```

`ToolDefinition`描述owner、factory、capability、supported scope、resource claim、transaction policy与shutdown policy；`ToolInstance`持有真实业务对象；`ToolRequest`是可取消的排队意图；`ToolLease`是唯一可释放authority；`ResourceKey`包含kind与scope，而不是固定全局enum。

### 8.2 生命周期

```text
Registered -> Prepared -> Queued -> Activating -> Active
                                            |         |
                                            |         +-> Accepting -> Completed
                                            |         +-> Canceling -> Canceled
                                            |         +-> Aborting  -> Aborted
                                            +-> Failed
owner/window/focus/shutdown loss ---------------------> Aborting
```

每条边生成一个revisioned transition batch。Setup失败不能占有资源；Active前输入不可路由；Deactivating开始先撤销capture，再调用业务shutdown；terminal receipt完成后释放lease。reentrant modal或callback只能提交新request，不能在旧transition中直接篡改容器。

### 8.3 Authority、outbox与query

`InteractiveToolAuthority`在一次锁内验证request、选择公平候选、提交instance/request/lease/resource state并写入outbox。dispatcher按revision发布；snapshot携带同一revision。bus失败不会改变已提交事实，而是把health标为Degraded并要求subscriber resync。跨scope sharding只在model test和性能证据完成后实施。

### 8.4 产品adapter

1. `SceneModeToolAdapter`拥有base/overlay instance和`SceneModeSlot(viewport)`。
2. `ViewportInputRouter`拥有pointer/focus capture，只有published owner收到输入。
3. Gizmo transform tool绑定Editor transaction，Accept提交、Cancel/OwnerLost回滚。
4. Modal owner使用`ModalSurface(window)`，nested modal遵守显式policy。
5. Export wizard的UI lease与后台job ticket分离，关闭wizard不伪造job取消。
6. Extension revoke先禁止新instance，再取消旧instance、force-end capture、释放lease，最后卸载代码。

## 9. 重构里程碑

### M0 · Truth freeze与算法止血

增加同ID不同集合、promotion fixpoint和并发publish顺序的negative regression；在产品UI/diagnostics中不得宣称现有scheduler正在保护Scene/Modal/Export。冻结当前caller与resource矩阵。

### M1 · Identity、definition与resource contract

引入Definition/Instance/Request/Lease ID、OwnerGeneration、ToolScope、ResourceKey和compiled catalog；提供hard-cutover迁移，不保留字符串release shim。

### M2 · Revisioned authority与公平调度

用单一transition engine替换分散single/set分支；实现冲突图、atomic multi-key claim、fair/aging/deadline/cancel、fixpoint promotion、不变量检查与snapshot/outbox。

### M3 · InputRouter与terminal lifecycle

建立pointer/window/device/focus capture、priority/handoff/focus-loss；实现Setup、Active、Accept/Cancel/Completed/Aborted、capture-first teardown和transaction bracket。

### M4 · Scene/Viewport/Gizmo/Modal产品切换

先接入SceneMode与Viewport/Gizmo，再接入Modal。每条现有直接路径硬切为authority adapter；未取得lease的instance不能接收输入或改变active mode。

### M5 · Extension、Export、shutdown与recovery

与Editor50 owner generation/revoke闭合；与Editor09 operation ticket分层接入Export；实现project/window/scene close、host shutdown、crash trace和last-session诊断。

### M6 · Qualification与性能

运行model/property/concurrency/fault/product/GUI/multi-viewport/soak测试；在正确性、资源清理与同workload证据通过后再做shard、pool和cache优化，并与参考引擎同条件比较。

## 10. 产品资格门

| Gate | 通过条件 |
|---|---|
| G01 | ToolDefinitionId、InstanceId、RequestId、LeaseId互不混用且全仓typed |
| G02 | owner generation、project/document/window/viewport scope随所有request与lease传播 |
| G03 | 同一instance的重复/并行claim不能覆盖或泄漏旧holder |
| G04 | 每次transition后resource holder、lease、request、instance双向索引一致 |
| G05 | multi-key claim零部分占有，失败零状态变化 |
| G06 | promotion执行到有界fixpoint，不留下可运行且无唤醒源的request |
| G07 | single/set公平策略有formal model与长期无饥饿测试 |
| G08 | 无关scope/resource的request不被全局set head阻塞 |
| G09 | deadline/cancel/withdraw均产生唯一terminal receipt |
| G10 | queue entries、bytes、per-owner与global hard ceiling在入队前检查 |
| G11 | authority commit和outbox revision原子，event顺序不能与snapshot逆转 |
| G12 | subscriber检测revision gap并从bounded snapshot/cursor resync |
| G13 | bus failure/backpressure进入可观测health，不静默丢弃 |
| G14 | tool factory Setup失败不占resource、不接收input且执行补偿 |
| G15 | Accept/Cancel/Completed/Aborted/OwnerLost的业务与transaction结果可区分 |
| G16 | teardown先撤销input capture，再运行可能reentrant的tool shutdown |
| G17 | window focus loss、pointer cancel、scene close和owner despawn强制终止capture |
| G18 | mouse/touch/pen/custom pointer identity与keyboard/text focus不混用 |
| G19 | SceneMode base/overlay全部通过viewport-scoped lease切换 |
| G20 | 未持有published capture的tool收不到ViewportInput |
| G21 | Gizmo Accept提交、Cancel/OwnerLost回滚且undo/redo闭环 |
| G22 | ModalSurface按window scope仲裁，nested policy明确且可测试 |
| G23 | Export UI lease与后台operation ticket生命周期分离 |
| G24 | plugin disable/reload禁止新request并清理旧instance/capture/lease后才卸载 |
| G25 | project/window/editor shutdown进入Quiescing并拒绝新acquire |
| G26 | poison/invariant failure进入typed fault/rebuild，不无条件继续服务 |
| G27 | serialized lifecycle schema有version、unknown variant与migration测试 |
| G28 | existing 43项测试保留并迁移到新authority语义 |
| G29 | model/property测试覆盖随机acquire/release/withdraw/reload序列 |
| G30 | barrier测试确定性复现并禁止unlock/publish因果倒置 |
| G31 | 真实host产品测试覆盖SceneMode、Gizmo、Modal、Export和extension unload |
| G32 | 多window、多viewport、多document和同definition多instance隔离通过 |
| G33 | 1K/10K request workload报告wait percentile、CPU、allocation与resident bytes |
| G34 | soak结束后0 holder、0 capture、0 request、0 stale owner generation |
| G35 | qualification receipt绑定BuildSet、source fingerprint、tool catalog和input trace |
| G36 | 在同场景同工具同输入同硬件且correctness gate全过前，不宣称优于Unreal |

## 11. 缺失测试矩阵

1. 同一ToolId先申请集合A、再申请互不冲突/部分交叠集合B，断言不能覆盖旧lease或留下holder。
2. 随机single/set acquire/release/withdraw/release_all序列与reference model逐步比对。
3. 构造多个同时可运行set，断言一次transition执行fixpoint且没有沉睡request。
4. 长时间混合single/set竞争，验证公平、aging、deadline与无饥饿。
5. 用barrier控制Acquire解锁、Release发布、Acquire发布顺序，验证revision仍保持因果。
6. 注入bus error/drop/backpressure、mutex poison与subscriber gap，验证health/resync。
7. mouse/touch/custom pointer、多window焦点切换、capture steal与focus loss。
8. SceneMode base/overlay、Gizmo drag transaction、Escape cancel和owner lost rollback。
9. nested modal、窗口关闭、project切换与Editor shutdown清理。
10. plugin disable/reload与active/queued tool并发，验证旧generation不能复活。
11. Export wizard关闭但后台job继续、显式cancel job、job完成后UI重连。
12. 1K/10K工具定义/request、多viewport soak与内存/延迟/分配基准。

## 12. Owner、依赖与非目标

| 域 | Canonical owner | Editor53依赖/交接 |
|---|---|---|
| Tool definition/instance/lease/capture/lifecycle | Editor53 | 本报告唯一owner |
| Scene/Prefab/Selection/Gizmo业务 | Editor03 | 提供真实tool adapter与transaction disposition，不复制业务算法 |
| Background job/Export operation | Editor09 | UI lease与operation ticket分层 |
| Message bus admission/retention/shutdown | Editor48 | Editor53只拥有revisioned outbox与tool resync语义 |
| Extension contribution/revoke/reload | Editor50 | owner generation与quiesce协议 |
| Transaction/undo/save | Editor02及现有editing engine | tool terminal disposition驱动，不另建undo authority |

本报告不优化PowerShell/Python tooling，不重写Scene/Gizmo具体算法，不把后台job变成interactive tool，也不承诺照搬Unreal UObject或Unity Editor内部结构。性能目标是保留完整语义后以数据布局、批处理、分scope并行和可测workload超越参考实现，而不是删除生命周期与恢复路径。

## 13. 禁止的临时修补

1. 禁止只在SceneMode切换前调用一次`acquire()`而不保存lease和terminal cleanup。
2. 禁止用更多`ExclusiveResource` enum variant冒充scoped/extensible resource model。
3. 禁止用`ToolId`字符串约定拼接window/plugin/instance身份。
4. 禁止只给`active_sets.insert`加“已存在就拒绝”后宣布调度器工程化完成。
5. 禁止在scheduler mutex内直接同步调用任意plugin/subscriber代码；使用commit + outbox。
6. 禁止忽略bus dispatch report或以零subscriber为Delivered产品证据。
7. 禁止让SceneMode、Gizmo和Viewport继续保留旁路写authority的兼容路径。
8. 禁止把Modal与Export后台job粗暴纳入同一全局互斥锁。
9. 禁止删除FIFO/atomic-set既有测试来适配新实现。
10. 禁止在无model、fault、soak和同workload receipt时宣称性能优于Unreal。

## 14. 状态与产出记录

本轮完成20份Zircon聚焦源码、43项聚焦测试、全量production caller反查及19份参考源码对照，新增本专项报告并同步Editor索引、顶层索引、coverage与跨报告P0 owner总账。未修改production/tests，未运行Cargo、GUI、并发模型、插件reload、焦点丢失或性能基准。

静态review完成不表示重构完成。先按M0为三个P0建立negative regression和产品truth freeze，再按M1-M5硬切到instance/lease/capture/terminal authority；只有G01-G36全部通过，Editor才可声明交互工具调度达到工程级产品闭环。
