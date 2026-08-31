---
related_code:
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/frame_publication.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-input-admission-current-architecture-review.md
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-api-extract-fallback-ui-current-review.md
  - docs/plans/performance/01/2026-08-23-runtime-host-intent-outbox-transaction-architecture-review.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
tests:
  - current runtime UI session surface 1 of 1 Rust file and 2 inline tests reviewed
  - supporting retained surface publication, input manager, interface event and session call chains reviewed
  - M0 static performance contract 3 of 3 passed after RED
  - focused rustfmt 1.94.1 plus scoped diff check passed
  - current-source Cargo, UI/input scale, WPR, allocator, power and F4 RenderDoc pending
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime dynamic session UI surface路由与publication复审（2026-08-23）

## 范围与当前性

已逐行复读`dynamic_api/session/runtime_ui.rs`当前**1/1**个Rust文件、**555行、21,075 B、2 tests**，
实施前SHA256为`a337c2327043978f7438dccc675556d435e62aff5d08e0c83f47636462cc5b53`；M0后为
**577行、21,952 B、2 tests**，SHA256为
`6aa503babe05af60d0f64c6a83c962f836b17aac09b5cf2b7c7ed62dff48a6b8`。同时沿
`events/extract/state -> RuntimeUiSurfaceSet -> UiSurface/UiInputManager`复核输入、IME host request、
accessibility和render publication。该文件已有其他Session新增的IME drain bridge，本轮保留并围绕它工作，
不撤销或改写其所有权。

## 当前源码判定

### Stable retained surface仍在session边界深clone command stream

每个`UiSurface`内部已有dirty generation与`surface_frame() -> Arc<UiSurfaceFrame>` publication：只有dirty或
transient state变化才封存arranged tree、render extract、hit grid等宽数据。但dynamic session的
`render_extract`绕过这个generation artifact，逐surface调用`rebuild_dirty`后直接遍历
`surface.render_extract.list.commands`，深clone每条`UiRenderCommand`、重写node id，并为每次
capture/present构造新聚合Vec。稳定S个surface、C条commands仍产生O(S+C) visits、C个command deep clones和
聚合分配；command内style/text/image等owned字段会放大bytes。

这不是在session旁挂一个last-viewport cache就能正确修复：输入、layout、font、hot reload、accessibility与render
必须消费同一surface generation。Runtime09/Render14应让`UiSurfaceFrame`或更窄的render artifact拥有稳定
global node-id/range projection，dynamic session只借用Arc/ranges；同generation aggregate build<=1，stable
capture deep clone=0。fallback menu/HUD继续由PERF-MVP-433统一，不建立第二UI extract authority。

### Multi-surface input多做一次完整owned event clone

`dispatch_input`逆序遍历surface，并在每次迭代调用`event.clone()`；pointer无capture时同样为全部surface clone。
因此未被任何surface消费的S-surface事件执行S次clone，keyboard/IME/analog/drag等会复制String、Vec或payload。
最后访问的surface已经是event最后一个consumer，完全可以move原值，使clone数**S降为S-1**。pointer capture
命中当前已经直接move到唯一surface，clone=0，应保持。

更大的`events * surfaces` route/rebuild问题继续归input batch计划：M0不把event共享成跨surface可变对象，不合并
barrier，不跳过`rebuild_dirty`，也不改变stop-propagation。最终Runtime10/12应让producer admission有界、UI
consumer批处理并复用route scratch，owned String只在真正需要的surface/ABI边界物化。

### Capture与host request聚合缺少session级容量owner

`pointer_capture_surfaces: BTreeMap<Option<u64>, usize>`按pointer id保存capture。Up/Cancel会回收，但恶意或故障
producer可持续发送不同pointer id的capturing Down而不结束，active capture数量/bytes/age没有硬界。正确修复应在
Runtime10/12统一active-pointer admission定义最大并发、cancel/overflow和失焦清理，而不是只给这个BTreeMap加
静默淘汰。

每个surface的`UiInputManager`又持独立IME request Vec；新增bridge逐surface`mem::take`后extend到session Vec，
再与core input host requests聚合。该路径解决了功能接线，但producer aggregate、semantic coalescing与
continuation仍归PERF-MVP-425；本轮不覆盖foreign bridge。

### Startup与accessibility仍是显式宽事务

`load`先让`project_ui_prototype_store`扫描完整asset registry并同步load全部UI layout/widget/style，再只对roots
构建store和surface；这是PERF-MVP-638/Runtime09的root dependency demand问题。accessibility query已有items/
bytes/depth/time预算，但在session锁内逐surface rebuild、构造、globalize并聚合完整snapshot；锁外JSON与domain
generation归Runtime10，不能误报为稳态每帧热点。

## Unreal源码依据

Unreal `SlateApplication.cpp:5493-5543,5761-5825,6478-6517`用`const FPointerEvent&`贯穿route，并把同一
transformed event借给tunnel/bubble widget callback；它支持“一个事件owner，路由只借用/最后消费”，而不是每个
surface复制owned payload。Zircon当前by-value API仍允许内部状态安全拥有event，因此M0只利用Rust最后消费move，
不照抄UE引用生命周期。

Unreal `SlateInvalidationRoot.cpp:356-444`在允许fast path时推送cached element data，只有needs slow path时清空并
重建，否则执行`PaintFastPath`；`1281-1404`按invalidation处理局部更新。可转移原则是persistent render artifact
与dirty-driven publication，不是采用UE widget结构或常量。Zircon已有`UiSurfaceFrame` Arc generation基础，
dynamic session应接入该owner而非重复克隆commands。

## 本轮M0与动态验收

本轮让generic与noncapture pointer reverse route以`Option<UiInputEvent>`持有唯一原值：index 0作为最后consumer
直接take/move，其余surface才clone；若高层surface提前stop propagation，原值随owner正常释放。pointer capture
判断保持在Option包装前，仍直接把原event move给唯一surface。由此unconsumed传播clone从**S降为S-1**，
单surface从**1降为0**，capture保持**0**；surface顺序、rebuild、sequence、pointer capture与reply语义不变。

`tools/tests/test_runtime_session_ui_surface_last_consumer_m0_performance_contract.py`先得到**0/3 RED**，实施后
**3/3 GREEN**；测试58行、2,121 B、SHA256
`b28705809208c6403ee05894dd264b06a21f82b68be49faaddf0d7d1b13824d8`。focused
`rustfmt +1.94.1 --edition 2021 --check`与scoped diff check通过。current-source Cargo不可执行，已有2条
Rust tests未运行；S到S-1只是源码clone计数，不冒充wall time、allocator或功耗改善。

动态按surfaces 0/1/8/64、nodes/commands 1/1K/100K、events 125/500/1K/10K Hz、payload 0/16 B/1 KiB/
1 MiB、stable/1% dirty/resize/font/hot reload运行。记录surface visits、event clones/bytes、rebuild fast/slow、
command clones/bytes、aggregate builds、Arc generation hits、capture entries/bytes/age、IME requests、session lock、
p50/p95/RSS/energy。M0要求generic/noncapture clone=max(S-1,0)、capture clone=0；终态要求stable render aggregate
build/command clone=0、changed build<=1/generation，active pointer与request队列count/bytes/age硬有界。

WPR/ETW负责CPU、lock、wake、allocation stack与power，allocator负责String/command/RSS；F4 RenderDoc验证UI
draw/pass/upload、z-order、clip和像素等价，不作为input CPU结论。current-source binary/Cargo不可得前继续留在
`pending.md`，不提交milestone、不发送完成企微。
