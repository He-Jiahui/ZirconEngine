# 开发工具

根目录只保留 `README.md` 和 `.gitignore`，所有脚本和实现按职能放在子目录中。

## 目录结构

```
tools/
  build/          构建脚本：编辑器打包、引擎/Hub/插件构建
  dev/            本地开发入口：Cargo 命令、开发会话、快速构建别名
  audits/         规范与质量检查：约定检查、源码审计、插件结构审计
  docs/           Wiki 源校验、导航钩子与网站构建
  setup/          安装与注册：Jenkins 托盘登录启动项、Codex session hook
  jenkins/        Jenkins 全部实现：执行引擎、托盘(tray/)、封存输入(pilot/)、
                  Gap 缺口追踪(gap/)、独立启动器(launcher/)
  analysis/       数据分析与验证：性能(performance/)、性能采集(profiling/)、
                  验证批次(validation/)、视觉检查(visual/)
  maintenance/    维护工具：清理旧 target、Windows 路径解析模块
  mvp/            MVP 产品输入与验收流程
  export/         项目导出实现
  cargo/          Cargo 子命令源码
  workbench/      编辑器 Workbench 预览（Node.js）
  penpot/         ZUI 设计插件、锁定依赖及独立验证入口
  tests/          工具回归测试
```

## 常用入口

| 用途 | 入口 |
| --- | --- |
| 本地 Cargo 命令 | `dev/local-cargo.ps1` 或 `dev/local_cargo.py` |
| 编辑器构建与打包 | `build/build-editor.ps1` |
| 引擎/Hub/插件构建 | `build/zircon_build.py` |
| 仓库规范检查 | `audits/check-conventions.ps1` |
| Jenkins 托盘启动 | `jenkins/zircon-jenkins-start.py` 或打包后的 `dist/zircon-jenkins.exe` |
| Jenkins Gap 工作流 | `python -m tools.jenkins.gap <command>` |
| 构建状态仪表盘 | `python tools/jenkins/dashboard.py --open` |

## 约定

- 一次性迁移和统计脚本放在 `.codex/outbox/`，不放工具目录。
- Python 工具使用 `python -B` 运行，避免生成字节码缓存。
- 编译产物和编译缓存必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下；本地命令使用 `zircon-local` 命名空间。
