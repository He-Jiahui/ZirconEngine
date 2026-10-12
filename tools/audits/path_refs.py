"""Resolve Rust mounts and rewrite physical path references without API renames."""
from __future__ import annotations

from collections import defaultdict
import json
import os
from pathlib import Path, PurePosixPath
import re

from . import separate_rust_tests as rust

PATH_TEXT = re.compile(r"(?:res://)?(?:[A-Za-z]:[\\/])?(?:\.{1,2}/)*[A-Za-z0-9_.@+-]+(?:[/\\][A-Za-z0-9_.@+-]+)+")
QUOTED = re.compile(r'"([^"\r\n]+)"')


def normalized(value: Path) -> Path:
    return Path(os.path.normpath(value))


def module_target(path: Path, module: rust.Module, known: set[Path]) -> Path | None:
    explicit=rust.PATH.search(module.attributes)
    if explicit:
        base=rust.module_base(path).joinpath(*module.parents) if module.parents else path.parent
        candidate=normalized(base/explicit[1])
        return candidate if candidate in known else None
    base=rust.module_base(path).joinpath(*module.parents)
    for candidate in (base/(module.name+".rs"),base/module.name/"mod.rs"):
        if candidate in known:return candidate
    # Attributed files can own their containing directory. The caller records
    # this resolution explicitly, so the new physical name cannot change it.
    if not module.parents:
        for candidate in (path.parent/(module.name+".rs"),path.parent/module.name/"mod.rs"):
            if candidate in known and candidate!=path:return candidate
    return None


def relative(target: Path, base: Path) -> str:
    return Path(os.path.relpath(target,base)).as_posix()


def concat_rewrite(source: str, old: Path, new: Path, moves: dict[Path,Path],
                   known: set[Path]) -> tuple[str,list[dict]]:
    """Keep static and local single-path asset macros bound to the same bytes."""
    masked,literals=rust.mask_rust(source)
    edits=[];edges=[]
    patterns=re.compile(r'\b(include|include_str|include_bytes)!\s*\(\s*concat!\s*\(')
    for match in patterns.finditer(masked):
        start=match.end();depth=1;end=start
        while end<len(masked) and depth:
            depth+=(masked[end]=='(')-(masked[end]==')');end+=1
        body=source[start:end-1]
        strings=list(QUOTED.finditer(body))
        if not strings:continue
        variable=re.search(r'\$(\w+)',masked[start:end-1])
        env=re.search(r'env!\s*\(\s*"([^"]+)"\s*\)',body)
        if env and env[1]!="CARGO_MANIFEST_DIR":continue
        # Remove the env name by its exact quoted range, not by its value.
        if env:strings=[m for m in strings if not (env.start()<=m.start()<env.end())]
        if not strings:continue
        base=old.parent
        if env:
            base=next((p for p in old.parents if (p/"Cargo.toml").is_file()),old.parent)
        prefix="".join(m[1] for m in strings)
        if variable:
            declarations=list(re.finditer(r'\bmacro_rules!\s*(\w+)\s*\{',masked[:match.start()]))
            if not declarations:continue
            name=declarations[-1][1]
            calls=[]
            for call in re.finditer(r'\b'+re.escape(name)+r'!\s*\(\s*"([^"\n]*)"\s*\)',source):
                if not masked[call.start():call.start()+len(name)].strip():continue
                target=normalized(base/(prefix+call[1]).lstrip('/') if env else base/(prefix+call[1]))
                if target not in known:continue
                calls.append((call,target,moves.get(target,target)))
            if not calls:continue
            edges.extend({"macro":name,"target":str(target),"destination":str(destination),"kind":"concat-call"}
                         for call,target,destination in calls)
            if old==new and all(target==destination for call,target,destination in calls):continue
            destination_base=Path(os.path.commonpath([str(v[2].parent) for v in calls]))
            new_prefix=("/"+relative(destination_base,base) if env else relative(destination_base,new.parent))+"/"
            edits.append((start+strings[0].start(1),start+strings[0].end(1),new_prefix))
            edits.extend((start+m.start(1),start+m.end(1),"") for m in strings[1:])
            for call,target,destination in calls:
                edits.append((*call.span(1),relative(destination,destination_base)))
        else:
            rest=QUOTED.sub('""',body)
            rest=re.sub(r'env!\s*\(\s*""\s*\)','',rest)
            if re.sub(r'[\s,"]','',rest):continue
            target=normalized(base/prefix.lstrip('/') if env else base/prefix)
            if target not in known:continue
            destination=moves.get(target,target)
            edges.append({"macro":match[1],"target":str(target),"destination":str(destination),"kind":"concat"})
            if old==new and target==destination:continue
            value="/"+relative(destination,base) if env else relative(destination,new.parent)
            edits.append((start+strings[0].start(1),start+strings[0].end(1),value))
            edits.extend((start+m.start(1),start+m.end(1),"") for m in strings[1:])
    for start,end,value in sorted(set(edits),reverse=True):source=source[:start]+value+source[end:]
    return source,edges


def rust_rewrite(source: str, old: Path, new: Path, moves: dict[Path,Path],
                 known: set[Path], future_known: set[Path] | None = None) -> tuple[str,list[dict],list[dict]]:
    edits=[];edges=[];unresolved=[]
    if future_known is None:future_known={moves.get(p,p) for p in known} if moves else known
    for module in rust.modules(source):
        if module.inline:continue
        target=module_target(old,module,known)
        if target is None:
            unresolved.append({"name":module.name,"line":source.count("\n",0,module.keyword)+1})
            continue
        destination=moves.get(target,target)
        edges.append({"name":module.name,"parents":list(module.parents),"target":str(target),"destination":str(destination)})
        explicit=rust.PATH.search(module.attributes)
        base=rust.module_base(new).joinpath(*module.parents) if module.parents else new.parent
        default_base=rust.module_base(new).joinpath(*module.parents)
        default=(default_base/(module.name+".rs"),default_base/module.name/"mod.rs")
        choices=[p for p in default if p in future_known]
        ambiguous=not explicit and (len(choices)>1 or (choices and choices[0]!=destination))
        # Every changed resolution gets an explicit physical mount. Module names,
        # visibility, cfg, public reexports and executable code remain intact.
        needs=(old!=new or target!=destination or ambiguous or (not explicit and not module.parents))
        if not needs:continue
        if explicit:
            start=module.start+explicit.start(1);end=module.start+explicit.end(1)
            edits.append((start,end,relative(destination,base)))
        elif destination not in default or ambiguous or not module.parents:
            line_start=source.rfind("\n",0,module.keyword)+1
            indent=source[line_start:module.keyword]
            indent=indent if not indent.strip() else ""
            newline="\r\n" if "\r\n" in source else "\n"
            attribute="#[path = "+json.dumps(relative(destination,base),ensure_ascii=False)+"]"
            edits.append((module.keyword,module.keyword,attribute+newline+indent))
    masked,_=rust.mask_rust(source)
    includes=[]
    for match in rust.INCLUDE.finditer(source):
        if not masked[match.start():match.start()+len(match[1])].strip():continue
        target=normalized(old.parent/match[2])
        destination=moves.get(target,target)
        if old!=new or target!=destination:
            edits.append((*match.span(2),relative(destination,new.parent)))
        includes.append({"macro":match[1],"target":str(target),"destination":str(destination),"existing":target in known})
    for start,end,value in sorted(edits,reverse=True):source=source[:start]+value+source[end:]
    source,concats=concat_rewrite(source,old,new,moves,known)
    return source,edges+includes+concats,unresolved


class References:
    def __init__(self, root: Path, moves: dict[str,str], all_paths: list[str]):
        self.root=root
        self.moves=moves
        self.path_parts={part for old in moves for part in old.split('/')[:-1]}
        self.paths=set(all_paths)
        self.manifests=set(PurePosixPath(p).parent.as_posix()
                           for p in all_paths if PurePosixPath(p).name=="Cargo.toml")
        self.directories={}
        directories=defaultdict(set)
        for old,new in moves.items():
            oldp=PurePosixPath(old);newp=PurePosixPath(new)
            if oldp.name=="mod.rs":directories[oldp.parent.as_posix()].add(newp.parent.as_posix())
            # The deepest possible common owner is used only when unambiguous.
            for parent in oldp.parents:
                name=parent.as_posix()
                if name==".":continue
                try:tail=oldp.relative_to(parent).parts
                except ValueError:continue
                if len(newp.parts)>=len(tail) and newp.parts[-len(tail):]==tail:
                    directories[name].add(PurePosixPath(*newp.parts[:-len(tail)]).as_posix())
        for old,values in directories.items():
            if len(values)==1:self.directories[old]=next(iter(values))
        self.assets={}
        for old,new in moves.items():
            a=PurePosixPath(old).parts;b=PurePosixPath(new).parts
            if "assets" in a and "assets" in b:
                key="res://"+"/".join(a[a.index("assets")+1:])
                value="res://"+"/".join(b[b.index("assets")+1:])
                if key in self.assets and self.assets[key]!=value:
                    self.assets[key]=None
                else:self.assets[key]=value

    def mapped(self, value: str) -> str | None:
        return self.moves.get(value,self.directories.get(value))

    def crate(self, source: str) -> str:
        for p in PurePosixPath(source).parents:
            if p.as_posix() in self.manifests:return p.as_posix()
        return PurePosixPath(source).parts[0] if source.startswith("zircon_") else ""

    def rewrite(self, text: str, old: str, new: str, *, rust_source: bool = False) -> str:
        oldp=self.root/old;newp=self.root/new
        crate=self.crate(old)
        protected=[]
        if rust_source:
            masked,_=rust.mask_rust(text)
            for module in rust.modules(text):
                path=rust.PATH.search(module.attributes)
                if path:protected.append((module.start+path.start(1),module.start+path.end(1)))
            for call in re.finditer(r'\b(?:include|include_str|include_bytes)!\s*\(',masked):
                start=call.end();end=start;depth=1
                while end<len(masked) and depth:
                    depth+=(masked[end]=='(')-(masked[end]==')');end+=1
                protected.append((start,end))
                if re.search(r'\$\w+',masked[start:end]):
                    declarations=list(re.finditer(r'\bmacro_rules!\s*(\w+)\s*\{',masked[:call.start()]))
                    if declarations:
                        name=declarations[-1][1]
                        for invocation in re.finditer(r'\b'+re.escape(name)+r'!\s*\(\s*"([^"\n]*)"\s*\)',text):
                            if masked[invocation.start():invocation.start()+len(name)].strip():protected.append(invocation.span(1))
        def replace(match):
            original=match.group()
            if any(start<=match.start() and match.end()<=end for start,end in protected):return original
            token=original.replace("\\\\","/").replace("\\","/")
            # A sentence's trailing period is not part of a path.
            punctuation=""
            if token.endswith(".") and not token.endswith(".."):
                token=token[:-1];punctuation="."
            if token.startswith("res://"):
                mapped=self.assets.get(token)
                return mapped+punctuation if mapped else original
            first=token.lstrip('./').split('/',1)[0]
            if first not in self.path_parts and not token.startswith(self.root.as_posix()+"/"):
                return original
            absolute=token.startswith(self.root.as_posix()+"/")
            if absolute:
                key=token[len(self.root.as_posix())+1:]
                mapped=self.mapped(key)
                result=self.root.as_posix()+"/"+mapped if mapped else None
            else:
                mapped=self.mapped(token)
                result=mapped
                if result is None:
                    for base,style in ((oldp.parent,"relative"),(self.root/crate,"crate")):
                        candidate=normalized(base/token)
                        try:key=candidate.relative_to(self.root).as_posix()
                        except ValueError:continue
                        mapped=self.mapped(key)
                        if mapped:
                            if style=="relative":result=relative(self.root/mapped,newp.parent)
                            else:result=relative(self.root/mapped,self.root/self.crate(new))
                            break
            if result is None:return original
            if "\\" in original:
                result=result.replace("/","\\\\" if "\\\\" in original else "\\")
            return result+punctuation
        return PATH_TEXT.sub(replace,text)
