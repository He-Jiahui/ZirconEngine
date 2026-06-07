import { icon } from "../../foundation/icons.js";

export function toast() {
  return `<div class="zr-toast"><span class="zr-toast-status">${icon("check")}</span><span>Operation completed successfully</span><strong>UNDO</strong>${icon("x")}</div>`;
}
