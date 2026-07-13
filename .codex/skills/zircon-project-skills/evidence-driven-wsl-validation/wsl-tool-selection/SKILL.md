# WSL Tool Selection

## Progressive Disclosure Index

- Read this file when choosing or installing tools for debugging and validation in WSL.
- If you still need the documentation requirements for acceptance, go to `../acceptance-and-evidence/SKILL.md`.
- If you need the test placement rules, go to `../plan-tests-under-tests-directory.md`.

## Workflow

1. Confirm WSL is justified.
   - Read `../../prefer-windows-validation/SKILL.md`.
   - State the Linux-specific failure, CI reproduction need, or Linux-only tool requirement.
   - Return to Windows when the same evidence can be collected natively.

2. Allocate mounted-drive outputs.
   - Acquire through the coordinator using the complete repository, `wsl` platform, toolchain, target architecture, workspace, and canonical build-configuration key.
   - Use only the granted mounted equivalent of the D/E/F drive-root `cargo-targets`, `targets`, or `ZirconBuilds` allowlist.
   - Keep the Windows host owner alive while its WSL child runs; direct unleased WSL Cargo is forbidden.
   - Never use an ad-hoc per-Session leaf, `~`, `$HOME`, `/home/<user>`, or the same leaf as a Windows build.

3. Match the tool to the symptom.
   - Crash or wrong branch: `gdb` or `lldb`.
   - Heap corruption, invalid reads or writes, or leaks: `asan`, `lsan`, `valgrind`, `heaptrack`.
   - Undefined behavior, bad casts, signed overflow, invalid shifts: `ubsan`.
   - Concurrency, races, or lock misuse: `helgrind` or `tsan`.
   - Long-running memory growth or allocation hotspots: `heaptrack`.

4. Verify the tool exists before proceeding.
   - Check with commands such as `gdb --version`, `lldb --version`, `valgrind --version`, `heaptrack --version`, or `clang --version`.
   - If a required tool is missing and the environment allows it, install it in WSL with `apt`.

5. Install missing mainstream tools when needed.
   - Typical packages include `gdb`, `lldb`, `valgrind`, `heaptrack`, `clang`, `gcc`, `g++`, `cmake`, and `ninja-build`.
   - Run `sudo apt-get update` before installation when package metadata may be stale.
   - Record the installed package names and version output in the acceptance evidence.

6. Prefer reproducible commands over interactive improvisation.
   - Pre-write `.gdb` or `.lldb` scripts when the session is likely to be repeated.
   - Keep reusable debugger scripts under the relevant `tests/` subtree or the repo skill scripts when they will be used again.

7. Use sanitizer-specific compatibility keys.
   - Put sanitizer flags and tool mode in the canonical build configuration so the coordinator grants a compatible primary pool distinct from normal debug builds.
   - Example patterns:
     - Address and undefined behavior: `-fsanitize=address,undefined -fno-omit-frame-pointer`
     - Leak detection: `-fsanitize=address,leak -fno-omit-frame-pointer`
     - Thread checking: `-fsanitize=thread -fno-omit-frame-pointer`
   - Record the exact configure and test commands used for each sanitizer run.

## Reporting

- State which tool was chosen and why it fit the observed symptom.
- State why WSL was necessary and record the exact mounted target directory.
- State whether the tool was already available or installed during the session.
- State the exact command, binary, input, and result that supplied the evidence.
