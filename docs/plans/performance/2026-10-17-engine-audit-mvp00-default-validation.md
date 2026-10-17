# 当前源码 MVP00 默认包验证合同

任务：engine-audit-20261009-01a12097；源 owner：Root 01a12097-5c69-7753-b694-f02c04d45968。执行窗口仍待协调，本文及请求处于独立复核阶段；尚未提交新原生操作。

## 本次执行范围

新 request/operation：engine-audit-20261017-mvp00-runtime-default-s50-r1；session 沿用 engine-audit-20261009-01a12097。请求见 .codex/outbox/engine-audit-20261009-01a12097/s50-mvp00-runtime-default-check.request.json。采用公开 submit-jenkins-management.ps1 的 validate 入口、实际 checkout、full-file-index-v2、完整 external root E:/Git/zr_vm。请求 buildRoot=E:/cargo-targets；构建产物与编译缓存须物理位于该根下，实际子路径与依赖缓存输入由受管准备和原生收据核验。

本次只执行默认 features 的 Runtime 包检查，语义参数为 cargo check -p zircon_runtime --locked --offline --jobs 1。payload 的 profile=dev；已审接收端不显式追加 --profile，最终 argv 以 prepare_command 及实际原生收据为准。features=[]、noDefaultFeatures=false，check 不附加 lib/noRun/测试harness参数。实际 argv、toolchain、产物根及 native PID/creation-time/exit/双EOF/waiter/job终态须由原生收据记录。沿用现有受管执行预算，不在 payload 中添加未支持字段。

Runtime 默认包通过后，按 [MVP00](../mvp/00-current-source-baseline-recovery.md) 的同相关源码输入依次推进 Editor、App 默认包检查。每个阶段须有实际终态；首个失败从最低 owner 修复后只重跑受影响检查。本次单 Runtime 请求不提前调度后继包。

## 与原 App 冲突的关联

原 engine-audit-20261016-current-app-compile-s42-strict / queue3662 / management #82 保留其 complete、preparation-conflict、compileStarted=false、preparationStarted=false。原 inputDigest=8a604b1f06eeca5464c3439d94d3dfc75cde5a33b6d447e76ef0b0dda0b180d4；expectedSourceDigest=46b224860234f69290e0d4432f7ebeeb0be347bbfc40ce44f7885e4a68d9eda1；actualSourceDigest=2bc5131a8b38cf83ecb300dfd4a8b4e61e2adbba5d344abef0c7491c915123e4。

Root 实比原 durable delivery 与 #82 receipt，仅 externalRoots[0].source.filesDigest 及派生 digest 变化；engine/cache/所有 HEAD、index 与 gitlink 子树摘要相等。外部 E:/Git/zr_vm 的具体变化文件及 actor 尚未证明。当前 engine commits 和 Cargo.lock 漂移发生于原 receipt 之后，保留其时间边界。

新 attempt 独立捕获实际完整输入；由公开入口绑定新的 expectedSourceDigest/inputDigest。原 request、operation、marker、失败收据及摘要不改写。上述 predecessor 关联只记录于本文及账本，payload 不新增未经支持的 predecessor 字段。开始捕获前完成 Root 的源码/文档写入；协调 engine、external root、index 与依赖配置写入窗口，保留所有 strict before/after guards。任何 guard 冲突保留实际诊断与未验收状态。

## 聚焦行为与产品边界

后续 App focused 回归需 App features dynamic-api,platform-winit,native-host、lib test、filter entry::tests::product_composition::close_owner::、testThreads=1；当前有三个普通测试及一个 ignored，不启用 include-ignored。原 App 的两个 feature 未启用 App/native-host。Runtime 同名 feature 不会反向启用 App feature。

本合同暂只授权上文 Runtime 默认包检查请求；后继 Editor/App 和 focused 路线按用户既有目标另建具名受管阶段并保留关联。三项关闭测试仅覆盖对应 owner/retry/typed-primary 合同；真实 DLL、窗口、渲染、编辑、保存重开及产品退出继续待验收。冻结 native16 与未知 A1 保持原账本。

## 验收与证据

本阶段需要：公开 schema 与源码 feature 独立复核、实际源码/配置/toolchain/产物指纹、原生进程终态、Cargo exit 0，以及完整输入 guard 的实际结果。submitted、observed、queued 和 CLI wrapper exit 0 只按其真实含义记录。

MVP00 的 M0.1 工具行为批次、M0.2 resolver focused/upward、M0.3 三包默认 check 仍逐项对账。本阶段通过只关闭 Runtime 默认包检查。Tracy/RenderDoc/GPU timestamp/WPR 实际产品采样继续开放；优化前后各五次独立运行、预热后稳态三十秒要求保持。

Root 不执行提交、推送、发布、源码副本、共享缓存清理或权限配置维护。本任务本轮删除零文件、释放零字节；必要原始日志与失败证据按任务 artifacts.json 保留。
