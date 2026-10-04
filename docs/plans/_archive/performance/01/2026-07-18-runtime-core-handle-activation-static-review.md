---
related_code:
  - zircon_runtime/src/core/runtime/handle/activation
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - twelve production Rust files reviewed
  - one source-level RED to GREEN bounded-poll guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, startup CPU counters and WPR trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core handle activation逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/handle/activation/**`共12/12个生产Rust文件，当前1,598行/1个inline源码守卫。范围覆盖single/batch activation、module lifecycle、startup service resolution、reactivation rollback、blocked unload与1至5 service专用依赖检测。

## PERF-MVP-321：module ready主线程忙轮询已止损

`wait_until_module_ready`在非零timeout内原先持续`yield_now`后立即再次调用`ready()`，异步模块长时间未ready时会占满一个CPU并把启动线程困在高频回调。已以RED→GREEN守卫改为最多1 ms且不超过剩余budget的sleep polling；初次ready、零timeout、睡眠后先检查ready再判deadline的边界语义保留。

最终契约不应永久依赖1 ms轮询。Runtime02需让async ready owner通过Condvar/event/future generation通知，activation等待可取消且不占用主线程；poll interval只作为兼容fallback并有调用计数。

## 生命周期重复投影与unload全图扫描

single activation的build/ready/finish分别重新锁module map、clone lifecycle Arc、分配module name并创建Weak context；batch还先clone所有完整`ModuleDescriptor`排序，再为每module复制service/startup lists并分五轮遍历。deactivate的blocked check对全部live services和dependencies扫描；超过5个unload services还临时建HashMap，小规模则由大量手写arity分支换取无map路径。

这些操作主要发生在启动/hot reload而非每帧，本轮不以代码形状贸然合并事务。Runtime02应建立冻结的module activation DAG/order、Arc module activation record与service reverse-dependency index；单次activation generation只投影一次lifecycle/context/service slices，blocked unload随受影响reverse edges而非全registry。

## 验收要求

对modules/services/dependencies各1/100/10k、ready delay 0/1/100/10,000 ms、single/batch/reactivate/unload记录ready calls、busy CPU、module/service locks、descriptor/name/list clone bytes、service/dependency visits与p95：等待CPU接近0且poll fallback≤ceil(delay/1ms)+1；最终通知路径poll=0，activation context owner=1，blocked unload与reverse affected edges成正比。rollback/error ordering、zero/overflow timeout、current-source Cargo与F0/F2 WPR通过前，12文件留在`pending.md`。
