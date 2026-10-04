---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_trait_binding
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework
tests:
  - current render-framework core-owner slice 12 of 12 Rust files reviewed, 624 lines
  - all 1 inline test read; capture state/lock/trait call graph traced
  - scoped source contracts and diff check passed
  - current-source Cargo, F2 multi-viewport trace and RenderDoc CLI capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics render-framework core owner静态审查（2026-07-18）

## 当前源覆盖

`graphics_debugger_capture/**`、`render_framework_state/**`、`render_framework_trait_binding/**`及`wgpu_render_framework/**`当前12/12个Rust文件、624行已逐文件静态阅读，1条锁中毒恢复测试已读。覆盖RenderDoc env/request/status、pending/queued/active状态机、capture begin/stop/relock/error merge、全局state字段、公开RenderFramework转发面、operation/state/planar锁及GPU测试辅助入口。连同此前各切片及后续history Arc accessor，`runtime/render_framework/**`当前124/124个Rust文件、15,003行、133条测试已完成静态覆盖。

## 当前正确边界

graphics debugger请求最多保留一个pending和一个latest queued viewport；active期间重复请求覆盖queued而不无界增长。finish把queued原子提升为pending，destroy会清理相关请求并报告错误。capture stop先在state锁内取得backend stop owner，随后释放state锁再执行可能阻塞的stop/poll，最后重锁提交generation/error；submit/stop双错误只在错误路径格式化。未发现需要为队列、正常路径String或DTO增加局部修复。

RenderFramework trait binding只转发owned extract/UI/surface/profile参数；WGPU lock helper对poison恢复一致。测试专用readback/diagnostics入口虽持operation/state锁跨GPU读取，但由`#[cfg(test)]`隔离，不计产品热路径。

## 剩余根因

`RenderFrameworkState`仍把SceneRenderer、pipeline/cache、providers、全部viewports、RenderStats与graphics debugger放在单一Mutex。capture stop虽释放state锁，但外层operation lock仍覆盖stop/poll，因而一次RenderDoc capture可以阻塞所有viewport lifecycle/submit和pipeline/profile control；start/prepare stop仍在state锁内调用backend。该根因继续归PERF-MVP-411的render-owner ordered lane与PERF-MVP-023的异步有界capture/readback，stats/query共享归418。

`GraphicsDebuggerStatus`查询在state锁内clone backend/error String，当前payload小且低频，不单独改公共DTO。最终由generation-owned debugger status Arc与if-newer query锁外materialize；Editor工具不得按UI tick高频轮询。RenderDoc CLI在当前环境不可用，本批没有生成或冒充`.rdc`证据。

本地Unreal Engine `RenderCaptureInterface`通过`ENQUEUE_RENDER_COMMAND`把begin/end capture交给render command owner；本地Bevy把GPU command buffers集中在render schedule末端提交。采用“capture/submit同一render owner、调用线程只交ticket、完成按generation发布”的边界原则，不复制其RHI或schedule API。

## 验收状态

静态与diff门禁完成，无简单代码改动。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，1条测试没有current-source结果；`renderdoccmd.exe`不可用且本切片无capture。1/2/8/64 viewports、capture request burst、stop/poll 0/10/100 ms、destroy/error race的operation/state wait、queue overwrite、status clone与submit p95证据未完成，继续留在`pending.md`，不进入`review.md`。
