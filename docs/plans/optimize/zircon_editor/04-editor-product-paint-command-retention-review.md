# Editor 产品 Paint 命令保留审查

日期：2026-08-28

状态：设计与确定性压力模型完成；产品实现因共享 owner 正在修改 command-stream、presenter、template paint 链而暂缓。

## 1. 结论

当前 Editor 已经不是“每个事件都无条件全屏重画”的旧模型：

- presenter 持有 backbuffer，并在 damage 存在时执行 region repaint；
- template paint 先把 damage 转为 effective clip，优先通过 generation-owned row paint index 或 transform streaming 只访问候选行；
- native resize 交互期冻结首张已提交图像并缩放该 snapshot，不重新构建 presentation；
- image command stream 已按 resource key + generation 查询 residency，resident 资源不重复携带上传字节。

这些机制解决了像素提交范围、候选节点范围、resize 交互期重排和 GPU 资源驻留问题，但还没有形成完整的 retained command model。每次普通 region repaint 仍会：

1. 重新调用 `build_chrome_command_stream`；
2. 重新进入 `extract_chrome_commands`；
3. 在 `draw_template_nodes_with_transform` 中创建新的 `Vec<HostPaintCommand>`；
4. 为 damage 候选节点重新解析状态、文本、图片和几何并生成命令；
5. 再执行 icon atlas pack、stream construction 和 image-resource compaction。

因此，按钮 hover/click 即使只损伤一个很小区域，也可能重复构造该区域内所有重叠节点的命令。局部像素重放不等于局部命令重建。

## 2. Current-Source 证据

产品入口：

- `zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs` 每次 present 都调用 `build_chrome_command_stream`；
- `zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/backbuffer.rs` 只在 stream 不是 full rebuild 且 backbuffer 尺寸匹配时执行 region replay；
- `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs` 每次调用创建新的 command vector，但会通过 damage clip 与 row index 缩小节点访问范围；
- `zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs` 每次 build 仍执行 extraction、atlas pack 和 resource compaction。

已有的 `test_editor_chrome_command_stream_allocation_performance_contract.py` 只锁定 extraction command vector 被 stream 直接接管，删除了 extract-vector 到 stream-vector 的第二次分配和头部搬运。它不证明跨 present 的命令复用。

`zircon_runtime_interface::UiRenderCachePlan` 是 debug/contract DTO，不是当前 Editor product paint spine；优化它不能作为按钮响应改善证据。

## 2.1 Unreal 参考合同

Unreal Slate 的 checked-in 实现缓存的是 widget 对应的 draw elements、render batches 与 clip states，而不是每帧重新运行 painter 后仅缓存最终像素：

- `SlateInvalidationRoot.cpp:179-187`把 `Layout/Visibility/ChildOrder` 等 pre-update 原因与 `Paint/Volatility/RenderTransform` 等 post-update 原因分开；
- `SlateInvalidationRoot.cpp:389-424`仅在 slow-path 条件成立时清空并重建全部 cached element list，否则执行 `PaintFastPath`；
- `DrawElements.cpp:181-208`在单个 widget 重新 paint 时清除并替换该 widget 的 cached elements，并显式携带 direct/indirect volatility；
- `DrawElements.cpp:425-513`由 `FSlateCachedElementData`统一拥有 cached element lists、render batches 与 clip states，cache handle 绑定 owning widget；
- `WidgetProxy.cpp:221-243`分别处理 Layout、Paint 与 RenderTransform，纯 render transform 跳过 prepass/parent layout；
- `WidgetProxy.cpp:351-375`计算 window-space transform delta，并将同一个 delta 应用到可见 subtree 的 cached elements；
- `SlateInvalidationRoot.cpp:1765-1769`验证不可见 widget 不得继续持有 cached elements。

Zircon 不需要复制这些 C++ 容器，但必须保留同样的所有权和失效分层：稳定 widget/node identity 持有 replay-ready fragment；volatility 不污染静态 sibling；render transform 能在缓存命令上应用 delta；visibility/child order/clip 有独立的结构验证；slow-path fallback 是显式、可计数的异常路径。

## 3. 目标算法

建立单一 presentation-instance-owned `HostPaintFragmentCache`，由 pane/window presentation 实例拥有，而不是进程全局缓存或 painter-local thread cache。每次 publication 发布一个新的不可变 `Arc<HostPaintFragmentSet>`；未变化 fragment 的 `Arc` 从上一 publication 直接复用。

`presentation_generation` 只标识一次原子 publication，不能放入逐 fragment 的命中 key。否则一次 hover 导致 publication generation 推进时，所有 fragment key 都会改变，缓存会退化成全量 miss。

缓存身份与内容 revision 必须分开：

```text
FragmentIdentity = {
    presentation_instance_id,
    stable_node_identity,
    fragment_role,
}

FragmentRevision = {
    node_content_revision,
    geometry_revision,
    interaction_revision,
    inherited_effect_revision,
    order_revision,
    text_metrics_generation,
    resource_generation,
}
```

身份与 revision 的具体字段应复用 current-source 已发布的 typed generation。若节点还没有独立 revision，第一阶段允许以 pane/row generation 为粒度，禁止在 paint 热路临时 hash 整个节点来制造“revision”。`order_revision`只更新 fragment set 的排序/index 元数据；若 command 使用局部坐标，纯 render-transform revision 也应以 replay metadata patch 处理，而不是重建 command payload。

一个节点可以发布多个 role fragment，至少区分：

- `Static`：背景、边框、稳定图标与稳定文本；
- `Interaction`：hover、press、focus、selection、caret 等状态层；
- `Volatile`：动画、视频、连续进度、viewport frame 等明确逐帧产品。

volatile fragment 每帧重建自身，但不得使同节点 static fragment、兄弟节点或整个 presentation miss。父节点 volatility 传播必须是显式 `volatile_indirectly` 依赖，不能由 painter 临时猜测。

每个 fragment 保存：

- replay-ready、局部坐标的 immutable command segment，例如 `Arc<[ChromeCommand]>` 或等价的 recorded-command slice；
- conservative paint bounds，用于 damage 查询；
- stable z/order key 与 clip/effect/transform replay metadata；
- 引用的 image/atlas resource key + generation 集合，不保存重复 RGBA 所有权；
- text metrics/resource generations；
- fragment byte cost、class 与 last-used publication generation。

`HostPaintCommand` 只能是 fragment miss 时 painter 产生的短生命周期构建输入，不能作为终态缓存边界。否则 warm present 仍会重复执行 `HostPaintCommand -> ChromeCommand` 转换、icon atlas pack、stream flatten 和 resource compaction。最终 `ChromeCommandStream` 或其替代 submission 必须支持借用多个 immutable fragment segments；不得为提交重新扁平化全部 command。

icon atlas 绑定在 fragment 构建/patch 时解析。fragment 保存 atlas page key、page generation 与 UV。atlas page 被 LRU 淘汰或 generation 改变时，只失效反向引用该 page generation 的 fragment；稳定 present 不再扫描所有 command 调用 `pack`。resource residency 在 submit 时按 fragment 的引用表查询，resident payload 不携带 RGBA，且不得通过就地修改共享 fragment 来执行 compaction。

publication/rebuild 负责构建或 patch fragment 与空间索引。paint 只执行：

1. 用 damage 查询相交 fragment handles；
2. 按稳定 z/paint order 合并 handles；
3. 仅为 changed/volatile handles 重建 replay-ready segment；
4. 将 borrowed command segments 直接交给 softbuffer replay 或 RHI submission；
5. 从统一 residency registry 查询需要上传的资源。

事件热路不得扫描全部 template nodes，不得按事件 hash 节点，不得 lazy 构建 fragment cache。

## 4. 失效合同

| 变化 | 必须失效 | 不应失效 |
| --- | --- | --- |
| hover/press/focus | 目标节点的 dynamic/state fragment；必要时祖先 state layer | 无关 pane、无关文本与图片 |
| 文本内容 | 目标 text fragment、其测量依赖的 geometry fragment | 无关兄弟节点 |
| resize 交互期 | 不重建 fragment；继续缩放 frozen snapshot | presentation、文本、资源 cache |
| resize settle | geometry dependency set；breakpoint/tier 变化时升级对应 pane subtree | 无尺寸依赖的资源 payload |
| SVG/image generation | 引用该 resource generation 的 fragment/resource binding | 无关资源与全部 atlas |
| theme/design token generation | 引用变化 token domain 的 fragment；无法归因时 typed pane fallback | 全 workspace 无条件清空 |
| topology change | 对应 subtree 或 presentation generation | 其它稳定 Surface generation |
| visibility/collapse | 目标 subtree 的 spatial/order membership、hit authority 与必要 fragment | 不可见节点继续持有可重放 fragment |
| z/paint order | fragment-set order/index metadata；paint payload revision 不变 | 重跑节点 painter |
| parent opacity/clip/effect | 受影响 subtree 的 inherited-effect metadata；clip 形状无法 patch 时 typed subtree fallback | 无依赖的 sibling/pane |
| render transform | local-coordinate fragment 的 replay transform delta、bounds 与 hit geometry | text shaping、image decode、static command rebuild |
| direct/indirect volatility | 只重建 volatile fragment；volatility membership 变化时更新集合 | static fragment 与无关 sibling |
| atlas page eviction | 仅引用旧 page key/generation 的 fragment resource binding | 全 fragment set、无关 page |

旧 fragment 必须在新 publication 原子可见前保持可读；事件与 presenter 只能读取同一已发布 generation，不能混合新 hit/layout 与旧 command fragment。

## 5. 内存和退化边界

- 第一阶段缓存只覆盖 visible + bounded overscan 节点，不缓存完整逻辑数据集；
- fragment command 容量上限必须由当前 visible command count 推导，并有 typed eviction/full-rebuild reason；
- resource bytes 只由统一 image/SVG residency owner 保存，fragment 只持 handle + generation；
- fragment query scratch 由 presenter/cache owner 复用，稳定容量下不得每 present 新分配；
- publication 保留 current immutable set；presenter 可持有至多一个 in-flight previous set。若第三代发布会超过 byte budget，必须 backpressure 或 typed full-stream fallback，不得无界堆积 retired generations；
- cache 使用 stable identity map、spatial bounds index、order index、resource reverse-dependency index 与 volatile handle set；这些 index 在 publication/patch 更新，不在 input/present 热路重建；
- fragment 内 String/command payload 只在 miss 时拥有一次，跨 publication 复用 `Arc`；image pixels 继续只由统一 residency/atlas owner 以 `Arc` 持有；
- 若命令顺序、clip stack 或资源 generation 校验失败，整次 present 退化为现有完整 extraction，不能发布部分错误结果；
- fallback 次数、原因、fragment hit/miss、static/interaction/volatile rebuild、transform/order metadata patch、rebuilt/borrowed command count、retained/current/in-flight/retired bytes 与 atlas reverse-invalidations 必须可观测。

## 6. 验收顺序

1. 下层 cache：同一 publication 两次查询返回相同 segment identity 与顺序，第二次 rebuilt count 为 0。
2. 跨 publication 复用：只改变一个 hover 节点并推进 `presentation_generation`；所有无关 fragment `Arc` identity 必须保持，证明 generation 没有误入 fragment key。
3. 局部状态：一个 hover 节点只重建 interaction fragment，目标 static fragment、旧节点与无关 pane 保持 identity。
4. volatile：动画/caret 连续1000帧只重建 volatile fragment；同节点 static fragment 与 sibling rebuild 为0；volatility membership 退出后恢复复用。
5. clip/z/visibility：跨 fragment 重叠、嵌套 clip、popup、负 z、order-only变化与 hide/show 保持完整 extraction 的像素/顺序等价；隐藏节点没有可重放 fragment。
6. transform：平移、旋转、非均匀缩放通过 replay delta 更新 fragment bounds、hit geometry 与像素，text shape/image decode/static rebuild 为0；无法表示的 clip/effect 使用 typed subtree fallback。
7. 资源：resident SVG/image 不携带 RGBA；resource generation 或 atlas page eviction 只使反向引用者 miss；旧 generation/UV 不被错误复用。
8. segmented submission：warm present 直接迭代 borrowed replay-ready segments，不创建与总 command 数同阶的新 `Vec`，不执行 `HostPaintCommand -> ChromeCommand` 转换或全 command atlas scan。
9. 生命周期/内存：current + 一个 in-flight previous set 后第三代压力触发已声明的 backpressure/fallback；retired bytes 在 quiescence 回到0，无 fragment/resource 泄漏。
10. resize：交互期 fragment build/patch 为0；settle 后只 patch geometry dependency set，tier变化允许 typed subtree fallback。
11. 压力：10,000 visible nodes、4,096次局部 repaint、每次12个候选且1个 changed fragment；证明 warm command materialization 从 `E * V * C` 降为 `E * D * C`。
12. 产品 profile：真实 Editor button storm、window resize、popup、Asset Browser SVG 列表分别采集 CPU、RSS、p50/p95/p99、fragment hit ratio、rebuilt commands、image upload bytes、full/region paint count。

在第12步完成前，确定性模型只能证明算法工作量边界，不能宣称按钮/窗体响应已达到目标。

### 确定性压力模型

`tools/editor_template_command_fragment_cache_pressure.py` schema v2 将“每个节点总 command 数”与“changed role fragment command 数”分开。默认场景为10,000个可见节点、4,096次 region repaint、每次12个候选、1个 interaction-changed node、每节点4条总 command、changed interaction fragment 1条 command：

- current region extraction：49,152次 candidate visit，196,608次 command materialization；
- replay-ready role fragments：49,152次 fragment lookup，4,096次 changed-fragment rebuild/materialization；
- 消除192,512次 command materialization，结构比值48倍；candidate visit不宣称下降。

旧 schema v1 的12倍结果等价于不做 role split、changed node 的4条 command 全部重建；它保留为保守对照，不再代表目标算法。v2 artifact 为 `E:\zircon-profiles\editor-template-command-fragment-cache-pressure-20260828.json`，SHA-256 `A1A84BDF14857B1423A6B4CB2D77D2D99C0006B07265A6E2E13CEC3BD9C6EA9E`。fragment、button appearance 与text decoration三组focused确定性/源码合同合计12/12通过。模型不含CPU、allocator、RSS、latency、segment merge、atlas或GPU计时。

### 产品预算与参考解释

动态验收复用现有 Editor profile gate，而不是为本缓存建立平行指标：同一 current-source PID 执行 warmup、measured、quiescence；button click、pointer 与 resize 场景分开，每个场景至少3轮。暖态 input-to-damage p95 不高于1 ms，CPU damage-to-submit p95 不高于8 ms，端到端 present 必须落在16.67 ms frame budget内；局部 click 的 Full target、full redraw 与无 typed reason fallback 均为0。

RSS 同时记录 warmup末端、measured peak/end和quiescence末端。measured peak 相对 warm baseline 增长不得超过96 MiB，quiescence增长不得超过64 MiB，并且增长必须能由 `fragment_current_bytes + fragment_in_flight_bytes + resource_resident_bytes`解释；`fragment_retired_bytes`在quiescence必须为0。这是现有产品门的绝对防线，另外还必须证明重复场景没有单调增长趋势。

fragment 专属结构门：

- warm button事件中，rebuilt command数不超过实际 changed interaction fragment command数；static/无关 fragment rebuild为0；
- `presentation_generation`推进时无关 fragment identity保持，fragment full-set rebuild为0；
- warm present的command flatten、`HostPaintCommand -> ChromeCommand`转换、全command atlas scan和resident RGBA upload均为0；
- volatile rebuild只等于已发布volatile handle数，render-transform只产生metadata patch；
- popup order/clip、SVG generation/atlas eviction的fallback和反向失效均带typed reason，普通稳定帧为0；
- command/fragment candidate、rebuilt、borrowed与submitted计数守恒，缺样、覆盖、跨PID或旧二进制直接fail closed。

Unreal在这里提供的是合理复杂度与失效边界，而不是可跨硬件照抄的毫秒数：普通Paint/RenderTransform/Volatility走widget级fast path，ChildOrder/Layout或验证失败才进入slow path。Zircon只有同时满足上述结构计数与同机frame-budget数据，才可以声称达到同类引擎的合理行为。

## 7. 本轮不做的内容

- 不修改正在共享重构的 command-stream、presenter、template paint 文件；
- 不为 `UiRenderCachePlan` debug DTO 做与产品链无关的临时分配微优化；
- 不增加第二套 SVG 像素缓存或 Editor 私有 GPU residency；
- 不把当前 region repaint、row index 或 resize snapshot 重新实现一遍；
- 不在 dirty 后首个输入事件 lazy 建 cache。
