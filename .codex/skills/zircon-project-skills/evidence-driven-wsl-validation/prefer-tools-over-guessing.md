# Prefer Tools Over Guessing

- Start from observed symptoms, not from a preferred fix.
- Choose a debugging or validation tool that can confirm or falsify the current hypothesis.
- Use `gdb` or `lldb` for crashes, bad control flow, unexpected state transitions, and call-path verification.
- Use `valgrind` for invalid memory access and leak investigations when sanitizer builds are not enough.
- Use `helgrind` from Valgrind for thread-safety analysis when concurrency is relevant. If the user says "halgrind", treat that as `helgrind`.
- Use `heaptrack` when allocation growth, ownership churn, or memory hotspots need attribution.
- Use `asan`, `ubsan`, `lsan`, or `tsan` builds when compiler instrumentation can expose the fault faster or more precisely.
- Capture the evidence that drove the conclusion: command, binary, input case, observed output, and why that output supports the diagnosis.
- Reject statements such as "this looks correct", "probably fixed", or "should be fine now" unless backed by a concrete test or tool result.
