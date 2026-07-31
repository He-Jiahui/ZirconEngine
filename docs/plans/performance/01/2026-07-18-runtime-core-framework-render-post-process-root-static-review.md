---
related_code:
  - zircon_runtime/src/core/framework/render/post_process/chain.rs
  - zircon_runtime/src/core/framework/render/post_process/color_lut_readback.rs
  - zircon_runtime/src/core/framework/render/post_process/color_space.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/exposure_readback.rs
  - zircon_runtime/src/core/framework/render/post_process/exposure_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/graph_resource_names.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/resolved_stack.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - remaining post-process root twelve of twelve Rust files reviewed
  - complete post-process directory thirty-five of thirty-five Rust files reviewed
  - production and cfg-test readback caller boundaries traced
  - scoped git diff check passed
  - focused Cargo scale counters F2 trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime post-process剩余root逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/post_process`剩余root 12/12个Rust文件、1,526行；结合已完成的effect settings、stack和Volume切片，目录当前35/35个Rust文件、7,135行均已静态读完。剩余root未发现需要另建编号的独立生产热点：链/格式/曝光/资源名主要是Copy或static契约；graph构建中的String/Vec/BTree clone、拓扑排序和多owner问题已由PERF-MVP-362覆盖；Volume求值与extract问题由PERF-MVP-363/364覆盖。

## 生产路径核对

`PostProcessPassGraph::validate_stack`把effect settings投影为owned pass nodes，clone required/produced/dependency Strings与Vec，再创建多个`BTreeMap/BTreeSet`完成拓扑和资源验证，最后再次clone ordered nodes和output-transfer name。它当前由每帧stack rebuild触发，属于PERF-MVP-362的同一最低根因；应随generation-compiled dense-ID artifact整体删除稳定帧构建，而不是局部替换一个集合后继续保留重复权威。

`PostProcessGraphResourceNames`、`PostProcessChainSlot`、`PostProcessEffectKind`、color-space和exposure settings均为static label或Copy数值路径，没有循环内格式化、锁、I/O或线程调度热点。`RenderResolvedPostProcessSettings`也是小型Copy结果，真正重复成本位于调用次数与override参数临时Vec，已记PERF-MVP-363/364。

## readback边界

color LUT与exposure report的字节遍历只从`#[cfg(test)]`的产品产物测试readback路径调用；32³/64³ LUT会做同步GPU readback并在线性CPU扫描中同时核对reference/identity，但不进入非test产品构建，不能作为运行时帧热点，也不能把测试计时冒充产品性能。正式RenderDoc/GPU timestamp验证仍应在capture/diagnostic显式开启时采样，默认产品帧不得新增同步readback。

## 验收要求

目录仍留在`pending.md`，直到PERF-MVP-361..364的generation artifact、共享resolved settings和dense report落地，并完成current-source post-process/volume/render-product Cargo批次、effects/volumes/cameras规模counter、F2 CPU/GPU trace及RenderDoc pass/resource对拍。验收同时断言非test默认帧readback copy/map/wait=0，capture/test开启时记录LUT字节数、map wait和CPU scan；顺序、色彩空间、exposure、Volume、插件和像素结果不得漂移。
