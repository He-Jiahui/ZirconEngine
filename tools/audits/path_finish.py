"""Complete a reviewed path migration against the actual current checkout."""
from __future__ import annotations

from pathlib import Path
import json

from .path_moves import ROOT, TEXT_SUFFIXES, digest, write, register, report, tracked_paths
from .path_names import files, audit
from .path_name_rules import make_moves
from .path_refs import References, rust_rewrite


def finish(directory: Path) -> None:
    if not (directory/"artifacts.json").is_file():
        raise ValueError("Task ownership must be registered before migration")
    if (directory/"extension-plan.json").exists():
        raise ValueError("Extension evidence exists; reconcile its journal instead of replaying moves")
    plan=json.loads((directory/"plan.json").read_text())
    paths=files();extra,issues=make_moves(paths)
    if issues:raise ValueError(issues)
    all_paths=sorted(set(paths+[p for p in tracked_paths() if (ROOT/p).is_file()]))
    known={ROOT/p for p in all_paths};absolute={ROOT/a:ROOT/b for a,b in extra.items()}
    future={absolute.get(p,p) for p in known}
    refs=References(ROOT,extra,all_paths)
    original={e["old"]:e for e in plan["moves"]}
    reverse={e["new"]:e["old"] for e in plan["moves"]}
    updates={e["path"]:e for e in plan["reference_updates"]}
    changes={};replacement_edges={}
    candidates=sorted(set(paths+[p for p in all_paths if
        (p.startswith(("tools/","docs/","templates/",".codex/skills/",".github/"))
         and not p.startswith(("docs/plans/","docs/ui/","docs/_data/"))) or "/" not in p]))
    for n,name in enumerate(candidates):
        path=ROOT/name
        if path.suffix.lower() not in TEXT_SUFFIXES:continue
        if name.startswith(("tools/audits/path_","tools/tests/test_path_names")):continue
        raw=path.read_bytes()
        if b"\0" in raw:continue
        try:text=raw.decode()
        except UnicodeError:continue
        new=extra.get(name,name)
        if path.suffix==".rs":
            text,edges,missing=rust_rewrite(text,path,ROOT/new,absolute,known,future)
            replacement_edges[reverse.get(name,name)]=[
                {"source":reverse.get(name,name),**e} for e in edges]
        text=refs.rewrite(text,name,new,rust_source=path.suffix==".rs")
        encoded=text.encode()
        if raw!=encoded:changes[name]=(new,digest(raw),encoded)
        if n%4000==0:print("Canonical source/reference pass",n,len(candidates),flush=True)
    for old,new in extra.items():
        if (ROOT/new).exists():raise ValueError("Destination exists: "+new)
        before=digest((ROOT/old).read_bytes());owner=reverse.get(old,old)
        if owner in original:original[owner]["new"]=new
        else:original[owner]={"old":owner,"new":new,"before":before}
    for entry in updates.values():entry["destination"]=extra.get(entry["destination"],entry["destination"])
    for name,(new,before,data) in changes.items():
        owner=reverse.get(name,name);prior=updates.get(owner)
        updates[owner]={"path":owner,"destination":new,
                        "before":prior["before"] if prior else before,"after":digest(data)}
    edges=[e for e in plan["rust_edges"] if e["source"] not in replacement_edges]
    for edge in edges:
        destination=Path(edge["destination"])
        if destination.is_relative_to(ROOT):
            current=destination.relative_to(ROOT).as_posix()
            edge["destination"]=str(ROOT/extra.get(current,current))
    edges += [e for batch in replacement_edges.values() for e in batch]
    plan["moves"]=list(original.values());plan["reference_updates"]=list(updates.values())
    plan["rust_edges"]=edges
    register(directory,["extension-plan.json","extension-operations.jsonl",
                        "extension-applied.json","plan.before-extension.json"])
    report(directory,"extension-plan.json",{
        "moves":[{"old":a,"new":b,"before":digest((ROOT/a).read_bytes())} for a,b in extra.items()],
        "references":[{"path":a,"new":v[0],"before":v[1],"after":digest(v[2])} for a,v in changes.items()],
        "reason":"Short feature folders and canonical target modules excluded by broad build-ignore patterns",
        "source_backups":False})
    print("Applying",len(extra),"extra renames and",len(changes),"reference edits",flush=True)
    with (directory/"extension-operations.jsonl").open("x",encoding="utf-8") as log:
        def event(value):
            log.write(json.dumps(value,ensure_ascii=False)+"\n");log.flush()
        for old,new in extra.items():
            source=ROOT/old;destination=ROOT/new;before=digest(source.read_bytes())
            destination.parent.mkdir(parents=True,exist_ok=True);source.rename(destination)
            if digest(destination.read_bytes())!=before:raise ValueError("Rename changed bytes")
            event({"action":"rename","old":old,"new":new,"sha256":before})
        for name,(new,before,data) in changes.items():
            path=ROOT/new
            if digest(path.read_bytes())!=before:raise ValueError("Concurrent reference edit: "+name)
            write(path,data);event({"action":"references","path":new,"sha256":digest(data)})
        parents={p for old in extra for p in (ROOT/old).parents if p!=ROOT and ROOT in p.parents}
        for parent in sorted(parents,key=lambda p:len(p.parts),reverse=True):
            try:parent.rmdir()
            except OSError:continue
            event({"action":"empty-source-directory","path":str(parent)})
    (directory/"plan.json").rename(directory/"plan.before-extension.json")
    report(directory,"plan.json",plan)
    result=audit(files(),str(ROOT))
    report(directory,"extension-applied.json",{
        "renamed":len(extra),"reference_files":len(changes),"path_audit":result,"source_backups":False})
    print(json.dumps({k:v for k,v in result.items() if k!="violations"}),flush=True)


if __name__=="__main__":
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reports",type=Path,required=True)
    finish(parser.parse_args().reports)
