# Sound M1.1 Kira 依赖与 canonical lock closure

Plan: docs/plans/zircon_plugins/02-sound.md
Milestone: M1.1
Status: accepted
Files: ["Cargo.lock", "zircon_plugins/Cargo.lock", "zircon_plugins/sound/runtime/Cargo.toml"]

## Scope Delivered

- `zircon_plugins/sound/runtime/Cargo.toml` 从 optional `cpal = "0.15"` 与 `cpal-backend` feature 硬切到 `kira = "0.12.2"`。
- 根工作区与 `zircon_plugins` 工作区 canonical lockfile 均绑定唯一 Kira 0.12.2、CPAL 0.18.1 及其跨平台依赖闭包；Sound package entry 在两锁中均依赖 `kira`。
- 该接受范围只关闭 M1.1 的依赖与锁文件契约，不关闭父 M1 的 KiraEngine 生命周期、Mixer Graph 编译、播放控制或图 diff/Tween 热更。

## Fresh Testing Evidence

- Windows canonical rustc 1.94.1 managed job `a0f2c2945eb7431fab6ff12650f41266`：`cargo +1.94.1 metadata --locked --format-version 1`，exit 0。
- Windows canonical rustc 1.94.1 managed job `736bb82d6ace4c5aa55ce6595f269c3c`：`cargo +1.94.1 metadata --manifest-path zircon_plugins/Cargo.toml --locked --format-version 1`，exit 0。
- Render01 launch-window job `d57cb5f09ee24b9685c36e63fd445457` 在根锁 SHA-256 `FED7DA1BF408C9FD58D37768ECC4F92CF72571B51E957C40ADE880CC460822A5` 下执行 `--locked` directional parity，1/1 passed、8178 filtered、exit 0；根锁在整个运行窗口保持不变。
- Sound package managed job `63a9a9d2f7794d488b61b530e5436fd2` / run `7c05366c20554ea7a1a8f9ae68fb5faa` 已在相同双 lockfile 下完成 `--locked` 依赖解析并编译到 `zircon_plugin_sound_runtime`；随后因父 M1 尚未完成的 Kira 0.12.2 API 接线以 18 个 Sound 源码错误终止。该结果证明 M1.1 lock closure 不再阻断 Cargo，同时保留父 M1 源码验收为未完成。
- 最终 SHA-256：根 `Cargo.lock` 为 `FED7DA1BF408C9FD58D37768ECC4F92CF72571B51E957C40ADE880CC460822A5`，`zircon_plugins/Cargo.lock` 为 `9CF31E50ABCC41EC77EDBEA7A18E37950595A1442224821C482CB7D95C202169`，Sound manifest 为 `72A8A1E86301871183C088DF8278C013E8BE6904616436524913201818D36234`。
- Exact scope `git diff --check` 通过，共 3 files changed、683 insertions、395 deletions；共享 Git index staged count 为 0。

## Review

- 独立 reviewer Session `plugins02-sound-m1-kira-lock-review-20260717`：Critical 0、Important 0。
- Root lock 为 +31/-13 packages，plugin lock 为 +31/-25 packages；新增包全部属于 Kira 闭包，删除包全部属于旧 CPAL 0.15 闭包，静态 lock dependency reference unresolved count 为 0，未发现无关升级或错误删除。
