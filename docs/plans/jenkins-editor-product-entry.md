# Jenkins 编辑器产物入口

`tools.dev.jenkins_editor` 将主工作区中已经应用的文件哈希交给 Jenkins。入口只写请求元数据；源码封存、物化、编译和产物发布由协调器执行。它不应用补丁，不创建代理持有的源码快照，也不取得共享源码的排他所有权。

## 请求输入

同一请求固定仓库、会话、请求 ID、尝试 ID、执行版本和物理构建根目录。准备以下 JSON 元数据：

- `current-hashes.json`：完整且明确选择的构建输入，键为仓库相对路径，值为当前文件 SHA-256。只有已经删除的受 Git 跟踪文件可以使用 `null`。
- `owned-paths.json`：本任务修改的路径清单，必须是上述输入的非空子集。该字段记录任务范围，不声明其他依赖文件的所有权。
- `untracked-paths.json`：输入中尚未被 Git 跟踪的文件清单。协调器核对 Git 实际状态，不自动纳入其他未跟踪文件。
- `external-inputs.json`：可选的 `zr_vm` 依赖元数据数组。每项只能包含 `name`、`repositoryRoot`、`baseHead`、`currentHashes` 和 `untrackedPaths`。仅允许主仓库的物理同级 `zr_vm` 目录；没有外部所有权或写入字段。

输入必须包含 Cargo 工作区及本地依赖、锁文件、相应 Rust 源码和测试、编辑器资源、字体描述及字体文件、BuildSet 规范和打包工具的导入闭包。Shipping 还需要产品构建配置。遗漏的必需输入会被拒绝，不从主工作区补读未封存内容。

下面展示命令参数，尖括号内容需要替换为本次真实值。哈希及路径清单可以保存在本任务的 `.codex/outbox` 目录中；不要为生成清单复制源码。

```powershell
& .jenkins/runtime/python/python.exe -B -m tools.dev.jenkins_editor dry-check `
  --session-id <session-id> --request-id <request-id> `
  --base-head <immutable-git-head> --build-root D:\cargo-targets `
  --current-hashes <absolute-current-hashes-json> `
  --owned-paths <absolute-owned-paths-json> `
  --untracked-paths <absolute-untracked-paths-json> `
  --external-inputs <absolute-external-inputs-json> `
  --profile development --required-test <exact-cargo-test-id>
```

`dry-check` 校验参数，不提交请求，也不形成构建或验收证据。将动作改为 `submit` 后，入口先注册不可变的 `current-source` 意图，再交给 `zircon-flow` 的 `prepare-current-source` 操作。协调器检查当前字节、Git 基线和授权范围后，签发 `sealedInputRef` 与 `sourcePreparationRef`。外部输入在每个源码身份独立的目录中准备；执行时纳入同一准备池写入日志，保持 Cargo 的同级相对路径关系。

路径数量较少时可以重复使用 `--owned-path` 和 `--untracked-path`。大量文件使用 JSON 清单，避免 Windows 命令长度限制；同一种路径输入不能同时使用两个形式。`--required-test` 可以重复，每一项必须是本次实际需要运行的完整 Cargo 测试名称。

### 先准备源码，再派发产品流程

需要先确认源码已封存时，可以在选中执行版本和正常授权下，依次调用协调器现有 `current-source/register` 与 `flow/prepare-current-source` 协议。登记输入必须与普通 operator 的规范化 intent 完全一致；准备控制保留原 identity、`currentSourceRef`、`buildRoot`、`driverDigest` 和 `runtimeOperationId`。这是协调器的源码准备入口，代理不直接调用捕获或物化函数。

人工源码静默确认不是这两个 API 的参数或授权要求。完成当前 HEAD、文件清单及内容哈希核对后，可以在既有任务授权内正常调用；协调器在登记、封存前后及完成准备前重复检查主仓库和外部输入。共享源码发生变化时保留原失败或未完成记录，下游构建必须等待完整 preparation。结果未知时先核对原请求，不能新建身份掩盖未知派发。

只有 `prepared` 响应、权威 `source_preparation` operation 的 `complete` 状态及匹配的 preparation record/result 都核对后，才可以派发同一输入。尚未有任何投递时，正常 `current-source/submit` 接受原六字段 identity、`currentSourceRef`、`buildRoot`、`runtimeOperationId`；它继续使用正常 Jenkins Pipeline 和幂等 submission marker，不要求代理创建或改写 operator lifecycle。已存在在途或未知 marker 时续查原派发，不能另发一次。

后续 Pipeline 的缓存准备分支校验已完成 operation、不可变主仓库与外部 CAS 对象，再使用原签发输入。普通 operator `submit` 的重新登记则仍会在复用 slot 前检查外部 `zr_vm` 的当前 HEAD、哈希和未跟踪路径；外部仓库在成功封存后继续修改时，这条重新登记路径可能被拒绝。此时只能使用已经核对的原准备结果及上述首次投递协议，不能改原 intent 或替换外部基线。

准备返回 `current_source_base_changed` 或 `current_source_hash_mismatch` 时，保留原身份、回应、operation 和已有 CAS，先查询同一请求并定位变化。没有有效 preparation 的请求不能走缓存续接；明确过期后取得新的稳定输入，才能建立有历史引用的新请求。等待时间或空闲采样不是源稳定、终态回收或验收证明。

## 续查与产物

提交后保留原身份查询：

```powershell
& .jenkins/runtime/python/python.exe -B -m tools.dev.jenkins_editor query `
  --session-id <original-session-id> --request-id <original-request-id> `
  --attempt-id <original-attempt-id> --build-root D:\cargo-targets
```

重复提交相同请求只查询已有请求。注册或投递超时后仍查询原请求，不生成新尝试掩盖未知结果。执行版本、运行代次或输出根目录改变时，入口拒绝悄悄替换身份，需要对原请求进行归属明确的协调器恢复。

### 确认未登记后的原身份恢复

注册大清单可能需要多遍物理路径和内容检查。`--timeout` 仍是查询与投递的有限预算，默认 60 秒、上限 300 秒。`--registration-timeout` 单独控制注册预算，未指定时沿用 `--timeout`，上限 900 秒；参数均拒绝非有限值。这个预算不覆盖入口在启动注册子进程之前的元数据校验。

`resume-registration` 只允许现有 `registration_unknown` 阶段，并直接读取原 `intent.json`。恢复命令不接受新的 hashes、base、owned/untracked/external、profile、tests 或 source ref；仓库、会话、请求、尝试、driver、runtime 和构建根仍保持原值。普通 `submit` 不自动执行这项恢复。

```powershell
& .jenkins/runtime/python/python.exe -B -m tools.dev.jenkins_editor resume-registration `
  --session-id <original-session-id> --request-id <original-request-id> `
  --attempt-id <original-attempt-id> --build-root D:\cargo-targets `
  --registration-terminal-receipt <absolute-root-owned-terminal-receipt-json> `
  --registration-timeout 900 --timeout 300
```

必须先由原控制调用的观察者提供以下终态收据；所有哈希取自真实字节。该收据只证明 bounded protocol subprocess 已由原调用回收以及其统一 exec 已退出，不是受管 Cargo 的 NativeJob 终态或产品验收证据。旧原调用没有编号时使用 `original`；新调用按 lifecycle 的 `registrationCallId`，不能重复使用旧调用的证明。

```json
{
  "schemaVersion": 1,
  "kind": "jenkins-editor-registration-terminal",
  "identity": {
    "repositoryId": "<original repository ID>",
    "sessionId": "<original session ID>",
    "requestId": "<original request ID>",
    "attemptId": "<original attempt ID>",
    "generation": 1,
    "stageImplementationDigest": "<original driver digest>"
  },
  "runtimeOperationId": "<original runtime>",
  "buildRoot": "D:\\cargo-targets",
  "intentDigest": "<canonical JSON digest of original intent.json>",
  "registrationCallId": "<original or lifecycle call ID>",
  "protocolInputSha256": "<original current-source-register.json file SHA-256>",
  "protocolImplementationSha256": "<tools/dev/jenkins_validate.py file SHA-256 used by the call>",
  "operatorHandle": {"kind": "unified-exec", "id": 123, "status": "terminal", "exitCode": 0},
  "registration": {
    "domain": "current-source", "action": "register", "status": "terminal",
    "cause": "timeout", "timeoutSeconds": 300, "processReaped": true
  },
  "evidence": {
    "dispatchResult": {"path": "<absolute original dispatch-result path>", "sha256": "<file SHA-256>"},
    "dispatchIntent": {"path": "<absolute original dispatch-intent path>", "sha256": "<file SHA-256>"}
  }
}
```

原 dispatch-result 必须保留 operator `exitCode: 2`、`operatorResult.reasonCode: protocol_delivery_unknown`、`retryable: true` 和 `formalAcceptance: false`，并通过 `dispatchIntentSha256` 绑定所引用的原 dispatch-intent。终态收据也绑定不可变注册输入和实际 `jenkins_validate.py` 实现哈希。不要为仍在途、归属未知或没有这项证据的调用填写终态。

恢复在原请求锁内先查询同一协调器身份。只有无冲突、非 retryable 的明确 `not_found` 才按原输入继续一次注册；查询超时或未知状态保留原未知结果。迟到的 ref 必须是完整 SHA-256，并与只读 State 中的原不可变意图匹配。存在 submission marker、准备结果或 request 时仅观察；确实只有登记而尚未投递时，复用 exact ref 进行一次正常投递，协调器的原幂等 marker 继续约束并发。

原 lifecycle、终态引用和每次新查询保存在该 operator slot 的 `registration-recovery` 追加收据中。恢复不删除历史。若本次再次超时，需要这一次调用的新终态证明；执行版本改变时不能把新 driver 偷换进旧意图。源码检查、封存和捕获仍全部由原选中协调器负责。

固定的 `development` 和 `shipping` 模板依次执行包检查、测试编译、测试发现、测试运行、编辑器 EXE、Runtime DLL 和打包。模板关闭默认特性，使用部署配置中明确的 `editor-product` 资源估计。Shipping 生成配套目录和 ZIP，并执行已有体积规则。

EXE 和 DLL 必须来自同一已封存输入并携带配套 BuildSet 清单。打包校验字体、资源和官方 CRT；各编译阶段需要实际进程结束、子进程结束、日志流结束及产物字节回执。所有编译产物、缓存和临时编译文件物理保存在盘根 `D/E/F:\cargo-targets` 的 `zircon-local` 命名空间中。

源码准备完成、Jenkins 队列 ID 或构建结束都不代表产品验收。`formalAcceptance` 只取自匹配身份的协调器验收结果。该结果还不能替代真实窗口的视觉、鼠标键盘操作、保存重开、Play/Stop、1080p 性能和分发目录启动验证。

## 启用条件

运行上述提交前，当前 Jenkins 执行版本必须包含此入口及固定产物模板，并已完成相应支撑代码回归和归属明确的版本切换。旧的本地协调器及其注册、租约、队列均保持停用；这些命令不恢复旧服务，也不改写旧任务结果。
