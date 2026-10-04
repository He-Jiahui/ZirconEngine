# WSL Tool Selection

## Progressive Disclosure Index

- Read this file when choosing or installing tools for debugging and validation in WSL.
- If you still need the documentation requirements for acceptance, go to `../acceptance-and-evidence/index.md`.

## Workflow

1. Confirm WSL is justified.
   - Read `../../prefer-windows-validation/guide.md`.
   - State the Linux-specific failure, CI reproduction need, or Linux-only tool requirement.
   - Return to Windows when the same evidence can be collected natively.

2. Allocate mounted-drive outputs.
   - Use `python3 -B -m tools.dev.local_cargo` with outputs physically below `/mnt/d/cargo-targets`, `/mnt/e/cargo-targets`, or `/mnt/f/cargo-targets`.
   - Record the repository, Linux toolchain, target architecture, profile and features. Keep incompatible configurations and Windows output in separate leaves.
   - Keep the invoking owner alive while its WSL child runs and preserve Cargo's native locks. The retired coordinator's registration and leases are not prerequisites.
   - Never place compiler outputs or caches under `~`, `$HOME`, `/home/<user>`, a repository-local target, or a path alias.

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
   - Record sanitizer flags and tool mode and select a physically checked output leaf distinct from normal debug builds.
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
