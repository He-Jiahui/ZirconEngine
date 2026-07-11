# M1 Control-Plane Foundation Output Records

- Owner plan: `../01-workflow-control-center-and-tray.md`
- Session: `workflow-control-center-20260711-1915`
- Testing state: implementation slices use lightweight syntax evidence; full unit/integration evidence belongs to M1-T.

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | M1.1 enums and schema v14 | `implementation_complete` | 2026-07-11 | `tools/session_coordinator/models.py`; `migrations.py`; `tests/test_workflow_schema.py`; `python -m compileall -q tools/session_coordinator` exit 0；完整测试待 M1-T |
| M1 | M1.2 workflow persistence and projections | `implementation_complete` | 2026-07-11 | `tools/session_coordinator/workflows/{models,store,projections}.py`; `tests/test_workflow_store.py`; `tests/test_workflow_projections.py`; latest accepted attempt contract written；完整测试待 M1-T |
| M1 | M1.3 snapshot and event replay | `implementation_complete` | 2026-07-11 | `control_plane/{contracts,snapshot,events}.py`; `tests/test_control_snapshot.py`; `tests/test_control_events.py`; single SQLite snapshot and bounded cursor contracts written；完整测试待 M1-T |
| M1 | M1.4 Observer bootstrap and loopback security | `implementation_complete` | 2026-07-11 | `control_plane/{auth,http_security}.py`; `tests/test_control_auth.py`; `tests/test_control_security.py`; opaque one-use ticket and HttpOnly cookie contracts written；完整测试待 M1-T |
| M1 | M1.5 modular HTTP routing and CLI composition | `implementation_complete` | 2026-07-11 | `control_plane/{router,http}.py`; `server.py`; `client.py`; `cli.py`; `tests/test_control_http.py`; legacy endpoints retained and control routes delegated；完整测试待 M1-T |
| M1 | M1.6 operator and module documentation | `implementation_complete` | 2026-07-11 | `docs/cli-and-tooling/{local-session-coordinator,workflow-control-center}.md`; `docs/tools/session_coordinator/{control-plane,workflows}.md`; machine-readable related-code headers and M1 trust/recovery boundaries written；完整测试待 M1-T |
| M1 | M1-T control-plane acceptance | `accepted` | 2026-07-11 | 计划指定套件 46 项通过；完整协调器套件拆分为 58 + 36 + 60 = 154 项全部通过（单次 discover 604.6 秒超时后按模块拆分取得可判定证据）；`python -m compileall -q tools/session_coordinator` 通过；`git diff --check` 通过；独立复审 7 项 Important 均完成架构修复，最终无 Critical/Important |
