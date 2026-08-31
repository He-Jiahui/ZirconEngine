# `zircon_hub` 差距审查

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Hub必须按运维工作流和失败闭环验收，不能按UI页面数量验收。前五轮分别覆盖Project/Engine/Delivery后端、Web与catalog、remote provider、内部控制面和Application Host；第六轮对当时全部272个tracked Hub文件做全量E3静态刷新；第七轮按当前工作树深挖Engine/BuildSet resolver、launch attempt、Child supervision、Ready/Focus与recent一致性。真实Update/release repository/install/rollback控制面仍由`zircon_tooling/09`拥有，Hub只拥有产品编排、安装发现、项目工作流、进程与宿主生命周期。

| 编号 | 主题 | 状态 | 报告 |
|---|---|---|---|
| 01 | Project、Engine、Build、Editor Launch、Process、Persistence、Delivery | review complete / implementation pending | [01-project-engine-build-editor-launch-process-persistence-delivery-review.md](01-project-engine-build-editor-launch-process-persistence-delivery-review.md) |
| 02 | Web Shell、Catalog、Settings、Team/Cloud、Accessibility、Performance | review complete / implementation pending | [02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md](02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md) |
| 03 | Marketplace、Account Auth、Organization/RBAC、Cloud Repository Provider | review complete / implementation pending | [03-marketplace-account-auth-organization-cloud-repository-provider-review.md](03-marketplace-account-auth-organization-cloud-repository-provider-review.md) |
| 04 | Command、Action、Message Delivery、Task、History、ViewModel、Localization | review complete / implementation pending | [04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md](04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md) |
| 05 | Application Host、Bootstrap、Window、IPC、Close、Shutdown、Crash Recovery | review complete / implementation pending | [05-application-host-bootstrap-window-ipc-close-shutdown-crash-recovery-review.md](05-application-host-bootstrap-window-ipc-close-shutdown-crash-recovery-review.md) |
| 06 | Product Control Plane、Project Lifecycle、Process、Delivery、Web、Host、Test Evidence当前源码全量刷新 | review complete / implementation pending | [06-current-source-product-control-plane-project-lifecycle-process-delivery-web-host-test-evidence-review.md](06-current-source-product-control-plane-project-lifecycle-process-delivery-web-host-test-evidence-review.md) |
| 07 | Engine、BuildSet、Launch Attempt、Child Supervision、Ready、Focus、Recent当前工作树刷新 | review complete / implementation pending | [07-engine-buildset-launch-attempt-child-supervision-ready-focus-recent-current-working-tree-review.md](07-engine-buildset-launch-attempt-child-supervision-ready-focus-recent-current-working-tree-review.md) |

七轮唯一账本仍是14个P0、254个P1和55个P2，共323项：**317 Open、5 Partial、1 Closed**。唯一Closed仍是Hub01 `ZHUB-P0-01` 的错误参数调用已改为 `persist_unchecked()`；5个Partial是Hub07按当前行为证据重判的`ZHUB-P1-06`、`ZHUB-P1-09`、`ZHUB-P1-16`、`ZHUB-P1-17`和`ZHUB-P2-08`。其余13项P0全部Open；Hub05三个B0只是继承alias，不重复计数。Hub07没有运行Cargo，所以这些状态只确认当前源码语义，不宣称构建或动态资格通过。

当前production仍由单一`Arc<Mutex<HubRuntimeSession>>`承载config、draft、navigation、catalog、queue和task；Settings folder modal、扫描、Git与persist可在锁内阻塞。command无version/principal/capability/immutable target/budget，无界串行worker、单一TaskStatus和detached Child没有durable终态。Package/Install仍是未签名递归copy，没有cook/build-set/dependency closure/atomic activation/rollback。Web state失败仍可退成可操作fallback，而action继续连接live backend；WebView也直接关闭窗口，宿主没有close/shutdown/quiesce协议。

测试接线继续有局部进展：当前Hub Rust owner source有284项inline test attribute，40个integration文件有271项test attribute；queue、focus、handshake、recent已出现真实行为测试，因此`ZHUB-P2-08`只重判为Partial。仍缺Hub+Editor真实双进程、Child kill/reap、PID reuse、mailbox fault、offline recent tombstone、ACL与scale资格。后续先按Hub07的M0-M7建立`BuildSetResolver + OperationService + EditorProcessSupervisor + request-bound LaunchAttempt + durable RecentProjectService + versioned HubReadModel`，再回到Hub06完整控制面闭环。
