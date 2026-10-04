# Modularize Large Files

Around 1000 lines is a design warning, not a hard limit or an approval trigger. Inspect responsibilities when a touched file receives substantial new logic.

- Split when the change adds an independent behavior family or exposes mixed ownership.
- Follow [module boundary discipline](../zr-module-boundary-discipline/guide.md) for the actual decomposition rules; move coherent types, behavior, callers, and tests together.
- Prefer semantic names and narrow interfaces. Avoid numbered parts and generic helper buckets.
- Generated files, vendored code, cohesive tables, and single-purpose files may remain large. A bounded fix does not require unrelated restructuring.
- If a split needed by the current change is deferred, explain the concrete constraint and affected boundary. Routine exceptions do not require a separate permission request.
- Verify the affected paths and behavior using the repository validation cadence.

Use a line count to locate a potential problem; use responsibility and change scope to decide whether to refactor.
