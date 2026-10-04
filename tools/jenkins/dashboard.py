"""ZirconEngine Jenkins build status dashboard generator.

Reads coordination SQLite, job build records, and failure evidence files,
then produces a self-contained HTML report.

Usage:
    python tools/jenkins/dashboard.py [--out PATH] [--open]
"""
from __future__ import annotations

import argparse
import json
import os
import sqlite3
import webbrowser
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

_CSS = """
:root {
  --bg:       #141414;
  --bg-raised:#1e1e1e;
  --border:   #2e2e2e;
  --text:     #f0f0f0;
  --muted:    #888;
  --accent:   #4a9eff;
  --success:  #3ecf8e;
  --warn:     #f5a623;
  --danger:   #e05252;
  --r-sm:     4px;
  --r-md:     8px;
}
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{background:var(--bg);color:var(--text);font:15px/1.6 system-ui,sans-serif;padding:32px 24px}
h1{font-size:1.5rem;font-weight:600;margin-bottom:4px}
h2{font-size:1.1rem;font-weight:600;margin:32px 0 12px;color:var(--text)}
.meta{color:var(--muted);font-size:.85rem;margin-bottom:32px}
.card{background:var(--bg-raised);border:1px solid var(--border);border-radius:var(--r-md);padding:20px;margin-bottom:12px}
table{width:100%;border-collapse:collapse;font-size:.9rem}
th{text-align:left;padding:8px 12px;border-bottom:1px solid var(--border);color:var(--muted);font-weight:500;white-space:nowrap}
td{padding:8px 12px;border-bottom:1px solid var(--border);vertical-align:top}
tr:last-child td{border-bottom:none}
.badge{display:inline-block;padding:2px 8px;border-radius:99px;font-size:.78rem;font-weight:600;letter-spacing:.02em}
.badge-open    {background:#4a9eff22;color:var(--accent)}
.badge-claimed {background:#f5a62322;color:var(--warn)}
.badge-submitted{background:#9b59b622;color:#c77dff}
.badge-accepted{background:#3ecf8e22;color:var(--success)}
.badge-fixing  {background:#f5a62322;color:var(--warn)}
.badge-P0{background:#e0525222;color:var(--danger)}
.badge-P1{background:#f5a62322;color:var(--warn)}
.badge-P2{background:#4a9eff22;color:var(--accent)}
.badge-SUCCESS{background:#3ecf8e22;color:var(--success)}
.badge-FAILURE{background:#e0525222;color:var(--danger)}
.badge-ABORTED{background:#88888822;color:var(--muted)}
.badge-UNSTABLE{background:#f5a62322;color:var(--warn)}
.stat-row{display:flex;gap:16px;flex-wrap:wrap;margin-bottom:24px}
.stat{background:var(--bg-raised);border:1px solid var(--border);border-radius:var(--r-md);padding:16px 24px;min-width:120px}
.stat-num{font-size:2rem;font-weight:700;line-height:1;color:var(--accent)}
.stat-label{color:var(--muted);font-size:.8rem;margin-top:4px}
.bar-wrap{background:var(--border);border-radius:99px;height:8px;margin-top:6px;overflow:hidden}
.bar-fill{height:100%;border-radius:99px;background:var(--accent)}
.empty{color:var(--muted);font-size:.9rem;padding:16px 0}
a{color:var(--accent);text-decoration:none}
a:hover{text-decoration:underline}
"""


def _now_str() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")


def _badge(text: str, extra: str = "") -> str:
    cls = f"badge badge-{text.replace(' ', '-')}"
    if extra:
        cls += f" {extra}"
    return f'<span class="{cls}">{text}</span>'


def _read_gap_issues(db_path: Path) -> list[dict]:
    if not db_path.is_file():
        return []
    try:
        con = sqlite3.connect(str(db_path), timeout=5)
        con.row_factory = sqlite3.Row
        rows = con.execute(
            "SELECT issue_id, doc_path, priority, gate_id, summary, status, updated_at "
            "FROM gap_issues ORDER BY priority, status, issue_id LIMIT 200"
        ).fetchall()
        con.close()
        return [dict(r) for r in rows]
    except Exception:
        return []


def _read_builds(jobs_dir: Path) -> list[dict]:
    builds = []
    if not jobs_dir.is_dir():
        return builds
    for job_dir in sorted(jobs_dir.iterdir()):
        if not job_dir.is_dir():
            continue
        builds_dir = job_dir / "builds"
        if not builds_dir.is_dir():
            continue
        for build_dir in sorted(builds_dir.iterdir(), reverse=True)[:10]:
            if not build_dir.is_dir():
                continue
            build_xml = build_dir / "build.xml"
            if not build_xml.is_file():
                continue
            try:
                text = build_xml.read_text(encoding="utf-8", errors="replace")
                import re
                result = (re.search(r"<result>(.*?)</result>", text) or
                          re.search(r"<currentResult>(.*?)</currentResult>", text))
                ts = re.search(r"<startTime>(\d+)</startTime>", text)
                num = re.search(r"<number>(\d+)</number>", text)
                builds.append({
                    "job": job_dir.name,
                    "number": int(num.group(1)) if num else 0,
                    "result": result.group(1) if result else "UNKNOWN",
                    "ts": int(ts.group(1)) // 1000 if ts else 0,
                })
            except Exception:
                continue
    builds.sort(key=lambda b: b["ts"], reverse=True)
    return builds[:50]


def _read_failures(failures_dir: Path) -> list[dict]:
    if not failures_dir.is_dir():
        return []
    out = []
    for f in sorted(failures_dir.glob("failure-*.json"), reverse=True)[:20]:
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
            data["_file"] = f.name
            out.append(data)
        except Exception:
            out.append({"_file": f.name, "error": "unreadable"})
    return out


def _ts_str(ts: int) -> str:
    if not ts:
        return "—"
    try:
        return datetime.fromtimestamp(ts, tz=timezone.utc).strftime("%Y-%m-%d %H:%M")
    except Exception:
        return str(ts)


def generate(out_path: Path) -> None:
    db_path = REPO / ".jenkins" / "state" / "coordination.sqlite3"
    jobs_dir = REPO / ".jenkins" / "jenkins_home" / "jobs"
    failures_dir = REPO / ".jenkins" / "builds" / "zircon-jenkins" / "failures"

    issues = _read_gap_issues(db_path)
    builds = _read_builds(jobs_dir)
    failures = _read_failures(failures_dir)

    # Stats
    total_issues = len(issues)
    by_status: dict[str, int] = {}
    by_priority: dict[str, int] = {}
    for i in issues:
        by_status[i["status"]] = by_status.get(i["status"], 0) + 1
        by_priority[i["priority"]] = by_priority.get(i["priority"], 0) + 1

    success_builds = sum(1 for b in builds if b["result"] == "SUCCESS")
    fail_builds = sum(1 for b in builds if b["result"] == "FAILURE")

    # ── Gap issue stats ──────────────────────────────────────────────────────
    stats_html = '<div class="stat-row">'
    stats_html += f'<div class="stat"><div class="stat-num">{total_issues}</div><div class="stat-label">Total Gap Issues</div></div>'
    for st, count in sorted(by_status.items()):
        stats_html += f'<div class="stat"><div class="stat-num">{count}</div><div class="stat-label">{st.title()}</div></div>'
    stats_html += '</div>'

    # Priority bar
    if total_issues:
        p0 = by_priority.get("P0", 0)
        p1 = by_priority.get("P1", 0)
        p2 = by_priority.get("P2", 0)
        stats_html += f'''<div style="margin-bottom:24px;font-size:.85rem;color:var(--muted)">
          Priority breakdown &nbsp;
          {_badge("P0")} {p0} &nbsp;
          {_badge("P1")} {p1} &nbsp;
          {_badge("P2")} {p2}
        </div>'''

    # ── Gap issues table ─────────────────────────────────────────────────────
    if issues:
        rows = ""
        for i in issues:
            gate = f'<span style="color:var(--muted);font-size:.8rem">{i["gate_id"]}</span>' if i.get("gate_id") else ""
            summary = (i.get("summary") or "")[:120]
            rows += (
                f'<tr>'
                f'<td><code style="font-size:.82rem">{i["issue_id"]}</code></td>'
                f'<td>{_badge(i["priority"])}</td>'
                f'<td>{_badge(i["status"])}</td>'
                f'<td>{summary}{gate}</td>'
                f'<td style="color:var(--muted);font-size:.8rem;white-space:nowrap">'
                f'{(i.get("updated_at") or "")[:16]}</td>'
                f'</tr>'
            )
        issues_html = f'''<div class="card">
          <table>
            <thead><tr>
              <th>Issue ID</th><th>Priority</th><th>Status</th>
              <th>Summary</th><th>Updated</th>
            </tr></thead>
            <tbody>{rows}</tbody>
          </table>
        </div>'''
    else:
        issues_html = '<p class="empty">No gap issues found in database (run <code>python -m tools.jenkins.gap extract</code> first).</p>'

    # ── Build history ─────────────────────────────────────────────────────────
    if builds:
        brows = ""
        for b in builds:
            brows += (
                f'<tr>'
                f'<td>{b["job"]}</td>'
                f'<td style="text-align:right;font-variant-numeric:tabular-nums">#{b["number"]}</td>'
                f'<td>{_badge(b["result"])}</td>'
                f'<td style="color:var(--muted);font-size:.8rem">{_ts_str(b["ts"])}</td>'
                f'</tr>'
            )
        build_summary = f'<p style="color:var(--muted);font-size:.85rem;margin-bottom:12px">{success_builds} succeeded · {fail_builds} failed of {len(builds)} shown</p>'
        builds_html = build_summary + f'''<div class="card">
          <table>
            <thead><tr><th>Job</th><th style="text-align:right">#</th><th>Result</th><th>Started</th></tr></thead>
            <tbody>{brows}</tbody>
          </table>
        </div>'''
    else:
        builds_html = '<p class="empty">No build records found under .jenkins/jenkins_home/jobs/.</p>'

    # ── Failures ─────────────────────────────────────────────────────────────
    if failures:
        fcards = ""
        for f in failures:
            ts = f.get("failedAt", 0)
            ts_str = _ts_str(int(ts)) if ts else "—"
            req = f.get("requestId", "—")
            err = str(f.get("error", ""))[:200]
            failed_stages = f.get("failedStages", [])
            stage_list = ", ".join(s.get("name", "?") for s in failed_stages) if failed_stages else "—"
            fcards += f'''<div class="card" style="margin-bottom:8px">
              <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px">
                <code style="font-size:.82rem">{f["_file"]}</code>
                <span style="color:var(--muted);font-size:.8rem">{ts_str}</span>
              </div>
              <div style="font-size:.85rem;color:var(--muted)">
                Request: <code>{req}</code> &nbsp;·&nbsp; Failed stages: {stage_list}
              </div>
              {f'<div style="margin-top:6px;font-size:.82rem;color:var(--danger);font-family:monospace">{err}</div>' if err else ""}
            </div>'''
        failures_html = fcards
    else:
        failures_html = '<p class="empty">No failure evidence files found.</p>'

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>ZirconEngine Jenkins Dashboard</title>
<style>{_CSS}</style>
</head>
<body>
<h1>ZirconEngine Jenkins Dashboard</h1>
<p class="meta">Generated {_now_str()} · {db_path}</p>

{stats_html}

<h2>Gap Issues</h2>
{issues_html}

<h2>Recent Builds</h2>
{builds_html}

<h2>Recent Failures</h2>
{failures_html}

</body>
</html>"""

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(html, encoding="utf-8")
    print(f"Dashboard written: {out_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--out", type=Path,
                        default=REPO / ".jenkins" / "state" / "dashboard.html")
    parser.add_argument("--open", action="store_true",
                        help="Open the dashboard in the browser after generation")
    args = parser.parse_args()
    generate(args.out)
    if args.open:
        webbrowser.open(args.out.as_uri())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
