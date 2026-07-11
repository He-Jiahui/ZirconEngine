import type { CargoJobProjection } from "../../api/contracts";
import { BoundedTable } from "../BoundedTable";
import { StatusText } from "../StatusText";

const value = (row: CargoJobProjection, key: keyof CargoJobProjection) => String(row[key] ?? "—");
export function ValidationLaneTable({ jobs }: { jobs: CargoJobProjection[] }) {
  return <BoundedTable rows={jobs} rowKey={(row) => value(row, "job_id")} columns={[
    { key: "lane", label: "验证通道", render: (row) => value(row, "lane_kind") },
    { key: "state", label: "状态", render: (row) => <StatusText value={value(row, "status")} /> },
    { key: "target", label: "Cargo 目标根", render: (row) => value(row, "target_dir") },
    { key: "pid", label: "PID", render: (row) => value(row, "pid") },
    { key: "exit", label: "退出码", render: (row) => value(row, "exit_code") },
    { key: "time", label: "创建时间", render: (row) => value(row, "created_at") },
  ]} />;
}
