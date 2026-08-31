# PFO-4d2e Forward Receiver Binding Profile Plan

## 状态

- 日期：2026-08-27
- 当前状态：`profile_instrumentation_source_implemented_static_checks_passed_dynamic_capture_pending`
- 范围：forward receiver standard/full bind-group创建的逐帧计数与CPU scope
- 前置：PFO-4d2c/d已修复provider/lightmap generation publication；receiver cache候选仍禁止在无profile时实施。

## 结构重审与观测缺口

`MeshPipelineCache`的forward receiver group聚合shadow atlas、light grid、reflection probe、lightmap、volumetric、transmission、cookie与irradiance bindings。standard路径当前由depth、gbuffer、shadow、velocity、TAA reactive mask等多个consumer调用；full路径还绑定graph transient light-grid/volumetric/transmission资源。源码调用次数只能证明候选规模，不能证明native create CPU成本或它是否为实际瓶颈。

现有profiling系统支持frame-indexed counter与命名CPU scope，RenderGraph也已有TAA/HZB create count先例，但forward receiver创建没有任何样本。直接加入generation cache会把不同资源失效语义混在一起，并违反“先测量、后优化”的门槛。

## Unreal对齐与设计

Unreal通过stat/RDG event/RHI resource diagnostics先区分scene/view snapshot创建与pass-local descriptor work，再决定缓存层级。Zircon先建立同等观测边界：standard固定shape与full graph-transient shape分别计量。

1. `MeshPipelineCache`拥有两个frame-local计数：standard receiver bind-group create、full receiver bind-group create。
2. direct/compiled render唯一frame入口在pipeline submission usage recording之后清零计数；不依赖历史帧扫描。
3. standard创建函数使用`forward_receiver/standard_bind_group_create` CPU scope，full创建函数使用`forward_receiver/full_binding_prepare` scope；后者明确包含volumetric params buffer准备与bind-group创建。
4. 每次真实`device.create_bind_group`成功返回后对应计数saturating +1。
5. direct/compiled唯一成功出口各上报一次`render` counter；失败帧不混入成功帧性能分布。profiling feature关闭时宏保持零运行时记录开销。
6. 不改变bind-group创建、缓存、资源owner、queue/submission或graph生命周期；本切片只提供性能工具证据入口。

## 动态判定门槛

- Windows产品场景采集environment-only/full deferred、1/4/16/64 shadow slot与transparent/OIT组合至少300稳定帧。
- 分别报告standard/full count p50/p95/p99、scope inclusive CPU p50/p95/p99与占frame-thread比例，并用RenderDoc/API validation核对native create事件。
- 只有standard create count稳定大于0且CPU占比达到已记录阈值，才允许设计generation snapshot cache；full路径必须按graph transient identity单独评估，禁止复用standard key。
- 源码阶段仅要求counter reset/increment/emit顺序、局部`rustfmt --check`与scoped `git diff --check`。未取得动态数据前不得宣称瓶颈或缓存收益。

## 当前完成项与静态结果

- `MeshPipelineCache`新增standard/full两个frame-local create计数。direct与compiled帧入口分别清零，唯一成功出口分别上报一次profile counter。
- standard路径带`forward_receiver/standard_bind_group_create` scope；full路径带`forward_receiver/full_binding_prepare` scope，明确覆盖volumetric params准备和最终bind-group创建。
- 两条真实创建路径各在`device.create_bind_group`返回后执行一次saturating increment；helper仍只有一个native create调用，未改变任何绑定内容或缓存行为。
- 五个触及Rust文件的局部`rustfmt --check`、scoped `git diff --check`、reset/increment/emit数量和成功出口顺序检查通过。未运行Cargo、产品profile、WGPU、PNG、RenderDoc或功耗验收。
