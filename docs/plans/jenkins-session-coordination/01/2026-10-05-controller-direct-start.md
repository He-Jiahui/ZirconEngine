---
doc_type: runtime-evidence
related_code:
  - tools/jenkins/deployment/recovery.py
  - tools/jenkins/deployment/startup_observations.py
  - tools/jenkins/deployment/manager.py
  - tools/jenkins/deployment/host.py
plan_sources:
  - "user: 直接启动 Jenkins 服务，不要求 Windows 重启"
  - ../01-single-host-jenkins-coordination.md
tests:
  - tools/jenkins/tests/test_abandoned_startup.py
  - .jenkins/state/migration-baseline/patch-support-regression-1791198493052996000.json
created: 2026-10-05
---

# Jenkins 控制器直接启动记录

控制器已在 `http://127.0.0.1:18080/` 直接启动，Windows 没有重启。当前状态为 `controller-running-quiet`，`serviceReady=true`、`ready=false`。管理页面与认证 API 可用，79 个锁定插件的版本和实际启用状态复验通过；controller 为零执行槽，agent 与执行 broker 暂停。账号、密钥、任务记录及历史构建保留。

## 恢复依据

旧启动 `start-1791091874631246000` 使用已核验的特定封存驱动，记录证明尚未尝试启动 agent。专用恢复器再次核对驱动内容、进程包含机制、旧 host／controller 的精确出生身份已经离开、工程运行组件 owner 库存为空、Home 现有控制文件可独占读取及新端口可绑定后，允许仅控制器恢复。

这份恢复证据标记为 `same-boot-control-plane-recovery`；它没有原生终止回执、日志 EOF、构建验收或执行保护释放权限。其他缺少可信终止证明的实例继续保持原有保护。

## 实际服务身份与回执

| 项目 | 核验结果 |
| --- | --- |
| 当前 operation | `start-1791198640971844700` |
| 当前封存 driver | `7ec2af2f6dd0b96ef7dc99a155f7626cec902fd7a08aeca452288690ebfe427d` |
| host | PID `34304`，birth `134356722422105163` |
| controller | PID `24600`，birth `134356722462773539` |
| Home | `E:\Git\ZirconEngine\.jenkins\jenkins_home` |
| 专用 Java／JVM 临时目录 | 工程 `.jenkins\runtime\jdk`／`.jenkins\tmp` |
| Jenkins／Java | `2.580.1`／Microsoft `21.0.12.1` |
| 任务 | `zircon-flow`、`zircon-execution`、`zircon-maintenance` |
| 受保护预约／writer | 各 2 项，启动前后内容未改变 |

上述 PID 与 birth 是当时观察值；后续状态须按实时出生身份核验，不能单凭 PID 管理。启动回执为 `.jenkins/state/migration-baseline/prepared-driver-start-1791198633615555000.json`。实时服务回执由 `.jenkins/state/deployment/service-health.json` 引用并绑定 SHA-256；控制器在线状态及阶段门槛记录于 `.jenkins/state/migration-baseline/acceptance-index.json`。

## 验证与仍待完成的门槛

所选驱动的完整支持层 pytest 发现覆盖 297 项，零失败／错误，1 项因 Windows 无法创建目录 symlink 而跳过，源码／配置输入在测试前后哈希一致。回执 `.jenkins/state/migration-baseline/patch-support-regression-1791198493052996000.json` 仅适用于对应封存驱动；启动后的工作树修复须另附验证并另行部署。

旧执行 `0e14eb5450d57b2f48df030f42693e6f`、`ed8734ffd554e3db0dc12406dcf07fb4` 仍缺少可信终止证据。它们的预约、writer 和历史请求保留，没有改号重发、补造成功回执或发布未验收产物。完整 agent／托盘部署验收、M5 增量、M6–M7 并行与恢复、M8 唯一入口仍待运行核验。

当前所选驱动保留历史工程内 `buildRoot`，但编译没有启用。当前工作约定要求编译产物及缓存物理位于真实驱动器根级 `D/E/F:\cargo-targets`；再次启用编译前还须同步规格、检查器、登记所有权及路径准入证据。Jenkins 控制组件和运行数据继续留在工程 `.jenkins`。
