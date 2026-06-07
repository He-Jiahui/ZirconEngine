import { stack } from "../../../foundation/layout.js";

export function labColumn(title, children) {
  return stack({ className: "zr-showcase-col", gap: "sm", children: [`<h3 class="zr-col-title">${title}</h3>`, children] });
}
