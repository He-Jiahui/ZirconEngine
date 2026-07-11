import { Table, TableBody, TableCell, TableContainer, TableHead, TableRow } from "@mui/material";
import type { ReactNode } from "react";

export interface Column<T> { key: string; label: string; render: (row: T) => ReactNode }

export function BoundedTable<T>({ rows, columns, rowKey, limit = 200, empty = "暂无记录" }: {
  rows: T[]; columns: Column<T>[]; rowKey: (row: T) => string; limit?: number; empty?: string;
}) {
  const visible = rows.slice(0, limit);
  return <TableContainer tabIndex={0} aria-label={`数据表，最多显示 ${limit} 行`}>
    <Table size="small" stickyHeader>
      <TableHead><TableRow>{columns.map((column) => <TableCell key={column.key}>{column.label}</TableCell>)}</TableRow></TableHead>
      <TableBody>
        {visible.map((row) => <TableRow hover key={rowKey(row)}>{columns.map((column) => <TableCell key={column.key}>{column.render(row)}</TableCell>)}</TableRow>)}
        {!visible.length && <TableRow><TableCell colSpan={columns.length}>{empty}</TableCell></TableRow>}
      </TableBody>
    </Table>
    {rows.length > limit && <p role="status">已限制显示前 {limit} 行，共 {rows.length} 行。</p>}
  </TableContainer>;
}
