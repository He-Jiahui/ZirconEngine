---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline
  - zircon_runtime/src/graphics/runtime/render_framework/set_pipeline_asset
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile
tests:
  - current pipeline and profile management slice 14 of 14 Rust files reviewed, 1406 lines
  - reload, set-pipeline and set-profile compile-outside-state-lock source regressions added
  - quality profile is moved into viewport state with name-only clone
  - scoped rustfmt and diff check passed
  - current-source Cargo 22 tests and F2 switch/reload traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics render-framework pipeline/profile管理静态审查（2026-07-18）

## 当前源覆盖

当前14/14个Rust文件、1,406行已逐文件静态阅读，覆盖capability validation、quality-profile compile options、pipeline register/reload/set及profile set。当前22个测试均已阅读；本轮新增三条锁顺序源码回归，但Cargo未执行。

## 直接止损

`register_pipeline_asset`已在取得framework锁前compile validation graph；`reload_pipeline`、`set_pipeline_asset`和`set_quality_profile`原先却在全局operation+state mutex内深clone pipeline并完整compile graph。三条路径先增加RED顺序门禁，本轮保留operation transaction，改为短state锁快照asset/capabilities，锁外`compile_pipeline_for_validation`，再短state锁按原executor→capability→profile错误优先级validate/publish。源码门禁确认三条均为snapshot < compile < relock，graph compile的state-lock hold=0。

profile提交原先`profile.clone()`保存viewport、再move原profile name到stats，导致整份features/config深clone。本轮只在锁外clone短name，随后把profile整体move进record；错误顺序和generation bump不变。

## 剩余根因

set/reload/profile仍为每次操作深clone完整`RenderPipelineAsset`并重新compile validation graph，即使handle+revision+executor/capability generation已验证；全局operation mutex也仍跨compile，多个viewport和编辑器操作互相等待。新增`PERF-MVP-412`要求register/reload发布immutable validated revision artifact，set只做O(1) handle/compatibility检查，reload通过single-flight ticket锁外compile并CAS发布last-good。

quality compile options反复构造feature/capability sets并写plugin feature String，default/pipeline/profile variants没有共享generation artifact；该部分联动`PERF-MVP-362/365/412`。capability mismatch成功路径只创建空Vec且要求数很小，错误路径保留全部missing detail和顺序，不做微优化。

## 验收状态

三文件已通过rustfmt、锁顺序/profile ownership源码合同和`git diff --check`。Cargo协调器仍在启动前JSON解析失败，22个selection/reload/capability tests没有current-source结果；pipeline 1/16/100/1k、passes 16/256/1k、viewports 1/8/64并发set/reload、F2热切换/plugin executor与失败rollback均未验收。继续留在`pending.md`，不进入`review.md`。
