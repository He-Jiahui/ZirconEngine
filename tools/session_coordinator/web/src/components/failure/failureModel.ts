import type { JsonObject } from "../../api/contracts";

export function failureClass(node: JsonObject): "applicable" | "fixed" | "foreign" | "invalid" | "open" {
  const status = String(node.status ?? node.resolution ?? "open").toLowerCase();
  if (status.includes("invalid")) return "invalid";
  if (status.includes("fixed") || status.includes("resolved")) return "fixed";
  if (node.applicable === false || node.applicable === 0) return "foreign";
  if (node.applicable === true || node.applicable === 1) return "applicable";
  return "open";
}
