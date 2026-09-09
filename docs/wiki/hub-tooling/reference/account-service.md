---
related_code:
  - zircon_hub/src/account/mod.rs
  - zircon_hub/src/account/config.rs
  - zircon_hub/src/account/credential.rs
  - zircon_hub/src/account/service.rs
  - zircon_hub/src/service/mod.rs
  - zircon_hub/src/service/storage/mod.rs
implementation_files:
  - zircon_hub/src/account
  - zircon_hub/src/service
plan_sources:
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
tests:
  - zircon_hub/src/account/config.rs
  - zircon_hub/src/account/mod.rs
  - zircon_hub/tests/service_authority_contract.rs
doc_type: api-reference
---

# 账户、凭据与本地服务

账户模块把 OIDC 配置、浏览器登录、凭据存储、操作取消和服务请求组合成 `AccountBroker`。本地 service 模块提供 catalog、cloud、identity、organization、storage 等服务边界。Hub UI 只能看到 `AccountView`/snapshot，不应直接读取 refresh token 或数据库连接。

## 配置契约

```rust
pub struct AccountConfig {
    pub issuer: String,
    pub client_id: String,
    pub service_url: String,
    pub callback_port: u16,
    pub allow_loopback_http: bool,
    pub operation_journal_path: Option<PathBuf>,
}

impl AccountConfig {
    pub fn load(path: &Path) -> Result<Self, AccountError>;
    pub fn endpoint(&self, value: &str) -> Result<Url, AccountError>;
    pub fn redirect(&self) -> String;
    pub fn journal_path(&self) -> Result<PathBuf, AccountError>;
}
```

配置从不超过 65536 字节的常规文件读取，并拒绝未知 JSON 字段。issuer/service URL 必须是 HTTPS；只有 `allow_loopback_http=true` 时，127.0.0.1 等 loopback HTTP 才被允许。URL 不得带用户名、密码、query 或 fragment。callback port 必须不低于 1024。

```json
{
  "issuer": "https://identity.example/realm",
  "client_id": "zircon-hub",
  "service_url": "https://service.example",
  "callback_port": 8480,
  "allow_loopback_http": false
}
```

`redirect()` 生成 `http://127.0.0.1:<port>/callback`。操作 journal 是非 secret 状态，默认位于 Windows `LOCALAPPDATA/ZirconHub/Account/operations.dat`；自定义路径必须是绝对路径、带文件名且不能包含 parent-dir 组件。

## Broker 与凭据

```rust
pub struct AccountView { /* configured, status, subject, operation fields */ }
pub struct AccountBroker { /* config, CredentialStore, cancellation */ }

impl AccountBroker {
    pub fn new(config: AccountConfig) -> Result<Self, AccountError>;
    pub async fn view(&self) -> AccountView;
    // login, logout and organization calls are async source-owned methods
}

pub trait CredentialStore: Send + Sync {
    // load/save/clear/revoke methods are source-owned
}
pub struct WindowsCredentials;
impl WindowsCredentials {
    pub fn new(issuer: &str, client: &str) -> Result<Self, AccountError>;
}
```

`SavedSession` 包含 issuer、subject、nonce、refresh_token 和 revoke-only 状态；它只能存在于 CredentialStore 内。`AccountView` 序列化前必须确认不包含 nonce/refresh_token。logout 即使身份提供商不可用，也应清理本地凭据，并把无法完成的撤销记录为 pending。

## 状态和错误

`AccountError` 覆盖 configuration、network、cancelled、operation store、revocation pending 等类别。调用方应区分“已登出但撤销待重试”和“仍然登录”；不能因为远端 down 就继续把旧 token 暴露给 UI。

```text
load AccountConfig
  -> AccountBroker::new
  -> CredentialStore load
  -> OIDC browser callback
  -> AccountView
  -> ServiceRequest with scoped credential
```

## 本地 service

`service::run` 是本地服务进程入口；`Database::open(path)` 打开迁移数据库，`Database::memory()` 供测试使用。服务层使用 receipt、path guard、job admission 和授权检查；HTTP/cloud/catalog API 的公共类型不能绕过这些门控直接操作 blob 或数据库。

```rust
let database = Database::open(storage_path)?;
// Service lifecycle owns migrations, supervisor and shutdown.
```

服务请求应有幂等 operation ID。storage receipt 的 `validate_id`、`fingerprint`、`replay`、`commit`、`lookup` 用于防止重复执行和结果不一致。

## 安全实践

- 生产 issuer/service 只用 HTTPS。
- 不把 refresh token 放进日志、Tauri snapshot、action history 或命令行。
- Windows 凭据使用系统 credential store，不写普通 TOML/JSON。
- journal 只写非 secret operation 状态，并限制绝对路径。
- local service 端点要做 loopback/来源/授权检查。
- 失败重试要有退避和 revoke-pending 状态，不能无限重放。

## 与 Unreal/Godot 的对照

Unreal Launcher 常把用户账户和项目服务集成在 Launcher 层；Godot 编辑器更少依赖集中式账户。Zircon 把账户 broker 与本地 service 分离，使离线项目管理可工作，同时让云 catalog、组织和授权成为显式能力。

## 恢复流程

1. 配置错误：修复 JSON/HTTPS/端口后重新 `AccountConfig::load`。
2. callback 失败：确认 loopback 端口未占用且浏览器回调只在本机监听。
3. token 过期：走 broker refresh/login，不让前端自行刷新。
4. logout 远端不可用：保留 revoke-pending，继续显示 signed-out。
5. service 数据库损坏：停止 admission，备份数据库，执行受控迁移/恢复。

## 测试要求

配置测试覆盖字节上限、常规文件、URL 限制、journal 路径和 loopback HTTP。账户测试覆盖并发操作取消、logout 清理凭据和 pending revoke。service 合同测试覆盖 authority、幂等 receipt、path guard 和 shutdown。

## 相关页面

- [Hub 状态与命令](hub-state-commands.md)
- [服务与账户总览](../zircon-hub.md)
- [发布 Receipt](packages-receipts.md)
