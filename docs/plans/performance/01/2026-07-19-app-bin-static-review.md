---
related_code:
  - zircon_app/src/bin
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - zircon_app/src/runtime_presenter.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
tests:
  - zircon_app/src/bin/zircon_shader_pbr_viewer
  - current-source Windows Cargo and F2 viewer traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App bin逐文件性能静态审查（2026-07-19）

## 范围与覆盖

`zircon_app/src/bin/**`当前源 **11/11** 个Rust文件、**2,428** 行、**43** 条测试已逐文件阅读。`editor.rs`与`runtime_preview.rs`只是参数转发入口，无独立热循环；其余9个文件构成F2 PBR/HDRI诊断viewer，覆盖winit app、后台加载、HDRI/PMREM staging、临时project asset、camera、scene render、Softbuffer present和RenderDoc触发。

5个viewer文件包含其他会话的当前改动，本轮只读并保留。当前实现已具备background load、event-loop proxy唤醒、1秒loading title限频、redraw transition合并、单次HDR decode、runtime compute-pool IBL staging、持久IBL cache和静态preview复用；这些方向正确，不能回退。

## 性能结论

- **PERF-MVP-428 / Runtime04**：每次viewer启动仍生成固定96×192 UV sphere，共18,721 vertices和110,592 indices，序列化大TOML并写临时project；`write_viewer_project_assets`先`ProjectManager::open + scan_and_import`以解析引用，随后runtime `AssetManager::open_project`再次打开/投影同一project。F2冷启动因此混入稳定fixture生成、文件I/O和重复project discovery。
- scene每次orbit/zoom render把`EnvironmentExtract` clone进packet；source-cubemap内含mip-chain payload，需复用PERF-MVP-352/414的generation-owned environment artifact，而不是在viewer另建私有缓存。
- viewer通过`SceneRenderer::render`得到完整CPU RGBA再Softbuffer present，属于诊断fallback；真实native present/readback预算继续归PERF-MVP-023与Render17，不能把该viewer的同步读回当产品稳态路径。
- HDRI exposure最多约128×64采样，staging使用runtime compute pool且input只读/解码一次；background task只发送一个结果，当前mpsc无界形态没有增长源，不新增任务。

## 本轮直接止损

`zircon_shader_pbr_viewer/presenter.rs`按RED→GREEN测试改为完整RGBA payload直接覆盖XRGB surface，仅截断payload预清零未覆盖像素，删除每次present一次完整surface写入。两分支测试形态、`rustfmt`与`git diff --check`已通过；该修复复用PERF-MVP-008的既有合同，current-source Cargo仍因受管validator JSON入口失败而pending。

## 动态验收

F2分别执行cold/warm IBL cache、startup与一次orbit redraw，记录sphere generation、TOML bytes、project open/scan/import、environment clone bytes、RGBA readback/copy、CPU p95和wall；warm生成物路径固定fixture work应为0。Softbuffer完整/截断像素测试与截图对拍必须通过；真实graphics backend补GPU timestamp和RenderDoc capture。完成前模块保留在`pending.md`，不进入`review.md`。

## 责任计划交接

Runtime04：`runtime/04/failure-2026-07-19-pbr-viewer-generated-project-rebuild.md`。native present/readback复用PERF-MVP-023，source-cubemap ownership复用PERF-MVP-352/414。
