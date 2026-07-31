---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/runtime_feedback_batch.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs
tests:
  - current submit root-contract slice 7 of 7 Rust files reviewed, 1374 lines
  - all 11 tests read; mutable generation-guard lookup regression added
  - mutable viewport generation lookup gate changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 transaction trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics submit root contracts静态审查（2026-07-18）

## 当前源覆盖

`submit_frame_extract`六个root Rust文件及`frame_submission_context/tests.rs`当前7/7个文件、1,374行已逐文件静态阅读，11条测试均已读。覆盖submission context完整owned/borrowed surface、advanced feature降级清理、TAA jitter、prepared sideband与feedback move DTO、submission record scalar snapshots、viewport generation validation及模块出口。连同已记录的各子切片及后续history Arc accessor，`submit_frame_extract/**`当前56/56个Rust文件、10,856行、88条测试已完成静态覆盖。

## 直接止损

`viewport_record_mut_after_generation_check`原先先调用只读guard做一次`viewports.get`，随后再`get_mut`，helper内部同一state guard做两次HashMap probe；submit owner在调用helper前还已做一次公共validation，合计3次。本轮mutable helper改为一次`get_mut`后原位比较generation，保持UnknownViewport/ViewportChanged错误字段与返回record语义；helper内部lookup 2→1，owner总路径3→2。新增单lookup源码回归先RED后GREEN。

`PreparedRuntimeSubmission::into_prepared_runtime_sidebands`、`RuntimeFeedbackBatch::into_parts`及`SubmissionRecordUpdate`已经采用move/scalar snapshot，没有发现简单深clone修复；保留现有owner边界。

## 剩余根因

`FrameSubmissionContext`仍是四十余参数的大型per-camera聚合，owned保存quality String、visibility、camera order、post graph、advanced/Solari、GI/VG extract/plan/feedback及多组Vec。虽source extract与compiled pipeline已Arc化、provider/VG outputs已move，但multi-camera仍按camera构造/恢复大量相同payload，继续归PERF-MVP-414/417。feedback sealed owner归415，stats scalar snapshot是418最终sealed report的局部基础。

owner camera当前仍在render完成后先执行公共`validate_viewport_generation`，紧接着mutable helper再次比较同generation；同一state guard下语义安全但仍2次map probe。PERF-MVP-411 Phase A应预留stable viewport slot/ticket，Phase C用slot+generation直接CAS publish；destroy/recreate让slot generation失效，不以裸handle二次查表。禁止在没有并发race/Cargo验证时局部删除前置validation及capture error handling。

本地Bevy RenderApp用extract/prepared resources在render schedule内传递owned generation数据，本地Unreal Engine render command携带调用侧封存payload交给render owner；采用“context封存一次、执行/提交借用、完成按generation发布”的原则，不复制其ECS resource或RHI command API。

## 验收状态

静态、single-lookup RED→GREEN、rustfmt、source contract与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，11条测试没有current-source结果；RenderDoc CLI不可用且无capture。1/8/64 cameras、viewport destroy/recreate/resize race、context payload 0/64 MiB的map probes、clone bytes、state lock与failure/capture parity未完成，继续留在`pending.md`，不进入`review.md`。
