# Runtime11 Dynamic Scene Writer Runtime Owner Hard Cut 架构与验证计划

> 日期：2026-08-26
> 所属 failure：`runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 当前源码结论

`RuntimeSessionArchiveWriter` 已有 bounded keyed lane、path generation authority、before-start cancel
capability 和 typed terminal，但公开 `new(limits)` 仍静默取得
`TaskPools::process_default().io()`。全仓没有该 constructor 的调用者；现有 `with_scheduler` 仅被
`cfg(test)` lib-test fixture 使用。

因此当前 API 允许未来产品调用者在没有 Runtime owner、scope 或 shutdown generation 时创建可执行
writer，并让 lane 越过 runtime 生命周期。这个旁路与 writer 的 bounded queue correctness 无关，必须
在进入产品调用链前硬切，不能等出现调用者后再保留兼容构造器。

## 2. Unreal Engine 对照与本仓裁决

主要参考 Unreal Engine
`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncWork.h`：异步任务显式接收 queued pool，
task owner 销毁前必须完成 idle/terminal 协议；owner 缺失不会静默切到另一个 pool。

Zircon 的 writer production 构造只接受 `&CoreHandle`，从该 handle 的 I/O pool 派生唯一 scheduler；
内部保存 `CoreWeak` 作为后续 admission authority。测试可以在 `cfg(test)` 下显式注入 isolated scheduler，
但该入口不进入 production 编译面。

## 3. 目标状态机与所有权

writer 内部 owner 只有两态：

- `Runtime(CoreWeak)`：production `with_runtime` 唯一生成；
- `Fixture`：只在 `cfg(test)` 生成。

`try_submit` 在任何 path canonicalization、generation reservation 或 lane admission 前 upgrade Runtime
owner；失败立即返回 `RuntimeUnavailable`。成功取得的 `CoreHandle` 只活到 armed admission activate
完成，保证提交线性化期间 owner 存活，随后显式释放。不能把 handle 捕获进 work closure，否则可能形成
`runtime services -> writer owner -> lane closure -> runtime` 强引用环。

现有 `reserve path -> lane admit -> path admit -> ticket publish -> activate` 顺序不变；owner 失败不会
推进 path generation，也不会触碰 filesystem。

## 4. 复杂度、资源与性能假设

| 路径 | 修改前 | 修改后 |
|---|---:|---:|
| production constructor | process-global pool lookup | CoreHandle I/O pool clone，`O(1)` |
| submit owner gate | 无 | CoreWeak upgrade，`O(1)` |
| lane admission / write | bounded keyed lane | 不变 |
| process-global writer constructor | 1 | 0 |

本切片增加一个 writer-sized weak owner和每次 submit 一次原子 weak upgrade，不改变 JSON serialization、
temp write、flush 或 atomic rename 算法，也不宣称吞吐、CPU、RSS 或功耗改善。后续产品 profile 必须测
submit latency、weak-gate cost、queue age、write service P50/P95/P99、shutdown wall、线程恢复、CPU、RSS
与功耗；只有数据证明 owner gate 或 global path map 是瓶颈时才优化。

## 5. 确定性验证计划

- 创建 explicit `CoreRuntime` 与 writer，随后 drop runtime；
- submit 必须在 path reservation 前返回 `RuntimeUnavailable`，目标文件不存在；
- source guard 断言 production writer 不再包含 `TaskPools::process_default` 或 `pub fn new`；
- 既有 capacity-rejection 和 before-start cancel 回归继续使用 isolated single-worker fixture。

执行 scoped `rustfmt --check`、owner/admission 源码断言、owned trailing whitespace 与 diff check。受管
Cargo、slow-I/O、burst、CPU、RSS 与功耗矩阵保持 pending；没有匹配哈希的回执前 failure 保持 open，
不提交 milestone、不发送手工企微。

## 6. 本切片完成定义

- production writer 只能从 live Runtime owner 构造；
- expired owner 在任何 path intent 或 filesystem work 前 typed 拒绝；
- isolated scheduler constructor 不进入 production 编译面；
- 新回归和 no-process-fallback guard 已挂载；
- source/static 与 managed/performance 状态分开记录。

## 7. 2026-08-26 源码验证结果

- production `RuntimeSessionArchiveWriter::new` 已删除，唯一 production constructor 为
  `with_runtime(&CoreHandle)`；scheduler 从该 Runtime I/O pool 派生；
- writer 保存 `CoreWeak`，submit 在 canonical path、generation reservation 和 lane admission 前取得短
  owner lease，activate 后显式释放；
- isolated `with_scheduler` 已限制为 `cfg(test)`，production process-global constructor 命中为 0；
- expired-owner typed rejection 与 no-process-global source guard 共 2 项新回归已挂载；既有
  admission-intent/cancel 回归保持不变；
- scoped `rustfmt --check`：2/2 Rust 文件通过；
- owner/admission/path-intent 源码断言：14/14 通过；
- Rust 文件规模：writer 211 行、私有 tests 178 行，均低于结构限制；
- 受管 Cargo、slow-I/O、burst、CPU、RSS、shutdown wall 与功耗样本：0，保持 pending。

本切片把 archive writer 的 production process-global constructor 数从 1 收敛为 0；没有产品调用者需要
兼容迁移。它不关闭完整 async reader/result-retention、sync path facade 或统一 ExecutionScope/code lease。
