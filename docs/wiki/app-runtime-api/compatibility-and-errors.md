---
related_code:
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_shape.rs
  - zircon_runtime_interface/src/runtime_api/abi/host_api_shape.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime_host/src/foreign_output/error.rs
  - zircon_runtime_host/src/foreign_output/metrics.rs
implementation_files:
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_runtime_host/src/foreign_output/error.rs
  - zircon_runtime_host/src/foreign_output/state.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki
  - docs/zircon_runtime_interface/runtime_api.md
tests:
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_host/src/foreign_output/tests.rs
  - zircon_app/src/entry/runtime_library/tests.rs
doc_type: testing-guide
---

# 兼容性、错误处理与排错

## ZrStatus

所有动态调用用 `ZrStatus` 返回同步结果。`diagnostics` 是 borrowed `ZrByteSlice`，失败时必须在 provider 仍加载、同线程下一次动态状态生成之前同步读取。最大允许 4 KiB；宿主应先 `checked_slice`，再用 UTF-8 lossy 方式记录无法严格解码的诊断。

| code | 含义 | 常见处理 |
| --- | --- | --- |
| `Ok` | 调用成功 | 继续校验输出 carrier，成功状态不代表任意外部 pointer 自动可信 |
| `Error` | 一般运行时错误或未知原始码 | 记录 diagnostics，按操作决定是否终止会话 |
| `UnsupportedVersion` | DTO/host/table 版本不被接受 | 停止握手，不做旧版本猜测 |
| `InvalidArgument` | 空指针、非法 handle、 malformed JSON/DTO | 修正 caller；不要原样重试 |
| `NotFound` | session、viewport、token、ticket 或 allocation 不存在 | 检查生命周期和 owner 归属 |
| `CapabilityDenied` | 能力策略拒绝 | 更换 profile/插件/权限，不能当作暂时错误 |
| `Panic` | runtime 导出捕获到 panic | 会话状态可能不可继续，记录并停止 |
| `BridgeNotEnabled` | 所需 bridge 未启用 | 检查 runtime build/profile 与插件选择 |
| `LimitExceeded` | 字节、item、深度、时长、帧尺寸等超预算 | 缩小请求或分页；不得把它归类为 malformed input |

未知原始状态码通过 `ZrStatusCode::from_raw` 收敛为 `Error`，保持宿主分支穷尽。

## 版本兼容规则

1. API table：只接受 version 8 且 `size_bytes == size_of::<ZrRuntimeApiV8>()`。任意增删字段都必须发布新表并协调硬切。
2. Host table：V1 同样要求精确版本与大小；callback capability 在握手后单独判断。
3. DTO：检查自身 `abi_version`。一个 V8 表引用 V1/V2/V3 DTO 是正常的独立版本族。
4. BuildSet：artifact sidecar、接口 spec、目标模型和 payload schema identity 必须匹配预期；表形状相同也不能绕过 BuildSet 校验。
5. JSON DTO：多数类型启用 `deny_unknown_fields`。新增字段即可能是破坏性变更，应升级 DTO/表或同步硬切，而不是假设旧 consumer 忽略它。
6. 持久化格式：通过 schema version 和显式 migration 前向升级，与 ABI table 版本无关。

## 常见故障

### 无法找到动态库

确认 runtime library 与产品可执行文件的 staging 位置，或设置 `ZIRCON_RUNTIME_LIBRARY`。相对覆盖路径按产品可执行文件解析，不应依赖不稳定的当前工作目录。检查错误中的 `requested_path` 和 recovery。

### 找到库但入口符号失败

当前只解析 `zircon_runtime_get_api_v8`。出现 V7 或无版本符号意味着制品来自不兼容 BuildSet。重新 stage 匹配的 runtime，不要添加 loader fallback。

### API table rejected

依次检查 pointer 对齐、`abi_version`、精确 `size_bytes`、所有 required slot。可选 surface 三件套需要整体判断，不能因存在其中一个就开启 native present。

### create_session 失败

检查 profile 拼写、项目根是否存在且包含 `zircon-project.toml`、Play 场景是否项目相对，以及 wake sink 的 token/callback 是否成对。若错误报告 retained cleanup，必须在创建线程调用 `zircon_app::retry_runtime_startup_cleanup` 后再重试。

### destroy_session 返回失败

最常见原因是仍有 runtime allocation 未释放，或在 wake callback 内同步销毁。停止新调用，释放所有 `ZrOwnedResultV2`，等待 callback quiescence，然后重试销毁。

### foreign-output session fused

这意味着此前输出发生协议违规：非法 pointer/len/allocation 组合、超预算 JSON、typed item 超限、解码超时或释放失败。读取 `RuntimeForeignOutputState::diagnostic_line()` 和 metrics，定位第一个拒绝；熔断会话不应恢复使用，应有序销毁并重新创建可信 session。

### 世界查询总是 NotModified

确认 cache 使用的是该 session 的最近 generation，而不是跨 session 或 world replacement 保留的 generation。处理 `WorldReplaced` 后清空依赖旧 epoch 的本地事务和缓存。

## 宿主清理顺序

```text
停止提交新事件和操作
  -> 收割已终止 operation；取消未完成 viewport-pick ticket
  -> unwatch + unsubscribe
  -> drain/释放所有 runtime-owned outputs
  -> unbind native surfaces
  -> 等待 wake callback 退出
  -> destroy_session（必要时在释放后重试）
  -> drop LoadedRuntime / unload library
  -> drop ProductComposition
```

具体产品可能在 composition 和动态 session 之间采用不同 owner nesting，但不可违反两条底线：函数 pointer 使用期间动态库必须存活；runtime allocation 释放前 session 和 provider 必须存活。

## 验证边界

本文档是调用契约，不替代测试。ABI 修改至少要覆盖表字段顺序/数量、`#[repr(C)]`、shape validation、required-slot gate、错误 diagnostics 上限、output ownership、重复/错误 session 释放和 fuse 行为。由于本轮仅编写 Wiki，未运行 Cargo；状态来自当前源码及现有测试清单。
