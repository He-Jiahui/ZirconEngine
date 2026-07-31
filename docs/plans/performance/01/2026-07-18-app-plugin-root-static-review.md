---
related_code:
  - zircon_app/build.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/prelude.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_app/src/plugins
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
tests:
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/runtime_presenter.rs
  - current-source Windows zircon_app plugin/root tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App plugin composition/root逐文件性能静态审查（2026-07-18，2026-07-22 current-source复核）

## 范围与覆盖

`zircon_app/{build.rs,src/{lib.rs,prelude.rs,runtime_presenter.rs,plugins/**}}`当前源 **8/8** 个Rust文件、**973** 行、**13** 个单元测试已逐文件阅读并通过独立`rustfmt --check`。范围覆盖plugin group builder/default profiles、root/prelude/build wiring及Softbuffer runtime presenter；entry调用链进一步核对到`BuiltinEngineEntry::for_config/bootstrap`。

## 性能结论

Plugin group的HashMap、order Vec、anchor position、descriptor activation sort与module Arc构造只在entry/profile启动装配执行，默认规模约5–12 modules，不进入frame/update/plugin callback热路径。当前源把每个enabled module descriptor冻结在`ResolvedPluginGroup`，nested group不再重复生成descriptor，disabled outer generation也不重复生成；2026-07-22复核确认`builder.rs`/`tests.rs`仍承载共享工作区改动，本切片只读且不覆盖。排序阶段复用既有module key仍有一次启动期短字符串分配机会，但规模与频率不足以越过MVP独立立项门槛，继续并入PERF-MVP-427的startup alloc counter观察。

`module_keys`与selection diagnostics会分配短Vec/String，但只用于启动报告、测试或显式diagnostics。`build.rs`仅输出rerun/link stack参数，root/prelude只有导出。`runtime_presenter`是F2 fallback热路径，但完整RGBA frame已由PERF-MVP-008删除无条件surface preclear；剩余RGBA→XRGB逐像素转换和present是当前fallback必要工作，需由产品scope证明是否还值得SIMD/格式直通，不凭静态形状新增重复计划。

## 参考引擎对照

Bevy `PluginGroupBuilder`同样采用typed map保存entry、Vec保存顺序，before/after通过position/insert完成；其设计假定这是启动配置面，不为小规模组装引入运行时scheduler。Zircon当前结构与该边界一致，性能重点应放在module activation、动态插件加载与每帧hook，而不是优化十余项启动Vec操作。

## 动态验收

待受管Cargo运行plugin/root/presenter聚焦测试；Editor/Runtime/Headless/Minimal各记录descriptor generation、activation sort次数与F0 wall/alloc，单次entry构造每enabled module descriptor应为1，bootstrap不得重新生成。Softbuffer记录`copy_rgba`/present p50/p95与完整/截断payload像素对拍。完成前保持`pending.md`，不进入`review.md`。
