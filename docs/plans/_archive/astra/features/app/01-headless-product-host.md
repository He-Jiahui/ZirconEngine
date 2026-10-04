---
status: implemented_pending_validation
source_recheck_required: true
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_runtime/301-runtime-core-lifecycle-taskgraph-session-shutdown-review.md
  - docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
---

# Headless 产品宿主闭环

## Finding 与当前证据

对应 Runtime301 `CORE-LIFE-P0-02`、App01/App08 `P0-1`，状态为
`implemented_pending_validation`。`target-server` 已有 `zircon_server` binary；
`entry/entry_runner/headless/` 从固定 owner 线程创建真实 Runtime DLL session，消费
`tick_frame`、host request、wake 和 destroy receipt，首个 tick 与 drain 成功后发布 Ready。

控制器在取消或开始销毁时固定进程 shutdown deadline，owner join、失败清理重试、
进程日志和 Windows console 等待复用该截止时间。超时保留同一线程及 DLL owner，
显式恢复仍可继续回收；产品进程在截止时间后仍有活动 owner 时按安全合同中止。
BuildSet、项目错误、deadline、重试及线程亲和已有定向回归源码，真实 Windows
server/DLL、feature 构建、长时运行及性能验收尚未执行，不能据此关闭原 finding。

## 架构与参考

App 拥有 binary、参数、循环、等待/唤醒和进程退出；Runtime 拥有项目、模块、时间、
World/ECS、服务调度和关闭。保留现有 Runtime DLL session 合同，server 不创建窗口、
surface 或 GPU device，不通过套用桌面 EventLoop 构造无窗口模式。

主参考 `dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp`
的 Tick/Exit 将时间、服务工作、退出意图和诊断放在产品循环；Rust 次参考
`dev/Fyrox/fyrox-impl/src/engine/executor.rs::run_headless` 保留 readiness、实际 update、
节拍和退出 controller。此处复用现有 Zircon time policy，不复制参考中的裸 sleep 比例。

落点为 `zircon_app/src/bin/server.rs`、`entry/entry_runner/headless/` 的启动、控制、
调度、终态和测试；`headless.rs` 保持入口接线。RuntimeSession/wake 的共享修改由
本计划唯一 owner 迁移 windowed 与 headless 消费者。产品 feature 和 BuildSet 必须能
证明不需要 desktop window/input bundle；构建路径调整只服务该产品，不扩展 tooling。

## 实施范围

1. 增加 target-server 对应的 `zircon_server` binary 和公开可取消 headless runner。
   参数复用现有项目、profile 和日志解析 owner；拒绝重复/未知或互相冲突输入，help
   不加载 Runtime。显式控制运行 tick 数用于产品自动验收，默认持续运行直到退出请求。
2. 通过已有 artifact/BuildSet/profile admission 创建真实 session，首个成功 tick 后才
   记录 Ready。缺 required module/plugin、项目失败、版本不兼容都返回非零终态。
3. 调度使用 monotonic deadline 与 Runtime product time policy；标准 signal/admin
   退出及 owner cancellation 能唤醒等待，不丢 wake，不因早唤醒重置固定节拍。
   固定 tick 不因无 render demand 停止模拟；超载不无界补帧，也不返回假成功。
4. Runtime 请求窗口/剪贴板等不支持宿主能力时保留 typed unsupported 结果及诊断；
   不忽略或把请求留成无界队列。需要新 wire outcome 时由 Interface owner 版本化。
5. 任意 startup/tick/stop 错误保留 primary failure；按共享 deadline/receipt 关闭 session、
   模块、任务与诊断，成功后卸载 DLL。超时 owner 不释放，进程退出按现有安全合同。
6. 连接 WOC native server 的真实 service ingress/simulation/egress/persistence 属下个
   子里程碑；本入口完成不关闭 App05/WOC 的网络和持久化 findings。

WOC 的 `woc_headless` role 也已接入同一真实 ZrVM/transaction path，并明确将
`presentationReady=false`、`renderedFrames=0` 发布为非可视化结果；这不替代
`zircon_server` 的 engine DLL/headless runner 或 WOC 服务层验收。

## 验收

- `--no-default-features --features target-server` 构建 binary 和匹配 Runtime；解析无 Winit
  依赖，运行无 native window/GPU device。实际子进程加载同一封存 BuildSet。
- 真项目 N tick 后检查 World/operation 可观测输出、Ready/terminal 顺序；空项目、
  初始化失败、tick 失败、用户取消、signal、重复关闭均有行为测试。
- early wake、连续 wake、等待中取消、deadline 溢出、时间回退输入、慢 tick 和超载
  都有确定性 clock/controller 测试；不可用 sleep 排序来假设竞态。
- 平稳 release tick 热路径无新逐帧 allocation；沿用 time/tick 有效门槛，p95 回退
  不超过 5%。记录空闲 CPU、tick p50/p95/p99、jitter、最长取消/退出时间和 RSS。
- 单元 fixture 只证明边界，真实 Windows server/DLL、进程退出、零遗留任务及长时运行
  证据齐全后才能 accepted。与首批修复合并验证，提交后继续独立工作。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Shared shutdown deadline | task graph scope drain、worker join、owner cleanup 的绝对 deadline 传递与重试 census | `implemented_pending_validation` | 2026-09-11 | `shutdown_until_does_not_restart_budget_for_later_scope_drains` 覆盖首个 scope 中段收敛、后续 scope 不重置预算、超时后释放并重试；既有 retry/repeat census 回归与 scoped rustfmt 通过；受管 Cargo、真实 Windows server/DLL、完整 host teardown 与性能证据仍待执行 |
