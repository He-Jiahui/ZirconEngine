# Icon Scale

The project needs four tiers. Four cover the real application hierarchy without creating a pseudo-precise catalogue of 15--27 logical-pixel values.

| Authored tier | Logical render extent | Use |
| --- | ---: | --- |
| `s` | 16 | Inline text, dense list rows, compact metadata |
| `m` | 20 | Menus, compact controls, field-leading icons |
| `l` | 24 | Standard toolbar/action and icon button |
| `xl` | 32 | Empty states, focal status, prominent feedback |

The logical values are renderer implementation details. In `.zui` and component props use a named value such as `icon_size = "m"`; do not author `17`, `18`, `19`, or another near-duplicate value. A legacy numeric declaration may be preserved only while migrating an untouched owner, then map it to the nearest tier when the owner changes.

Choose by task hierarchy, not by screenshot measurement. If a container is too small for its required tier, fix the flow and control density before shrinking the icon. Only use `xl` when the icon itself carries the region's primary feedback; it is not a generic spacing tool.
