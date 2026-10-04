---
title: Runtime Foundation Current Source Protected Plan Routing
date: 2026-08-23
status: routing_only
related_report:
  - docs/plans/performance/01/2026-08-23-runtime-foundation-current-source-performance-adoption.md
protected_targets:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Runtime Foundation Current Source Protected Plan Routing

本记录只提供受保护台账和其他计划 owner 的归并输入；本轮没有修改受保护文件。

## `review.md` 建议

暂不写入。`zircon_runtime/src/foundation` 已完成 14/14 当前源码复读，Optimize118 也完成全产品静态审查，但 3 个 P0、39 个 failed gate 和 current-source 动态证据缺口仍在。

## `pending.md` 建议

`zircon_runtime/src/foundation`：静态审查完成；等待 M118.0 四个产品 RED、BootConfig/typed config authority/scoped persistence/event consumer 硬切，以及 managed Windows multi-runtime/fault/scale/WPR 证据。

## 唯一 owner 路由

| 问题 | 目标计划 |
|---|---|
| Foundation 纵向组合、boot precedence、durable projection、多 Runtime path owner、真实 Config/Event consumer | Optimize Runtime118 (`99s...current-source-review.md`) |
| execution domain、worker shutdown、EventBus 内部调度 | Runtime01/02 |
| typed config registry/layer/schema 与 Core direct-write hard cut | Runtime03 |
| path identity、atomic recovery、cross-process lock/CAS | Runtime24/25 |
| module descriptor/capability truth/lifecycle | Runtime42/46 |
| preference scope/backend/Editor settings | Runtime45 |
| manager handle/generation/lifecycle | Runtime50 |
| dynamic session/product host 构建与关闭 | Runtime43/App01 |
| 旧同步整文件写 failure 的 source-bound closeout | `docs/plans/zircon_runtime/runtime/02/failure-2026-07-18-config-manager-synchronous-full-file-rewrite.md` |

不得新增 `ConfigFacade`、第二个 persistence worker 或第二套 event bus 作为解决方案；不得以后台线程存在、空 driver 已注册、单元 roundtrip 通过或一次 flush 成功宣称 Foundation Ready。

## 晋级门

1. 四个产品 RED 先失败后通过，且证明旧 Core raw path 已删除或不可达。
2. durable write work 与 durable projection 成比例，不再与 Core 全部 key/value 总量绑定；规模数据包含 copied/serialized/written bytes 与 RSS。
3. Editor/PIE/tool/dynamic session 的 address/owner/shared-or-isolated 语义可证明，无 silent supersede。
4. 至少一个真实产品 consumer 使用 typed Event service；publication、drop/gap、generation 和 shutdown receipt 可观察。
5. 受管 Windows current-source Cargo、fault/soak、WPR/ETW 全部绑定同一 artifact/build identity；完成前继续 pending。

