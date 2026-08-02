---
related_code:
  - zircon_app/src/entry/
  - zircon_app/src/entry/cli
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
tests:
  - inline unit tests: 9
  - zircon_app/src/entry/tests
  - current-source managed Windows Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App entry根与CLI当前源码复核（2026-07-23）

## 范围与基线

`zircon_app/src/entry`根9个Rust文件及`cli/**`2个Rust文件，当前源码 **11/11**、**1,816** 行、**9** 条内联测试已逐文件阅读；path+Git-blob清单SHA-256为`d00b3fa541415ab9bcfce120a5c311d3a209569d976ce1274eba6b30652cebfc`。`engine_entry.rs`、`entry_config.rs`和`mod.rs`有外部未提交修改，`platform_preferences.rs`未跟踪；本轮只读保留。

## 关键结论

- **PERF-MVP-004 / Runtime02**：`ResolvedPluginGroup` current已让report、排序和bootstrap共享冻结descriptor snapshot，旧的每模块重复生成已静态止损；但`DescriptorBackedEngineModule::new`仍为每个动态plugin module把`descriptor.name`与`description`分别clone后`Box::leak`为`'static`。module-selection/report/bootstrap或reload重复构造entry时，wrapper Drop不会释放这两段文本，进程RSS永久增长。
- **PERF-MVP-427 / Editor01+Editor12**：`EntryConfig::project_plugin_manifest`复制完整manifest；module selection又复制effective manifest、registration/feature reports和module descriptors，重建catalog/extension report。`BuiltinEngineEntry::bootstrap`每次重建builtin devtools catalog，并在activation前后两次写入同一entry config。single prepared startup artifact仍缺失。
- first-party render plugin补选只枚举3个固定feature，并在manifest中线性查重；window/platform preference安装和CLI commandlet路由都是启动期工作，没有独立每帧热点。export-root discovery构造少量ancestor candidates且只在显式export bootstrap发生，不单独编号。

## 目标与验收

Runtime02把`EngineModule::module_name/module_description`硬切为借用`&str`，或等价地让冻结module artifact显式拥有`Arc<str>`并保证借用不逃逸；`DescriptorBackedEngineModule`直接借用自身descriptor，生产源码`Box::leak`为0。已有descriptor snapshot继续作为report/sort/register唯一generation，不新增global interner或editor/bin旁路缓存。

Editor01/12消费同一prepared project/plugin/registration/catalog/module artifact；config/catalog/descriptor每entry generation各构建≤1，activation前后同值store为0或由明确changed-generation证明必要。用dynamic modules 1/100/1,000、entry/report/reload 1/1,000/100,000记录descriptor calls、String owners/leaked bytes、catalog/manifest clone bytes、config writes、RSS和F0 wall/p95；drop/retire后dynamic name bytes回收。current-source Cargo和counter未完成，不进入`review.md`。
