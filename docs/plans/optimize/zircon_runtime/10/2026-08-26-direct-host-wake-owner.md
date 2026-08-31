# Runtime10 Direct Host Wake Owner 架构与测量计划

> 日期：2026-08-26
> 所属 failure：`runtime/10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 当前源码结论

`RuntimeWakeRegistration::register` 接收并登记 `EventLoopProxy`，但 registration 本身只保存 token。
`RuntimeSession::wake_host` 已经持有 registration，却调用 `wake_token(token)`，再次取得 process-global
`Mutex<HashMap<u64, EventLoopProxy>>`、clone proxy 后才 wake。只有 DLL 回调
`runtime_wake_trampoline(token)` 缺少 Rust owner，确实需要 token registry。

因此 host-owned wake 和 foreign callback wake 被错误合并成同一查询路径。每个 host wake 都承担一次
全局 mutex acquire、HashMap lookup 与 proxy clone；更重要的是它把本来 session-local 的唤醒与所有
session 的 callback register/unregister 放进同一锁域。

## 2. 参考与 owner 边界

本地 Unreal `ApplicationCore/Private/HAL/InputThread.cpp` 与
`Core/Internal/IO/GenericPlatformIoDispatcher.cpp` 均由持有 `WakeEvent` 的 owner 直接调用
`Trigger()`；只有跨边界定位 owner 时才需要 registry/dispatch。Zircon 保留 winit proxy 和 token ABI，
不复制 UE 的线程或全局类型。

目标边界：

- `RuntimeWakeRegistration` 同时保存 token 和 owned `EventLoopProxy`；
- host `wake()` 在 registration 仍有效时直接调用 owned proxy，不访问 registry；
- FFI trampoline 继续以 token 在 registry 中取得短生命周期 proxy clone，并保留 panic containment；
- `unregister()` 先从 registry 删除 token，再将 token 置零；之后 host direct wake 和 stale callback
  均为 no-op；
- 不增加第二份 registry、兼容方法、thread-local cache 或 raw pointer。

## 3. 复杂度与风险

| 路径 | 修复前 | 修复后 |
| --- | --- | --- |
| host-owned wake | global mutex + HashMap + proxy clone | registration validity check + direct proxy call |
| FFI callback wake | global mutex + HashMap + proxy clone | 不变 |
| registration state | token | token + one proxy handle |
| unregister | map removal | 不变 |

`EventLoopProxy` 本来已经在 registry 中保有一个 clone，registration 新增一个 handle clone，不复制 event
loop state。该变化消除的是错误锁域，不在无动态证据时宣称 p95、CPU、功耗或跨引擎耗时改善。

## 4. 验证与性能计划

源码阶段：

- 行为回归分别触发 host direct wake 与 ABI callback wake，二者都 exactly once；
- unregister 后两条路径均不再唤醒；
- source guard 锁定 `wake()` 直接使用 owned proxy，禁止回调 `wake_token`；
- 保留 callback panic containment；
- 执行 Rust 2021 rustfmt、source assertions、owner line budget、trailing whitespace 与 scoped diff check。

受管阶段在非 `C:` 目标运行 Runtime10/App focused Cargo，并在 `1/1k/100k` host wake、并发
register/unregister 与 mixed callback 场景记录 registry lock wait/hold、lookup/clone count、wake p50/p95/p99、
main-thread wall 与 CPU。Windows host 允许时才采 WPR/功耗；无回执前不作量化结论。

## 5. 完成定义

- host-owned wake 的 registry lookup 和 lock 次数从每次 1 收敛为 0；
- FFI callback 的 token lifetime 与 panic boundary 保持；
- unregister 后无 stale wake；
- failure 只记录 source implemented/static checked，完整 typed host-request、failed-destroy quarantine 与
  产品性能矩阵仍保持 open。

## 6. 2026-08-26 源码实现与静态结果

- `RuntimeWakeRegistration` 现在保存 token 与 owned `EventLoopProxy`；registry 只保留供 FFI callback
  定位的 proxy clone；
- host `wake()` 在 token 有效时直接调用 owned proxy，方法体不再引用 `wake_token`、global registry、
  mutex、HashMap lookup 或 callback-time proxy clone；
- FFI trampoline 继续使用 token registry 并保持 `catch_unwind`；`unregister()` 后 host direct 与 stale
  callback 两条路径都 no-op；
- 现有行为测试扩展为 direct/callback 各一次 exact wake 与 unregister 后零新增 wake，并新增 source
  lock-domain guard；
- scoped `rustfmt --check` 1/1、static contracts 8/8、owner 155 行、scoped `git diff --check` 通过，
  仅有工作区 LF/CRLF 提示。

以上不等同于 Runtime10/App Cargo、storm、延迟、CPU、WPR 或功耗证据；这些动态结果均未执行，
本切片不作性能改善或跨引擎接近值声明。
