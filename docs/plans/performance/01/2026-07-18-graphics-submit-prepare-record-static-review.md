---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
tests:
  - current prepare/record slice 7 of 7 Rust files reviewed, 445 lines
  - provider Arc, shared compiled pipeline, centralized VG stats and traversal source regressions added
  - anti-pattern source gate changed from RED to GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 product trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics submit prepare/record静态审查（2026-07-18）

## 当前源覆盖

`prepare_runtime_submission/**` 2/2和`record_submission/**` 5/5，共7/7个Rust文件、445行已逐文件静态阅读。为确认调用语义，还完整读取`frame_submission_context.rs`、`virtual_geometry_stats.rs`、runtime history五文件、provider registration和viewport capture owner；这些支撑文件不冒充其父目录已验收。

## 直接止损

Hybrid GI与Virtual Geometry prepare原先为了绕过state可变借用，每个启用feature、每camera frame深clone整份provider registration，连`provider_id: String`一起复制。本轮为registration增加只共享trait-object `Arc`的窄接口，prepare只做原子引用计数，不复制provider metadata。

record与present原先每次成功提交都深clone完整`CompiledRenderPipeline`，包括graph、pass metadata及多组Vec；现由context到viewport只传`Arc<CompiledRenderPipeline>`。离屏record还在全局state锁内每帧执行`graph().dump().to_text()`；现用独立的last-capture pipeline identity复用稳定compiled generation的既有文本，且显式避免`A capture -> B present -> B capture`误用A文本。对外`CapturedFrame.graph_dump: Option<String>`合同未变。

Virtual Geometry executable segment count原先在record阶段新建合并`BTreeSet`扫描一次，`update_stats`随后又建resident/requested/seen/all四套集合并重扫；record侧重复owner已删除，由最终stats扫描发布同值。统计内部同一page state由每segment两次求值降为一次，node/cluster hierarchy traversal由两次完整BFS降为一次共享结果。源码反模式门禁先得到四项RED，再在修改后全部GREEN；新增三条源码回归，rustfmt与`git diff --check`通过。

后续逐调用点审查发现非主camera记录仍传递已删除的VG segment-count `0`实参；现已同步修正并把非主camera source纳入同一回归。Cargo仍未执行，所以这里只声明源码签名一致，不声明编译已通过。

## 剩余根因与参考

`record_history`仍在每camera frame复制history bindings、完整visibility history snapshot、static index和validation key；capture仍保存完整RGBA并在查询时复制owned frame。prepare、render、feedback、record、history release和stats也仍处于全局framework state锁事务内。新增`PERF-MVP-413`要求发布generation-owned submission/history artifact、on-demand/shared graph diagnostics及短锁capture/history snapshot；锁域与异步readback继续服从`PERF-MVP-023/410/411`，不建立第二套队列权威。

本地UE参考`RenderGraphBuilder.cpp`把RDG debug工作置于`RDG_ENABLE_DEBUG`，texture visualization还先检查`IsRequestedView()`/`ShouldCapture()`再创建capture pass；采用的是“诊断请求门控、compiled metadata共享”原则，不复制其C++ API或宏结构。

## 验收状态

当前只完成静态、源码RED→GREEN和格式检查。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，本轮不重复绕过协调器运行raw Cargo；provider/record新增测试及相关现有测试没有current-source结果。本机`renderdoccmd.exe`此前已确认command-not-found，且本切片没有可用capture，所以GPU/锁内wall、F2多camera与真实capture仍未验收。该切片继续留在`pending.md`，不进入`review.md`。
