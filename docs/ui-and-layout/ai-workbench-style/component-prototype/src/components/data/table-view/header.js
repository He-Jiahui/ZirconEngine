import { icon } from "../../../foundation/icons.js";

export function tableHeader() {
  return `<div class="zr-table-row zr-table-head"><span>Name</span><span>Type</span><span>Size</span><span>Modified</span>${icon("gear")}</div>`;
}
