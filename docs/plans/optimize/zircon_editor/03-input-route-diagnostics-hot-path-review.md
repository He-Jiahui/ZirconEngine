# UI input route diagnostics hot-path review

状态：`static_candidate + design_ready_p1`。本报告绑定 2026-08-28 current source，只包含源码证据和确定性算法模型；未运行 Cargo、Editor 产品 profile 或 allocator 采样，不宣称鼠标响应已经达标。

## 1. Current-source finding

普通 pointer dispatch 已经使用 frame-owned hit grid，但命中之后仍在每个事件上重复物化诊断路径：

1. `zircon_runtime/src/ui/surface/input/pointer.rs` 把 `UiPointerRoute`交给 `annotate_pointer_route_trace`，随后调用 `annotate_result_route_steps`。
2. `zircon_runtime/src/ui/surface/input/route_policy.rs` 从同一 bubble route 反向复制 preview tunnel，另取 focused route，复制 popup ID，并把root fallback复制进公开trace。
3. `zircon_runtime/src/ui/surface/input/route_steps.rs` 再按preview/bubble/focus路径生成逐节点`UiDispatchReplyStepTrace`。
4. `zircon_runtime/src/ui/surface/surface/event_routing.rs` 对同一个pointer route依次尝试range、scrollbar、table、tree与generic component default action；缺少target-kind dispatch table时，无关handler仍进入产品事件链。

因此“hit test不再全树扫描”并不等于pointer事件已经轻量。当前结果DTO同时承担产品提交结果、远程/测试序列化和完整路由调试三种职责，导致稳定hover也支付诊断展开成本。

输入积压已单独排除为当前Editor主因。Interface的`UiWindowInputPumpBatch::push_coalesced`确实只合并相邻redraw，ABI adapter也逐条`push`；但Editor native event loop已有唯一`UiIdlePointerMoveMailbox`，只对无button/capture/resize/tab-drag的同device idle mouse move保留latest value，任何非move边界先flush，并为被替换sequence发布`Coalesced`终态与批量counter。现有65,536 idle moves/256 batches模型只dispatch 256次，capture-sensitive 4,096次全部保留。不能在Interface再叠加不知Editor capture/drag状态的第二层coalescer；该batch问题只属于runtime preview/ABI host的独立backpressure工作，不作为本轮Editor hover解释。

raw device motion是可独立收口的特例。`UiInputEvent::MouseMotion`策略恒为`Unrouted`，旧实现仍进入通用trace构建。当前静态候选在`zircon_runtime/src/ui/surface/input/mouse_motion.rs`直接移动event并发布`Unrouted`，保留unhandled reply、`raw_mouse_motion` note与最终route-authority note，完整route trace保持default empty。

## 2. Reference-engine evidence

本地 Unreal 参考源码：

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp:257`起的direct/tunnel/bubble policy持有`const FWidgetPath&`，路由阶段借用同一path，不构造公开诊断路径副本。
- 同文件`ProcessMouseMoveEvent`在约6478行先通过hittest定位一个`FWidgetPath`，约6517行直接交给`RoutePointerMoveEvent`；输入debug scope只在`WITH_SLATE_DEBUGGING`存在。
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Trace/SlateTrace.cpp:229`等昂贵trace工作先检查`UE_TRACE_CHANNELEXPR_IS_ENABLED(SlateChannel)`；widget path/debug info只在trace channel需要时生成。
- `FHittestGrid`仍负责paint期空间索引；该事实不能替Zircon event-result DTO的额外复制辩护。命中authority与诊断authority是两个独立优化层。

可吸收的是“单一路径借用 + 显式调试开关”原则，不复制Unreal的宏、全局状态或C++对象模型。

## 3. Target architecture

### 3.1 Compact dispatch receipt

产品热路径默认只发布固定大小的`UiInputDispatchReceipt`：event identity、route policy、target/handler、disposition、effect/component/binding counts、capture/focus/hover generation和typed fallback reason。不得包含per-node Vec或String note。

现有`UiInputDispatchResult`保留行为payload，但route trace改为可选的generation-owned handle。内部Editor消费者没有请求trace时，不得展开preview/focus/popup/root/steps。

### 3.2 Shared route authority

`UiPointerRoute`发布单一共享route product：bubble path、stacked hits、entered/left和popup stack均由本次frame/hit generation拥有。dispatch policy借用slice；trace capture若开启，可在事件结束前一次序列化，不能为preview、bubble、focus和steps各建一份独立Vec。

preview tunnel应是bubble slice的反向view；route steps应在显式capture时由iterator输出。focus route与pointer bubble route相同时复用同一identity；不同才引用第二个published route handle。

### 3.3 Explicit diagnostics level

在window/input-manager边界发布`UiInputDiagnosticsLevel::{Off, Summary, Trace}`：

- `Off`：产品默认，只保留行为所需receipt与low-cardinality counters。
- `Summary`：保留policy、target、handler、disposition和typed reason，无路径展开。
- `Trace`：测试、debug reflector或远程诊断显式请求，完整route/step序列化。

level必须在路由前已知，不能先构造完整trace再丢弃。secure-text redaction与host request不受该开关影响。

### 3.4 Target-kind default action dispatch

Surface frame publication为可交互节点发布低基数`UiDefaultActionKind`或descriptor dispatch mask。pointer event只进入目标route可能消费的handler；scrollbar/range/table/tree/generic仍保留明确优先级和typed fallback，但不再逐个探测所有无关域。

popup、capture、rich-link、editable text和drag的跨域组合必须通过mask表达，不能假设每个target只有一种行为。mask缺失或generation不匹配时允许计数化保守链式fallback。

## 4. Complexity and evidence budget

raw motion模型使用100,000 events、12层focus、4 roots、3 popup：删除100,000次event clone、100,000次focused-route query、4,300,000个route identity copy和500,000次trace Vec allocation；仍保留200,000次公开diagnostic String allocation。工件：`E:\zircon-profiles\runtime-ui-mouse-motion-fast-path-pressure-20260828.json`，SHA-256 `118D6C87B7879DB93F79B2A60D8814659CCB1EEEC319F8F9D68C2A5115522BD5`。

普通pointer目标压力场景固定为100,000 move events、12层bubble、12层focus、3 popup、5个default-action handler。目标：

- `Off/Summary`模式每事件route trace Vec allocation、route identity String clone、route-step materialization均为0。
- stable same-target move只做一次hit-grid query、必要hover equality和一个dispatch-mask分支；无状态变化时component/binding/damage publication为0。
- `Trace`模式必须与现有公开route target、preview/bubble/focus/capture/root/popup顺序逐项等价。
- diagnostics off/on不得改变reply、effects、host requests、component events、binding reports、secure redaction或physical virtual pointer。

该场景的确定性目标模型计得current eager diagnostics为2,800,000个identity copy、400,000次trace Vec allocation和500,000次default-handler probe；`Off/Summary + dispatch mask`目标将前两项归零，并避免400,000次无关handler probe，同时保留100,000次匹配分支。工件`E:\zircon-profiles\runtime-ui-pointer-diagnostics-pressure-20260828.json`，SHA-256 `485808D409C4ADC86745A4A084E8746CC3CEFC6E255A675E0DD3E93602923B23`。`implementation_evidence=false`，不得把该数字写成已实现收益。

产品门必须分别采集same-target hover、cross-target hover、captured drag、popup hover和200-step resize中的CPU p50/p95/p99、allocation count/bytes、route path copies、default handler probes、damage/full redraw、RSS。只有current-source Editor二进制三轮结果通过预算，才能宣称交互改善。

## 5. Implementation order and ownership

1. 先执行raw mouse-motion lower Rust回归；当前源码/模型合同5/5已通过，lower测试已写但未managed执行。
2. Interface owner稳定后先加diagnostics level与compact receipt，不改pointer行为。
3. Runtime route owner把trace展开移到显式`Trace`分支，并让preview/steps借用单一route product。
4. 再发布default-action dispatch mask，逐类迁移range、scrollbar、table、tree、generic handler。
5. 最后跑Editor产品profile；若hit-grid query仍占主导，再回到frame publication/cell density，而不是继续压诊断常数。

当前`pointer.rs`、`route_policy.rs`、`event_routing.rs`和Interface dispatch result均有其它owner未提交改动。本轮不吸收这些路径，只提交独立raw-motion叶子、回归、模型和本报告；共享owner稳定前不得创建重复route cache或局部schema。

## 6. Static validation ledger

- raw-motion source/model、pointer target模型与hit-route focused合同：15/15通过。
- pointer trace ownership、hover diff、route-step capacity、projected-hit ordering、arranged input patch及Editor pointer-move相邻合同：31/31通过。
- 两个Rust文件`rustfmt --edition 2021 --check`、三个Python文件`py_compile`与候选路径`git diff --check`通过。
- 全量`test_*performance_contract.py`运行1,254项，其中1,251项通过、3项错误；三项均为外部`test_runtime64_readiness_noop_performance_contract.py`读取已删除的`zircon_runtime/src/core/resource/manager/readiness_projection.rs`，不是本候选断言失败。本轮按共享owner约束不修该block。日志`E:\zircon-profiles\all-performance-contracts-20260828-mouse-motion-fast-path.log`，SHA-256 `6C944B42E964C1BB2E514E27EFAA3CE18B8C3BDAA4656432186A54C4D9CFBABD`。
- 未运行managed Cargo或Editor产品profile；lower Rust empty-trace回归仍是待执行证据。
