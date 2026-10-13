---
doc_type: implementation-record
created: 2026-10-12
related_code:
  - tools/jenkins-management/entry.py
  - tools/jenkins-management/runner.py
  - tools/jenkins-management/source_preparation.py
  - tools/jenkins-management/zircon-management.groovy
tests:
  - "zircon-management #26: actual Cargo and 24 zr_math cases"
  - "zircon-management-tests #8: 42 reported checks, 17 test methods"
  - "zircon-management-tests #10: 49 reported checks, 22 test methods"
  - "zircon-management-tests #11: 60 reported checks, 27 test methods"
  - "zircon-management-tests #12: 60 reported checks, 27 test methods; large evidence and coverage seal regression"
  - "zircon-management #29: SUCCESS; three actual Cargo steps, 24 tests, Native proofs and read-only consumer passed"
  - "zircon-management #27 and #28: failed before Native/Cargo launch"
---

# 普通管理与封存输入复用

部署地址为 [本机 Jenkins](http://127.0.0.1:18080/)，配置为控制器零执行槽、唯一节点 `zircon-windows-agent` 两个执行槽。2026-10-13 01:49 UTC 页面与节点仍健康；02:08 UTC 核对时旧宿主、keeper、controller 和 agent 已退出，端口不再监听，托盘仍存活。正式宿主的丢失恢复正在准备，退出原因尚未证实；原构建、源码和历史回执保留。此前管理组件热更新没有重启这些服务。

运行身份为 `start-1791789846374705900`，driver 为 `121af96cb9d4e5dce3f3e830373203952324f380aa4b0ed9493f16ee783d4ecf`。Jenkins 运行数据在工程 `.jenkins` 内；Cargo 产物、构建池与编译缓存使用请求选定的真实 `D/E/F:\cargo-targets`，默认 D，明确选根后不换盘。

## 已核对的执行

`zircon-management #22` 合并了本会话两个已完成补丁的工作树，保留联合候选、原补丁和原输入身份，没有产品提交或推送。`#26` 在 F 根运行该候选的真实 `zr_math` 验证，24 项单测通过，Jenkins SUCCESS。最终 source receipt、原生 Job 全树退出、日志 EOF、execution terminal 与资源释放均已核对。

该次执行使用完整封存输入。原 engine、依赖缓存配置及执行输入保持匹配；可变外部 checkout 在封存后发生变化，回执明确保留 `inputsUnchanged=false`。这次通过证明的是该次封存输入及其覆盖，不能替代当前外部源码、完整产品或 M0–M8 验收。

`zircon-management-tests #8` 为 SUCCESS。JUnit 报告 42 项检查、17 个测试方法元素，零失败、错误和跳过；11 项相关源码及 driver manifest 哈希前后相同。真实测试进程已退出，Job 退出后活动进程为零，stdout/stderr EOF 完整。测试临时树实际删除 4 个文件、3 个目录，共 2,322 个逻辑字节；最小证明和日志保留。

该步验证的 capsule `95f209334d19ae3a6dea3db9a17ce3b3531358060cc04b12723ba11175f5f39f` 保留为历史版本。该轮多命令版本为 `5a2382f2792d733281c110d8ba20937c2f883b638b82c0fbf4cb0106775cd175`；`zircon-management-tests #10` 实测 SUCCESS，22 个测试方法和 27 项子检查通过，JUnit 共 49 项，源码哈希与 Native/双 EOF 证据完整。#9 的旧夹具失败及修复证据保留；#9、#10 的自有临时文件分别删除 2,988 与 2,879 个逻辑字节。

当前安装 capsule 为 `7936cd765025fd4f56347f418238f1a06eca50c8bb59500e4fcaa23a91027123`。旧版本 `dd38d5907b1ec513d4c5bcb2a071fd812f4ac6a7c12de729f446fd24b8d67443` 及 `zircon-management-tests #11` 的 60 项通过证据保留；该轮临时树删除 3,405 个逻辑字节。真实批次随后暴露了大请求读取上限，复核还发现旧夹具未将覆盖纳入 seal 身份。两处修复已由 `zircon-management-tests #12` 实测 SUCCESS：27 个测试方法、33 项子检查，JUnit 共 60 项，零失败、错误和跳过。11 项源码与 driver manifest 哈希前后匹配，Native 全树终止及双 EOF 完整，临时树实际删除 3,405 个逻辑字节。大型源码证据使用目的明确的 64 MiB 上限，普通回执保留 8 MiB 上限；消费器分别核对 donor 和新覆盖的 seal 身份。

## 提交与复用

普通入口为 `tools/dev/submit-jenkins-management.ps1`，执行 job 为 `zircon-management`。使用当前 State 中已有授权的 session、原请求和原 operation，输入文件携带明确源码、路径和构建根。示例中的变量须填入该任务已有身份：

```powershell
& 'E:\Git\ZirconEngine\tools\dev\submit-jenkins-management.ps1' `
  -Action validate -SessionId $sessionId -RequestId $requestId `
  -OperationId $operationId -InputFile $inputFile
```

对账使用相同参数加 `-QueryOnly`。投递结果不明时先核对原队列和回执；API 登录不能替代源码动作授权。分支、工作树、补丁、合并准备、受控文件生成及回收同样走该管理 job，并核对相应动作授权。

可选 `batch` 输入绑定原始合同文件及 SHA-256、完整有序 recipes 和每步超时。一个父任务捕获、封存和物化一次，连续执行全部命令，逐步保存实际进程、日志与回执；同配置使用既有稳定池。缺少任一步终止证明时保持待对账。原始大型输入通过控制文件交给子进程。

`zircon-management #27` 提交了真实三步 `zr_math` 请求：检查、仅编译测试、执行原 24 项测试。任务固定使用 F 根，在封存外部 `zr_vm` 的 `tests/tooling/test_layout_moves.py` 时发现捕获后字节变化，以 `before_hash_mismatch` 失败。Cargo 和 Native 均未启动，预约经 `durable-unlaunched` 证明释放；原失败请求、部分封存对象、热池和诊断保留。自有空临时目录保留为待受管清理，实际释放 0 B。随后 `#28` 显式引用 `#26` 的完整封存输入，计划重新运行三条命令，但读取旧 request.json 时遇到通用 8 MiB 上限；实际文件为 8,667,726 B，登记 SHA 与文件一致。`#28` 以 `validation_preparation_failed` 终态失败，Native/Cargo 未启动，预约经 `durable-unlaunched` 释放；原请求与终态保持不变。大型证据读取及旧、新覆盖 seal 消费修复已通过 `#12` 回归。新的 `#29` 使用独立请求 `mvp-shared-batch-bounded-f-20261012`，固定 F 根并复用 `#26` 的封存输入，现为 Jenkins SUCCESS。实际完成 `cargo check`、`cargo test --no-run` 和 `cargo test`，24 项测试通过。原安装 capsule 的只读消费器已核对三份原生进程身份与全树退出、双日志 EOF、实际命令、逐步回执、完整 capture 及新旧覆盖 seal；execution 为 passed，预约已释放。原 engine、依赖缓存与准备输入保持匹配，封存后的外部变化仍明确记录为 `inputsUnchanged=false`。任务缓存评估完成，本次实际回收 0 B；自有空 TMP 目录仍登记待受管清理，热池与历史证据保留。原失败任务保持原终态。

相同完整 capture 可以复用已验证的封存字节。复用前检查 manifest、原 producer/consumer、物理根及全部 CAS；覆盖变化时建立新覆盖与 consumer，保留旧记录。源码改变走新的封存准备；合法不同构建根使用所选根。损坏的匹配对象拒绝复用。每次验证仍实际运行测试。

## 清理与待处理输入

`zircon-scheduled-cleanup` 每半小时盘点 D/E/F 并执行有界受管 GC。`#42` 为 SUCCESS，本次符合条件的回收对象为空，实际回收 0 B。可复用热池、活跃与历史引用，以及归属不明对象继续保留；不能据此声称 D/E/F 积压全部清理完成。

多命令入口和 Git 消费批次回执的适配已有 `#12` Jenkins 回归证据，真实三步批次 `#29` 及原消费核对也已通过。两份本会话输入工作树的回收请求已通过公共管理入口分别进入 Jenkins 队列 977、979；对应原构建为 `#30`、`#31`；服务中断前分别在执行和等待 Git 资源锁。两次构建均尚未形成终态删除回执，回收继续等待宿主恢复后的原操作对账和物理结果核对，目前不计入已释放空间。联合候选、原分支、补丁和历史证据保留。原 Runtime/editor 的 15 条命令及 16 个 producer 合同完整归档；29 条旧源码 guard 中有 5 条内容变化，9 个旧路径均找到提交 `21da363625e644aacca45b8c90621547cd42ba5b` 的改名记录。迁移后的 Rust 模块别名与大部分必需函数仍存在；profiling-export 的一个必需函数缺失，另两项筛选覆盖仍需核对。原 intent 尚待有效更新交接，未提交旧批次。Engine/App 原请求也保持等待有效源码交接。归档和授权签发分别记录其实际状态。

注释审查补充 `source-comment-audit-20261012-reflection-review-001-ready.json` 已由 `zircon-records-followup #4` 归档并清除原 inbox 文件。保留副本 SHA 与原 13,082 B 文件一致；该次没有创建新 Native 验证，原任务继续等待，归档不构成源码验收。

本记录不关闭 M8、整产品或性能门槛。原始机器证据登记在 `E:\cargo-targets\zircon-local\reports\jenkins-parallel-activation-20261011\artifacts.json`。
