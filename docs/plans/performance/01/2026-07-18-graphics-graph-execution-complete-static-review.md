---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_builtin_postprocess_executors.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphPass.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
tests:
  - graph_execution current Rust source forty-seven of forty-seven files reviewed, 14478 lines
  - unused uber resource-routing scan source guard RED then GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, scale counters, F2 pixels, plugin reload and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics graph execution全目录逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/graph_execution/**/*.rs`当前47/47个Rust文件、14,478行，包括瞬态资源物化、execution record、pass context/GPU resource lookup、内建scene/post executors、SSR和executor registry及全部tests。先前分切片证据已经覆盖物化、record、context、GPU post和render-pass leaf；本记录关闭余下root、内建executor、SSR与registry逐文件静态覆盖，不代表动态验收完成。

## 本轮直接止损

`uber_postprocess_executor`原先调用`uber_input_resource(context)`扫描pass资源声明，但选出的名字存入下划线变量后从未传给GPU记录路径；真实`record_post_process_stack`会自行按post graph选择scene-color输入。已用结构源码守卫先复现RED，再删除产品路径的无效扫描，并把只服务routing单元测试的helper限制在`cfg(test)`。该改动不改变resource declaration、attachment、error或draw语义；scoped rustfmt与`git diff --check`已过，Cargo仍待协调器批量执行。

## PERF-MVP-399：编译期dense executor与资源绑定计划

`RenderPassExecutorRegistry`当前以`BTreeMap<RenderPassExecutorId, Arc<dyn RenderPassExecutor>>`保存executor。每个可执行pass仍用String ID做树查找；每个renderer owner构造时又逐项注册完整built-in表、推进generation并重置validation atomics。虽然compiled pipeline validation已有generation cache，但它只跳过验证全扫，没有消除每pass dispatch lookup与registry重复构造。

`product_postprocess_executor`还在每个effect pass从frame graph线性找node，随后深clone`required_inputs`和`produced_outputs`两组`Vec<String>`，再逐名字解析资源。内建scene/post executors普遍clone pass name或executor ID以绕开mutable context借用。SSR resolve/pyramid路径则对同一pass做多次name lookup，并通过`owned_texture_*_view`逐帧创建full-mip和per-mip views；view bundle继续归PERF-MVP-366，context metadata多owner继续归PERF-MVP-343，post graph feature mask继续归PERF-MVP-362。

Render01/Plugins01应在registry generation冻结时发布immutable executor slot table，并在pipeline compile/validation边界把String executor ID解析为dense `ExecutorSlot`；产品执行只做O(1) slot访问。插件register/revoke/hot reload通过新generation原子发布新表并使旧compiled binding失效，不能在pass循环回退String查找。后处理node的required/produced resources同样编译为declaration handles/ranges，执行器借用compiled metadata，诊断开启时才格式化名字。

Unreal RDG把execute lambda直接保存在`FRDGPass`并按`FRDGPassHandle`遍历执行；当前Bevy把render graph编译为`Schedule`并直接运行systems。两者均说明名字应停留在声明、诊断和热重载边界，而不是稳定帧dispatch键。

## 验收预算

按passes 16/64/256/1024、built-in/plugin executors 16/128/1k、renderer owners 1/8及stable/register/revoke/reload记录registry builds、BTree/String probes、executor-id clone/hash bytes、slot resolves、post node/resource-name clones、resource lookups、texture-view creates和CPU p95。要求同registry generation built-in table build不超过1，稳定pass String/BTree dispatch=0且dense resolve为O(1)，required/produced resource String clone=0，插件变更只使受影响generation重绑且旧slot不可执行；PERF-MVP-343/362/366的既有语义与预算同时满足。focused Cargo、F2像素、plugin reload、GPU timestamp与DX12 RenderDoc完成前保留在`pending.md`，不进入`review.md`。
