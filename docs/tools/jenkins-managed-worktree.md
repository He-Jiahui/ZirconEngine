---
doc_type: workflow-detail
created: 2026-10-15
related_code:
  - tools/jenkins/managed_worktree.py
  - tools/jenkins/managed_worktree_lifetime.py
  - tools/jenkins/product_inputs.py
  - tools/jenkins-management/runner.py
  - tools/dev/submit-jenkins-management.ps1
---

# Jenkins 固定工作树与增量验证

按用户 2026-10-17 最新要求，共享 main 必须保持纯净，任务从一开始就在 Jenkins 分配的 worktree 分支中实现。Jenkins 为任务创建并复用一个短路径 Git worktree，将选定时刻的完整输入登记、封存，再通过 zircon-flow 和 zircon-execution 构建。AI 只编辑所属任务工作树中获准的规范路径、提交补丁和输入 metadata；工作树创建、受管准备、合并、编译及释放均由 Jenkins 执行。下文观测共享 checkout 脏输入的字段保留用于历史请求对账，不能作为新任务继续修改 main 的许可。交付遵循[纯净主干与 PR 流程](jenkins-branch-integration.md)。

本文说明实现接口。是否正式可用，以当前 selected driver、management installed.json 的 managedWorktree.enabled、实际运行回执为准；静态文档和 Jenkins SUCCESS 不能替代源码、覆盖、产物及原生终结核验。当前任务证据索引为 E:/cargo-targets/zircon-local/reports/jenkins-one-worktree-flow-20261015/artifacts.json。

## 身份与目录

- 本机只使用 zircon-windows-agent；并行任务共享该节点的执行槽和资源预算。
- Jenkins runtime、Home、agent 与工作树在 E:/Git/ZirconEngine/.jenkins；MVP 的可复用工作树 ID 为 m15w-m，创建成功前该 ID 仅是计划。
- 请求明确指定 buildRoot=E:/cargo-targets；编译产物、构建池、编译缓存和构建 TEMP 位于 E:/cargo-targets/zircon-local/zircon-jenkins 的受管命名空间。不能把 target 放进工作树，也不按会话复制 target。
- 写入身份 W 只拥有任务的明确文件，具有 implementation/validation；读取身份 R 只具有 validation，绑定同一 principal、真实 W 授权快照及完整构建依赖闭包。由 Jenkins authorization actor 核验并签发，不直接改 State 或复用其他任务的权限。

## 一轮实际操作

1. W 调用 create-worktree，携带 name、不可变 baseHead、ownedPaths 和完整操作身份。保留返回的 worktreeId、路径及 operationKey。分支位于 codex/jenkins/<session>/<name>；重复观察原操作，不能为隐藏未知结果换 ID。
2. W 调用 inspect-worktree，传 worktreeId、targetSourceOperationKey 和真实 externalRoots。首次 targetSourceOperationKey 引用创建操作；以后引用上一次完整 preparation 或已完成补丁操作。Jenkins 检查目标原状并观测共享 checkout 的 HEAD、dirty/untracked/deletion、编译控制输入及所有外部 Gitlink。
3. 保存 Jenkins 发出的 preparationInputRef（path、sha256、bytes）、inventoryDigest、operationKey。清单位于实际 build namespace 的 management/reports/<operationKey>/source-inventory.json。清单通过有界哈希引用传递，不能展开塞入 Jenkins INPUT_JSON。
4. R 的 validation 范围必须精确覆盖该清单全部源码和 external/<name>/<path>，并绑定真实 W 快照。然后 R 调用 prepare-worktree，携带 worktreeId、inspectionOperationKey、原 preparationInputRef。Jenkins 再核对来源，封存输入并排他准备工作树；成功返回 worktreePreparationRef。
5. 有新增实现改动时，由 W 调用 apply-patch：传原始补丁引用、beforeHashes、expectedHead 和工作树身份。保留按顺序排列的已完成 patchOperationKeys；拒绝越权、外来后续编辑和不匹配的前像。首次验证已经观测的源码无需制造空补丁。
6. W 调用 prepare-flow，携带 worktreePreparationRef、patchOperationKeys、coverage 和独立 R 的 flowIdentity。返回的 currentSourceRef 与 flowPayload 绑定已准备工作树及封存外部输入。共享 checkout 后续变化不改写这些输入。
7. R 调用 submit-flow，顶层携带 R 的管理操作身份，并把返回的完整对象原样放入 flowPayload 字段。续查原 request/attempt、Flow、execution 与产物回执；核对实际命令、覆盖、封存源码、BuildSet、原生终结及归档字节。编译或测试失败保留失败身份和证据，不发布部分产物。
8. 所有该输入的请求、consumer、执行宿主与阶段进程均有真实终态、EOF 和验收/失败回执后，W 调用 retire-worktree-reader，传 worktreeId 与 currentSourceRef。该步骤核对证据并释放对应引用；取消单一 consumer 不会证明整个共享执行退出。
9. 下一轮复用同一路径重新 inspect/prepare；存在活动 reader、未知进程或未完成 writer 时拒绝覆盖。prepare 过程中中断会保留 writer fence 和证据，需显式恢复核对，不能自动清库或删锁。

提交、推送和通知分别要求原有明确授权。准备或编译通过不授予 commit/push；观察到的外来源码是构建输入，不转移其实现归属。工作树删除另走 release-worktree 的归属、独有改动交付及终结检查。

## 正式提交入口

使用当前已安装、哈希匹配的入口，并把上述动作的 JSON 文件放在获准控制目录。每一步都带 repositoryId、sessionId、requestId、operationId、driverDigest、runtimeOperationId、ownedPaths 和 buildRoot；具体动作增加对应字段。

```powershell
& 'E:/Git/ZirconEngine/tools/dev/submit-jenkins-management.ps1' `
  -Action inspect-worktree -SessionId '<W session>' `
  -RequestId '<request>' -OperationId '<operation>' `
  -InputFile 'E:/Git/ZirconEngine/.jenkins/state/controls/<request>.json'
```

-QueryOnly 用于核对同一操作，不代表已执行。以返回的规范 request/response 路径和 operationKey 继续核对；尚未获得真实结果时保持 pending。更换 driver/runtime 后先完成正式重绑定与旧执行对账，不能手改胶囊哈希。

详见[管理边界](agent-artifact-management.md)和[增量补丁工作流](../../.codex/skills/jenkins-coordination/references/incremental-patch-validation.md)。
