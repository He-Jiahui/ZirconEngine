---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
tests:
  - current submit execution slice 3 of 3 Rust files reviewed, 1036 lines
  - all 2 submit-slice inline tests read; 2 virtual-geometry ownership/lock support regressions added
  - virtual-geometry snapshot global/per-camera hot-path deep clones changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 multi-viewport trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics submit execution静态审查（2026-07-18）

## 当前源覆盖

`submit.rs`、`submit_runtime_frame.rs`与`present_frame_extract.rs`当前3/3个Rust文件、1,036行已逐文件静态阅读，2条内联测试均已读。覆盖extract/runtime-frame两类submit、surface present preflight/lease、context/prepare/history、GPU render、capture、feedback、generation validation、viewport/non-viewport record、temporal/particle提交与stats更新。

## 直接止损

`RenderVirtualGeometryDebugSnapshot`含二十余组实例、page、cluster、traversal、visbuffer与submission `Vec`。拥有shared viewport products的相机原先先把完整snapshot深clone到framework global state，随后`record_camera_product_reports`又按camera key深clone一次；公开query再按请求复制。内部frame、global state和per-camera map现统一持有`Arc<RenderVirtualGeometryDebugSnapshot>`，热路径两次复制降为引用计数增加；公开query在短锁内只clone Arc，释放state guard后才复制独立owned snapshot，外部接口未改变。新增`Arc::ptr_eq`与锁外clone顺序回归先RED后GREEN；resident VG upload入口借用Arc，renderer仍只读snapshot。

## 剩余根因

三个入口都持有全局`operation_lock`覆盖完整multi-camera loop。每camera又从`prepare_runtime_submission`前持有`RenderFrameworkState` mutex，跨history resolve、renderer CPU/GPU encode/submit或surface present、capture finish/relock、feedback take/merge、generation validation、record/history和stats。present还在同一state锁内lease并restore surface。任何慢pipeline/provider、driver、GPU submission、feedback或大stats都会串行阻塞其他viewport lifecycle/query/submit；直接runtime-frame入口还把context visibility深clone回frame，继续归PERF-MVP-417/414。

本批收紧`PERF-MVP-411`：Runtime07/Render10把三条近重复入口收敛为generation-owned `PreparedSubmissionTransaction`，Phase A短锁快照viewport/camera/pipeline/history/capture token，Phase B在ordered render-owner lane执行prepare/render/present/feedback且不持framework state锁，Phase C短锁校验generation并CAS提交record/stats/history。surface lease改为render-owner资源ticket，same viewport保持顺序，独立viewport和只读query不被慢提交阻塞。PERF-MVP-416继续要求debug-off不构造snapshot，debug-on每generation只构造一次Arc report；本轮只消除了构造后的内部深clone。

本地Bevy把各view生成的`PendingCommandBuffers`集中到render schedule末尾一次`queue.submit`，并用独立RenderApp资源承载render state；本地Unreal Engine用`ENQUEUE_RENDER_COMMAND`把game/editor线程请求移交render command lane。采用的是“提交有唯一render owner、调用线程传递generation payload、队列末端统一提交”的边界原则，不复制它们的ECS/RHI API或线程模型。

## 验收状态

静态、共享所有权/锁外query clone RED→GREEN、rustfmt、source contract与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，2条submit内联测试、2条support回归及受影响类型调用没有current-source Cargo结果；RenderDoc CLI不可用且无capture。1/2/8/64 viewports、慢driver 0/10/100 ms、VG snapshot 0/64 MiB的operation/state lock wait/hold、Arc clone、submit数量与frame p95动态证据未完成，继续留在`pending.md`，不进入`review.md`。
