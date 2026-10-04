# Codex file permissions

## Canonical configuration

`E:\Git\ZirconEngine\.codex\config.toml` defines the `zircon-strict` permission
profile and the sole maintained write allowlist. It extends `:read-only` and
grants writes only to the explicitly listed source, documentation, fixture,
skill, session, state and output paths. The checkout root is read-only; its
editable configuration files are granted individually.

Directory grants include descendants. A new file inside a granted directory
is permitted; a new sibling directory or unlisted root file is not. Reading an
external input never grants permission to move, overwrite or delete it.

Compiler products, compiler caches, scratch, exports and generated evidence use
the registered `zircon-local` namespaces under the physical drive-root
`D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets`. Source code in
`zircon_runtime\build` remains source, not a compiler output directory.
The existing Cargo wrapper retains physical-path checks and native locking.
The policy does not authorize clearing retired coordinator data or restoring
its services, registration, leases or validation queue.

Shell environment values route temporary, Cargo, Python, pip, npm and XDG
outputs into the E: namespace. Other tools must use their explicit output
options. A write denial is a reason to choose a permitted destination; it does
not authorize adding a grant or running the command without confinement.

## Activation

1. Open ZirconEngine as a trusted project. Codex loads project configuration
   only for trusted projects.
2. Select **zircon-strict** in the desktop permissions menu. Existing chats
   that already selected Full Access can retain that selection; the saved
   default does not retroactively replace an active turn's execution policy.
3. Use native Windows `elevated` sandboxing. If setup reports a problem, run
   the official sandbox setup and complete any Windows administrator prompt.
   Do not erase credentials, rotate sandbox accounts manually or weaken the
   profile as a workaround.
4. Verify real operations with the command below. Configuration parsing, profile
   discovery and `windowsSandbox/readiness = ready` are separate from passing
   runtime evidence.

The user configuration at `D:\CodexData\.codex\config.toml` replaces its old
`sandbox_mode = "danger-full-access"` with the equivalent permission-profile
default for other projects. This removes the legacy setting that otherwise
overrides project permission profiles. ZirconEngine selects `zircon-strict`
through its own project configuration.

## Tool coverage

The profile controls sandboxed local commands and their child processes. MCP
servers and app tools use separate execution boundaries, so this checkout
disables its configured RenderDoc and CUA MCP servers and default app access.
Command networking is disabled to avoid dispatching writes through unconstrained
local or remote services. Re-enabling a separate writer requires an explicitly
authorized, verified confinement design.

The configuration itself, hooks, AGENTS.md, Git internals, reference repositories,
nested worktrees, retired coordinator data, user directories and system temp
have no write grant. A Codex host's own state, logs and sandbox provisioning are
outside the command profile; this profile does not claim to confine every
background write by Windows or the desktop host.

These are project defaults, not an administrator-enforced device policy.
Explicitly selecting another profile or supplying a command-line override
changes that invocation's permissions. The working agreement prohibits using
that route to bypass this project's allowlist.

## Runtime verification

```powershell
& 'E:\Git\ZirconEngine\.jenkins\runtime\python\python.exe' -B -X utf8 `
  'E:\Git\ZirconEngine\tools\permissions\verify_codex_file_permissions.py' `
  --codex 'C:\Users\HeJiahui\AppData\Local\OpenAI\Codex\bin\8aaf1547b825b104\codex.exe'
```

The tool uses the installed CLI's `sandbox --permission-profile zircon-strict`
interface, includes managed requirements, and stores owned canaries and receipts
under `E:\cargo-targets\zircon-local\codex-permissions`.

It checks permitted create/modify/copy/rename/delete operations, denied root and
external writes, denied copy/move, preserved move sources, protected configuration
access, inherited subprocess restrictions and writes through a directory junction.
The parent verifier also checks forbidden destinations for residual output and
confirms that the permission configuration did not change. An interpreter failure,
timeout, missing directory or failed positive probe is a failed verification, not
proof of a successful denial. Negative probes must receive actual access denials.

The configured profile passed all 15 checks with Codex 0.160.0 and native Windows
`elevated` sandboxing. The final receipt records no stderr, no forbidden output
and an unchanged configuration:
[runtime receipt](E:/cargo-targets/zircon-local/codex-permissions/probe-6151a82962894e9d9dea183eb2e5b764/receipt.json).
This verifies the tested sandbox invocation; the desktop chat still requires the
profile selection described above.

Re-run after changing the allowlist, Codex version, native sandbox mode or
relevant Windows permissions. This is permission acceptance evidence, not Rust
build, Jenkins migration or whole-workspace acceptance.

## Official references

- [Permission profiles](https://learn.chatgpt.com/docs/permissions)
- [Windows sandbox](https://learn.chatgpt.com/docs/windows/windows-sandbox)
- [Configuration precedence](https://learn.chatgpt.com/docs/config-file/config-basic)
