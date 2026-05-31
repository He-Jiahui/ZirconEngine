# Hub Design Board Structure Geometry Evidence

This evidence records the browser-measured geometry of the primary overall
structure board. It supports manual screenshot review by making the shell
ownership boundaries explicit.

## Measurement Source

- Source page: `docs/ui-and-layout/hub-design-board/index.html?board=structure`
- Exported screenshot: `docs/ui-and-layout/hub-design-structure-layout.png`
- Machine baseline: `docs/ui-and-layout/hub-design-board/structure-geometry-baseline.json`
- Canvas: `1568x1003`
- Measured in Microsoft Edge with viewport `1568x1003`

## Primary Shell Geometry

All values below are rounded CSS pixels relative to `.hub-frame`.

| Region | Left | Top | Width | Height | Right | Bottom |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Hub frame | 0 | 0 | 1100 | 612 | 1100 | 612 |
| Dimension strip | n/a | n/a | n/a | 32 | n/a | n/a |
| Topbar | 1 | 1 | 1098 | 58 | 1099 | 59 |
| Sidebar | 1 | 59 | 178 | 552 | 179 | 611 |
| Workspace | 179 | 59 | 920 | 459 | 1099 | 518 |
| Bottom strip | 179 | 518 | 920 | 93 | 1099 | 611 |
| Source Engine overlay | 253 | 63 | 245 | 118 | 498 | 181 |
| Account overlay | 841 | 63 | 190 | 136 | 1031 | 199 |

## Structure Assertions

- Topbar spans the full Hub frame width.
- Sidebar starts immediately below Topbar and owns the full left column down to
  the frame bottom.
- Workspace starts immediately right of Sidebar and remains above Bottom strip.
- Bottom strip starts at the Workspace left edge and owns the lower shell band.
- Source Engine and Account overlays float over the header/workspace area, not
  inside Sidebar, and do not push Workspace or Bottom strip.

## Review Use

Use this file after opening `hub-design-structure-layout.png`. First check that
the visible proportions match the table, then use
`STRUCTURE_COVERAGE_MATRIX.md` to decide which supplemental artifact clarifies a
specific review item. Functional-content details remain secondary evidence.
