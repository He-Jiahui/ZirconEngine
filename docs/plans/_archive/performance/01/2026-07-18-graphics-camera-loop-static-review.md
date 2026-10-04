---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs
tests:
  - current camera-loop slice 3 of 3 Rust files reviewed, 1372 lines
  - all 15 tests read; terminal/planar hot-path and empty-sequence regressions added
  - terminal submission projection and planar target rescan gate changed from RED to GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 multi-camera trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics camera loop静态审查（2026-07-18）

## 当前源覆盖

`camera_loop.rs`、`camera_loop/tests.rs`与`camera_loop/tests/frame.rs`当前3/3个Rust文件、1,372行已逐文件静态阅读，15条测试均已读。覆盖camera sequence/stack flatten、terminal UI/output ownership、extract/frame streaming restore、planar reflection camera插入及legacy test projection helper。

## 直接止损

present preflight只需viewport terminal target，原实现却调用`camera_loop_submissions`完整构造全部camera submission descriptor再find owner；真正submit随后再次解析sequence/plan。本轮terminal lookup直接从resolved sequence定位最后primary base（无primary则最后base）的stack terminal camera并读取target，不再物化submission Vec，现有primary/offscreen测试语义保持。

每个planar probe原先对当前/augmented camera列表执行`iter().any`查texture target，P probes×C cameras线性放大。本轮先建立existing texture target id `HashSet`，成功派生camera后才insert；on-demand/every-frame/dirty/already-submitted与derive失败语义不变。两项源码门禁先RED后GREEN，新增hot-path回归，rustfmt与diff检查通过。

## 剩余根因

present仍先为terminal target resolve sequence，进入submit又resolve并flatten第二次；camera descriptor/stack/output policy尚无共享generation plan。多camera streaming为source post-process state clone volumes、stack、graph一次，并为后续每camera `clone_from/clone`恢复；frame path还clone完整`FrameVisibility`。每camera随后独立构建context、compile key、visibility、VG diagnostics、feedback/history，分别由PERF-MVP-414..416描述。

planar update mutex覆盖probe扫描、camera派生和首次完整camera Vec clone；新增target set只消除查重放大。一次loop后才批量mark captured，后续camera失败会让已完成probe下帧重做。新增`PERF-MVP-417`要求Render09发布generation-owned `CompiledCameraSubmissionPlan`，present preflight与submit共享；camera slot携target/order/UI/output policy，source post/visibility/advanced payload以Arc generation handle恢复。Render11按probe dirty/target/camera generation增量派生，Runtime07短锁快照plan并在成功camera级别提交进度；Editor01不得在UI/controller锁内等待整loop。

本地Bevy `bevy_render::camera::SortedCameras`使用单一可复用Vec resource由camera driver消费，change/order/target信息集中；采用的是“排序/计划单owner、driver借用消费”的原则，不复制其ECS schedule、target clone或ambiguity实现。

## 验收状态

静态、源码RED→GREEN、rustfmt与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，15条测试没有current-source结果；RenderDoc CLI不可用且无capture。1/8/64 cameras、0/1k probes、large post/visibility payload的plan build/clone/lock/CPU/GPU与failure retry动态证据未完成，继续留在`pending.md`，不进入`review.md`。
