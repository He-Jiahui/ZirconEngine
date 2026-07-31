---
related_code:
  - zircon_runtime/reflection_macros/src
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
reference_sources:
  - dev/bevy/crates/bevy_reflect/derive/src/derive_data.rs
tests:
  - zircon_runtime/reflection_macros/src/tests.rs::macro_expansion_sources_use_single_field_and_module_item_passes
  - current-source proc-macro Cargo and generated-module compile counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime reflection macros逐文件性能静态审查（2026-07-22）

## 范围与覆盖

计划账本原路径`zircon_runtime/src/reflection_macros`不存在；真实proc-macro crate为`zircon_runtime/reflection_macros/src/**`，当前源 **8/8** 个Rust文件、**927** 行、**11** 条测试已逐文件阅读。覆盖attribute/CLI-style meta解析、`ZirconScriptType` derive、host function/module展开、type-ref tokens和全部macro token测试。

## 发现与直接止损

- `derive_zircon_script_type_impl`原先分别调用`field_registration_tokens`和`field_projection_tokens`，每个field的全部`zircon_script` nested meta与type-ref inputs解析两遍。本轮合并为单个`field_tokens` pass，一次解析同时生成两个TokenStream，skip/name/type/documentation语义不变。
- `host_module_impl`原先对inline module items分别filter-map function和script type两遍。本轮单次遍历分类两类ident，保持原item order与descriptor/export order。
- `HostFunctionArgs`少量key字符串化、format_ident与quote属于编译期且输入规模很小；`syn = { features = ["full"] }`是ItemFn/ItemMod所需，不作为独立热点。
- 宏生成的host module descriptor函数仍会在被调用时构造owned module/type/function descriptor、capability和field Vec；它只在registration/docs control-plane可达，继续由PERF-MVP-331/Runtime13按registration generation共享，不为proc-macro另建运行时缓存真相。

两项修复均先得到源码契约RED，再完成GREEN、`rustfmt --edition 2021`与scoped `git diff --check`。受管Cargo槽不可用时没有运行raw Cargo。

## 参考约束与动态验收

Bevy reflection derive先把容器/field attributes收敛进`ReflectMeta`、`ReflectStruct.fields: Vec<StructField>`，后续多个展开阶段复用已解析字段元数据；本轮单次field parse与该方法一致。后续若descriptor规模成为构建瓶颈，应进一步建立本仓自己的parsed field model，而不是在registration/projection生成器里重新解析syn AST。

动态验收需要1/100/10k fields与module items的proc-macro wall、peak RSS、attribute parse count、expanded token bytes和incremental rebuild范围；同时验证macro token golden、Runtime13 host modules与reflection tests。current-source Cargo和规模计数完成前留在`pending.md`，不得进入`review.md`。
