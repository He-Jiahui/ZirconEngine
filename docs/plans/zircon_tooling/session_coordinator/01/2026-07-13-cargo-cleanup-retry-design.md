# Cargo 清理失败重试设计

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | Cargo 清理失败节流 | `designed` | 2026-07-13 | 已确认当前即时清理入口会同时拾取 `pending` 与 `failed`；设计改为 `pending` 立即处理，`failed` 由守护进程每 30 秒重试。 |

## 目标行为

- `pending` Cargo 清理任务继续立即执行，不延迟正常磁盘回收。
- 清理失败后不再被后续即时清理请求反复触发，统一由现有守护循环每 30 秒重试。
- 不新增调度进程或数据库字段；服务重启后最多等待 30 秒再次尝试。
- 成功、失败事件继续写入现有事件日志，便于后续定位占用进程或权限问题。

## 实现边界

为 `retry_pending_jobs()` 增加是否包含失败任务的最小参数：即时后台清理仅选择 `pending`；现有守护观察循环保持默认 30 秒周期，并继续选择 `pending` 与 `failed`。测试验证即时入口不会重复失败任务，同时固定默认观察周期为 30 秒。

本次不增加持久化重试时间、指数退避、最大重试次数、独立任务服务或新的网页配置项；固定 30 秒常量即可满足本地开发期使用。

## 关联文件

- `tools/session_coordinator/cleanup.py`
- `tools/session_coordinator/config.py`
- `tools/session_coordinator/tests/test_cleanup.py`
- `docs/cli-and-tooling/local-session-coordinator.md`
