"""Hash-bound physical path migration; no source backups, Git writes or builds."""
from __future__ import annotations

import argparse
from collections import Counter
import datetime
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import tomllib

from .path_names import ROOT, audit, files, path_units
from .path_name_rules import make_moves
from .path_refs import References, rust_rewrite, normalized, module_target
from . import separate_rust_tests as rust

TEXT_SUFFIXES = frozenset({
    ".rs",".md",".py",".ps1",".toml",".json",".yaml",".yml",".zui",
    ".wgsl",".zshader",".cpp",".h",".hpp",".c",".cmake",".txt",".ts",".tsx",
    ".js",".mjs",".html",".css",".scss",".ini",".svg",".xml",
})


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str) -> bytes:
    return subprocess.run(["git",*args],cwd=ROOT,capture_output=True,check=True,
                          env=dict(os.environ,GIT_OPTIONAL_LOCKS="0")).stdout


def write(path: Path, data: bytes) -> None:
    """Mutate the canonical handle, preserving unrelated bytes and line endings."""
    with path.open("r+b" if path.exists() else "x+b") as stream:
        stream.seek(0);stream.write(data);stream.truncate();stream.flush()


def report(directory: Path, name: str, value: dict) -> None:
    with (directory/name).open("x",encoding="utf-8",newline="\n") as stream:
        json.dump(value,stream,ensure_ascii=False,indent=2);stream.write("\n")


def register(directory: Path, names: list[str]) -> None:
    path=directory/"artifacts.json"
    value=json.loads(path.read_text(encoding="utf-8"))
    registered={entry["path"] for entry in value["retained"]}
    for name in names:
        target=str(directory/name)
        if target not in registered:
            value["retained"].append({
                "path":target,"kind":"rename-evidence",
                "reason":"Hash-bound migration and validation metadata; no source bodies",
                "depends_on":["canonical checkout path migration"],"review_after":None,
            })
    write(path,(json.dumps(value,ensure_ascii=False,indent=2)+"\n").encode())


class Permissions:
    def __init__(self):
        config=tomllib.loads((ROOT/".codex/config.toml").read_text(encoding="utf-8"))
        self.rules=[(Path(name),access) for name,access in
                    config["permissions"]["zircon-strict"]["filesystem"].items()
                    if not name.startswith(":")]
        self.checked_ancestors=set()

    def writable(self, path: Path) -> bool:
        ancestors={path,*path.parents}
        matches=[(len(str(parent)),access) for parent,access in self.rules
                 if parent in ancestors]
        return max(matches,default=(0,"read"))[1]=="write"

    def check(self, path: Path) -> None:
        if not self.writable(path):raise PermissionError("No canonical write grant: "+str(path))
        for item in (path,*path.parents):
            if item==ROOT:break
            if item in self.checked_ancestors:break
            try:metadata=item.lstat()
            except FileNotFoundError:metadata=None
            if metadata and (stat.S_ISLNK(metadata.st_mode) or
                             getattr(metadata,"st_file_attributes",0)&stat.FILE_ATTRIBUTE_REPARSE_POINT):
                raise PermissionError("Path alias rejected: "+str(item))
            self.checked_ancestors.add(item)


def tracked_paths() -> list[str]:
    return sorted(set(item.decode("utf-8") for item in
                      git("ls-files","-co","--exclude-standard","-z").split(b"\0") if item))


def build_plan() -> tuple[dict,dict[str,bytes]]:
    paths=files()
    moves,issues=make_moves(paths)
    if issues:raise ValueError({"unresolved_names":issues})
    permission=Permissions()
    for old,new in moves.items():
        permission.check(ROOT/old)
        permission.check(ROOT/new)
        if new in moves:raise ValueError("Destination overlaps another old source: "+new)
        if (ROOT/new).exists():raise ValueError("Destination exists: "+new)
    all_paths=[p for p in tracked_paths() if (ROOT/p).is_file()]
    refs=References(ROOT,moves,all_paths)
    absolute_moves={ROOT/old:ROOT/new for old,new in moves.items()}
    known={ROOT/p for p in all_paths}
    future_known={absolute_moves.get(p,p) for p in known}
    updates={};changes=[];edges=[];unresolved=[];parse_errors=[];readonly=[];preview_failures=[]
    before={}
    candidates=sorted(set(paths+[p for p in all_paths
                               if (p.startswith(("tools/","docs/",".codex/skills/",".github/"))
                                   and not p.startswith(("docs/plans/","docs/ui/","docs/_data/")))
                               or "/" not in p]))
    for index,old in enumerate(candidates):
        path=ROOT/old
        if not path.is_file():continue
        if index%2500==0:print(f"Inspect {index}/{len(candidates)}",flush=True)
        if old.startswith(("tools/audits/path_name","tools/audits/path_moves",
                           "tools/audits/path_refs","tools/tests/test_path_names")):continue
        if path.suffix.lower() not in TEXT_SUFFIXES:
            if old in moves:before[old]=digest(path.read_bytes())
            continue
        data=path.read_bytes()
        if old in moves:before[old]=digest(data)
        if b"\0" in data:continue
        try:text=data.decode("utf-8")
        except UnicodeDecodeError:continue
        new=moves.get(old,old)
        value=text
        if path.suffix==".rs":
            try:value,source_edges,source_unresolved=rust_rewrite(
                value,path,ROOT/new,absolute_moves,known,future_known)
            except ValueError as error:
                parse_errors.append({"path":old,"error":str(error)});continue
            edges.extend({"source":old,**edge} for edge in source_edges)
            unresolved.extend({"source":old,**entry} for entry in source_unresolved)
        value=refs.rewrite(value,old,new,rust_source=path.suffix==".rs")
        encoded=value.encode("utf-8")
        if data!=encoded:
            if path.suffix==".rs":
                observed=rust_rewrite(value,ROOT/new,ROOT/new,{},future_known)[1]
                seen={(e.get("macro"),e.get("name"),tuple(e.get("parents",[])),e["target"]) for e in observed}
                for edge in source_edges:
                    if edge.get("existing",True) and (edge.get("macro"),edge.get("name"),tuple(edge.get("parents",[])),edge["destination"]) not in seen:
                        preview_failures.append({"source":old,**edge})
            if not permission.writable(path):
                readonly.append({"path":old,"before":digest(data),"proposedAfter":digest(encoded)})
                continue
            permission.check(path)
            updates[old]=encoded
            changes.append({"path":old,"destination":new,"before":digest(data),"after":digest(encoded)})
        if old in moves:before[old]=digest(data)
    plan={
        "schema_version":1,"created_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "base_head":git("rev-parse","HEAD").decode().strip(),
        "permission_config_sha256":digest((ROOT/".codex/config.toml").read_bytes()),
        "git_index_sha256":digest((ROOT/".git/index").read_bytes()),
        "git_status_sha256":digest(git("status","--porcelain=v1","-z","--untracked-files=all")),
        "budget":{"total":128,"prefix_reserve":48,"relative":80},
        "inventory":audit(paths,str(ROOT)),
        "moves":[{"old":old,"new":new,"before":before[old]} for old,new in sorted(moves.items())],
        "reference_updates":changes,"rust_edges":edges,"unresolved_existing":unresolved,
        "parse_errors":parse_errors,"readonly_reference_updates":readonly,
        "preview_failures":preview_failures,
        "rust_api_names_preserved":True,"source_backups_created":False,
    }
    return plan,updates


def summarize(plan: dict) -> dict:
    return {"source_files":plan["inventory"]["files"],
            "over_budget_before":plan["inventory"]["long_files"],
            "renamed_files":len(plan["moves"]),
            "reference_files":len(plan["reference_updates"]),
            "rust_edges":len(plan["rust_edges"]),
            "unresolved_existing":len(plan["unresolved_existing"]),
            "parse_errors":plan["parse_errors"],
            "readonly_reference_files":len(plan["readonly_reference_updates"]),
            "preview_failures":plan["preview_failures"][:20],
            "maximum_after":max((path_units(v["new"]) for v in plan["moves"]),default=plan["inventory"]["max_relative_units"])}


def reviewed_updates(plan: dict) -> dict[str,bytes]:
    """Recreate only reviewed edits; exact before/after hashes bind their meaning."""
    moves={v["old"]:v["new"] for v in plan["moves"]}
    absolute={ROOT/a:ROOT/b for a,b in moves.items()}
    known={Path(edge["target"]) for edge in plan["rust_edges"]}|set(absolute)
    future={absolute.get(p,p) for p in known}
    refs=References(ROOT,moves,tracked_paths())
    permissions=Permissions()
    for old,new in moves.items():
        permissions.check(ROOT/old);permissions.check(ROOT/new)
        if (ROOT/new).exists():raise ValueError("Reviewed destination already exists: "+new)
    result={}
    for entry in plan["reference_updates"]:
        old=entry["path"];new=entry["destination"];path=ROOT/old
        permissions.check(path)
        data=path.read_bytes()
        if digest(data)!=entry["before"]:raise ValueError("Reviewed source changed: "+old)
        text=data.decode("utf-8")
        if path.suffix==".rs":text=rust_rewrite(text,path,ROOT/new,absolute,known,future)[0]
        data=refs.rewrite(text,old,new,rust_source=path.suffix==".rs").encode("utf-8")
        if digest(data)!=entry["after"]:raise ValueError("Reviewed transformation differs: "+old)
        result[old]=data
    return result


def apply(directory: Path, plan: dict, updates: dict[str,bytes]) -> None:
    if plan["preview_failures"]:raise ValueError("New Rust or include path resolution differs from the reviewed inputs")
    if plan["parse_errors"]:raise ValueError("Preexisting parse errors require explicit resolution")
    current={v["old"]:v for v in plan["moves"]}
    expected={v["path"]:v["before"] for v in plan["reference_updates"]}
    expected.update({v["old"]:v["before"] for v in plan["moves"]})
    for old,wanted in expected.items():
        if digest((ROOT/old).read_bytes())!=wanted:raise ValueError("Source drift: "+old)
    if git("rev-parse","HEAD").decode().strip()!=plan["base_head"]:
        raise ValueError("HEAD changed; regenerate and review the plan")
    if digest((ROOT/".codex/config.toml").read_bytes())!=plan["permission_config_sha256"]:
        raise ValueError("Permission configuration changed")
    register(directory,["applied.json","operations.jsonl","verify.json","partial-failure.json"])
    journal=directory/"operations.jsonl"
    moved=[];written=[];empty=[]
    with journal.open("x",encoding="utf-8",newline="\n") as log:
        def event(value):
            log.write(json.dumps(value,ensure_ascii=False)+"\n");log.flush()
        try:
            for index,entry in enumerate(plan["moves"]):
                old=ROOT/entry["old"];new=ROOT/entry["new"]
                if digest(old.read_bytes())!=entry["before"]:raise ValueError("Move source drift: "+str(old))
                new.parent.mkdir(parents=True,exist_ok=True)
                old.rename(new)
                if digest(new.read_bytes())!=entry["before"]:raise ValueError("Moved bytes changed: "+str(new))
                moved.append(entry["old"]);event({"action":"rename",**entry})
                if index%2000==0:print(f"Moved {index}/{len(plan['moves'])}",flush=True)
            for index,(old,data) in enumerate(updates.items()):
                target=ROOT/current[old]["new"] if old in current else ROOT/old
                if digest(target.read_bytes())!=expected[old]:
                    raise ValueError("Reference source drift: "+str(target))
                write(target,data)
                written.append(old);event({"action":"references","path":str(target),"after":digest(data)})
                if index%2000==0:print(f"References {index}/{len(updates)}",flush=True)
            directories=set(parent for old in moved for parent in (ROOT/old).parents
                            if parent!=ROOT and ROOT in parent.parents)
            for path in sorted(directories,key=lambda p:len(p.parts),reverse=True):
                if path.is_dir():
                    try:path.rmdir()
                    except OSError:continue
                    empty.append(str(path));event({"action":"empty-source-directory","path":str(path)})
        except Exception as error:
            report(directory,"partial-failure.json",{
                "error":repr(error),"moved":moved,"references_written":written,
                "empty_source_directories":empty,"replay_allowed":False,
            })
            raise
    report(directory,"applied.json",{
        "renamed":len(moved),"reference_files":len(written),
        "empty_source_directories_removed":len(empty),
        "plan_sha256":digest((directory/"plan.json").read_bytes()),
        "GitWrites":False,"builds":False,"sourceBackups":False,
    })


def verify(directory: Path, plan: dict) -> dict:
    updates={v["path"]:v for v in plan["reference_updates"]}
    failures=[]
    for move in plan["moves"]:
        old=ROOT/move["old"];new=ROOT/move["new"]
        expected=updates.get(move["old"],{}).get("after",move["before"])
        if old.exists():failures.append({"path":move["old"],"error":"old-source-remains"})
        if not new.is_file() or digest(new.read_bytes())!=expected:
            failures.append({"path":move["new"],"error":"destination-hash"})
    for update in plan["reference_updates"]:
        path=ROOT/update["destination"]
        if not path.is_file() or digest(path.read_bytes())!=update["after"]:
            failures.append({"path":str(path),"error":"reference-hash"})
    current_files=files()
    result=audit(current_files,str(ROOT))
    moves={v["old"]:v["new"] for v in plan["moves"]}
    source_cache={}
    edge_failures=[]
    known={ROOT/p for p in tracked_paths()}|{ROOT/p for p in current_files}
    for edge in plan["rust_edges"]:
        old=edge["source"];new=moves.get(old,old)
        if new not in source_cache:
            value=(ROOT/new).read_bytes().decode("utf-8")
            source_cache[new]=(value,rust.modules(value),rust_rewrite(value,ROOT/new,ROOT/new,{},known)[1])
        value,modules,observed=source_cache[new]
        if "macro" in edge:
            if edge.get("existing",True) and not any(v.get("macro")==edge["macro"] and v["target"]==edge["destination"] for v in observed):
                edge_failures.append({"source":new,"macro":edge["macro"],"error":"include-destination"})
            continue
        same=[m for m in modules if not m.inline and m.name==edge["name"]
              and list(m.parents)==edge["parents"]]
        destinations={module_target(ROOT/new,m,known) for m in same}
        if Path(edge["destination"]) not in destinations:
            edge_failures.append({"source":new,"name":edge["name"],"expected":edge["destination"]})
    result.update({"hash_failures":failures,"rust_edge_failures":edge_failures,
                   "rust_edges_checked":len(plan["rust_edges"]),
                   "unresolved_before":len(plan["unresolved_existing"]),
                   "git_index_unchanged":digest((ROOT/".git/index").read_bytes())==plan["git_index_sha256"],
                   "JenkinsAcceptance":False,"nativeRustAcceptance":False})
    result["passed"]=result["passed"] and not failures and not edge_failures
    report(directory,"verify.json",result)
    return result


def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode",choices=["plan","apply","verify"])
    parser.add_argument("--reports",type=Path,required=True)
    args=parser.parse_args()
    directory=args.reports
    if not (directory/"artifacts.json").is_file():
        parser.error("Register task ownership in artifacts.json before creating reports")
    if args.mode=="verify":
        plan=json.loads((directory/"plan.json").read_text())
        result=verify(directory,plan)
        print(json.dumps({k:v for k,v in result.items() if k!="violations"},ensure_ascii=False))
        return 0 if result["passed"] else 1
    if args.mode=="apply":
        accepted=json.loads((directory/"plan.json").read_text())
        updates=reviewed_updates(accepted)
        apply(directory,accepted,updates)
        print(json.dumps({"applied":len(accepted["moves"]),"references":len(updates)}))
        return 0
    plan,updates=build_plan()
    if args.mode=="plan":
        register(directory,["plan.json"])
        report(directory,"plan.json",plan)
        print(json.dumps(summarize(plan),ensure_ascii=False))
        return 0
    return 0


if __name__=="__main__":
    raise SystemExit(main())
