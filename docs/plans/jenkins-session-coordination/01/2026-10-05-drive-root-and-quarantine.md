---
related_code:
  - .jenkins/deployment-spec.json
  - .jenkins/workflow-spec.json
  - tools/jenkins/deployment/host.py
  - tools/jenkins/deployment/quarantine.py
  - tools/jenkins/deployment/manager.py
tests:
  - .jenkins/state/migration-baseline/patch-support-regression-1791207120105098800.json
doc_type: implementation-record
---

# 2026-10-05：直接启动复核与支持层候选驱动

正式 start 入口确认现有自有实例为 running。host PID 34304 与 controller PID 24600 的出生时间和可执行文件匹配；沿用操作 `start-1791198640971844700`。登录页及认证 API 可访问，79 个锁定插件健康。未重启 Windows，未停止或替换现有 controller。

Jenkins 运行数据与 Home 留在工程 `.jenkins`。工作树规格及检查器已同步为真实 `D/E/F:\cargo-targets`，默认 `D:\cargo-targets`。编译命名空间为 `<buildRoot>\zircon-local\zircon-jenkins`，适配当前写入授权；显式选根不自动换盘。历史工程内 builds 保留为证据，不接收新编译。

支持层修复涵盖实际表单与请求身份绑定、响应路径与动作授权、driver/operation/generation 的完整 broker 绑定、停止后的进程身份保存、部署历史证明归档、数据库损坏后的句柄关闭，以及封存对象和补偿目录同步。旧 launch 与身份不完整的记录不被新 broker 改写或启动；遗留预约和 writer 保护保持原版本。

显式 `zircon-strict` 子进程内完成全部核心支持层 pytest 发现：329 项收集，328 项通过，零失败或错误，1 项目录 symlink 测试因宿主限制跳过。测试前后源码与配置哈希一致。这是子进程验证记录，不证明当前桌面会话已启用权限配置，也不构成 Cargo 或运行里程碑接受。

回归回执：`.jenkins/state/migration-baseline/patch-support-regression-1791207120105098800.json`。日志 SHA-256：`be3cde9aa42062d3e3d2bee32a3a6c1ba358e26632d97b75e872c18e9d07e0a6`。

新候选驱动：`daf136cbe3043d9956d38b73fff9f5b8d77f1937e5cd5ec3a5e4dc4dec78a616`。准备记录：`.jenkins/state/migration-baseline/prepared-patch-driver-daf136cbe3043d9956d38b73fff9f5b8d77f1937e5cd5ec3a5e4dc4dec78a616.json`。准备器以 `--candidate-only` 生成候选，未更改 active selector、既有 prepared pointer 或历史保护，不启动新服务。

在线驱动仍为 `7ec2af2f6dd0b96ef7dc99a155f7626cec902fd7a08aeca452288690ebfe427d`，`serviceReady=true`、`ready=false`。controller 零执行槽并保持 quiet，agent 和执行 broker 暂停；新目录策略和修复尚未加载到在线实例。后续需显式建立新执行 generation、清点资源并保留历史隔离，再验证 agent、真实增量 Cargo、并行与联合输入、取消/恢复以及 M8 唯一入口。M2、M5–M8 门槛保持开放；没有产品仓库提交、推送或外部通知。

最新运行观察：`.jenkins/state/migration-baseline/service-start-confirmed-1791208672746848900.json`。验收索引：`.jenkins/state/migration-baseline/acceptance-index.json`。
