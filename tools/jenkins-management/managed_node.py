"""Bounded public8 Node preparation/execution; never a Cargo or shell fallback.

This module is part of the management capsule. The receiver owns identity and
delivery, ResourceManager owns admission, and the retained Windows Job owns
process-tree termination. Product source ownership stays with the public owner.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import time


ACTIONS = frozenset({"prepare-node", "validate-node"})
CAPABILITY = {"enabled": True, "schemaVersion": 1, "helper": "managed_node.py",
              "actions": sorted(ACTIONS), "package": "zircon-zui-plugin",
              "sourcePolicy": "actual-checkout-node-closure-v1", "formalAcceptance": False}
REPOSITORY = Path("E:/Git/ZirconEngine")
APP = REPOSITORY / "tools/penpot/apps/zircon-zui-plugin"
OUTPUT_ROOT = Path("E:/cargo-targets")
PUBLIC = OUTPUT_ROOT / "zircon-local/reports/penpot-unreal-01a0f04d"
DEPS = Path("D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot/node_modules")
NODE = Path("D:/cargo-targets/zircon-local/penpot-public/tools/node/node-v24.19.0-win-x64/node.exe")
NODE_SHA = "3602f2bb1a10f2cbab4c36886218a33c1ab3db87290e73b033c46c77147d0237"
VITEST = DEPS / ".pnpm/vitest@4.1.10_@types+node@2_ab335678513582c72db689a93f4931ca/node_modules/vitest/vitest.mjs"
VITEST_SHA = "39db22f579acf5639bbb17a261408debbde03f4692c0c439e77e7f13aeba74d6"
PLAYWRIGHT = DEPS / ".pnpm/playwright@1.62.1/node_modules/playwright/index.mjs"
BUILD_HELPER = PUBLIC / "ue-penpot-resume-20261023/local-execution/public-component-closure-01/managed-node-build.cjs"
BUILD_HELPER_SHA = "c41d4fa9481858cdbd99fac4b7187a9d1279b0a2417108d5de747ba7f7a5215d"
TREE_PATTERN = "SDK absent paint|SDK nil readback|nonempty optional SDK paint|unknown paint fields|rejects user "
CHARACTER_TABLE_PNG_SPECS = ("src/penpot-control-content.spec.ts", "src/penpot-table-controls.spec.ts",
                           "src/penpot-canvas-png.spec.ts")
CHARACTER_TABLE_PNG_PATTERN = (TREE_PATTERN + "|source-owned Table global geometry guard"
                               "|Canvas PNG SDK return diagnostics")
PUBLIC_SPECS = ("src/penpot-canonical-library.spec.ts", "src/penpot-native-component-copy.spec.ts",
                "src/penpot-native-component-ownership.spec.ts", "src/bridge/zui-component-review-host.spec.ts")
PUBLIC_CANONICAL_SPECS = ("tools/zui-layout-real-plugin-registration.spec.ts",
                          "tools/penpot-canonical-library-publication.spec.ts",
                          "src/penpot-canonical-library.spec.ts", "src/penpot-native-components.spec.ts",
                          "src/penpot-control-content.spec.ts", "src/penpot-capture-validation.spec.ts")
SEARCH_SPECS = ("tools/zui-layout-real-plugin-registration.spec.ts",
                "tools/penpot-canonical-library-publication.spec.ts", "src/penpot-canonical-library.spec.ts",
                "src/penpot-native-components.spec.ts", "src/penpot-control-content.spec.ts",
                "src/penpot-capture-validation.spec.ts", "src/penpot-field-controls.spec.ts",
                "src/penpot-export-guard.spec.ts", "src/penpot-authoring-layout.spec.ts")
STYLE_CODEC_SPECS = ("src/penpot-capture-validation.spec.ts", "src/bridge/zui-source-export.spec.ts",
                     "src/penpot-source-colors.spec.ts", "src/penpot-asset-renderer.spec.ts",
                     "src/penpot-tree-row-controls.spec.ts")
AI_PRIVATE_SPECS = ("src/bridge/zui-export-apply.spec.ts", "src/bridge/zui-component-review-host.spec.ts")
AI_EXPORT_SPECS = ("src/bridge/zui-source-export.spec.ts",)
AI_PRIVATE_READ_INPUTS = ("zircon_plugins/ai/editor/behavior_tree.zui",
                          "zircon_editor/assets/ui/theme/editor_base.zui",
                          "zircon_editor/assets/ui/theme/editor_material.zui",
                          "zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs")
REAL_UI_NODE_SOURCES = ("tools/zui-layout-real-ui.test.mjs", "tools/zui-layout-real-ui.mjs",
                        "tools/zui-layout-real-session-contract.mjs")
SESSION_CONTRACT_NODE_SOURCES = ("tools/zui-layout-real-session-contract.test.mjs",
                                 "tools/zui-layout-real-session-contract.mjs")
TREE_PROPERTY_SPECS = ("src/penpot-tree-row-controls.spec.ts", "src/penpot-text-overflow.spec.ts",
                       "src/penpot-control-content.spec.ts")
ACTOR_LAYOUT_SPECS = ("src/penpot-layout-allocation.spec.ts", "src/penpot-native-component-copy.spec.ts",
                      "src/penpot-field-controls.spec.ts", "src/penpot-control-content.spec.ts",
                      "src/penpot-asset-renderer.spec.ts", "src/penpot-native-painter-content.spec.ts",
                      "src/penpot-tree-row-controls.spec.ts", "src/penpot-canonical-library.spec.ts",
                      "src/penpot-export-guard.spec.ts")
COMPILE_HELPER_ROOT = OUTPUT_ROOT / "zircon-local/tmp/penpot-unreal-01a0f04d"
COMPILE_REPORT_ROOT = PUBLIC / "ue-penpot-resume-20261023/local-execution/public-component-closure-01"
COMPILE_OUTPUT_ROOT = OUTPUT_ROOT / "zircon-local/build/penpot-unreal-01a0f04d/penpot-plugin-production"
EXPORTER_REQUEST = "penpot-unreal-01a0f04d-actor-exporter-r01"
EXPORTER_RECIPE = "penpot-official-exporter-admission-r01"
EXPORTER_CANDIDATE = PUBLIC / "ue-penpot-resume-20261023/local-execution/public-component-closure-01/actor-native-exporter-route-r01/managed-build-request.json"
EXPORTER_CANDIDATE_SHA = "eac0fa0a524fbd3d71f994d088db69937a82052ed16816f7cc82e288d6059c27"
EXPORTER_SOURCE_TREES = tuple("third_party/penpot/" + name for name in (
    "exporter/src", "exporter/vendor", "common/src", "common/vendor", "common/resources"))
UI_AOT_SOURCE_TREES = ("third_party/penpot/plugins/libs/plugins-styles/src",)
UI_AOT_CONTROL_FILES = ("tools/penpot/angular.json", "tools/penpot/tsconfig.base.json",
                        "tools/penpot/apps/zircon-zui-plugin/tsconfig.app.json")
# Generated only by the admitted Jenkins phase, never supplied as command text.
UI_AOT_HELPER = r'''"use strict";
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto'),{createRequire}=require('node:module');
const cfg=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));
const repo='E:/Git/ZirconEngine',workspace=path.join(repo,'tools/penpot'),app=path.join(workspace,'apps/zircon-zui-plugin');
const dependencyRoot='D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot';
const build='E:/cargo-targets/zircon-local/build/penpot-unreal-01a0f04d/penpot-plugin-production';
const evidence='E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/field-number-publication-r01';
const out=path.join(build,'ui-field-number-r01'),entry=path.join(app,'src/main.ts'),receiptPath=path.join(evidence,'ui-compile-receipt.json');
const req=createRequire(path.join(dependencyRoot,'package.json')),sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const requireTrue=(v,m)=>{if(!v)throw Error(m)},hash=p=>sha(fs.readFileSync(p));
requireTrue(path.resolve(cfg.repo)===path.resolve(repo)&&path.resolve(cfg.mainApp)===path.resolve(app)&&path.resolve(cfg.dependencyRoot)===path.resolve(dependencyRoot)&&path.resolve(cfg.out)===path.resolve(build),'Fixed Angular package/configuration changed');
requireTrue((!fs.existsSync(out)||fs.readdirSync(out).length===0)&&!fs.existsSync(receiptPath),'Retained UI output cannot be overwritten');
function files(root){const result=[];for(const e of fs.readdirSync(root,{withFileTypes:true})){const p=path.join(root,e.name);requireTrue(!e.isSymbolicLink(),'UI source/output aliases are not admitted');if(e.isDirectory())result.push(...files(p));else{requireTrue(e.isFile(),'UI input must be a regular file');result.push(p)}}return result}
const controls=['angular.json','tsconfig.base.json','pnpm-lock.yaml','package.json'].map(n=>path.join(workspace,n)).concat(['tsconfig.json','tsconfig.app.json'].map(n=>path.join(app,n)));
const sourcePaths=[...files(path.join(app,'src')),...files(path.join(repo,'third_party/penpot/plugins/libs/plugins-styles/src')),...controls,__filename];
const sourceFacts=()=>Object.fromEntries([...new Set(sourcePaths)].sort().map(p=>[p,hash(p)]));
const before=sourceFacts(),write=(p,v)=>{fs.mkdirSync(path.dirname(p),{recursive:true});fs.writeFileSync(p,JSON.stringify(v,null,2)+'\n',{flag:'wx'})};
let receipt={schema:'managed-single-angular-ui/v1',status:'pending',project:'zircon-zui-plugin',entry,aot:true,watch:false,workerBuilt:false,publisherBuilt:false,sourceCopiesCreated:false,dependencyInstallation:false,sourceBeforeHashes:before};
(async()=>{
 const angular=JSON.parse(fs.readFileSync(path.join(workspace,'angular.json'),'utf8')),project=angular.projects['zircon-zui-plugin'];
 requireTrue(project.architect.build.builder==='@angular/build:application'&&project.architect.build.options.browser==='apps/zircon-zui-plugin/src/main.ts'&&project.architect.build.options.scripts.length===0,'Only the one current UI AOT entry is admitted');
 const ts=req('typescript'),parsed=ts.getParsedCommandLineOfConfigFile(path.join(app,'tsconfig.app.json'),{}, {...ts.sys,onUnRecoverableConfigFileDiagnostic:d=>{throw Error(ts.flattenDiagnosticMessageText(d.messageText,' '))}});
 requireTrue(parsed,'Actual UI TypeScript configuration did not parse');
 const aliases=Object.fromEntries(Object.entries(parsed.options.paths??{}).map(([k,v])=>[k,v.map(x=>path.resolve(parsed.options.pathsBasePath??workspace,x))]));aliases['*']=[path.join(dependencyRoot,'node_modules/*')];
 const corePath=req.resolve('@angular/core/package.json'),core=JSON.parse(fs.readFileSync(corePath,'utf8')),interop=core.exports['./rxjs-interop'].types;
 requireTrue(typeof interop==='string','Installed Angular interop types are missing');aliases['@angular/core/rxjs-interop']=[path.resolve(path.dirname(corePath),interop)];aliases['smol-toml']=[path.join(dependencyRoot,'node_modules/.pnpm/smol-toml@1.8.0/node_modules/smol-toml')];
 const compilerOptions={...parsed.options,paths:aliases,strict:true,noEmit:true,incremental:false};
 const program=ts.createProgram({rootNames:[entry],options:compilerOptions}),errors=ts.getPreEmitDiagnostics(program).filter(x=>x.category===ts.DiagnosticCategory.Error);
 requireTrue(errors.length===0,ts.formatDiagnosticsWithColorAndContext(errors,{getCanonicalFileName:p=>p,getCurrentDirectory:()=>workspace,getNewLine:()=> '\n'}));
 const programInputs=program.getSourceFiles().map(x=>path.resolve(x.fileName));
 const configPath=path.join(evidence,'ui-generated.tsconfig.json');write(configPath,{extends:path.join(app,'tsconfig.app.json'),compilerOptions:{outDir:path.join(out,'tsc'),incremental:false,strict:true,paths:aliases},files:[entry],include:[]});
 const options={...project.architect.build.options,...project.architect.build.configurations.production,outputPath:{base:out,browser:''},tsConfig:path.relative(workspace,configPath),polyfills:[req.resolve('zone.js')],aot:true,watch:false,progress:false,statsJson:true,sourceMap:false,deleteOutputPath:false,extractLicenses:true};
 requireTrue(options.optimization?.fonts===false,'Remote font optimization is outside this UI build');
 const {logging}=req('@angular-devkit/core'),logger=new logging.Logger('managed-single-angular-ui');logger.subscribe(e=>console.log(e.message));
 const teardowns=[],metadata={...project,cli:{cache:{enabled:false,path:path.join(evidence,'angular-cache')}}};
 const context={id:1,workspaceRoot:workspace,currentDirectory:workspace,target:{project:'zircon-zui-plugin',target:'build',configuration:'production'},logger,builder:{builderName:'@angular/build:application',description:'Managed single UI AOT',optionSchema:{}},getProjectMetadata:async()=>metadata,getBuilderNameForTarget:async()=>'@angular/build:application',getTargetOptions:async()=>options,reportStatus:()=>{},reportProgress:()=>{},addTeardown:f=>teardowns.push(f),validateOptions:async o=>o};
 const interopJs=req.resolve('@angular/core/rxjs-interop'),interopPlugin={name:'managed-existing-core-interop',setup(b){b.onResolve({filter:/^@angular\/core\/rxjs-interop$/},()=>({path:interopJs}))}};
 const {buildApplication}=req('@angular/build');let success=false;
 try{for await(const result of buildApplication(options,context,{codePlugins:[interopPlugin]})){console.log('MANAGED_SINGLE_UI_RESULT',JSON.stringify(result));requireTrue(result.success===true,'Angular UI AOT failed');success=true}}finally{for(const f of teardowns.reverse())await f()}
 requireTrue(success,'Angular UI AOT produced no successful result');
 const after=sourceFacts();requireTrue(JSON.stringify(before)===JSON.stringify(after),'UI source changed during AOT');
 const stats=JSON.parse(fs.readFileSync(path.join(out,'stats.json'),'utf8')),statInputs=[],virtualInputs=[];
 for(const name of Object.keys(stats.inputs??{})){const p=path.resolve(workspace,name);if(fs.existsSync(p)&&fs.statSync(p).isFile())statInputs.push(p);else{requireTrue(name.startsWith('angular:'),'Undiscovered Angular compiler input: '+name);virtualInputs.push(name)}}
 requireTrue(statInputs.length>0,'Angular AOT stats lack actual inputs');
 const allInputs=[...new Set([...sourcePaths,...programInputs,...statInputs,...Object.keys(require.cache)])].sort();
 const inputs=allInputs.map(p=>({path:p,sha256:hash(p)})),outputs=files(out).map(p=>({path:p,sha256:hash(p),bytes:fs.statSync(p).size}));
 const indexPath=path.join(out,'index.html'),index=fs.readFileSync(indexPath,'utf8');requireTrue(outputs.some(x=>/^main-[^/\\]+\.js$/.test(path.basename(x.path))&&index.includes(path.basename(x.path))),'UI index does not select its compiled main bundle');
 receipt={...receipt,status:'compiled-current',typescriptNoEmitPassed:true,typescriptStrictPassed:true,sourceUnchanged:true,sourceAfterHashes:after,inputs,programInputPaths:programInputs,statsInputPaths:statInputs,virtualInputs,outputs,indexSha256:hash(indexPath),configuration:{path:configPath,sha256:hash(configPath)}};
 write(receiptPath,receipt);console.log(JSON.stringify({status:receipt.status,project:receipt.project,inputs:inputs.length,outputs:outputs.length,indexSha256:receipt.indexSha256}));
})().catch(error=>{if(!fs.existsSync(receiptPath))write(receiptPath,{...receipt,status:'failed',error:String(error.stack??error)});console.error(error);process.exitCode=1});
'''
# Fixed product stages, materialized only after the Node resource/writer admission.
STYLE_CODEC_HELPER = r'''"use strict";
const fs = require('node:fs'), path = require('node:path'), crypto = require('node:crypto');
const {createRequire} = require('node:module'), {spawnSync} = require('node:child_process');
const repo = 'E:/Git/ZirconEngine', app = path.join(repo, 'tools/penpot/apps/zircon-zui-plugin');
const dependencyRoot = 'D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot';
const build = 'E:/cargo-targets/zircon-local/build/penpot-unreal-01a0f04d/penpot-plugin-production';
const evidence = 'E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/settings-style-codec-r01';
const control = 'E:/cargo-targets/zircon-local/tmp/penpot-unreal-01a0f04d';
const specs = ['src/penpot-capture-validation.spec.ts', 'src/bridge/zui-source-export.spec.ts',
  'src/penpot-source-colors.spec.ts', 'src/penpot-asset-renderer.spec.ts', 'src/penpot-tree-row-controls.spec.ts'];
const entry = path.join(app, 'src/plugin.ts'), receiptPath = path.join(evidence, 'worker-compile-receipt.json');
const output = path.join(build, 'runtime/assets/plugin-style-codec-r01.js');
const metafilePath = path.join(evidence, 'worker-metafile.json');
const testResult = path.join(evidence, 'managed-vitest-result.json');
const vitestConfig = path.join(control, 'settings-style-codec-vitest-r01.mjs');
const specConfig = path.join(control, 'settings-style-codec-tsconfig-r01.json');
const req = createRequire(path.join(dependencyRoot, 'package.json'));
const before = new Map(), requireTrue = (ok, message) => { if (!ok) throw Error(message); };
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
function remember(p) {
  p = path.resolve(p);
  requireTrue(fs.realpathSync.native(p).toLowerCase() === p.toLowerCase(), 'Aliased compiler input: ' + p);
  const digest = hash(p);
  requireTrue(!before.has(p) || before.get(p) === digest, 'Compiler input changed: ' + p);
  before.set(p, digest);
  return p;
}
function write(p, value) {
  fs.mkdirSync(path.dirname(p), {recursive:true});
  fs.writeFileSync(p, JSON.stringify(value, null, 2) + '\n', {flag:'wx'});
}
(async () => {
  const cfg = JSON.parse(fs.readFileSync(remember(process.argv[2]), 'utf8'));
  const expected = {repo, mainApp:app, dependencyRoot, out:build,
    playwright:path.join(dependencyRoot, 'node_modules/.pnpm/playwright@1.62.1/node_modules/playwright/index.mjs')};
  requireTrue(Object.keys(cfg).sort().join() === Object.keys(expected).sort().join() &&
    Object.keys(expected).every(k => path.resolve(cfg[k]) === path.resolve(expected[k])), 'Expanded style-codec configuration');
  for (const p of [output, receiptPath, metafilePath, testResult, path.join(evidence, 'managed-vitest-junit.xml')]) {
    requireTrue(!fs.existsSync(p), 'Retained output already exists: ' + p);
    fs.mkdirSync(path.dirname(p), {recursive:true});
    requireTrue(/^E:[\\/]cargo-targets[\\/]/i.test(p) &&
      fs.realpathSync.native(path.dirname(p)).toLowerCase() === path.resolve(path.dirname(p)).toLowerCase(),
      'Style-codec requires physical E output');
  }
  [__filename, vitestConfig, specConfig, entry, ...specs.map(p => path.join(app, p))].forEach(remember);
  const vitest = path.join(dependencyRoot, 'node_modules/.pnpm/vitest@4.1.10_@types+node@2_ab335678513582c72db689a93f4931ca/node_modules/vitest/vitest.mjs');
  remember(vitest);
  const test = spawnSync(process.execPath, [vitest, 'run', '--config', vitestConfig],
    {cwd:app, env:process.env, stdio:'inherit', windowsHide:true});
  requireTrue(!test.error && test.status === 0 && test.signal === null, 'Style-codec Vitest stage failed');
  const results = JSON.parse(fs.readFileSync(testResult, 'utf8'));
  const names = results.testResults?.map(row => path.resolve(row.name));
  requireTrue(results.success === true && names?.length === specs.length && new Set(names).size === specs.length &&
    specs.every(name => names.includes(path.join(app, name))) && results.testResults.every(row =>
      row.assertionResults?.length > 0 && row.assertionResults.every(item => item.status === 'passed')),
    'Style-codec requires actual unique discovery and no failed/skipped tests in every fixed spec');
  const ts = req('typescript'), esbuild = req('esbuild');
  const programs = [], programInputs = new Set();
  function typecheck(configPath, roots) {
    const read = p => { if (fs.existsSync(p)) remember(p); return ts.sys.readFile(p); };
    const config = ts.readConfigFile(configPath, read);
    requireTrue(!config.error, 'Cannot read strict TypeScript configuration');
    const parsed = ts.parseJsonConfigFileContent(config.config, {...ts.sys, readFile:read}, path.dirname(configPath));
    requireTrue(parsed.errors.length === 0, 'Invalid strict TypeScript configuration');
    const options = {...parsed.options, strict:true, noEmit:true, incremental:false,
      paths:{...parsed.options.paths,
        'vitest':[path.join(dependencyRoot, 'node_modules/.pnpm/vitest@4.1.10_@types+node@2_ab335678513582c72db689a93f4931ca/node_modules/vitest/dist/index.d.ts')],
        'smol-toml':[path.join(dependencyRoot, 'node_modules/.pnpm/smol-toml@1.8.0/node_modules/smol-toml')],
        '*':[path.join(dependencyRoot, 'node_modules/*')]}};
    const host = ts.createCompilerHost(options); host.readFile = read;
    const program = ts.createProgram(roots, options, host), diagnostics = ts.getPreEmitDiagnostics(program);
    requireTrue(diagnostics.length === 0, ts.formatDiagnosticsWithColorAndContext(diagnostics,
      {getCanonicalFileName:p=>p, getCurrentDirectory:()=>repo, getNewLine:()=> '\n'}));
    program.getSourceFiles().forEach(file => programInputs.add(remember(file.fileName)));
    programs.push({roots, strict:true, noEmit:true, incremental:false, diagnostics:0});
  }
  typecheck(specConfig, specs.slice(0, 2).map(p => path.join(app, p)));
  typecheck(path.join(app, 'tsconfig.plugin.json'), [entry]);
  const bundle = await esbuild.build({absWorkingDir:repo, entryPoints:[entry], outfile:output,
    bundle:true, minify:true, format:'esm', metafile:true, logLevel:'info',
    tsconfig:path.join(app, 'tsconfig.plugin.json'),
    nodePaths:[path.join(dependencyRoot, 'node_modules'), path.join(dependencyRoot, 'node_modules/.pnpm/smol-toml@1.8.0/node_modules')],
    plugins:[{name:'managed-style-codec-inputs', setup(build) {
      build.onLoad({filter:/\.[cm]?[jt]sx?$/}, args => { remember(args.path); return null; });
    }}]});
  const metaInputs = Object.keys(bundle.metafile.inputs).map(p => remember(path.resolve(repo, p)));
  Object.keys(require.cache).forEach(remember);
  const after = Object.fromEntries([...before].map(([p, digest]) => {
    requireTrue(hash(p) === digest, 'Style-codec input changed during execution: ' + p); return [p, digest];
  }));
  write(metafilePath, bundle.metafile);
  write(receiptPath, {schema:'managed-source-style-codec/v1', status:'compiled-current',
    typescriptNoEmitPassed:true, typescriptStrictPassed:true, sourceUnchanged:true, sourceCopiesCreated:false,
    specs, programs, entries:[entry], workerBuilt:true, publisherBuilt:false, runnerBuilt:false,
    angularBuilt:false, dependencyInstallation:false, deployed:false, formalAcceptance:false,
    inputBeforeHashes:Object.fromEntries(before), inputAfterHashes:after,
    inputs:[...before].map(([path, sha256]) => ({path, sha256})),
    programInputPaths:[...programInputs], metafileInputPaths:metaInputs,
    testResult:{path:testResult, sha256:hash(testResult)},
    output:{path:output, sha256:hash(output)}});
})().catch(error => {console.error(error); process.exitCode = 1;});
'''
COMPILE_RECIPES = {
    "style-codec": {"helper": "settings-style-codec-managed-r01.cjs",
                    "sha256": hashlib.sha256(STYLE_CODEC_HELPER.encode()).hexdigest(),
                    "entry": "src/plugin.ts", "bundle": "runtime/assets/plugin-style-codec-r01.js",
                    "receipt": "settings-style-codec-r01/worker-compile-receipt.json",
                    "metafile": "settings-style-codec-r01/worker-metafile.json"},
    "ui-aot": {"helper": "field-number-publication-angular-managed.cjs",
                "sha256": hashlib.sha256(UI_AOT_HELPER.encode()).hexdigest(), "entry": "src/main.ts",
                "bundle": "ui-field-number-r01/index.html",
                "receipt": "field-number-publication-r01/ui-compile-receipt.json",
                "metafile": "ui-field-number-r01/stats.json"},
    "publisher": {"helper": "material-library-build-tool-r01.cjs",
                  "sha256": "3a36a3c607093d52ee5b12a9b7cc3a9b50028cece212432bd3b7e75c4719e94d",
                  "entry": "tools/penpot-canonical-library-publication.ts",
                  "bundle": "tools/penpot-canonical-library-publication-incremental-r1.mjs",
                  "receipt": "material-tool-compile-r01.json", "metafile": "material-tool-metafile-r01.json"},
    "worker": {"helper": "material-library-build-worker-r01.cjs",
               "sha256": "826df09c9c596a83c9c2943df53191b6e5e26b767c78c59a91d2529eae1ea844",
               "entry": "src/plugin.ts", "bundle": "runtime/assets/plugin-canonical-library-material-r1.js",
               "receipt": "material-worker-compile-r01.json", "metafile": "material-worker-metafile-r01.json"},
    "layout-worker": {"helper": "actor-content-allocation-build-worker-r4.cjs",
                      "sha256": "56227a0857c740c3866b05e1a0e23f12e9715285b536f020cecc0a78625b7cc9",
                      "entry": "src/plugin.ts", "bundle": "runtime/assets/plugin-canonical-layout-r4.js",
                      "receipt": "actor-content-allocation-r01/worker-compile-receipt.json",
                      "metafile": "actor-content-allocation-r01/worker-metafile.json"},
    "session-runner": {"helper": "existing-plugin-session-runner-r02.cjs",
                       "sha256": "41009ec313e90f0361fcdbfa524c9e5f6350ef773f2046a2ea9f8c575a07f775",
                       "entry": "tools/zui-layout-real-session.ts",
                       "bundle": "public-component-closure-current/existing-plugin-session-r01/tools/zui-layout-real-session.mjs",
                       "receipt": "existing-plugin-session-r01/runner-compile-receipt.json",
                       "metafile": "existing-plugin-session-r01/runner-metafile.json"},
}
RECIPES = {
    "penpot-source-style-codec-r01": {"specs": list(STYLE_CODEC_SPECS), "compile": "style-codec",
        "pattern": None, "expectedPassed": None, "requireEachSpec": True,
        "requireUniqueSpecResults": True, "timeout": 180},
    EXPORTER_RECIPE: {"exporterAdmission": True, "timeout": 180},
    "penpot-real-ui-node-tests-r02": {"specs": [REAL_UI_NODE_SOURCES[0]], "builtin": True,
                                      "expectedPassed": 14, "timeout": 120},
    "penpot-existing-session-contract-tests-r01": {"specs": [SESSION_CONTRACT_NODE_SOURCES[0]],
        "builtin": True, "builtinSources": list(SESSION_CONTRACT_NODE_SOURCES),
        "expectedPassed": None, "timeout": 90},
    "penpot-existing-session-registration-r01": {"specs": ["tools/zui-layout-real-plugin-registration.spec.ts"],
        "pattern": None, "expectedPassed": None, "requireEachSpec": True,
        "requireUniqueSpecResults": True, "timeout": 90},
    "penpot-existing-session-runner-r01": {"specs": [], "compile": "session-runner", "timeout": 180},
    "penpot-ai-private-source-tests-r03": {"specs": list(AI_PRIVATE_SPECS),
        "configSpecs": list(AI_PRIVATE_SPECS + AI_EXPORT_SPECS), "commandSpecFilter": True,
        "pattern": "registered private AI source", "expectedPassed": 21,
        "requireEachSpec": True, "requireUniqueSpecResults": True,
        "readOnlyInputs": list(AI_PRIVATE_READ_INPUTS), "aiPrivateCache": True, "timeout": 120},
    "penpot-ai-source-export-tests-r03": {"specs": list(AI_EXPORT_SPECS),
        "configSpecs": list(AI_PRIVATE_SPECS + AI_EXPORT_SPECS), "commandSpecFilter": True,
        "pattern": "no-edit export|missing captured ownership|generated prefab internals|another asset identity",
        "expectedPassed": 4, "requireEachSpec": True, "requireUniqueSpecResults": True,
        "readOnlyInputs": list(AI_PRIVATE_READ_INPUTS), "aiPrivateCache": True, "timeout": 90},
    "penpot-tree-property-consumers-r02": {"specs": list(TREE_PROPERTY_SPECS), "pattern": None,
                                          "expectedPassed": None, "requireEachSpec": True, "timeout": 120},
    "penpot-settings-property-consumers-r01": {"specs": ["tools/zui-layout-dependencies.spec.ts"],
        "pattern": "original PropertyEditorRow height variants and typed value slot",
        "expectedPassed": 4, "requireEachSpec": True, "timeout": 120},
    "penpot-actor-content-allocation-tests-r02": {"specs": list(ACTOR_LAYOUT_SPECS), "pattern": None,
                                               "expectedPassed": None, "requireEachSpec": True, "timeout": 180},
    "penpot-character-tree-row-r1": {"specs": ["src/penpot-control-content.spec.ts"],
                                      "pattern": TREE_PATTERN, "expectedPassed": 31, "timeout": 120},
    "penpot-character-table-png-tests-r02": {"specs": list(CHARACTER_TABLE_PNG_SPECS),
        "pattern": CHARACTER_TABLE_PNG_PATTERN, "expectedPassed": 46,
        "expectedPassedBySpec": dict(zip(CHARACTER_TABLE_PNG_SPECS, (31, 8, 7))),
        "requireEachSpec": True, "requireUniqueSpecResults": True, "timeout": 120},
    "penpot-public8-tests-r1": {"specs": list(PUBLIC_SPECS), "pattern": None,
                               "expectedPassed": None, "timeout": 180},
    "penpot-public-canonical-tests-r03": {"specs": list(PUBLIC_CANONICAL_SPECS), "pattern": None,
        "expectedPassed": None, "requireEachSpec": True, "requireUniqueSpecResults": True, "timeout": 180},
    "penpot-public8-worker-r1": {"specs": [], "timeout": 180},
    "penpot-search-native-fill-tests-r1": {"specs": list(SEARCH_SPECS), "pattern": None,
                                          "expectedPassed": None, "requireEachSpec": True, "timeout": 180},
    "penpot-canonical-publisher-r1": {"specs": [], "compile": "publisher", "timeout": 180},
    "penpot-canonical-worker-r1": {"specs": [], "compile": "worker", "timeout": 180},
    "penpot-canonical-layout-worker-r4": {"specs": [], "compile": "layout-worker", "timeout": 180},
    "penpot-field-number-single-ui-aot-r01": {"specs": [], "compile": "ui-aot", "timeout": 180},
}
SOURCE_TREES = ("tools/penpot/apps/zircon-zui-plugin/src", "tools/penpot/apps/zircon-zui-plugin/tools",
                "tools/penpot/packages", "zircon_editor/assets/ui/editor",
                "third_party/penpot/plugins/libs/plugin-types")
CONTROL_FILES = ("tools/penpot/package.json", "tools/penpot/pnpm-lock.yaml", "tools/penpot/pnpm-workspace.yaml",
                 "tools/penpot/tsconfig.json", "tools/penpot/tsconfig.base.json",
                 "tools/penpot/apps/zircon-zui-plugin/package.json",
                 "tools/penpot/apps/zircon-zui-plugin/tsconfig.json",
                 "tools/penpot/apps/zircon-zui-plugin/tsconfig.plugin.json",
                 "tools/penpot/apps/zircon-zui-plugin/tsconfig.layout-visual.json")
MAX_INPUT_BYTES = 512 * 1024**2
PREPARE_SECONDS = 180


class NodePolicyError(ValueError):
    code = "managed_node_policy"


def require(value, message):
    if not value:
        raise NodePolicyError(message)


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
                      allow_nan=False).encode("utf-8")


def digest(value):
    return hashlib.sha256(encoded(value)).hexdigest()


def unique_json(raw):
    def pairs(items):
        value = {}
        for key, item in items:
            require(key not in value, "Duplicate Node configuration field")
            value[key] = item
        return value
    return json.loads(raw, object_pairs_hook=pairs,
                      parse_constant=lambda _value: require(False, "Nonfinite Node configuration"))


def physical(value, *, under=None, exists=False):
    path = Path(value)
    require(path.is_absolute() and ".." not in path.parts and not any(c in str(path) for c in "\0\r\n"),
            "Node paths must be literal absolute paths")
    for parent in (path, *path.parents):
        if parent.exists() or parent.is_symlink():
            facts = parent.lstat()
            require(not stat.S_ISLNK(facts.st_mode) and not getattr(facts, "st_file_attributes", 0) & 0x400,
                    "Node input/output aliases are not permitted")
    require(not exists or path.exists(), "A declared Node input is missing: " + str(path))
    require(under is None or path.is_relative_to(under), "Node path escaped its fixed root")
    return path


def file_hash(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def reference(value, *, under):
    require(isinstance(value, dict) and set(value) == {"path", "sha256"}
            and isinstance(value["sha256"], str) and re.fullmatch("[0-9a-f]{64}", value["sha256"]),
            "Node configuration needs its exact path and SHA256")
    path = physical(value["path"], under=under, exists=True)
    require(path.is_file() and file_hash(path) == value["sha256"], "Node configuration bytes changed")
    return path


def recipe(payload, repo):
    require(Path(repo) == REPOSITORY, "The public Node package uses the canonical checkout")
    allowed = {"action", "repositoryId", "sessionId", "requestId", "operationId", "driverDigest",
               "runtimeOperationId", "ownedPaths", "nodeRecipe", "sourceHashes", "dependencyHashes",
               "preparationRef", "observationOnly"}
    require(isinstance(payload, dict) and payload.get("action") in ACTIONS and not set(payload) - allowed,
            "Only the typed Node identity/recipe schema is accepted")
    selection = payload.get("nodeRecipe")
    require(isinstance(selection, dict) and set(selection) == {"schemaVersion", "id", "config"}
            and type(selection.get("schemaVersion")) is int and selection["schemaVersion"] == 1
            and selection.get("id") in RECIPES, "Unknown or expanded Node recipe")
    authored = payload.get("sourceHashes")
    require(isinstance(authored, dict) and authored, "Current source-owner hashes are required")
    for name, checksum in {**authored, **payload.get("dependencyHashes", {})}.items():
        require(isinstance(name, str) and not name.startswith((".", "/")) and "\\" not in name
                and ":" not in name and not set(name.split("/")) & {"", ".", ".."}
                and isinstance(checksum, str) and re.fullmatch("[0-9a-f]{64}", checksum),
                "Node source declarations require exact checkout paths and hashes")
    require(payload.get("action") != "validate-node" or isinstance(payload.get("preparationRef"), dict),
            "Node execution requires the original completed preparation receipt")
    identifier, rule = selection["id"], RECIPES[selection["id"]]
    for name in rule.get("readOnlyInputs", ()):
        require(name in payload.get("dependencyHashes", {}),
                "The private AI recipe requires its exact reviewed read-only dependency: " + name)
        require(not any(name == owned or name.startswith(owned.rstrip("/") + "/")
                        for owned in payload.get("ownedPaths", ())),
                "The private AI recipe cannot claim write ownership of a read-only dependency")
    config_path = reference(selection["config"], under=OUTPUT_ROOT)
    if rule.get("exporterAdmission"):
        require(payload["action"] == "prepare-node" and payload.get("requestId") == EXPORTER_REQUEST,
                "Exporter admission belongs to its original request and prepare-node only; compilation is not admitted")
        require(config_path == EXPORTER_CANDIDATE and selection["config"]["sha256"] == EXPORTER_CANDIDATE_SHA,
                "Exporter admission requires the unchanged original candidate")
        candidate = unique_json(config_path.read_text(encoding="utf-8-sig"))
        require(candidate.get("requestId") == EXPORTER_REQUEST and candidate.get("submitted") is False,
                "Original exporter proposal identity changed")
        return {"id": identifier, "exporterAdmission": True, "package": "penpot/exporter",
                "config": str(config_path), "candidateSha256": EXPORTER_CANDIDATE_SHA,
                "timeoutSeconds": rule["timeout"], "productCompilationSupported": False}
    raw = config_path.read_text(encoding="utf-8-sig").strip()
    worker = identifier == "penpot-public8-worker-r1"
    builtin = rule.get("builtin", False)
    compile_kind = rule.get("compile")
    compile_helper = None
    if builtin:
        cfg = unique_json(raw)
        require(set(cfg) == {"repo", "spec", "outputFile"}
                and Path(cfg["repo"]) == REPOSITORY and cfg["spec"] == rule["specs"][0],
                "The built-in Node recipe selects only its one fixed test file")
        for name in rule.get("builtinSources", REAL_UI_NODE_SOURCES):
            require((APP / name).relative_to(REPOSITORY).as_posix() in authored,
                    "The built-in Node test requires current source-owner hashes for its local modules")
        outputs = {"result": physical(cfg["outputFile"], under=PUBLIC)}
        require(outputs["result"] != PUBLIC and outputs["result"].suffix == ".json",
                "Node output destinations must be task scoped")
        command = [str(NODE), "--test", "--test-reporter=tap", str(APP / rule["specs"][0])]
    elif compile_kind:
        cfg = unique_json(raw)
        require(set(cfg) == {"repo", "mainApp", "dependencyRoot", "out", "playwright"}
                and Path(cfg["repo"]) == REPOSITORY and Path(cfg["mainApp"]) == APP
                and Path(cfg["dependencyRoot"]) == DEPS.parent and Path(cfg["playwright"]) == PLAYWRIGHT
                and Path(cfg["out"]) == COMPILE_OUTPUT_ROOT,
                "Canonical compilation accepts only its fixed package, dependencies and physical E output")
        compile_rule = COMPILE_RECIPES[compile_kind]
        compile_helper = physical(COMPILE_HELPER_ROOT / compile_rule["helper"], under=OUTPUT_ROOT,
                                  exists=compile_kind not in {"ui-aot", "style-codec"})
        if compile_helper.exists():
            require(file_hash(compile_helper) == compile_rule["sha256"], "The reviewed canonical compile helper changed")
        entry = (APP / compile_rule["entry"]).relative_to(REPOSITORY).as_posix()
        require(entry in authored, "The selected compile entry requires its current source-owner hash")
        if compile_kind == "style-codec":
            require(all((APP / name).relative_to(REPOSITORY).as_posix() in authored
                        for name in STYLE_CODEC_SPECS),
                    "Style-codec requires current owner hashes for all five unfiltered specs")
        outputs = {"bundle": physical(COMPILE_OUTPUT_ROOT / compile_rule["bundle"], under=OUTPUT_ROOT),
                   "compileReceipt": physical(COMPILE_REPORT_ROOT / compile_rule["receipt"], under=OUTPUT_ROOT),
                   "metafile": physical((COMPILE_OUTPUT_ROOT if compile_kind == "ui-aot" else COMPILE_REPORT_ROOT)
                                        / compile_rule["metafile"], under=OUTPUT_ROOT)}
        if compile_kind == "ui-aot":
            outputs["build"] = physical(COMPILE_OUTPUT_ROOT / "ui-field-number-r01", under=OUTPUT_ROOT)
        elif compile_kind == "style-codec":
            evidence = COMPILE_REPORT_ROOT / "settings-style-codec-r01"
            outputs["testResult"] = physical(evidence / "managed-vitest-result.json", under=OUTPUT_ROOT)
            outputs["junit"] = physical(evidence / "managed-vitest-junit.xml", under=OUTPUT_ROOT)
        command = [str(NODE), str(compile_helper), str(config_path)]
    elif worker:
        cfg = unique_json(raw)
        require(set(cfg) == {"repo", "mainApp", "dependencyRoot", "out", "playwright", "outputRoot", "expectedInputs"}
                and Path(cfg["repo"]) == REPOSITORY and Path(cfg["mainApp"]) == APP
                and Path(cfg["dependencyRoot"]) == DEPS.parent and Path(cfg["playwright"]) == PLAYWRIGHT,
                "Worker build accepts only the two existing public entry points")
        require(file_hash(physical(BUILD_HELPER, exists=True)) == BUILD_HELPER_SHA, "The pinned public worker helper changed")
        outputs = {"build": physical(cfg["outputRoot"], under=OUTPUT_ROOT)}
        require(outputs["build"].is_relative_to(OUTPUT_ROOT / "zircon-local/build/penpot-unreal-01a0f04d"),
                "Worker output requires the public owner's physical E build root")
        require(Path(cfg["out"]).is_relative_to(OUTPUT_ROOT / "zircon-local/build/penpot-unreal-01a0f04d"),
                "Worker output parent escaped its fixed root")
        require(isinstance(cfg["expectedInputs"], list) and cfg["expectedInputs"], "Worker source inputs are required")
        for item in cfg["expectedInputs"]:
            require(isinstance(item, dict) and {"path", "sha256"} <= set(item) <= {"path", "sha256", "byteLength"},
                    "Worker input declarations are invalid")
            input_path = reference({"path": item["path"], "sha256": item["sha256"]}, under=None)
            require(input_path.is_relative_to(REPOSITORY) or input_path.is_relative_to(PUBLIC)
                    or input_path.is_relative_to(OUTPUT_ROOT / "zircon-local/tmp/penpot-unreal-01a0f04d"),
                    "Worker helper inputs escaped the declared public source/control roots")
        command = [str(NODE), str(BUILD_HELPER), str(config_path)]
    else:
        require(raw.startswith("export default ") and raw.endswith(";"), "Vitest configuration must be literal JSON")
        cfg = unique_json(raw[len("export default "):-1])
        require(set(cfg) == {"root", "cacheDir", "resolve", "test"} and Path(cfg["root"]) == APP,
                "Vitest root or configuration controls changed")
        aliases = {"vitest": str(VITEST.parent / "dist/index.js"),
                   "smol-toml": str(DEPS / ".pnpm/smol-toml@1.8.0/node_modules/smol-toml/dist/index.js"),
                   "playwright": str(PLAYWRIGHT), "feather-icons/dist/icons.json": str(DEPS / "feather-icons/dist/icons.json")}
        require(set(cfg["resolve"]) == {"alias"} and set(cfg["resolve"]["alias"]) == set(aliases)
                and all(Path(cfg["resolve"]["alias"][name]) == Path(path) for name, path in aliases.items()),
                "Vitest aliases must use the existing pinned dependency packages")
        test = cfg["test"]
        expected = {"watch": False, "globals": True, "environment": "node", "maxWorkers": 1,
                    "fileParallelism": False, "include": rule.get("configSpecs", rule["specs"]), "reporters": ["default", "json"],
                    "coverage": {"enabled": False}, "testTimeout": 15000, "pool": "forks"}
        configured_pattern = rule["pattern"] is not None and not rule.get("commandSpecFilter")
        extra = {"outputFile"} | ({"testNamePattern"} if configured_pattern else set())
        require(set(test) == set(expected) | extra and encoded({k: test.get(k) for k in expected}) == encoded(expected),
                "Vitest permits only its fixed spec/filter and one-worker nonwatch recipe")
        require(not configured_pattern or test["testNamePattern"] == rule["pattern"], "Vitest test-name scope expanded")
        require(set(test["outputFile"]) == {"json"}, "Vitest has one JSON result destination")
        cache = physical(cfg["cacheDir"], under=OUTPUT_ROOT if rule.get("aiPrivateCache") else PUBLIC)
        if rule.get("aiPrivateCache"):
            require(cache == OUTPUT_ROOT / "zircon-local/cache/penpot-unreal-01a0f04d/vitest-focused",
                    "The private AI recipe uses only its original physical E focused cache")
        outputs = {"cache": cache,
                   "result": physical(test["outputFile"]["json"], under=PUBLIC)}
        require(outputs["cache"] != PUBLIC and outputs["result"] != PUBLIC
                and outputs["result"].suffix == ".json", "Node output destinations must be task scoped")
        for spec in rule.get("configSpecs", rule["specs"]):
            relative = (APP / spec).relative_to(REPOSITORY).as_posix()
            require(relative in authored, "The selected spec requires its current source-owner hash")
        command = [str(NODE), str(VITEST), "run", "--config", str(config_path)]
        if rule.get("commandSpecFilter"):
            command.extend([*rule["specs"], "-t", rule["pattern"]])
    return {"id": identifier, "rule": rule, "config": str(config_path), "configuration": cfg,
            "worker": worker, "builtin": builtin, "compileKind": compile_kind,
            "compileHelper": str(compile_helper) if compile_helper else None,
            "outputFiles": [name for name in outputs if name != "build"] if compile_kind else ["result"],
            "command": command, "cwd": str(REPOSITORY if worker or compile_kind or builtin else APP),
            "timeoutSeconds": rule["timeout"], "outputs": {k: str(v) for k, v in outputs.items()}}


def _tree_files(root, *, deadline):
    if not root.exists():
        return
    for directory, dirs, files in os.walk(physical(root, exists=True), followlinks=False):
        require(time.monotonic() < deadline, "Node source/dependency discovery timed out; capture is incomplete")
        dirs[:] = sorted(name for name in dirs if name not in {"node_modules", ".git", "dist", ".cache"})
        for name in dirs:
            physical(Path(directory) / name, exists=True)
        for name in sorted(files):
            yield physical(Path(directory) / name, exists=True)


def inspect_exporter_admission(candidate_path):
    """Inspect canonical inputs only; this cannot build, stage or approve reuse."""
    path = physical(candidate_path, under=OUTPUT_ROOT, exists=True)
    require(path == EXPORTER_CANDIDATE and file_hash(path) == EXPORTER_CANDIDATE_SHA,
            "Original exporter candidate changed before its native inspection")
    candidate = unique_json(path.read_text(encoding="utf-8-sig"))
    deadline = time.monotonic() + 150
    def census():
        files, roots, total = {}, {}, 0
        for tree in EXPORTER_SOURCE_TREES:
            root = physical(REPOSITORY / tree, under=REPOSITORY)
            roots[str(root)] = root.is_dir()
            require(not root.exists() or root.is_dir(), "An exporter source root is not a directory")
            for source in _tree_files(root, deadline=deadline):
                total += source.stat().st_size
                require(total <= MAX_INPUT_BYTES and len(files) < 50000,
                        "Exporter source inspection exceeded its bounded inventory")
                files[str(source)] = file_hash(source)
        return files, roots
    before, before_roots = census()
    rows, blockers = [], []
    for group in ("officialSources", "configs", "tools"):
        for item in candidate[group]:
            require(time.monotonic() < deadline, "Exporter input inspection timed out; no preparation is accepted")
            source = physical(item["path"], exists=True)
            actual = file_hash(source)
            expected = (candidate["sourcePatchAdmission"]["afterSha256"]
                        if source == REPOSITORY / candidate["sourcePatchAdmission"]["path"]
                        else item["sha256"])
            rows.append({"group": group, "path": str(source), "expectedSha256": item["sha256"],
                         "requiredCurrentSha256": expected, "actualSha256": actual,
                         "matches": actual == expected})
            if actual != expected:
                blockers.append({"code": "candidate_input_revision_changed", "path": str(source),
                                 "expected": expected, "actual": actual})
    patch = candidate["sourcePatchAdmission"]
    http = physical(REPOSITORY / patch["path"], under=REPOSITORY, exists=True)
    actual_http = file_hash(http)
    if actual_http != patch["afterSha256"]:
        blockers.append({"code": "exporter_http_source_revision_pending", "path": str(http),
                         "requiredAfter": patch["afterSha256"], "actual": actual_http,
                         "ownerThreadId": candidate["ownerThreadId"]})
    generated = candidate["generatedEnumReuse"]["candidate"]
    generated_path = physical(generated["path"], exists=True)
    generated_actual = file_hash(generated_path)
    # The r01 candidate contains a provenance gate, not a generation receipt.
    blockers.append({"code": "exporter_generated_enum_provenance_unproven",
                     "path": str(generated_path), "actualSha256": generated_actual,
                     "expectedSha256": generated["sha256"], "bytesMatch": generated_actual == generated["sha256"],
                     "required": "Original Jenkins generator/source closure, native and artifact reuse receipts; equality is not admission"})
    blockers.append({"code": "exporter_compiler_closure_unprepared",
                     "required": "Qualified Node production dependency resolution, actual compiler tools and Maven basis; exclude repo common/target/classes",
                     "resolvedClasspath": None, "sourceWriterAdmission": None})
    after, after_roots = census()
    if before != after or before_roots != after_roots:
        blockers.append({"code": "exporter_source_membership_changed_during_inspection",
                         "changed": sorted(p for p in set(before) | set(after) if before.get(p) != after.get(p)),
                         "changedRoots": sorted(p for p in before_roots if before_roots[p] != after_roots[p])})
    return {"schemaVersion": 1, "status": "blocked", "requestId": EXPORTER_REQUEST,
            "candidateSha256": EXPORTER_CANDIDATE_SHA, "sourceInputs": rows,
            "sourceCensus": {"files": after, "rootPresence": after_roots,
                             "missingConfiguredRoots": [p for p, present in after_roots.items() if not present],
                             "beforeDigest": digest({"files": before, "roots": before_roots}),
                             "afterDigest": digest({"files": after, "roots": after_roots}),
                             "observedStable": before == after and before_roots == after_roots,
                             "compilerClosureComplete": False},
            "httpAfterMatches": actual_http == patch["afterSha256"], "blockedBy": blockers,
            "captureReady": False, "productCompilationSupported": False, "productCommandsExecuted": [],
            "sourceCopiesCreated": 0, "generatedEnumReused": False, "serviceControlPerformed": False}


def run_exporter_admission(m, plan):
    """Original ordinary intake, bounded metadata reservation and retained native evidence."""
    from tools.jenkins.resources.capacity import Capacity, ResourceManager
    from tools.jenkins.state.locks import process_lock
    report = physical(OUTPUT_ROOT / "zircon-local/management-node/reports" / m.key, under=OUTPUT_ROOT)
    owner, writer = "management-node-" + m.key, "management-node-exporter-admission-output"
    require(not report.exists() and all(m.state.get(d, owner) is None for d in
            ("execution", "execution_launch", "management_checkout_read")), "Original exporter admission requires reconciliation")
    policy = m.state.get("resource_policy", "default")
    resource = ResourceManager(m.state, Capacity(policy["payload"]["cpuBudget"],
                               policy["payload"]["memoryBudget"], policy["payload"]["diskBudget"]))
    # This is a read/metadata budget, not a reservation for Node/Java/CLJS builds.
    amount = Capacity(1, 512 * 1024**2, 16 * 1024**2)
    reservation, launched, terminal, execution = None, False, False, None
    released, proof_id = False, None
    try:
        helper = _load(m._node_client_path(), "jenkins_exporter_admission_native")
        with process_lock(m.repo / ".jenkins/state/locks/management-git.lock", reason="management_git_busy", timeout=0):
            require(m.checkout_git_read_conflict(unknown_only=True) is None, "An unknown source reader needs reconciliation")
            m._node_write_json(report / "artifacts.json", {"ownerSession": m.session, "taskId": m.request,
                "operationId": m.operation, "sourceRoot": str(m.repo), "reportRoot": str(report),
                "sourceCopiesCreated": 0, "productOutputs": [],
                "retention": "Keep original admission/native/resource blockers; no build or reuse performed"})
            resource.queue(owner, request_id=owner, amount=amount, build_root=str(OUTPUT_ROOT), heavy_writer=True, writer_key=writer)
            inventory, generation = resource.scan_inventory(str(OUTPUT_ROOT))
            reservation = resource.admit(owner, amount, inventory_generation=generation, request_id=owner,
                                         build_root=str(OUTPUT_ROOT), heavy_writer=True, writer_key=writer)
            inventory_ref = m._node_write_json(report / "resource-inspection.json", {
                "policyVersion": policy["version"], "inventory": inventory, "inventoryGeneration": generation,
                "metadataBudget": amount.__dict__, "metadataReservation": reservation,
                "productReservation": None, "productPoolCacheWriterAdmitted": False})
            if reservation is None:
                queued = m.state.get("resource_queue", owner + ":" + owner)
                require(queued is not None and queued["payload"].get("status") == "waiting",
                        "Original metadata resource queue cannot be reconciled")
                m.state.put("resource_queue", queued["key"], {**queued["payload"], "status": "cancelled",
                            "reason": "one-shot exporter inspection capacity not admitted"}, expected_version=queued["version"])
                result = {"status": "complete", "exporterAdmissionStatus": "blocked", "reasonCode": "resource_capacity",
                          "resourceRef": inventory_ref, "nodeStarted": False, "productStarted": False,
                          "captureReady": False, "productReservation": None, "productCompilationSupported": False,
                          "productPoolCacheWriterAdmitted": False, "formalAcceptance": False}
                result["receipt"] = m._node_write_json(report / "receipt.json", result)
                return result
            execution = {"kind": "management_exporter_admission", "status": "preparing", "executionId": owner,
                         "requestId": m.request, "operationId": m.operation, "sessionId": m.session,
                         "inputDigest": m.input_digest, "reservationKey": reservation["key"], "report": str(report)}
            m.state.put("execution", owner, execution, expected_version=0)
            m._node_write_json(report / "request.json", {"payload": m.payload, "plan": plan})
            command = [str(m.repo / ".jenkins/runtime/python/python.exe"), "-B", "-X", "utf8",
                       str(Path(__file__).resolve()), "exporter-admission", plan["config"]]
            environment = {k: os.environ[k] for k in ("SYSTEMROOT", "WINDIR", "COMSPEC", "JOB_NAME", "NODE_NAME") if k in os.environ}
            environment.update({"PYTHONDONTWRITEBYTECODE": "1", "TEMP": str(report), "TMP": str(report), "TMPDIR": str(report)})
            m.state.put("management_checkout_read", owner, {**execution, "status": "active", "pinsReleased": False,
                "readKind": "metadata-inspection", "completeCompilerSourceGuard": False}, expected_version=0)
            m.state.put("execution_launch", owner, {**execution, "status": "claimed", "command": command}, expected_version=0)
        launched = True
        native = helper._run_owned_command(command, m.repo, environment, report, plan["timeoutSeconds"])
        native_ref = m._node_write_json(report / "native-terminal.json", native)
        proof = settled_native_proof(native, native_ref, m._node_native_proof)
        require(proof["complete"] is True and native["command"] == command and native["cwd"] == str(m.repo),
                "Exporter inspection native root/children/EOF remain unproven")
        terminal = True
        proof_id = owner + "-native"
        m.state.put("native_job", proof_id, {"owner": owner, "executionId": owner, "status": "terminal",
                    "role": "management_exporter_admission", "completeProof": proof, "nativeEvidence": native}, expected_version=0)
        details, inspection_error = None, None
        if native["rootExitCode"] == 0 and not native.get("cleanup", {}).get("attempted"):
            try:
                details = unique_json((report / "stdout.bin").read_text(encoding="utf-8-sig"))
                require(details.get("status") == "blocked" and details.get("requestId") == m.request
                        and details.get("candidateSha256") == plan["candidateSha256"]
                        and details.get("captureReady") is False and details.get("productCompilationSupported") is False
                        and details.get("productCommandsExecuted") == [] and details.get("sourceCopiesCreated") == 0
                        and details.get("generatedEnumReused") is False and details.get("serviceControlPerformed") is False
                        and isinstance(details.get("blockedBy"), list) and details["blockedBy"],
                        "Exporter inspection did not return its restricted, non-build evidence")
            except Exception as error:
                details, inspection_error = None, str(error)
        result = {"status": "complete" if details is not None else "failed", "exporterAdmissionStatus": "blocked",
                  "reasonCode": "exporter_source_provenance_pending" if details is not None else "exporter_inspection_failed",
                  "details": details, "inspectionError": inspection_error, "nativeTerminal": native_ref,
                  "nativeProof": proof, "nativeProofRef": proof_id, "resourceRef": inventory_ref,
                  "nodeStarted": False, "productStarted": False, "captureReady": False, "productReservation": None,
                  "productCompilationSupported": False, "productPoolCacheWriterAdmitted": False, "formalAcceptance": False}
        resource.release(reservation["key"], writer_key=writer, native_proof_ref=proof_id)
        released = True
        for domain in ("management_checkout_read", "execution_launch", "execution"):
            row = m.state.get(domain, owner)
            m.state.put(domain, owner, {**row["payload"], "status": "released" if domain == "management_checkout_read" else "terminal",
                        "pinsReleased": True, "nativeProofRef": proof_id}, expected_version=row["version"])
        result.update(metadataReservationReleased=True, sourceReadersClosed=True)
        result["receipt"] = m._node_write_json(report / "receipt.json", result)
        return result
    except BaseException as error:
        unknown = launched and not terminal
        if reservation is not None and execution is not None and not unknown:
            row = m.state.get("execution", owner)
            m.state.put("execution", owner, {**row["payload"], "status": "failed", "message": str(error)}, expected_version=row["version"])
            if terminal and not released:
                resource.release(reservation["key"], writer_key=writer, native_proof_ref=proof_id)
                released = True
            elif not released and m.state.get("execution_launch", owner) is None:
                resource.release_unlaunched(reservation["key"], writer_key=writer, execution_id=owner)
                released = True
            elif not released:
                unknown = True
        for domain in ("management_checkout_read", "execution_launch"):
            row = m.state.get(domain, owner)
            if row is not None:
                m.state.put(domain, owner, {**row["payload"], "status": "unknown" if unknown else
                            "released" if domain == "management_checkout_read" else "terminal",
                            "pinsReleased": not unknown, "nativeProofRef": proof_id}, expected_version=row["version"])
        return {"status": "unknown" if unknown else "failed", "reasonCode": "exporter_admission_failed",
                "message": str(error), "captureReady": False,
                "originalReservationRetained": reservation is not None and not released,
                "productStarted": False, "formalAcceptance": False}


def _resolve_package(start, name):
    require(isinstance(name, str) and re.fullmatch(r"(?:@[\w.-]+/)?[\w.-]+", name), "Invalid dependency package name")
    for parent in (start, *start.parents):
        if not parent.is_relative_to(DEPS.parent):
            break
        candidate = parent / "node_modules" / name
        if (candidate / "package.json").is_file():
            resolved = candidate.resolve(strict=True)
            physical(resolved, under=DEPS, exists=True)
            return candidate, resolved
    return None


def _seed_package(name):
    found = _resolve_package(DEPS.parent, name)
    if found is None and name == "smol-toml":
        # Existing compiler/Vitest aliases select this installed package directly;
        # a missing top-level link does not authorize installation or another version.
        package = physical(DEPS / ".pnpm/smol-toml@1.8.0/node_modules/smol-toml", under=DEPS, exists=True)
        manifest = unique_json((package / "package.json").read_text(encoding="utf-8-sig"))
        require(manifest.get("name") == "smol-toml" and manifest.get("version") == "1.8.0",
                "The fixed existing smol-toml alias identity changed")
        found = package, package
    return found


def capture(plan, payload, *, deadline):
    files, bindings, total = {}, {}, 0
    def add(path):
        nonlocal total
        require(time.monotonic() < deadline, "Node input hashing timed out; capture is incomplete")
        path = physical(path, exists=True)
        require(path.is_file(), "Node input is not a regular file")
        if str(path) not in files:
            total += path.stat().st_size
            require(total <= MAX_INPUT_BYTES and len(files) < 50000, "Node closure exceeded its bounded inventory")
            files[str(path)] = file_hash(path)
    ui_aot = plan.get("compileKind") == "ui-aot"
    for name in CONTROL_FILES + (UI_AOT_CONTROL_FILES if ui_aot else ()):
        path = REPOSITORY / name
        if path.exists():
            add(path)
    if plan.get("compileKind") == "style-codec":
        for path in style_codec_generated_inputs():
            add(path)
        add(APP / "tsconfig.spec.json")
    require(str(REPOSITORY / "tools/penpot/pnpm-lock.yaml") in files, "The Node package lock is missing")
    for name in SOURCE_TREES + (UI_AOT_SOURCE_TREES if ui_aot else ()):
        for path in _tree_files(REPOSITORY / name, deadline=deadline):
            add(path)
    add(Path(plan["config"]))
    require(files[plan["config"]] == payload["nodeRecipe"]["config"]["sha256"],
            "Node configuration changed before source capture")
    require(file_hash(physical(NODE, exists=True)) == NODE_SHA, "Node24.19.0 executable identity changed")
    add(NODE)
    require(file_hash(physical(VITEST, exists=True)) == VITEST_SHA, "Vitest4.1.10 launcher changed")
    roots = [VITEST.parent, PLAYWRIGHT.parent]
    packages = ("smol-toml", "feather-icons", "typescript", "esbuild")
    if ui_aot:
        packages += ("@angular/build", "@angular/core", "@angular/compiler", "@angular/compiler-cli",
                     "@angular/common", "@angular/platform-browser", "@angular-devkit/core", "rxjs", "zone.js")
    for name in packages:
        found = _seed_package(name)
        require(found is not None, "An existing Node dependency is missing; installation is not permitted")
        logical, resolved = found
        bindings[str(logical)] = str(resolved)
        roots.append(resolved)
    visited = set()
    while roots:
        package = roots.pop()
        if str(package) in visited:
            continue
        visited.add(str(package))
        require(len(visited) <= 256, "Node dependency package closure exceeded its bounded scope")
        manifest = unique_json((package / "package.json").read_text(encoding="utf-8-sig"))
        if manifest["name"] in {"vitest", "typescript", "esbuild"}:
            require(manifest["version"] == {"vitest": "4.1.10", "typescript": "6.0.3", "esbuild": "0.28.1"}[manifest["name"]],
                    "A selected Node dependency version changed")
        for path in _tree_files(package, deadline=deadline):
            add(path)
        required = manifest.get("dependencies", {})
        optional = {**manifest.get("optionalDependencies", {}), **manifest.get("peerDependencies", {})}
        for name in sorted(set(required) | set(optional)):
            found = _resolve_package(package, name)
            require(found is not None or name not in required or name in manifest.get("optionalDependencies", {}),
                    "An installed transitive Node dependency is missing: " + name)
            if found:
                logical, resolved = found
                bindings[str(logical)] = str(resolved)
                roots.append(resolved)
    if plan["worker"]:
        add(BUILD_HELPER)
        require(files[str(BUILD_HELPER)] == BUILD_HELPER_SHA, "The pinned public worker helper changed before capture")
        for item in plan["configuration"]["expectedInputs"]:
            add(Path(item["path"]))
    if plan.get("compileHelper"):
        add(Path(plan["compileHelper"]))
        require(files[plan["compileHelper"]] == COMPILE_RECIPES[plan["compileKind"]]["sha256"],
                "The reviewed canonical compile helper changed before capture")
    for name, expected in {**payload["sourceHashes"], **payload.get("dependencyHashes", {})}.items():
        path = physical(REPOSITORY / name, under=REPOSITORY, exists=True)
        add(path)
        require(files[str(path)] == expected, "The source-owner revision changed: " + name)
    for logical, resolved in bindings.items():
        require(str(Path(logical).resolve(strict=True)) == resolved, "Node package resolution changed during capture")
    return {"files": dict(sorted(files.items())), "bindings": dict(sorted(bindings.items())),
            "inputBytes": total, "packageCount": len(visited), "sourceCopiesCreated": 0}


def verify_test_result(plan, result):
    require(isinstance(result, dict) and result.get("success") is True, "Vitest did not report success")
    expected_names = {str((APP / name).resolve()) for name in plan["rule"]["specs"]}
    results = result.get("testResults")
    require(isinstance(results, list) and results, "Vitest produced no discovered spec results")
    require({str(Path(row["name"]).resolve()) for row in results} == expected_names, "Vitest spec discovery differs from the recipe")
    if plan["rule"].get("requireUniqueSpecResults"):
        require(len(results) == len(expected_names), "Vitest must report each selected spec exactly once")
    assertions = [case for row in results for case in row.get("assertionResults", [])]
    passed = sum(case.get("status") == "passed" for case in assertions)
    failed = sum(case.get("status") == "failed" for case in assertions)
    require(passed > 0 and failed == 0, "The selected Node tests did not actually pass")
    require(plan["rule"]["expectedPassed"] in (None, passed), "The selected Node test count differs from the source-owner scope")
    if plan["rule"].get("requireEachSpec"):
        require(all(any(case.get("status") == "passed" for case in row.get("assertionResults", []))
                    for row in results), "Every selected spec must discover and pass an actual test")
    per_spec = []
    expected_counts = plan["rule"].get("expectedPassedBySpec")
    if expected_counts is not None:
        for row in results:
            spec = Path(row["name"]).resolve().relative_to(APP.resolve()).as_posix()
            cases = row.get("assertionResults", [])
            actual = sum(case.get("status") == "passed" for case in cases)
            require(actual == expected_counts[spec], "The selected Node per-spec test count differs from the source-owner scope")
            per_spec.append({"spec": spec, "passed": actual,
                             "skipped": sum(case.get("status") != "passed" for case in cases)})
    return {"passed": passed, "failed": failed, "skipped": len(assertions) - passed - failed,
            "specs": sorted(expected_names), "testsExecuted": True,
            **({"perSpec": sorted(per_spec, key=lambda row: row["spec"])} if expected_counts is not None else {})}


def verify_builtin_result(plan, stdout):
    """Require real TAP discovery, complete totals and no failed/skipped test."""
    text = "\n".join(stdout.decode("utf-8").splitlines())
    require(text.startswith("TAP version 13\n"), "The built-in Node test did not produce TAP13")
    counts = re.findall(r"^# (tests|pass|fail|cancelled|skipped|todo) ([0-9]+)$", text, re.M)
    require(len(counts) == 6 and len({name for name, _value in counts}) == 6,
            "The built-in Node test lacks unique complete TAP totals")
    totals = {name: int(value) for name, value in counts}
    expected = plan["rule"]["expectedPassed"]
    if expected is None:
        expected = totals["tests"]
        require(expected > 0, "The built-in Node test must discover actual tests")
    require(totals == {"tests": expected, "pass": expected, "fail": 0,
                       "cancelled": 0, "skipped": 0, "todo": 0},
            "The built-in Node test count, failures or skipped cases differ from its scope")
    rows = re.findall(r"^ok ([1-9][0-9]*) - ([^\n]+)$", text, re.M)
    require([int(number) for number, _name in rows] == list(range(1, expected + 1))
            and len({name for _number, name in rows}) == expected
            and re.findall(r"^1\.\.([0-9]+)$", text, re.M) == [str(expected)]
            and not re.search(r"^not ok\b", text, re.M),
            "The built-in Node test discovery differs from its single-file TAP plan")
    return {"passed": expected, "failed": 0, "skipped": 0, "testsExecuted": True,
            "specs": plan["rule"]["specs"], "testNames": [name for _number, name in rows],
            "reporter": "tap", "headlessExecuted": False, "engineRustExecuted": False}


def materialize_ui_helper(m, plan, report):
    """Generate one pinned helper after resource/writer admission, before capture."""
    if plan.get("compileKind") != "ui-aot":
        return
    path = physical(plan["compileHelper"], under=COMPILE_HELPER_ROOT)
    raw = UI_AOT_HELPER.encode()
    require(path == COMPILE_HELPER_ROOT / COMPILE_RECIPES["ui-aot"]["helper"]
            and hashlib.sha256(raw).hexdigest() == COMPILE_RECIPES["ui-aot"]["sha256"],
            "UI helper generation escaped its fixed source")
    exists = path.exists()
    m.write_record(report / "helper-generation.json", {
        "schemaVersion": 1, "operationId": m.operation, "kind": "fixed-single-ui-aot-helper",
        "path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw),
        "reused": exists, "sourceCopiesCreated": 0,
        "retention": "Retain this pinned managed helper; do not overwrite a foreign helper or product"})
    if not exists:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("xb") as stream:
            stream.write(raw)
    require(file_hash(path) == COMPILE_RECIPES["ui-aot"]["sha256"], "Generated UI helper changed")


def style_codec_generated_inputs():
    return (COMPILE_HELPER_ROOT / "settings-style-codec-vitest-r01.mjs",
            COMPILE_HELPER_ROOT / "settings-style-codec-tsconfig-r01.json")


def materialize_style_codec_helper(m, plan, report):
    """Generate only the pinned five-spec/typecheck/one-worker stages after admission."""
    if plan.get("compileKind") != "style-codec":
        return
    helper = physical(plan["compileHelper"], under=COMPILE_HELPER_ROOT)
    rule = COMPILE_RECIPES["style-codec"]
    require(helper == COMPILE_HELPER_ROOT / rule["helper"], "Style-codec helper escaped its fixed path")
    vitest_path, spec_path = style_codec_generated_inputs()
    aliases = {"vitest": str(VITEST.parent / "dist/index.js"),
               "smol-toml": str(DEPS / ".pnpm/smol-toml@1.8.0/node_modules/smol-toml/dist/index.js"),
               "playwright": str(PLAYWRIGHT),
               "feather-icons/dist/icons.json": str(DEPS / "feather-icons/dist/icons.json")}
    config = {"root": str(APP), "cacheDir": str(COMPILE_REPORT_ROOT / "settings-style-codec-r01/cache"),
              "resolve": {"alias": aliases},
              "test": {"include": list(STYLE_CODEC_SPECS), "globals": True, "environment": "node",
                       "watch": False, "coverage": {"enabled": False}, "maxWorkers": 1,
                       "fileParallelism": False, "testTimeout": 15000, "pool": "forks",
                       "reporters": ["default", "json", "junit"],
                       "outputFile": {"json": plan["outputs"]["testResult"], "junit": plan["outputs"]["junit"]}}}
    strict = {"extends": str(APP / "tsconfig.spec.json"),
              "compilerOptions": {"strict": True, "noEmit": True, "incremental": False},
              "files": [str(APP / name) for name in STYLE_CODEC_SPECS[:2]], "include": []}
    generated = {helper: STYLE_CODEC_HELPER.encode(),
                 physical(vitest_path, under=COMPILE_HELPER_ROOT):
                     ("export default " + json.dumps(config, ensure_ascii=False) + ";\n").encode(),
                 physical(spec_path, under=COMPILE_HELPER_ROOT): encoded(strict) + b"\n"}
    require(hashlib.sha256(generated[helper]).hexdigest() == rule["sha256"], "Style-codec helper source changed")
    for path, raw in generated.items():
        require(not path.exists() or path.read_bytes() == raw,
                "Retained style-codec generated input differs; reconcile its original owner")
    m.write_record(report / "style-codec-generation.json", {
        "schemaVersion": 1, "operationId": m.operation, "recipe": plan["id"],
        "inputs": [{"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw),
                    "reused": path.exists()} for path, raw in generated.items()],
        "sourceCopiesCreated": 0, "retention": "Keep pinned controls and original test/typecheck/build/native evidence"})
    for path, raw in generated.items():
        if not path.exists():
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open("xb") as stream:
                stream.write(raw)


def verify_style_codec_result(plan, captured, result):
    import xml.etree.ElementTree as ET
    entry = str(APP / "src/plugin.ts")
    require(result.get("schema") == "managed-source-style-codec/v1"
            and result.get("typescriptStrictPassed") is True
            and result.get("specs") == list(STYLE_CODEC_SPECS) and result.get("entries") == [entry]
            and result.get("workerBuilt") is True
            and all(result.get(name) is False for name in (
                "publisherBuilt", "runnerBuilt", "angularBuilt", "dependencyInstallation", "deployed")),
            "Style-codec receipt expanded the fixed five-spec and worker scope")
    expected_roots = [[str(APP / name) for name in STYLE_CODEC_SPECS[:2]], [entry]]
    require(result.get("programs") == [{"roots": roots, "strict": True, "noEmit": True,
                                        "incremental": False, "diagnostics": 0} for roots in expected_roots],
            "Style-codec lacks both actual strict minimal TypeScript programs")
    expected_inputs = {item["path"]: item["sha256"] for item in result["inputs"]}
    require(result.get("inputBeforeHashes") == expected_inputs and result.get("inputAfterHashes") == expected_inputs,
            "Style-codec input hashes must match before and after execution")
    program = result.get("programInputPaths")
    require(isinstance(program, list) and program and len(program) == len(set(program))
            and all(root in program for roots in expected_roots for root in roots)
            and set(program) <= set(expected_inputs), "Style-codec lacks its actual TypeScript input census")
    metafile = unique_json(Path(plan["outputs"]["metafile"]).read_text(encoding="utf-8"))
    meta_inputs = [str((REPOSITORY / name).resolve()) for name in metafile["inputs"]]
    require(result.get("metafileInputPaths") == meta_inputs and set(meta_inputs) <= set(expected_inputs),
            "Style-codec lacks its actual esbuild input census")
    tests = Path(plan["outputs"]["testResult"])
    require(result.get("testResult") == {"path": str(tests), "sha256": file_hash(tests)},
            "Style-codec test result identity changed")
    details = verify_test_result(plan, unique_json(tests.read_text(encoding="utf-8")))
    require(details["skipped"] == 0, "Style-codec cannot substitute skipped/todo/cancelled tests")
    junit = Path(plan["outputs"]["junit"])
    xml = ET.fromstring(junit.read_bytes())
    cases = list(xml.iter("testcase"))
    require(len(cases) == details["passed"]
            and all(not any(case.find(tag) is not None for tag in ("failure", "error", "skipped")) for case in cases),
            "Style-codec JUnit differs from actual complete passing discovery")
    suites = [suite.get("name", "").replace("\\", "/") for suite in xml.iter("testsuite")]
    require(all(sum(name == spec or name.endswith("/" + spec) for name in suites) == 1
                for spec in STYLE_CODEC_SPECS), "Style-codec JUnit must identify all five unique spec files")
    return {**result, "testDetails": details, "junit": {"path": str(junit), "sha256": file_hash(junit)},
            "deployed": False, "engineRustExecuted": False}


def verify_ui_compile_result(plan, captured, result):
    require(result.get("schema") == "managed-single-angular-ui/v1"
            and result.get("status") == "compiled-current"
            and result.get("project") == "zircon-zui-plugin"
            and Path(result.get("entry", "")) == APP / "src/main.ts"
            and result.get("aot") is True and result.get("watch") is False
            and result.get("typescriptNoEmitPassed") is True and result.get("typescriptStrictPassed") is True
            and result.get("sourceUnchanged") is True and result.get("workerBuilt") is False
            and result.get("publisherBuilt") is False and result.get("dependencyInstallation") is False
            and result.get("sourceCopiesCreated") is False,
            "Single UI AOT did not prove its bounded strict build")
    before, after = result.get("sourceBeforeHashes"), result.get("sourceAfterHashes")
    require(isinstance(before, dict) and before and before == after
            and all(captured["files"].get(name) == checksum for name, checksum in before.items()),
            "UI AOT source seal differs from its read guard")
    items = result.get("inputs")
    require(isinstance(items, list) and items and len({item["path"] for item in items}) == len(items)
            and any(Path(item["path"]) == APP / "src/main.ts" for item in items)
            and all(captured["files"].get(item["path"]) == item["sha256"] for item in items),
            "UI compiler discovered an unguarded input")
    discovered = {item["path"] for item in items}
    require(all(isinstance(result.get(field), list) and result[field]
                and set(result[field]) <= discovered for field in ("programInputPaths", "statsInputPaths")),
            "UI AOT lacks its TypeScript/stats input census")
    require(str(APP / "src/main.ts") in result["programInputPaths"], "UI TypeScript entry was not consumed")
    stats = unique_json(Path(plan["outputs"]["metafile"]).read_text(encoding="utf-8"))
    require(isinstance(stats.get("inputs"), dict) and stats["inputs"], "UI AOT stats are empty")
    stat_inputs, virtual_inputs = [], []
    for name in stats["inputs"]:
        path = (REPOSITORY / "tools/penpot" / name).resolve()
        if path.is_file():
            require(str(path) in discovered, "UI stats discovered an unguarded input")
            stat_inputs.append(str(path))
        else:
            require(name.startswith("angular:"), "UI stats input is not a captured file or Angular virtual input")
            virtual_inputs.append(name)
    require(result["statsInputPaths"] == stat_inputs and result.get("virtualInputs", []) == virtual_inputs,
            "UI AOT stats input identity differs from its actual metafile")
    directory = physical(plan["outputs"]["build"], under=COMPILE_OUTPUT_ROOT, exists=True)
    outputs = result.get("outputs")
    require(isinstance(outputs, list) and outputs and len({item["path"] for item in outputs}) == len(outputs),
            "UI AOT lacks its complete emitted output list")
    actual = {str(path) for path in _tree_files(directory, deadline=time.monotonic() + 30)}
    require(actual == {item["path"] for item in outputs}, "UI AOT output membership changed")
    for item in outputs:
        path = physical(item["path"], under=directory, exists=True)
        require(path.is_file() and file_hash(path) == item["sha256"] and path.stat().st_size == item["bytes"],
                "UI AOT output identity changed")
    require(result.get("indexSha256") == file_hash(Path(plan["outputs"]["bundle"])),
            "UI AOT index identity differs from its actual output")
    return {"passed": True, "typescriptNoEmitPassed": True, "typescriptStrictPassed": True,
            "aot": True, "project": "zircon-zui-plugin", "profile": "production",
            "inputs": len(items), "outputs": outputs, "workerBuilt": False, "publisherBuilt": False,
            "testsExecuted": False, "headlessExecuted": False, "engineRustExecuted": False}


def verify_compile_result(plan, captured):
    """Bind the fixed noEmit/esbuild output and discovered imports to read pins."""
    outputs, rule = plan["outputs"], COMPILE_RECIPES[plan["compileKind"]]
    result = unique_json(Path(outputs["compileReceipt"]).read_text(encoding="utf-8"))
    if plan["compileKind"] == "ui-aot":
        return verify_ui_compile_result(plan, captured, result)
    require(result.get("status") == "compiled-current" and result.get("typescriptNoEmitPassed") is True
            and result.get("sourceUnchanged") is True and result.get("sourceCopiesCreated") is False,
            "Canonical compilation did not prove its actual noEmit/build result")
    if plan["compileKind"] in {"layout-worker", "style-codec"}:
        require(result.get("typescriptStrictPassed") is True,
                "The layout worker requires its actual strict TypeScript result")
    items = result.get("inputs")
    require(isinstance(items, list) and items and len({item["path"] for item in items}) == len(items),
            "Canonical compilation needs its complete discovered input list")
    entry = APP / rule["entry"]
    require(any(Path(item["path"]) == entry for item in items), "Canonical compiler did not consume its fixed entry")
    for item in items:
        require(captured["files"].get(str(Path(item["path"]))) == item["sha256"],
                "Canonical compiler discovered an unguarded input")
    bundle = physical(outputs["bundle"], under=COMPILE_OUTPUT_ROOT, exists=True)
    declared = result.get("output", {})
    require(Path(declared.get("path", "")) == bundle and declared.get("sha256") == file_hash(bundle),
            "Canonical bundle identity differs from the fixed output")
    metafile = unique_json(Path(outputs["metafile"]).read_text(encoding="utf-8"))
    emitted = metafile.get("outputs", {})
    require(len(emitted) == 1 and len(metafile.get("inputs", {})) > 0,
            "Canonical esbuild output discovery differs from its single bundle")
    for name, facts in emitted.items():
        require((REPOSITORY / name).resolve() == bundle and
                (REPOSITORY / facts.get("entryPoint", "")).resolve() == entry,
                "Canonical esbuild entry/output differs from the recipe")
    for name in metafile["inputs"]:
        require(str((REPOSITORY / name).resolve()) in captured["files"],
                "Canonical esbuild discovered an unguarded input")
    if plan["compileKind"] == "style-codec":
        return verify_style_codec_result(plan, captured, result)
    if plan["compileKind"] == "session-runner":
        require(result.get("typescriptStrictPassed") is True
                and result.get("entries") == [str(entry)]
                and result.get("workerBuilt") is False and result.get("angularBuilt") is False
                and result.get("dependencyInstallation") is False,
                "The existing-session runner requires its one strict entry without worker/install work")
        expected_inputs = {item["path"]: item["sha256"] for item in items}
        require(result.get("inputBeforeHashes") == expected_inputs
                and result.get("inputAfterHashes") == expected_inputs,
                "Runner compilation must pin every discovered input before and after reads")
        program = result.get("programInputPaths")
        require(isinstance(program, list) and program and str(entry) in program
                and len(program) == len(set(program)) and set(program) <= set(expected_inputs),
                "Runner compilation lacks its complete TypeScript input census")
        meta_inputs = [str((REPOSITORY / name).resolve()) for name in metafile["inputs"]]
        require(result.get("metafileInputPaths") == meta_inputs and set(meta_inputs) <= set(expected_inputs),
                "Runner compilation lacks its actual esbuild input census")
    return {**result, "metafile": {"path": outputs["metafile"], "sha256": file_hash(Path(outputs["metafile"]))},
            "deployed": False, "engineRustExecuted": False}


def _load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_preparation(m, checksum):
    ref = m.payload["preparationRef"]
    root = OUTPUT_ROOT / "zircon-local/management-node/reports"
    path = reference(ref, under=root)
    prepared = unique_json(path.read_text(encoding="utf-8"))
    key = prepared.get("operationKey")
    require(isinstance(key, str) and re.fullmatch("[0-9a-f]{64}", key)
            and path == root / key / "receipt.json", "Node preparation is not a canonical server receipt")
    row = m.state.get("management_operation", key)
    require(row is not None and row["payload"].get("phase") == "complete"
            and row["payload"].get("action") == "prepare-node"
            and row["payload"].get("result", {}).get("receipt") == ref,
            "Node preparation lacks its actual completed server operation")
    require(prepared.get("kind") == "managed-node-preparation-v1" and prepared.get("status") == "prepared"
            and prepared.get("sessionId") == m.session and prepared.get("requestId") == m.request
            and prepared.get("repositoryId") == m.repo_id and prepared.get("driverDigest") == m.config["driverDigest"]
            and prepared.get("runtimeOperationId") == m.config["runtimeOperationId"]
            and prepared.get("nodeRecipe") == m.payload["nodeRecipe"] and prepared.get("inputsDigest") == checksum
            and prepared.get("reservationReleased") is True and prepared.get("sourcePinsReleased") is True,
            "Node preparation is stale or belongs to another request")
    return prepared


def settled_native_proof(native, reference, verifier):
    """Keep natural completion separate from a proven owned timeout cleanup."""
    proof = verifier(native, reference)
    if proof["complete"]:
        return {**proof, "naturalCompletion": True, "ownedCleanup": False}
    cleanup = native.get("cleanup", {})
    tree = native.get("processTree", {})
    evidence = native.get("nativeJobEvidence", {})
    records = evidence.get("records", {})
    identity = proof.get("identity", {})
    birth = identity.get("creationTime")
    settled = (cleanup.get("complete") is True and cleanup.get("attempted") is True
        and cleanup.get("reason") == "timeout" and cleanup.get("jobTerminationProven") is True
        and cleanup.get("rootSignaled") is True and cleanup.get("originalWaiterProven") is True
        and not cleanup.get("errors") and not evidence.get("errors")
        and type(identity.get("pid")) is int and identity["pid"] > 0
        and isinstance(birth, str) and birth.isascii() and birth.isdigit() and int(birth) > 0
        and Path(identity.get("executable", "")).is_absolute()
        and tree.get("scope") == "retained-windows-job" and tree.get("nativeWaiterJoined") is True
        and tree.get("readersFinished") is True and tree.get("pipeEOF") is True
        and all(tree.get("pipes", {}).get(name, {}).get("eof") is True for name in ("stdout", "stderr"))
        and type(native.get("rootExitCode")) is int
        and type(records.get("afterCleanup", {}).get("activeProcesses")) is int
        and records["afterCleanup"]["activeProcesses"] == 0
        and type(records.get("afterClose", {}).get("activeProcesses")) is int
        and records["afterClose"]["activeProcesses"] == 0)
    return {**proof, "complete": settled, "childrenGone": settled,
            "naturalCompletion": False, "ownedCleanup": settled,
            "terminalErrors": native.get("terminalErrors", [])}


def run(management):
    """One original operation. An unknown native claim retains its reservation."""
    from tools.jenkins.resources.capacity import Capacity, ResourceManager
    from tools.jenkins.state.locks import process_lock
    m = management
    plan = recipe(m.payload, m.repo)
    require(m.config.get("managedNode") == CAPABILITY, "The bounded Node capability is not installed")
    if plan.get("exporterAdmission"):
        return run_exporter_admission(m, plan)
    report = physical(OUTPUT_ROOT / "zircon-local/management-node/reports" / m.key, under=OUTPUT_ROOT)
    owner, writer = "management-node-" + m.key, "management-node-public8-output"
    for domain in ("execution", "execution_launch", "management_checkout_read"):
        require(m.state.get(domain, owner) is None, "Original Node preparation/launch needs reconciliation; it cannot be replayed")
    require(not report.exists(), "Original Node evidence exists; reconcile it instead of overwriting")
    policy = m.state.get("resource_policy", "default")["payload"]
    resources = ResourceManager(m.state, Capacity(policy["cpuBudget"], policy["memoryBudget"], policy["diskBudget"]))
    # A separate, bounded Node phase budget; preserve the current global disk floor.
    amount = Capacity(1, 2 * 1024**3, 1024**3)
    guard, reservation, launched, terminal = None, None, False, False
    execution_created, read_created = False, False
    proof_id = None
    started, deadline = time.time(), time.monotonic() + PREPARE_SECONDS
    try:
        with process_lock(m.repo / ".jenkins/state/locks/management-git.lock", reason="management_git_busy", timeout=0):
            conflict = m.checkout_git_read_conflict(unknown_only=True)
            require(conflict is None, "An unknown checkout reader requires its original reconciliation")
            resources.queue(owner, request_id=owner, amount=amount, build_root=str(OUTPUT_ROOT), heavy_writer=True, writer_key=writer)
            _inventory, generation = resources.scan_inventory(str(OUTPUT_ROOT))
            reservation = resources.admit(owner, amount, inventory_generation=generation, request_id=owner,
                                          build_root=str(OUTPUT_ROOT), heavy_writer=True, writer_key=writer)
            if reservation is None:
                return {"status": "waiting", "reasonCode": "resource_capacity", "nodeStarted": False,
                        "captureReady": False, "resourcePolicyVersion": m.state.get("resource_policy", "default")["version"],
                        "budget": amount.__dict__, "formalAcceptance": False}
            execution = {"kind": "management_owned_node", "status": "preparing", "executionId": owner,
                         "repositoryId": m.repo_id, "sessionId": m.session, "requestId": m.request,
                         "operationId": m.operation, "inputDigest": m.input_digest, "sourceRoot": str(m.repo),
                         "reservationKey": reservation["key"], "preparationKey": writer, "report": str(report)}
            m.state.put("execution", owner, execution, expected_version=0)
            execution_created = True
            materialize_ui_helper(m, plan, report)
            materialize_style_codec_helper(m, plan, report)
            inputs = capture(plan, m.payload, deadline=deadline)
            for kind, value in plan["outputs"].items():
                target = physical(value, under=OUTPUT_ROOT)
                require(not target.exists() or kind not in plan.get("outputFiles", ["result"])
                        and target.is_dir() and not any(target.iterdir()),
                        "Retained Node output must be reconciled; no existing artifacts will be overwritten")
            checksum = digest(inputs)
            guard_module = m._node_guard_helper()
            trusted = {}
            for name, value in inputs["files"].items():
                require(time.monotonic() < deadline, "Node native input preparation timed out")
                if Path(name).is_relative_to(DEPS):
                    trusted[name] = guard_module.capture_readonly_node_input(name, value)
            roots = [str(REPOSITORY), str(DEPS), str(NODE.parent)]
            roots += [str(Path(name).parent) for name in inputs["files"]
                      if Path(name).is_relative_to(OUTPUT_ROOT) and not Path(name).is_relative_to(REPOSITORY)]
            roots = list(dict.fromkeys(roots))
            interests = {root: sorted({Path(name).relative_to(Path(root)).as_posix()
                                      for name in inputs["files"] if Path(name).is_relative_to(Path(root))})
                         for root in roots}
            interests[str(DEPS)] += [Path(name).relative_to(DEPS).as_posix() for name in inputs["bindings"]]
            # Entire source subtrees are interests so added imports/members cannot escape.
            interests[str(REPOSITORY)] += list(SOURCE_TREES) + list(CONTROL_FILES)
            if plan.get("compileKind") == "ui-aot":
                interests[str(REPOSITORY)] += list(UI_AOT_SOURCE_TREES) + list(UI_AOT_CONTROL_FILES)
            guard = guard_module.CheckoutReadGuard(REPOSITORY, files=inputs["files"], watch_roots=roots,
                watch_interests=interests, trusted_native_inputs=trusted, soft_pins=False,
                admission_deadline=deadline).acquire()
            require(capture(plan, m.payload, deadline=deadline) == inputs, "Node input membership or resolution changed during admission")
            guard.assert_stable()
            registration = {"schemaVersion": 1, "taskId": m.request, "ownerSession": m.session,
                            "operationId": m.operation, "reportRoot": str(report), "sourceRoot": str(m.repo),
                            "sourceCopiesCreated": 0, "outputs": plan["outputs"], "temporaryRoot": str(report / "tmp"),
                            "dependencyRootsReadOnly": [str(DEPS)], "reservation": reservation,
                            "retention": "Keep native/test/build failures and reusable products; cleanup only through managed ownership"}
            artifact_ref = m._node_write_json(report / "artifacts.json", registration)
            request_ref = m._node_write_json(report / "request.json", {"payload": m.payload, "plan": plan, "inputs": inputs,
                                                                      "inputsDigest": checksum, "preparedAt": started})
            read = {**execution, "status": "active", "pinsReleased": False, "inputCopiesCreated": 0,
                    "checkoutReadRef": request_ref, "guardFacts": guard.facts()}
            m.state.put("management_checkout_read", owner, read, expected_version=0)
            read_created = True
        if m.action == "prepare-node":
            finish = guard.finish()
            require(finish["complete"] is True and finish["stable"] is True, "Node preparation guard did not close stably")
            guard.close()
            guard = None
            row = m.state.get("execution", owner)
            m.state.put("execution", owner, {**row["payload"], "status": "cancelled", "reason": "prepare_only_no_native_launch"}, expected_version=row["version"])
            resources.release_unlaunched(reservation["key"], writer_key=writer, execution_id=owner)
            result = {"schemaVersion": 1, "status": "prepared", "kind": "managed-node-preparation-v1",
                      "operationKey": m.key, "operationId": m.operation,
                      "repositoryId": m.repo_id, "sessionId": m.session, "requestId": m.request,
                      "driverDigest": m.config["driverDigest"], "runtimeOperationId": m.config["runtimeOperationId"],
                      "nodeRecipe": m.payload["nodeRecipe"], "inputsDigest": checksum, "request": request_ref,
                      "sourceGuard": finish, "reservation": reservation, "reservationReleased": True,
                      "sourcePinsReleased": True, "captureReady": False, "readyForReAdmission": True,
                      "nodeStarted": False, "nativeTerminal": None, "sourceCopiesCreated": 0,
                      "elapsedSeconds": time.time() - started, "artifactManifest": artifact_ref,
                      "formalAcceptance": False, "m8Accepted": False}
        else:
            verify_preparation(m, checksum)
            temporary = physical(report / "tmp", under=OUTPUT_ROOT)
            temporary.mkdir()
            for kind, value in plan["outputs"].items():
                (Path(value).parent if kind in plan.get("outputFiles", ["result"]) else Path(value)).mkdir(parents=True, exist_ok=True)
            environment = {key: os.environ[key] for key in ("SYSTEMROOT", "WINDIR", "COMSPEC") if key in os.environ}
            environment.update({"TEMP": str(temporary), "TMP": str(temporary), "TMPDIR": str(temporary),
                                "HOME": str(temporary), "USERPROFILE": str(temporary), "APPDATA": str(temporary),
                                "LOCALAPPDATA": str(temporary), "PATH": str(NODE.parent), "CI": "true",
                                "ZUI_LAYOUT_REPO_ROOT": str(REPOSITORY), "ZUI_LAYOUT_TEST_ARTIFACT_DIR": str(temporary),
                                "PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD": "1", "npm_config_cache": str(temporary / "npm-cache")})
            helper = _load(m._node_client_path(), "jenkins_managed_node_native")
            guard.assert_stable()
            m.state.put("execution_launch", owner, {**execution, "status": "claimed", "command": plan["command"],
                        "cwd": plan["cwd"], "environmentDigest": digest(environment), "request": request_ref}, expected_version=0)
            launched = True
            native = helper._run_owned_command(plan["command"], Path(plan["cwd"]), environment, report, plan["timeoutSeconds"])
            native_ref = m._node_write_json(report / "native-terminal.json", native)
            proof = settled_native_proof(native, native_ref, m._node_native_proof)
            require(native["command"] == plan["command"] and native["cwd"] == plan["cwd"] and proof["complete"] is True,
                    "Node retained Job / descendants / dual EOF remain unproven")
            terminal = True
            proof_id = owner + "-native"
            m.state.put("native_job", proof_id, {"owner": owner, "executionId": owner, "role": "management_owned_node",
                        "status": "terminal", "completeProof": proof, "terminalProof": proof, "nativeEvidence": native}, expected_version=0)
            finish = guard.finish()
            require(finish["complete"] is True and finish["stable"] is True, "Node source guard changed or did not join")
            guard.close()
            guard = None
            passed = native["rootExitCode"] == 0 and not native.get("cleanup", {}).get("attempted")
            details, error = None, None
            try:
                require(passed, "Node process did not finish naturally with exit zero")
                if plan.get("builtin"):
                    details = verify_builtin_result(plan, (report / "stdout.bin").read_bytes())
                    m._node_write_json(Path(plan["outputs"]["result"]), details)
                elif plan.get("compileKind"):
                    details = verify_compile_result(plan, inputs)
                elif plan["worker"]:
                    output = Path(plan["outputs"]["build"])
                    details = unique_json((output / "compile-receipt.json").read_text(encoding="utf-8"))
                    require(details.get("typescriptNoEmitPassed") is True and details.get("deployed") is False,
                            "Worker receipt lacks its two bounded typechecks/builds")
                    for item in details.get("inputs", []):
                        require(inputs["files"].get(str(Path(item["path"]))) == item["sha256"], "Worker discovered an unguarded input")
                    for item in details.get("outputs", []):
                        require(file_hash(physical(item["path"], under=output, exists=True)) == item["sha256"], "Worker output hash differs")
                    require(len(details.get("outputs", [])) == 2, "Worker did not produce both fixed bundles")
                else:
                    details = verify_test_result(plan, unique_json(Path(plan["outputs"]["result"]).read_text(encoding="utf-8")))
            except Exception as failure:
                passed, error = False, str(failure)
            resources.release(reservation["key"], writer_key=writer, native_proof_ref=proof_id)
            result = {"status": "passed" if passed else "failed", "nodeStarted": True, "nativeTerminal": native_ref,
                      "nativeProofRef": proof_id, "nativeProof": proof, "inputsDigest": checksum, "request": request_ref,
                      "sourceGuard": finish, "reservationReleased": True, "sourcePinsReleased": True,
                      "command": plan["command"], "cwd": plan["cwd"], "details": details, "error": error,
                      "artifactManifest": artifact_ref, "sourceCopiesCreated": 0, "deployed": False,
                      "formalAcceptance": False, "m8Accepted": False}
        read_row = m.state.get("management_checkout_read", owner)
        m.state.put("management_checkout_read", owner, {**read_row["payload"], "status": "released", "pinsReleased": True,
                    "sourceGuard": finish}, expected_version=read_row["version"])
        if m.action != "prepare-node":
            row = m.state.get("execution", owner)
            m.state.put("execution", owner, {**row["payload"], "status": result["status"], "nativeProofRef": proof_id}, expected_version=row["version"])
            row = m.state.get("execution_launch", owner)
            m.state.put("execution_launch", owner, {**row["payload"], "status": "terminal", "nativeProofRef": proof_id}, expected_version=row["version"])
        receipt = m._node_write_json(report / "receipt.json", result)
        return {**result, "receipt": receipt}
    except BaseException as error:
        if guard is not None and (not launched or terminal):
            try:
                finish = guard.finish()
                guard.close()
                guard = None
            except Exception:
                pass
        unknown = launched and not terminal or guard is not None
        if reservation is not None and execution_created and not unknown:
            row = m.state.get("execution", owner)
            m.state.put("execution", owner, {**row["payload"], "status": "failed", "message": str(error)}, expected_version=row["version"])
            if terminal:
                resources.release(reservation["key"], writer_key=writer, native_proof_ref=proof_id)
            else:
                resources.release_unlaunched(reservation["key"], writer_key=writer, execution_id=owner)
        if read_created:
            row = m.state.get("management_checkout_read", owner)
            m.state.put("management_checkout_read", owner, {**row["payload"], "status": "unknown" if unknown else "released",
                        "pinsReleased": not unknown, "message": str(error)}, expected_version=row["version"])
        waiting = (reservation is None and (getattr(error, "retryable", False)
                   or getattr(error, "code", None) in {"management_git_busy", "checkout_source_busy"}))
        return {"status": "unknown" if unknown else "waiting" if waiting else "failed", "reasonCode": getattr(error, "code", "managed_node_preparation_failed"),
                "message": str(error), "nodeStarted": launched, "originalReservationRetained": unknown,
                "sourcePinsReleased": not unknown, "captureReady": False, "formalAcceptance": False, "m8Accepted": False}


if __name__ == "__main__":
    import sys
    require(len(sys.argv) == 3 and sys.argv[1] == "exporter-admission"
            and os.environ.get("JOB_NAME") == "zircon-management"
            and os.environ.get("NODE_NAME") == "zircon-windows-agent", "Exporter inspection requires its formal Jenkins native lifecycle")
    print(encoded(inspect_exporter_admission(sys.argv[2])).decode("utf-8"))
