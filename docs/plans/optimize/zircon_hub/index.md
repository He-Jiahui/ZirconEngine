# `zircon_hub` 差距审查

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Hub必须按运维工作流和失败闭环验收，不能按UI页面数量验收。Project/Engine/Build/Editor launch/persistence/delivery后端，web shell/catalog/settings/Team/Cloud/accessibility，Marketplace/Account Auth/Organization/Cloud Repository provider，Command/Action/Message Delivery/Task/History/ViewModel/Localization内部控制面，以及Application Host/Bootstrap/Window/IPC/Close/Shutdown/Crash Recovery已完成首轮E3静态审查；真实Update/release repository/install/rollback控制面由`zircon_tooling/09`拥有，其余跨平台bundle细节随发布实施复核。

| 编号 | 主题 | 状态 | 报告 |
|---|---|---|---|
| 01 | Project、Engine、Build、Editor Launch、Process、Persistence、Delivery | review complete / implementation pending | [01-project-engine-build-editor-launch-process-persistence-delivery-review.md](01-project-engine-build-editor-launch-process-persistence-delivery-review.md) |
| 02 | Web Shell、Catalog、Settings、Team/Cloud、Accessibility、Performance | review complete / implementation pending | [02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md](02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md) |
| 03 | Marketplace、Account Auth、Organization/RBAC、Cloud Repository Provider | review complete / implementation pending | [03-marketplace-account-auth-organization-cloud-repository-provider-review.md](03-marketplace-account-auth-organization-cloud-repository-provider-review.md) |
| 04 | Command、Action、Message Delivery、Task、History、ViewModel、Localization | review complete / implementation pending | [04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md](04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md) |
| 05 | Application Host、Bootstrap、Window、IPC、Close、Shutdown、Crash Recovery | review complete / implementation pending | [05-application-host-bootstrap-window-ipc-close-shutdown-crash-recovery-review.md](05-application-host-bootstrap-window-ipc-close-shutdown-crash-recovery-review.md) |

五轮报告累计记录14个P0、254个P1和55个P2。除后端编译、删除、recent复活和进程owner问题外，production state load会把任意后端/协议故障吞成“Ready”演示状态；catalog在全局锁内同步递归扫描，Settings逐按键交换完整snapshot，Team把Git作者和Git identity误投影为成员/账户，undecorated窗口没有drag region，前端也没有行为测试。远程服务当前保持disabled是正确基线；启用前必须建立secure account credential、organization RBAC/audit、signed Marketplace/entitlement、共享Package Service以及revision/CAS Cloud snapshot/sync，且先封闭native plugin pre-admission code execution。

Hub04进一步确认WebView可把任意绝对输出目录交给系统shell，排队动作不冻结target且执行时会改写全局选择，进程内无界队列与单一TaskStatus没有取消、恢复和终态证明，外部effect与history/config不是可恢复commit，history还会持久化并投影未分级命令和日志。39个integration文件中38个不链接Hub业务crate，270个测试只有9个直接执行production business type；后续必须以versioned command envelope、immutable target lease、durable TaskRegistry、effect ledger、redacted history和typed MessageCatalog/read model替换当前字符串与快照拼装链。

Hub05进一步确认native host没有统一close/exit协议：WebView直接销毁窗口，Rust侧不处理`CloseRequested`/`ExitRequested`，后台线程与Editor `Child`没有可join/cancel的宿主owner；声明过的`HubWindowState`也没有production读写链。前端initial snapshot与event listener分离，事件无generation/sequence，连接失败无重试，后端或schema错误还能被降级成可操作的fallback shell。后续必须建立分阶段bootstrap、instance admission、window session handshake、stop-admission、close decision、worker/process quiesce、terminal checkpoint与crash/restart recovery；现有source-shape测试不能作为真实Tauri窗口生命周期资格。
