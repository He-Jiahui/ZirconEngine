---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
tests:
  - current WGPU render-framework construction Rust source census 3 of 3 files reviewed, 629 lines
  - provider winner clone-count regression test added
  - provider selection source contract moves winner and borrows provider ids
  - scoped rustfmt and diff check passed
  - current-source Cargo and F0/F2 startup traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics render-framework construction静态审查（2026-07-18）

## 当前源覆盖

`zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/**`当前3/3个Rust文件、629行已逐文件静态阅读：`construct.rs`606行、`create_default_pipelines.rs`21行、module wiring 2行。四个provider选择合同测试均已阅读，其中一个为本轮新增clone-count回归；未执行Cargo，不能声明动态通过。

## 直接止损

通用`select_provider`接收owned `Vec<T>`，原先仍要求`T: Clone`并返回`providers[best_index].clone()`；duplicate检查还把每个borrowed provider id转成新String存入HashSet。本轮先增加clone-count RED测试，再删除Clone bound，以`HashSet<&str>`完成生命周期内去重，验证priority/tie后用`providers.into_iter().nth(best_index)`移动winner。源码合同确认winner clone=0、per-id `to_string`=0，duplicate id、highest priority与tie错误文本逻辑不变。

## 剩余瓶颈

最终构造器仍把八类extension/provider输入逐一collect为Vec，render features又clone一份给SceneRenderer并为forward/deferred默认pipeline分别clone descriptor；随后同步构造WGPU backend、renderer、capability/debugger、默认pipelines和framework state。它与上游module descriptor/factory的多轮clone共同归`PERF-MVP-409`的immutable extension catalog和异步single-flight device-init ticket，不在当前函数叠加第二套cache。

无显式compute pool的公共构造入口还会创建新的compute `TaskPool`；module-host产品路径已传入`core.task_pools().compute().clone()`，因此暂按非产品/测试构造成本记录，后续调用图若证明产品可达再单独升级。provider selection当前O(N)去重+O(N)best scan是合理算法，不改为排序。

## 验收状态

局部文件通过rustfmt、源码合同与`git diff --check`。Cargo协调器JSON解析阻塞仍在，故provider duplicate/tie/highest/clone-count测试没有current-source执行结果；adapter/device cold/warm、extension 0/1/100/1k、caller blocked time和F0/F2启动trace也未完成。继续保留`pending.md`，不进入`review.md`。
