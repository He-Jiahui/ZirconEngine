---
related_code:
  - zircon_runtime/src/graphics/runtime_builtin_graphics
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
tests:
  - current runtime_builtin_graphics Rust source census 14 of 14 files reviewed, 401 lines
  - eight extension catalog to_vec calls and ten explicit clone calls accounted for
  - lazy RenderFramework factory traced into synchronous WGPU and renderer construction
  - current-source Cargo and F0/F2 startup traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics runtime_builtin_graphics静态审查（2026-07-18）

## 当前源覆盖

`zircon_runtime/src/graphics/runtime_builtin_graphics/**`当前14/14个Rust文件、401行已逐文件静态阅读。范围包括`GraphicsModule` extension owners、module descriptor/factory、render framework创建、driver/rendering-manager service及全部module wiring；当前源有8处`.to_vec()`和10处显式`.clone()`，逐处核对了调用阶段与owner。

## 启动瓶颈

`GraphicsModule::descriptor(&self)`为render features、geometry sources、shading models、pass executors、runtime collectors及三类runtime providers深clone八张Vec。`module_descriptor_with_render_features`随后把它们重新collect为八个`Arc<Vec<_>>`；Lazy `RenderFramework` factory首次执行又对八张表逐一`to_vec`，WGPU framework构造器再次collect，并额外clone render features传入renderer/default pipeline。插件越多，启动clone bytes与分配按多份完整catalog放大。

Lazy这里只延后成本，没有异步化：首次manager服务解析在调用线程创建`ProjectAssetManagerAccess`，同步进入WGPU adapter/device请求及SceneRenderer/pipeline/resource构造。冷driver、无adapter、device创建失败或重试都可能阻塞MVP主/UI线程。新增`PERF-MVP-409`要求单一generation-owned `GraphicsExtensionCatalog`和Initializing→Ready/Error ticket；catalog每generation只冻结一次，device init在受控render lane推进并以module readiness通知。

本地Bevy参考把adapter/device逻辑封装为async `initialize_renderer`并通过future resource承接，但desktop路径最终仍`block_on`；Zircon只借其初始化结果与消费阶段分离，不复制desktop阻塞。最终必须利用现有task/module readiness合同，同时保持native surface线程边界、init失败诊断、single-flight与确定重试。

`RenderingManager::backend_info`每次构造一个短backend-name String，但该管理查询不是帧热路径；`graphics_core_error`的字符串只在失败边界。它们不单独编号，也不以微小局部改动掩盖device-init主停顿。

## 验收状态

本切片没有修改生产代码：在缺少Cargo与启动trace时改module factory所有权会影响失败重试和服务生命周期。须按0/1/100/1k extensions、cold/warm adapter、first service/viewport、failure/retry/device-loss采集clone/alloc、init stage wall、caller blocked time、ticket age与线程证据；Cargo、F0/F2和RenderDoc对象核对完成前保留`pending.md`，不进入`review.md`。
