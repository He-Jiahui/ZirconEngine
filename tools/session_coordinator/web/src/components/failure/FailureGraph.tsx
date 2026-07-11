import { Stack, Typography } from "@mui/material";
import type { FailureProjection } from "../../api/contracts";
import { StatusText } from "../StatusText";
import { failureClass } from "./failureModel";

export function FailureGraph({ nodes, diagnostics }: FailureProjection) {
  return <Stack spacing={1} aria-label="Failure 依赖图">{nodes.slice(0, 200).map((node) => {
    const state = failureClass(node);
    return <Stack key={node.node_id} direction="row" spacing={2} sx={{ alignItems: "center" }} className="graph-node">
      <StatusText value={state} /><Typography>{String(node.summary_slug ?? node.summary ?? node.node_id ?? "未命名 Failure")}</Typography>
      <Typography variant="caption">{String(node.origin_plan ?? "未知来源计划")} → {String(node.fixing_plan ?? "未知修复计划")}</Typography>
    </Stack>;
  })}{!nodes.length && <Typography>当前没有 Failure 节点。</Typography>}<Typography variant="h6">图诊断</Typography>{diagnostics.slice(0, 100).map((diagnostic) => <Stack key={diagnostic.diagnosticId} className="graph-node"><Typography><strong>{diagnostic.code}</strong>：{diagnostic.message}</Typography><Typography variant="caption">{diagnostic.paths.join("、") || "无关联路径"}</Typography></Stack>)}{!diagnostics.length && <Typography>当前没有图诊断。</Typography>}</Stack>;
}
