---
related_code:
  - .github/workflows/wiki-pages.yml
  - mkdocs.yml
  - requirements-docs.txt
  - tools/wiki_site.py
  - tools/wiki_mkdocs_hook.py
  - docs/wiki/navigation.yaml
implementation_files:
  - .github/workflows/wiki-pages.yml
  - mkdocs.yml
  - requirements-docs.txt
  - tools/wiki_site.py
  - tools/wiki_mkdocs_hook.py
plan_sources:
  - user: 2026-09-09 搭建 GitHub Pages Wiki 自动化流程
  - docs/plans/mvp/index.md
tests:
  - .github/workflows/wiki-pages.yml
  - tools/wiki_site.py
  - docs/wiki/navigation.yaml
doc_type: workflow-detail
---

# GitHub Pages Wiki 发布

本页说明如何把 `docs/wiki` 发布为 GitHub Pages 站点。仓库把 Markdown 页面和 `navigation.yaml` 作为唯一内容源，把校验、静态生成和部署拆成三个明确阶段：

```text
docs/wiki + navigation.yaml
        |
        v
tools/wiki_site.py validate
        |
        v
MkDocs + Material + wiki_mkdocs_hook.py
        |
        v
site/ -> upload-pages-artifact -> deploy-pages -> github-pages
```

站点地址默认是 `https://he-jiahui.github.io/ZirconEngine/`。正式运行时，`actions/configure-pages` 提供仓库实际的 Pages 基础 URL，构建钩子会覆盖本地默认值，因此项目页和用户页都能正确处理子路径。

## 发布触发矩阵

| 事件 | 构建与校验 | 部署 |
| --- | --- | --- |
| `push` 到 `main`，且修改命中文档路径 | 是 | 是 |
| `pull_request`，且修改命中文档路径 | 是 | 否 |
| `workflow_dispatch`（默认分支） | 是 | 是 |
| `workflow_dispatch`（其他分支） | 是 | 否 |

工作流文件是 `.github/workflows/wiki-pages.yml`。路径过滤覆盖 Wiki 内容、MkDocs 配置、依赖清单、构建脚本和工作流自身；只修改 Rust 源码不会无谓触发文档发布。
同一 Pull Request 的旧校验会被新提交取消；`main` 分支的构建和部署不会被取消，以免正在进行的生产发布被中断。

## 一次性 GitHub 配置

在仓库的 **Settings > Pages > Build and deployment > Source** 选择 **GitHub Actions**。这一步让仓库接受工作流上传的 Pages artifact，而不是寻找 `gh-pages` 分支。官方流程、权限和环境说明见 [GitHub Pages 自定义工作流文档](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)。

首次部署会使用名为 `github-pages` 的 environment。可以在 **Settings > Environments** 为它增加审批人、分支限制或部署保护规则；工作流已经在部署 job 中声明该 environment，并只授予 `pages: write` 和 `id-token: write` 所需权限。

若仓库启用了组织级 Actions 限制，需要允许以下官方 action：

- `actions/checkout@v5`
- `actions/setup-python@v5`
- `actions/configure-pages@v5`
- `actions/upload-pages-artifact@v4`
- `actions/deploy-pages@v4`

不需要配置个人 token、云存储密钥或 `gh-pages` 分支；部署使用 GitHub Actions 自动生成的短期 `GITHUB_TOKEN`。

## 本地预览

建议使用仓库外部或被忽略的 `.wiki-venv`，避免污染源码树。PowerShell 示例：

```powershell
python -m venv .wiki-venv
.\.wiki-venv\Scripts\python.exe -m pip install --disable-pip-version-check -r requirements-docs.txt
.\.wiki-venv\Scripts\python.exe tools/wiki_site.py validate --json
.\.wiki-venv\Scripts\python.exe tools/wiki_site.py build --output site --json
.\.wiki-venv\Scripts\python.exe tools/wiki_site.py serve --dev-addr 127.0.0.1:8000
```

`validate` 不依赖 MkDocs 运行时，只检查 frontmatter、链接、代码围栏、导航覆盖率和声明的源码路径。代码、测试或计划可能与文档分开提交，因此当前 checkout 中暂时不存在的元数据目标默认记为 warning；需要完整源码快照时可加 `--strict-metadata` 让它们成为错误。`build` 会先执行同一校验，再用 `mkdocs build --strict --clean` 生成 `site/`；未找到的站内页面、导航项或锚点仍会使命令失败。`serve` 启动 MkDocs 开发服务器，修改 Markdown 后会自动刷新。

## 构建阶段

### 1. 依赖安装

`requirements-docs.txt` 固定 MkDocs、Material、PyYAML 和 pymdown 扩展的直接版本。CI 使用 Python 3.12，并启用 pip 缓存；升级版本时应在本地重新执行完整 `validate` 和 `build`，确认主题、钩子和严格链接校验仍然通过。

### 2. 内容校验

`tools/wiki_site.py validate` 将 `navigation.yaml` 展平后与实际 Markdown 文件集合做双向比较：遗漏页面和不存在页面都会报错。每页 frontmatter 必须按 `related_code`、`implementation_files`、`plan_sources`、`tests`、`doc_type` 顺序出现；声明路径必须是仓库内的安全相对路径。外部 URL 和 `user:` 记录不检查存在性；源码、测试或计划路径不在当前 checkout 时输出 warning，完整快照门禁使用 `--strict-metadata`。

### 3. MkDocs 生成

`tools/wiki_mkdocs_hook.py` 在 MkDocs 加载配置时把仓库自己的导航格式转换为 MkDocs `nav`，并在 Pages 构建中注入 `base_url`。钩子还把 `navigation.yaml` 复制到站点根目录，保留机器可读的导航清单，便于后续 Wiki 前端或索引工具消费。

## 部署阶段

构建 job 以 `contents: read` 和 `pages: read` 读取源码及 Pages 元数据，然后将 `site/` 上传为 Pages artifact。站点根目录同时包含机器可读的 `navigation.yaml` 和 `wiki-build-info.json`；后者记录站点地址、导航版本和本次构建的 `github.sha`（本地构建没有 revision 时为 `null`）。部署 job 等待构建成功后调用 `actions/deploy-pages`，并把部署地址写入 `github-pages` environment。Pull Request 只执行校验和构建，不获得部署权限；来自 fork 的请求也不会读取仓库 secrets。

成功发布后，打开 **Actions > Publish ZirconEngine Wiki** 可以看到构建日志和部署 URL。页面缓存或搜索索引更新可能需要短暂时间；先确认 workflow 和 environment 均显示成功，再判断站点内容是否异常。

## 新增或修改页面

1. 在 `docs/wiki` 下创建带标准 frontmatter 的 Markdown 页面。
2. 在 `docs/wiki/navigation.yaml` 的合适 section 中增加唯一 `slug`、相对 `path` 和显示标题。
3. Wiki 内部页面使用相对于当前 Markdown 文件的 `.md` 链接；指向仓库源码、测试目录或计划文件的链接使用 GitHub `blob/main` 或 `tree/main` URL，避免被静态站点误认为页面。
4. 本地执行 `validate` 和 `build --strict`，再提交变更。

导航清单是人工维护的稳定接口，不要在 MkDocs 配置中复制第二份页面树。页面组织、frontmatter 和状态标签的完整约定见[贡献文档](../contributing-docs.md)。

## 故障排查

| 现象 | 检查项 |
| --- | --- |
| `PyYAML is required` | 使用工作流同一份 `requirements-docs.txt` 安装依赖，并确认命令调用的是虚拟环境 Python。 |
| `navigation.yaml omits Markdown page` | 页面已创建但未加入导航；补齐 `path`、`slug` 和 `title`。 |
| MkDocs 报 `target not found` | 检查站内 `.md` 链接的相对路径；仓库源码链接改为 GitHub URL，不要关闭 strict 校验。 |
| PR 通过但没有部署 | 这是预期行为；只有 `main` push 或默认分支手工运行才会进入 deploy job。 |
| Pages 显示 404 | 确认 Settings > Pages 的 Source 是 GitHub Actions，并检查 `github-pages` environment 最近一次部署记录。 |
| 页面样式或子路径错误 | 检查 `WIKI_SITE_URL` 是否由 `configure-pages` 输出，并避免在 Markdown 中硬编码站点根绝对路径。 |

## 回滚与审计

部署内容完全由 `main` 分支提交重建。要回滚到已知版本，可在默认分支上恢复对应文档提交后重新运行工作流；不应直接编辑 Pages artifact 或手工推送 `gh-pages`。站点根目录的 `wiki-build-info.json` 记录 `github.sha`，可据此把线上页面与源码快照对应起来。

相关实现入口：[发布工作流](https://github.com/He-Jiahui/ZirconEngine/blob/main/.github/workflows/wiki-pages.yml)、[MkDocs 配置](https://github.com/He-Jiahui/ZirconEngine/blob/main/mkdocs.yml)、[Wiki 校验与构建脚本](https://github.com/He-Jiahui/ZirconEngine/blob/main/tools/wiki_site.py)、[MkDocs 导航钩子](https://github.com/He-Jiahui/ZirconEngine/blob/main/tools/wiki_mkdocs_hook.py)。
