# Rust 测试目录迁移记录

2026-10-04：仓库自有 Rust 测试已迁入名称为 `tests` 的目录，覆盖引擎、编辑器、Hub、插件、工具、模板和示例。第三方及 `dev` 参考仓库不在迁移范围内。

迁移了 3,042 个内联测试模块、2,157 个既有测试／夹具文件，以及 20 个独立测试函数。扫描 24,052 个 Rust 文件后，测试属性数量仍为 34,151；普通源码中没有测试用例或内联测试模块。9 个未接入 Cargo 的 virtual-geometry 测试快照仅迁移位置，保持原来的未接入状态。

单元测试保留原模块名、条件编译、私有访问和测试名称，通过 `#[cfg(test)]` 与 `#[path = "tests/…"]` 加载。原来直接位于父模块的 20 个测试函数使用条件 `include!`，保持原命名空间。构建脚本测试放在 `tests/unit`，避免增加 Cargo 自动发现的集成测试目标。必要的测试状态钩子仍保留在被测类型中，没有扩大公开接口。

同步更新了测试文件引用、相对资源路径以及读取测试源码的 Python／Rust 结构断言。业务文件与测试文件分别读取，既有断言保留；历史记录仍保留当时的路径。当前目录约定和 CI 已加入迁移工具回归及目录检查。

## 验证结果

| 检查 | 结果 |
| --- | --- |
| Rust 测试目录检查 | 24,052 个文件，零违规 |
| 测试定义数量 | 34,151 → 34,151 |
| 迁移工具回归 | 10 项通过，覆盖原始字符串、条件编译、资源路径和子模块查找 |
| Rust 语法 | 初次 9,794 个相关文件通过；后续修改的 100 个文件通过 |
| 模块查找 | 182 个包入口全部通过 rustfmt 模块遍历 |
| 代码空白检查 | 9,967 个相关路径，按 Windows CRLF 检查并修复 EOF 空行后通过 |
| 独立 Hub 结构测试 | 9 个文件使用 MSVC 环境编译成功；15 个相关测试中 10 项通过，5 项仍失败 |
| Python 结构测试 | 316 个模块、1,349 项测试；仍有 24 项失败和 4 项错误。最后修改的 3 个模块重新执行，20 项全部通过 |

Hub 剩余失败涉及缺失的 `editor_recent_sync.rs`、缺失的旧 Hub 文档，以及现有 action 路由约定差异。Python 剩余问题涉及缺失文档／视觉工具，以及布局、颜色、生命周期、输入接口和字体等源码约定差异。失败日志保留，没有删减断言来取得通过结果。

## Cargo 验证仍待执行

通过 `tools/dev/local-cargo.ps1 -IndependentPreview` 尝试了：

```text
check -p zircon_runtime_interface -p zircon_reflect_derive -p zr_math --tests --locked
```

代理 `127.0.0.1:7897` 无法连接；离线尝试缺少 `image` 缓存；直接连接遇到 Schannel `SEC_E_NO_CREDENTIALS`。检查均在依赖解析阶段停止，没有获得完整 Cargo 编译或测试通过的证据。依赖下载环境恢复后需补跑上述检查和各受影响包的测试。

编译和临时产物位于 `E:/cargo-targets/zircon-local/win32/rust-test-separation/target`。独立检查不代表 Jenkins 或里程碑验收。

## 记录与复查

迁移路径、源码哈希、计数和验证输出位于 `.codex/outbox/rust-tests-separation/`。`receipt.json`、`prepared-before-resume.json`、`unwired-receipt.json` 和 `standalone-cases.json` 保留各阶段记录；语法、模块图、独立 Rust 检查和 Python 检查使用各自的 JSON 记录，没有保存整份源码备份。

```powershell
python -B -m unittest tools.tests.test_separate_rust_tests -v
python -B tools/audits/separate_rust_tests.py --check
```
