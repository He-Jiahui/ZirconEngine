---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/record_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/release_previous_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
tests:
  - current feedback/history state slice 7 of 7 Rust files reviewed, 758 lines
  - all 9 tests in the slice read; history reuse and particle capacity regressions added
  - history comparison and particle Vec source gate changed from RED to GREEN
  - stale non-viewport VG stats argument found and repaired during full call-site review
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 camera-stack trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics feedback/history静态审查（2026-07-18）

## 当前源覆盖

submit执行路径中的feedback/history状态切片当前7/7个Rust文件、758行已逐文件静态阅读，包含runtime feedback merge、history handle resolve/release、非主camera记录、particle previous-state和temporal camera更新；9条测试均已读。另读取viewport particle/motion camera owner和particle ambiguity算法确认容量与重复扫描边界。

## 直接止损

frame context已经计算并保存完整`FrameHistoryInvalidationReason`，`resolve_history_handle`却再次查同一camera history并比较size、pipeline、bindings和validation key。本轮删除第二次兼容性计算，rotation直接消费context reason；`FrameInputsChanged`仍只失效内容而不重分配，resize/pipeline/no-history等分类不变。

particle previous-state原先每camera frame收集新Vec后`HashMap::insert`替换旧Vec，稳定粒子数也丢弃容量。本轮camera slot通过entry取得既有Vec，原地clear/extend；新增容量回归确认同camera复用allocation，anonymous ambiguity过滤及billboard basis语义不变。源码门禁先得到两项RED，修改后均GREEN，rustfmt和diff检查通过。

逐调用点复核还发现前一批删除VG冗余segment-count参数后，非主camera记录路径仍残留`0`实参。本轮同步删除并把该路径加入源码回归；这是current-source类型错误修复，不把未运行Cargo写成已编译通过。

## 剩余根因

每个camera提交都从renderer和prepared sideband各take一份Hybrid GI/VG/particle readback并合并大量Vec；context中的Hybrid GI/VG visibility feedback随后又clone进入runtime feedback。VG debug snapshot还读取同一feedback，说明当前没有single sealed feedback owner。camera stack的非终端路径构造完整particle feedback后直接丢弃，global/viewport simulation feedback的消费身份不明确。

particle previous-state与velocity path分别构建anonymous entity BTreeMap/BTreeSet，frame context的matched/missing/ambiguity统计又重复扫描；capacity复用只止住结果Vec，不解决重复分类。history allocation/release和provider update仍在全局state transaction中，history内容大clone继续由`PERF-MVP-413`负责。

新增`PERF-MVP-415`要求generation-tagged `RuntimeFeedbackTicket`和唯一shared-product camera owner：GPU readback通过固定in-flight ring跨帧完成，renderer/sideband在producer边界合并一次；debug/runtime/stats借用同一sealed artifact；particle ambiguity/history发布一次report并复用scratch。该计划复用`PERF-MVP-341/379/396/410/411/413`，不另建插件或particle私有队列。

本地Bevy `gpu_readback.rs`采用跨帧map completion而非提交点同步取回；本地UE `RHIGPUReadback`使用可复用staging与fence。这里只采用“有界跨帧ticket、无frame-thread wait”的边界，Zircon仍需保持自己的camera/plugin generation和错误语义。

## 验收状态

静态、源码RED→GREEN、rustfmt与diff门禁已完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，9条测试及残留调用修复没有current-source编译结果；本机RenderDoc CLI不可用且无capture。feedback bytes/merge visits、camera owner、readback age/drop、particle规模和history rotation动态证据均未完成，继续留在`pending.md`，不进入`review.md`。
