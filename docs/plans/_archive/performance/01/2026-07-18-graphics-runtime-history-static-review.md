---
related_code:
  - zircon_runtime/src/graphics/runtime/mod.rs
  - zircon_runtime/src/graphics/runtime/history
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
tests:
  - current graphics runtime history slice 8 of 8 Rust files reviewed, 294 lines
  - all 1 ownership regression read; wide validation-key sharing regression added
  - per-record validation-key deep clone gate changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 temporal-history trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics runtime history静态审查（2026-07-18）

## 当前源覆盖

`graphics/runtime/history/**`七个Rust文件与runtime root wiring当前8/8个文件、294行已逐文件静态阅读，1条新增所有权回归已读。覆盖history construct/update/access、target/render/pipeline/bindings/validation compatibility、visibility/static snapshot及模块出口；同时复核context与record调用边界。

## 直接止损

`FrameHistoryValidationKey`包含camera descriptor、全mesh validation rows、完整lighting、animation poses、post-process、particles与effective feature Strings。context每camera已构造该wide key，`record_history`原先在history update/new时再完整clone一次。本轮context和`ViewportFrameHistory`内部统一持有`Arc<FrameHistoryValidationKey>`，record只clone Arc；`is_compatible`仍对`Arc::as_ref()`与当前key做完整内容比较，ViewportResized→RenderSizeChanged→PipelineChanged→HistoryBindingChanged→FrameInputsChanged错误优先级不变。源码所有权门禁先RED后GREEN。

## 剩余根因

validation key首次构造仍逐camera clone lighting/post/particles，遍历全部meshes与animation poses并复制动态layer/pose数据；随后compatibility按完整结构深比较。stable multi-camera/scene generation没有复用sealed hash/revision。`record_history`还每帧clone compiled history bindings、visibility history snapshot与static index；这些大payload在state锁内提交。继续归PERF-MVP-413/414与Render06：scene/camera/post/particle/feature generation形成compact validation token，history只持Arc token与共享snapshot handles，changed按component revision定位失效原因。

本地Bevy view history/temporal resources按camera/view entity在render world中跨帧保留，并由change/extract generation更新；本地Unreal temporal history由view state owner持久管理。采用“stable view slot持有共享generation history、输入变化用revision失效”的原则，不复制其ECS/RDG类型。

## 验收状态

静态、Arc ownership RED→GREEN、rustfmt、source contract与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，新增回归及现有graphics history集成测试没有current-source结果；RenderDoc CLI不可用且无capture。meshes/poses 0/1k/100k、cameras 1/8/64、stable/1% input change的key build/clone/compare、bindings/visibility/static clone bytes、state lock与history parity未完成，继续留在`pending.md`，不进入`review.md`。

## 2026-08-29 当前源码取代说明

本记录的Arc共享结论仍有效，但“wide validation key仍深比较”和“内容component revision变化即全局失效”已被后续正确性复核取代。当前key只比较world identity、camera结构合同和effective feature集合；正常scene/camera/light/post/particle变化不再清空所有history。camera cut连续性判定与velocity共享。P0-2分域generation、资源共享clone/lock成本和0/1k/100k规模实测仍未完成，最新量化计划与静态结果见`2026-08-29-temporal-history-compatibility-structural-review.md`。
