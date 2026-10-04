# Hub 本地服务验证记录

status: `accepted`
milestone: `HUB-S0-S5-local-service-windows`
date: `2026-09-07`

受管 Windows 验证使用不可变 `zr_vm` 快照 `1966223f74da1d7df8ace543ca98588594cca34e`，
外部内容摘要为 `c6c17cdc4ae51d83a62dea4accaf670b5bb793d69c06bf9cd2fa60da3ddfc500`。
完整 workspace manifest 与当前 Hub overlay 封存于 validation-copy job
`6b91ac7cea634fefa3db80aee52efb89`，输入 manifest 摘要为
`cded504771c3555e5cb924d6ab99a22e88dbbd9d88cfca4c32c112e3878e2ccc`。

命令：

```text
cargo test -p zircon_hub --no-default-features --features local-service --lib --locked
```

持久化 request `44413c49b9b541879e0e39033576ea6e` 已完成，run 与 job 均为
`6b91ac7cea634fefa3db80aee52efb89`，退出码为 `0`。测试结果为 **78 passed, 0 failed,
0 ignored**，覆盖 service catalog、OIDC、组织授权、云 snapshot/blob/retention/maintenance、
HTTP 路由、SQLite migration、Windows PathGuard 与旁路文件安全。

本次修复关闭了受管执行暴露的两项测试层问题：Windows PowerShell Core 没有可自动加载的
`Set-Acl` 模块，测试夹具改用内置 `icacls.exe` 设置与部署脚本一致的受保护 owner/SYSTEM
DACL；热回滚 journal fixture 在 detached copy 上写入 SQLite 固定 hot-journal marker，
避免活动 writer 的 Windows 缓存显示零头。HTTP mutation fixture 改用服务枚举实际接受的
`invitation_id` 字段。生产 `PathGuard`、服务授权和数据库事务代码未通过放宽安全约束来绕过
失败；`rustfmt --edition 2021 --check` 与 scoped `git diff --check` 通过。

此前同一输入的首轮结果为 43/78（ACL fixture 未建立），修复后为 37/78（仅夹具故障），
随后热 journal 定向回归 `1/1`、HTTP cloud route 定向回归 `1/1`，最终全量为 78/78。
该记录接受 Hub 本地服务 Rust 单元验证；DPAPI/native broker、长离线恢复、release 存储规模
及磁盘掉电测试仍由 S1/S4/S6/G28 门禁单独追踪。

## 真实 OIDC 与服务恢复

`HUB-S1-S2-S5-process-probe` 于 `2026-09-07` 接受 **40/40** 检查，其中 6 项验证环境与
输入准入，34 项通过真实 Keycloak/HTTP 交互验证授权和持久化行为。Node 仅承担客户端：
真实 Chromium 执行三个临时用户的 authorization-code + S256 PKCE 登录，Rust 服务分别以
PID `4020`、`18112` 监听 `127.0.0.1:8787`；真实 Keycloak PID `3100` 提供 `8080` realm
与 `9000` readiness，回调使用 `8480`。覆盖组织隔离、分页邀请/撤销、角色及所有权移交、
下一请求撤权、签名 catalog/许可/发布者撤销、并发 snapshot 单赢家与稳定 conflict receipt、
显式新 operation 解决冲突、丢响应查询 receipt、进程强制终止后重放不重复写及 blob 恢复。
状态为 `true` 的组合检查前均有实际 HTTP 状态断言；重放还比较独立请求结果和重启后查询，
没有用客户端自己构造的响应代替服务结果。

受管 binary job `fca1cef59202441cb068cda8bc45ae9a`、run request
`5f68f738d455442bb4e0177dc2d84be1` 以退出码 `0` 完成
`cargo build -p zircon_hub --no-default-features --features local-service --bin zircon_hub_service --locked`。
复用上述 `zr_vm` pin；输入 manifest SHA-256 为
`6d91aba3efa22bf6b728e0126e6ccf764d13798d1d4932617b8be38e76f5ef79`，pool 原位 executable
SHA-256 为 `1952f333dbeeaaaa71b9ec2dc0d6fb40af0f7116b23648223c18014d8f6afbe3`。
本轮 dev profile 依赖工具链 `std-9bb8353cbe481272.dll`，其 SHA-256 为
`d1171903ffa1598e292ca4784799d2195fabfdb4c129d790662ca2a844208bf5`；探针仅为子进程补充该
DLL 目录，不复制产物、不更改全局 PATH。每次启动前重验 executable 摘要。

探针入口：[service-probe.mjs](../../../../../zircon_hub/deploy/local/probes/service-probe.mjs)
SHA-256 `45dc01462aa1b97b3b518a6539e242b60a69011936d58181a10427761f3dfde0`；
[oidc-client.mjs](../../../../../zircon_hub/deploy/local/probes/oidc-client.mjs)
SHA-256 `daeb2ab9ea5d64d9cd1ccd61bb6700211f24fbd64694c824792d7ce2e41601d4`。
复现复用现有本地部署，先在另一终端执行下面的 `start.ps1`，保持 Keycloak 运行，再运行探针；
需要原有私有部署配置和 Playwright Chromium，不重新导入 realm 或重置密钥：

```powershell
$hubProbeRuntime = 'C:/Users/HeJiahui/.codex/visualizations/2026/09/06/01a0780b-97a2-7380-9b15-1e90591a6b4b/hub-local-runtime'
& "$hubProbeRuntime/start.ps1"
# 在另一终端设置同一 hubProbeRuntime 后运行：
& 'C:/Users/HeJiahui/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node.exe' zircon_hub/deploy/local/probes/service-probe.mjs `
  --runtime $hubProbeRuntime `
  --executable 'E:/cargo-targets/zircon-engine/pool/672137e9ccccba71125de548cff9b4ebbd03131dcbad2ee27602d6c650c70c1e/debug/zircon_hub_service.exe' `
  --build-job fca1cef59202441cb068cda8bc45ae9a `
  --input-hash 6d91aba3efa22bf6b728e0126e6ccf764d13798d1d4932617b8be38e76f5ef79 `
  --executable-hash 1952f333dbeeaaaa71b9ec2dc0d6fb40af0f7116b23648223c18014d8f6afbe3 `
  --runtime-dll-directory 'C:/Users/HeJiahui/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin' `
  --playwright-root 'C:/Users/HeJiahui/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules' `
  --chromium 'C:/Users/HeJiahui/AppData/Local/ms-playwright/chromium-1217/chrome-win64/chrome.exe'
```

本轮 evidence 位于上述 runtime 下
`service-run-fb5d5d3d-5b36-4818-ba70-f0a9e96cd6c4/evidence.json`，SHA-256
`5af14ea42861f119f506a99ff80e5936dfcc3849bbd42dfba1a6d27a56e83866`，同目录有真实登录页
`keycloak-login-1280.png`。探针已终止两个服务进程并删除三个临时用户；Keycloak 通过 Ctrl+C
停止，随后批处理退出，`8080/9000/8787/8480` 均无监听者。原服务配置摘要保持一致，既有
凭据/云密钥/realm/database 保留，探针不输出 token、密码或私钥。

边界：catalog 此次只验证服务返回的已授权 release 元数据，**未下载或安装 package**；
`nativeBrokerExecuted`、`tauriExecuted`、`installationExecuted` 均为 `false`。
服务重启采用 Windows 强制终止，不能代替优雅关闭、断电或 Windows Credential Manager
恢复验收。先前缺开发期 std DLL 的启动失败及 JSON 对象键序比较导致的探针失败均保留在
同一 runtime 的其他 `service-run-*` 中，不计作已通过产品断言。

## 2026-09-25 受管复核增量

本记录的 Rust local-service 回归在当前工作区快照上重新执行。受管测试作业
`ace5a376fb1444469fbaa43b84c66200`（`zircon_hub`、`local-service`、lib tests、单线程）
实际执行 **84 passed, 0 failed, 0 ignored**；本轮修复的 Windows junction fixture 和
schema-v4 migration fixture 均包含在该结果中。Hub 静态合同套件为 **81/81**，目标相关
插件、生命周期、asset/glTF、RenderGraph replay、hit-grid 和 Editor native-window
静态合同合计 **65/65**。

同一 `local-service` feature 的 release-profile build 作业
`35af2d2753ae4528896e3c8f3537219f` 以 `exit_code=0` 完成，`zircon_hub` library 与
`zircon_hub_service` binary 均完成链接。验证包装器曾在成功编译后报告 coordinator
内部错误；该作业随后按实际编译结果以 `exit_code=0` 完成，不能把包装器错误误记为
编译失败。上述证据只升级本地服务 Rust 回归和 release compile 的当前快照状态，
不替代本记录中仍明确开放的 DPAPI/native broker、真实 Tauri 和长时服务恢复验收。
