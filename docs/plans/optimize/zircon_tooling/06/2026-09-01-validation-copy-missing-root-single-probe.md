# Validation copy missing root single probe

## Change

Validation-copy missing-root recovery now uses one `lstat` probe per scan phase
to distinguish an absent path from any existing file, directory, or dangling
symlink. The previous path called both `exists()` and `is_symlink()` during the
initial scan and again under the running lock. The two-phase TOCTOU recheck and
cleanup-root policy validation remain unchanged; non-not-found OS errors remain
fail-closed as existing paths.

## Performance evidence

Real Windows filesystem benchmark with 1,000 absent paths and both recovery scan
phases, 41 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 73,579,900 ns | 53,565,500 ns | 27.201% lower, 1.37x |
| p95 | 94,215,100 ns | 78,299,400 ns | 16.893% lower, 1.20x |
| metadata probes | 4,000 | 2,000 | 50.000% lower |

## Validation

Six focused missing-root recovery and race tests passed 6/6 in 79.819 seconds,
covering terminal-copy convergence, running/local reservations, and root
recreation after lock and mutation-gate transitions. The static performance
contract, Python compilation, and scoped `git diff --check` passed. Complete
workspace-copy coverage remains in the accumulated asynchronous coordinator
validation batch.
