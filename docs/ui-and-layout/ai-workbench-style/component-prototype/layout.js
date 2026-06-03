const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export function cx(...parts) {
  return parts.flatMap((part) => Array.isArray(part) ? part : [part]).filter(Boolean).join(" ");
}

function childHtml(children) {
  return Array.isArray(children) ? children.flat(Infinity).filter(Boolean).join("") : String(children ?? "");
}

function layoutAttrs({ align, justify, direction, gap, wrap, columns } = {}) {
  return [
    align ? `data-zr-align="${esc(align)}"` : "",
    justify ? `data-zr-justify="${esc(justify)}"` : "",
    direction ? `data-zr-direction="${esc(direction)}"` : "",
    gap ? `data-zr-gap="${esc(gap)}"` : "",
    wrap ? 'data-zr-wrap="true"' : "",
    columns ? `data-zr-columns="${esc(columns)}"` : ""
  ].filter(Boolean).join(" ");
}

export function box({ as = "div", className = "", align, justify, direction, gap, wrap, children = "" } = {}) {
  const attrs = [
    `class="${esc(cx("zr-layout", className))}"`,
    layoutAttrs({ align, justify, direction, gap, wrap })
  ].filter(Boolean).join(" ");
  return `<${as} ${attrs}>${childHtml(children)}</${as}>`;
}

export function stack({ as = "div", className = "", align = "stretch", justify = "start", gap = "sm", children = "" } = {}) {
  return box({ as, className: cx("zr-stack", className), align, justify, direction: "column", gap, children });
}

export function cluster({ as = "div", className = "", align = "center", justify = "start", gap = "sm", wrap = false, children = "" } = {}) {
  return box({ as, className: cx("zr-cluster", className), align, justify, direction: "row", gap, wrap, children });
}

export function grid({ as = "div", className = "", align, justify, gap = "sm", columns, children = "" } = {}) {
  const attrs = [
    `class="${esc(cx("zr-grid", className))}"`,
    layoutAttrs({ align, justify, gap, columns })
  ].filter(Boolean).join(" ");
  return `<${as} ${attrs}>${childHtml(children)}</${as}>`;
}
