# Runtime 08 ECS kernel filters current result

Date: 2026-07-10

Status: in progress

## Results

| Filter | Available binary | Current-source follow-up |
|---|---:|---|
| `entity` | 78 passed / 3 failed | two owned guards 2/2; one external render behavior failure remains |
| `observer` | 16 passed / 1 failed | stale naming guard 1/1 |
| `command` | 144 passed / 5 failed | four owned guards 4/4; render GPU-context 800-line budget remains |
| `change_tick` | 4/4 | complete for this binary |
| `messages` | 24/24 | complete for this binary |
| `ecs` | 330 passed / 10 failed | all ten old failures covered by current owner-tree/kernel/naming/performance evidence |

The `ecs` current-source follow-up comprises owner-tree 3/3, kernel split 1/1, naming M2 44/44 aggregate evidence, and Runtime 07 ECS/extract guards 2/2.

Fresh default-feature filter reruns are required before `entity`, `observer`, `command`, or `ecs` can be promoted. The strict render file-budget failure is retained as an open structure issue.
