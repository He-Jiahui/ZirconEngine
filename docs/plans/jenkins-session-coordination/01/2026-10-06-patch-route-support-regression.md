---
related_code:
  - tools/jenkins/deployment/stop_binding.py
  - tools/jenkins/deployment/manager.py
  - tools/jenkins/deployment/host.py
  - tools/jenkins/patch_context.py
  - tools/jenkins/patches.py
  - tools/jenkins/workflow/handler.py
  - tools/jenkins/deployment/job_provisioning.py
  - .jenkins/pipeline/steps.groovy
  - .jenkins/pipeline/zircon-flow.groovy
plan_sources:
  - docs/plans/jenkins-session-coordination/01-single-host-jenkins-coordination.md
  - "user: 通过增量 patch 由 Jenkins 自动合并并编译验证"
tests:
  - "zircon-strict 完整支持层 pytest：488 collected，487 passed，1 skipped，0 failed/error"
  - "补丁 checkout 与 metadata 专项：38 passed"
  - "停机与 lifecycle 专项：29 passed"
  - "通知独立授权与 Pipeline 协议：28 passed"
doc_type: support-regression-record
---

# 2026-10-06 增量补丁接入回归

已封存经过完整回归的候选实现，尚未选为运行版本。源码在完整测试期间保持不变；静态与隔离功能测试不替代真实 Cargo、部署、M5–M8 验收。

停机调用方先核对精确的 host/controller/agent 身份并保存绑定，收到真实原生进程树、退出和 EOF 回执后才补齐历史 host 缺失的元数据。原始停止回执按实际字节存入不可变历史文件；绑定不合成 native proof，也不释放历史 execution holds。重复停止已终止代次只核对现有证据。新 host 不重放已有结果的历史命令。

新增部署所有的原始验收 checkout 注册接口，仅允许工程 .jenkins/tmp/acceptance 下直接子目录，绑定请求、授权路径、运行代次和 sealed driver。patch 注册通过绑定引用选择源 checkout；准备、补偿和加锁使用源 checkout，工具链政策及调度身份始终从工程部署上下文取得。该接口只用于原始验收夹具，未形成产品仓库接受结果。

补丁封存后由支持库收集 Cargo metadata；工具路径和哈希来自已核验环境政策。复用证据必须匹配完整政策、源码、repository、buildRoot 和该源码的封存运行目录。metadata 查询不等同编译验收，测试默认仍须实际运行。相同封存输入可绑定新的合法消费者。

Pipeline 响应递归转换为可序列化 JSON 类型并仅保留固定字段，补丁 identity/ref 在恢复后仍可使用。每次 control 调用注入选定运行代次和 driver digest，拒绝外来代次，不改写嵌套工作流身份。

通知需 ALLOW_NOTIFY、显式目标和独立 notify 授权；纯注释模板不通知。先完成持久化 finalization，再尝试投递。已交付和交付不明的 outbox 不重新发送，投递失败独立重试或对账；提交意图不推导通知授权。

完整回归：.jenkins/state/migration-baseline/patch-support-regression-1791265320988217900.json。回执 SHA-256：27dd423cc7f2a4e3c815f278963b2c8a3e7185ec6483164975edd3e36c0e4575。
候选 driver：16f826112f52f7d5c84353baaa3d5b89583fe8e12113a527e872029d85556286。
准备回执：.jenkins/state/migration-baseline/prepared-patch-driver-16f826112f52f7d5c84353baaa3d5b89583fe8e12113a527e872029d85556286.json。
候选封存未改变 selector、prepared pointer 或服务进程。

当前托盘 PID 11660 仍存活且创建身份匹配，两个右键回调已实际弹出九项菜单。原 controller PID 5300 与 host PID 12148 已退出，当前服务不可用。旧代次仍属于当前 Windows boot，缺失完整 native EOF 回执，不能用 PID 不存在来伪造终止接受结果。旧 selector 保持 7ec2af2f6dd0b96ef7dc99a155f7626cec902fd7a08aeca452288690ebfe427d。

protected holds digest 仍为 dddd82c8cf7573e54ab1f4be4ec6ac7ff5e506cea14a80bea36ca50dd04ac583。当前状态记录：tested-patch-support-current-1791265645736407000.json。后续先审计旧不可变实现的控制器 Job containment 与受限管理服务恢复路径；在取得新代次的真实终止证据后，由部署执行器激活候选，核对 agent、插件与物理路径，再执行原始夹具的增量 patch/Cargo 流程。
