# Plan Tests Under Tests Directory

- Put test code in the closest existing domain subtree under `tests/` whenever possible, such as `tests/parser`, `tests/instructions`, `tests/function`, `tests/module`, `tests/gc`, `tests/projects`, or `tests/scripts`.
- If no existing subtree cleanly owns the behavior, create a new focused `tests/<domain>/` directory and wire it into `tests/CMakeLists.txt`.
- Keep debugger scripts near the test domain they support, for example a `.gdb` script beside the owning test source or project fixture.
- Put source-language fixtures in the owning test subtree or in `tests/projects` when they represent end-to-end runnable programs.
- Treat `tests/scripts/test_cases/` and `tests/projects/` as integration-style fixture areas, not as substitutes for focused unit or subsystem coverage.
- Add a detailed acceptance document under `tests/acceptance/<feature-or-milestone>.md` whenever the work changes behavior, closes a bug, or claims milestone completion.
- The acceptance document must include:
  - scope and owning layers
  - baseline before changes
  - complete test inventory
  - boundary and failure cases
  - tool matrix and command lines
  - results and acceptance decision
- Do not leave a feature with only code changes and no test or acceptance documentation trail.
