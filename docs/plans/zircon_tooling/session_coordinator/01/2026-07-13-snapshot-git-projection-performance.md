# Snapshot Git Projection Performance Repair

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | 生产快照 Git 投影性能收束 | `completed` | 2026-07-13 | 真实状态库分段剖析显示 `_git` 单段耗时 `5102.128ms`，50 条 `finalize_requests.index_snapshot` 合计 `186,820,884` bytes；SQL 边界改为显式浏览器字段后，7 次完整真实库投影加 JSON 的 P95/最大值为 `304.754ms`，低于设计 `800ms` 目标。 |

## 根因与修复

`ControlSnapshotService._git()` 原来通过 `SELECT *` 读取 finalize 记录，再在 Python 中删除 `index_snapshot`。该内部二进制列用于 Git 原子收尾失败时恢复 index，不属于网页投影；先读取再丢弃仍会让 SQLite、Python 和内存承担全部约 178 MiB 历史快照成本。

修复在 SQL 层显式列出控制台需要的 finalize 元数据，彻底不读取 `index_snapshot`，且保持现有 `paths`、`categories`、`untracked`、`validation`、状态、SHA 与时间字段契约不变。

## 验证

- RED：新增 SQLite authorizer 回归，禁止读取 `finalize_requests.index_snapshot`；旧查询以 `access ... is prohibited` 失败。
- GREEN：聚焦回归通过，随后 `tools.session_coordinator.tests.test_control_snapshot` 为 `5/5` 通过。
- 真实库：修复前完整投影约 `5234.561ms`，其中 Git 投影 `5102.128ms`；修复后 7 次完整投影与 JSON 样本为 `182.903–304.754ms`，载荷 `1,373,339` bytes。
- 该证据针对当前代码直接读取真实生产数据库；生产 HTTP 进程仍须在安全排空后滚动，届时再核验真实 `/control/v1/snapshot`。

## 关联文件

- `tools/session_coordinator/control_plane/snapshot.py`
- `tools/session_coordinator/tests/test_control_snapshot.py`
- `docs/cli-and-tooling/workflow-control-center.md`
