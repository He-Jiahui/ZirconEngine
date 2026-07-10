# WSL Tool Selection

## Progressive Disclosure Index

- Read this file when choosing or installing tools for debugging and validation in WSL.
- If you still need the documentation requirements for acceptance, go to `../acceptance-and-evidence/SKILL.md`.
- If you need the test placement rules, go to `../plan-tests-under-tests-directory.md`.

## Workflow

1. Match the tool to the symptom.
   - Crash or wrong branch: `gdb` or `lldb`.
   - Heap corruption, invalid reads or writes, or leaks: `asan`, `lsan`, `valgrind`, `heaptrack`.
   - Undefined behavior, bad casts, signed overflow, invalid shifts: `ubsan`.
   - Concurrency, races, or lock misuse: `helgrind` or `tsan`.
   - Long-running memory growth or allocation hotspots: `heaptrack`.

2. Verify the tool exists before proceeding.
   - Check with commands such as `gdb --version`, `lldb --version`, `valgrind --version`, `heaptrack --version`, or `clang --version`.
   - If a required tool is missing and the environment allows it, install it in WSL with `apt`.

3. Install missing mainstream tools when needed.
   - Typical packages include `gdb`, `lldb`, `valgrind`, `heaptrack`, `clang`, `gcc`, `g++`, `cmake`, and `ninja-build`.
   - Run `sudo apt-get update` before installation when package metadata may be stale.
   - Record the installed package names and version output in the acceptance evidence.

4. Prefer reproducible commands over interactive improvisation.
   - Pre-write `.gdb` or `.lldb` scripts when the session is likely to be repeated.
   - Keep reusable debugger scripts under the relevant `tests/` subtree or the repo skill scripts when they will be used again.

5. Use sanitizer-specific build directories.
   - Keep sanitizer builds separate from normal debug builds.
   - Example patterns:
     - Address and undefined behavior: `-fsanitize=address,undefined -fno-omit-frame-pointer`
     - Leak detection: `-fsanitize=address,leak -fno-omit-frame-pointer`
     - Thread checking: `-fsanitize=thread -fno-omit-frame-pointer`
   - Record the exact configure and test commands used for each sanitizer run.

## Reporting

- State which tool was chosen and why it fit the observed symptom.
- State whether the tool was already available or installed during the session.
- State the exact command, binary, input, and result that supplied the evidence.
