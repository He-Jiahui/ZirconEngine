import type { JsonObject } from "../../api/contracts";
import { BoundedTable } from "../BoundedTable";

const text = (row: JsonObject, ...keys: string[]) => String(keys.map((key) => row[key]).find((value) => value != null) ?? "—");
export function LeaseTable({ leases }: { leases: JsonObject[] }) {
  return <BoundedTable rows={leases} rowKey={(row) => text(row, "path_key", "display_path")} columns={[
    { key: "path", label: "文件/模块", render: (row) => text(row, "display_path", "path_key") },
    { key: "owner", label: "所有者", render: (row) => text(row, "session_id") },
    { key: "expires", label: "到期", render: (row) => text(row, "expires_at") },
    { key: "base", label: "基线哈希", render: (row) => <code>{text(row, "base_hash").slice(0, 12)}</code> },
  ]} />;
}
