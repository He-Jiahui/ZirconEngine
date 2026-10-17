import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import {spawn} from 'node:child_process';
import {createGzip} from 'node:zlib';

const args = new Map();
for (let i=2;i<process.argv.length;i+=2) args.set(process.argv[i],process.argv[i+1]);
for (const key of ['--package-root','--output','--artifact-manifest','--browser'])
  if (!args.get(key)) throw Error('required argument '+key);
const packageRoot = fs.realpathSync(args.get('--package-root'));
const outputInput = path.resolve(args.get('--output'));
const parent = fs.realpathSync(path.dirname(outputInput));
const output = fs.existsSync(outputInput) ? fs.realpathSync(outputInput) : path.join(parent,path.basename(outputInput));
const inside = (root,candidate) => candidate.toLowerCase()===root.toLowerCase() || candidate.toLowerCase().startsWith(root.toLowerCase()+path.sep);
if (!['D:\\cargo-targets','E:\\cargo-targets','F:\\cargo-targets'].some(root=>fs.existsSync(root)&&inside(fs.realpathSync(root),output)))
  throw Error('browser output must physically remain under a drive-root cargo-targets directory');
const manifest = JSON.parse(fs.readFileSync(args.get('--artifact-manifest'),'utf8'));
if (!(manifest.artifacts??[]).some(entry=>fs.existsSync(entry.path)&&inside(fs.realpathSync(entry.path),output)))
  throw Error('register the output in the task artifact manifest before running this validator');
const cycles = Number(args.get('--cycles')??20);
if (!Number.isSafeInteger(cycles)||cycles<20||cycles>100) throw Error('cycles must be between 20 and 100');
for (const required of ['zircon_app.js','zircon_app_bg.wasm','scene/catalog.json'])
  if (!fs.existsSync(path.join(packageRoot,required))) throw Error('missing browser package input '+required);
fs.mkdirSync(output,{recursive:true});
const profile = path.join(output,'profile');
const browserLog = fs.openSync(path.join(output,'browser.log'),'a');
const html = String.raw`<!doctype html><meta charset="utf-8"><title>Zircon WebGPU memory acceptance</title>
<style>body{margin:0;background:#181818;color:white;font:14px sans-serif}canvas{display:block}pre{white-space:pre-wrap}</style>
<main id="host"></main><pre id="result">Running...</pre>
<script type="module">
import init,{WebEngine,browser_memory_snapshot} from '/zircon_app.js';
const cycles=CYCLE_COUNT;
const frame=()=>new Promise(requestAnimationFrame);
const parse=JSON.parse;
const check=(value,message)=>{if(!value)throw Error(message)};
window.__zirconAcceptance=(async()=>{
 const wasm=await init();
 const report={status:'running',cycles:[],errors:[],user_agent:navigator.userAgent,
  measurement_scope:'Rust requested heap bytes and engine texture/graph GPU counters; excludes JS/driver memory',
  baseline:parse(browser_memory_snapshot())};
 window.__zirconProgress=report;
 const adapter=await navigator.gpu?.requestAdapter();
 check(adapter,'WebGPU adapter unavailable');
 report.adapter={...adapter.info};
 report.limits={};
 for(const key of ['maxBindGroups','maxStorageBuffersPerShaderStage','maxSampledTexturesPerShaderStage','maxTextureArrayLayers'])report.limits[key]=adapter.limits[key];
 for(let cycle=0;cycle<cycles;cycle++){
  const canvas=document.createElement('canvas');canvas.width=320;canvas.height=240;
  document.getElementById('host').append(canvas);
  let engine;const before=parse(browser_memory_snapshot());
  const item={cycle,before,frames:[]};
  try{
   engine=await WebEngine.load(canvas,new URL('/scene',location.href).href);
   const loaded=parse(browser_memory_snapshot());
   check(loaded.live_engines===1&&loaded.registered_canvases===1&&loaded.event_listeners>0,'missing browser owner or listeners');
   item.loaded=loaded;
   for(let f=0;f<8;f++){
    if(f===3)engine.resize(480,270);
    if(f===6)engine.resize(320,240);
    const timestamp=await frame();
    const receipt=parse(await engine.render_frame(timestamp));item.frames.push(receipt);
    check(receipt.presented,'visible frame was not presented');
   }
   const last=item.frames.at(-1);
   check(last.animation_poses>0,'fixed scene did not evaluate its skeleton');
   check(last.visible_instances>0,'fixed scene did not submit visible meshes');
   check(last.ui_commands>0,'fixed scene did not produce text/UI commands');
   check(last.texture_resident_bytes>0,'fixed scene texture was not resident');
   if(cycle===0){
    window.__zirconScreenshotReady=true;
    await new Promise(resolve=>{window.__zirconContinue=resolve;setTimeout(resolve,5000)});
   }
   const closed=parse(await engine.close());item.close=closed;
   check(closed.modules_closed&&closed.task_graph_joined&&closed.pending_assets===0,'browser close did not finish draining');
  }finally{
   if(engine){try{await engine.close()}finally{engine.free()}}
   canvas.remove();
  }
  await frame();await frame();
  const after=parse(browser_memory_snapshot());item.after=after;
  item.wasm_linear_bytes=wasm.memory.buffer.byteLength;
  check(after.live_engines===0&&after.registered_canvases===0&&after.event_listeners===0,'browser owner count did not return to zero');
  check(after.rust_heap,'build the package with memory-validation enabled');
  item.allocation_calls=after.rust_heap.allocation_calls-before.rust_heap.allocation_calls;
  report.cycles.push(item);
  document.getElementById('result').textContent='Completed '+(cycle+1)+' / '+cycles;
 }
 const warm=report.cycles.slice(5).map(item=>item.after.rust_heap.live_requested_bytes);
 const count=warm.length,average=warm.reduce((a,b)=>a+b,0)/count,mid=(count-1)/2;
 const numerator=warm.reduce((sum,value,index)=>sum+(index-mid)*(value-average),0);
 const denominator=warm.reduce((sum,_,index)=>sum+(index-mid)**2,0);
 report.live_heap_slope_bytes_per_cycle=numerator/denominator;
 report.late_heap_span_bytes=Math.max(...warm.slice(-5))-Math.min(...warm.slice(0,5));
 check(report.live_heap_slope_bytes_per_cycle<=8192&&report.late_heap_span_bytes<=262144,'live requested heap did not stabilize after warm-up');
 report.status='passed';
 document.getElementById('result').textContent=JSON.stringify(report,null,2);
 return report;
})().catch(error=>{const report=window.__zirconProgress??{cycles:[]};report.status='failed';report.error=String(error?.stack??error);window.__zirconProgress=report;document.getElementById('result').textContent=JSON.stringify(report,null,2);return report});
</script>`;
const server = http.createServer(async(req,res)=>{
 try{
  const url = new URL(req.url,'http://localhost');
  if(url.pathname==='/__zircon_acceptance.html'){res.setHeader('Content-Type','text/html; charset=utf-8');res.end(html.replace('CYCLE_COUNT',String(cycles)));return}
  const candidate = path.resolve(packageRoot,'.'+decodeURIComponent(url.pathname));
  if(!inside(packageRoot,candidate)){res.writeHead(403);res.end();return}
  const actual=await fs.promises.realpath(candidate);
  if(!inside(packageRoot,actual)){res.writeHead(403);res.end();return}
  const types={'.js':'text/javascript','.wasm':'application/wasm','.json':'application/json'};
  res.setHeader('Content-Type',types[path.extname(actual)]??'application/octet-stream');
  res.setHeader('Cache-Control','no-store');
  const input=fs.createReadStream(actual).on('error',()=>res.destroy());
  if(String(req.headers['accept-encoding']??'').includes('gzip')){
   res.setHeader('Content-Encoding','gzip');res.setHeader('Vary','Accept-Encoding');
   input.pipe(createGzip()).on('error',()=>res.destroy()).pipe(res);
  }else input.pipe(res);
 }catch{res.writeHead(404);res.end()}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser = spawn(args.get('--browser'),[
 '--headless=new','--no-first-run','--no-default-browser-check','--disable-background-networking',
 '--remote-debugging-port=0','--enable-unsafe-webgpu','--user-data-dir='+profile,
 '--disk-cache-dir='+path.join(output,'cache'),'--crash-dumps-dir='+output,'about:blank',
],{windowsHide:true,stdio:['ignore',browserLog,browserLog],env:{...process.env,TEMP:output,TMP:output,TMPDIR:output}});
const receipt={status:'failed',browser_pid:browser.pid,started:new Date().toISOString(),package_root:packageRoot,cycles_requested:cycles};
fs.writeFileSync(path.join(output,'identity.json'),JSON.stringify(receipt,null,2));
let ws;
try{
 let port;
 for(let i=0;i<150;i++){
  if(browser.exitCode!==null)throw Error('browser exited with code '+browser.exitCode);
  try{port=fs.readFileSync(path.join(profile,'DevToolsActivePort'),'utf8').split('\n')[0];break}catch{}
  await new Promise(resolve=>setTimeout(resolve,200));
 }
 if(!port)throw Error('browser DevTools did not start');
 const targets=await(await fetch('http://127.0.0.1:'+port+'/json/list')).json();
 const page=targets.find(target=>target.type==='page');
 if(!page)throw Error('no browser page');
 ws=new WebSocket(page.webSocketDebuggerUrl);
 await new Promise((resolve,reject)=>{ws.onopen=resolve;ws.onerror=reject});
 let sequence=0;
 const pending=new Map();
 ws.onmessage=event=>{const value=JSON.parse(event.data);const entry=pending.get(value.id);if(entry){pending.delete(value.id);clearTimeout(entry.timer);value.error?entry.reject(Error(JSON.stringify(value.error))):entry.resolve(value.result)}};
 const call=(method,params={},timeout=30000)=>new Promise((resolve,reject)=>{
  const id=++sequence;
  const timer=setTimeout(()=>{pending.delete(id);reject(Error('DevTools timeout: '+method))},timeout);
  pending.set(id,{resolve,reject,timer});ws.send(JSON.stringify({id,method,params}));
 });
 await call('Page.enable');
 await call('Page.navigate',{url:'http://127.0.0.1:'+server.address().port+'/__zircon_acceptance.html'});
 let completed=false;
 for(let i=0;i<600;i++){
  if(browser.exitCode!==null)throw Error('browser exited with code '+browser.exitCode);
  const state=await call('Runtime.evaluate',{expression:'({ready:!!window.__zirconScreenshotReady,status:window.__zirconProgress?.status,loaded:!!window.__zirconAcceptance})',returnByValue:true});
  if(state.result?.value?.ready&&!receipt.screenshot){
   const image=await call('Page.captureScreenshot',{format:'png'});
   const file=path.join(output,'fixed-scene.png');
   fs.writeFileSync(file,Buffer.from(image.data,'base64'));receipt.screenshot=file;
   await call('Runtime.evaluate',{expression:'window.__zirconContinue?.()'});
  }
  if(['passed','failed'].includes(state.result?.value?.status)){completed=true;break}
  await new Promise(resolve=>setTimeout(resolve,250));
 }
 if(!completed)throw Error('browser acceptance did not complete within the bounded run');
 const result=await call('Runtime.evaluate',{expression:'window.__zirconAcceptance',awaitPromise:true,returnByValue:true});
 const report=result.result?.value;
 if(!report)throw Error('browser returned no acceptance receipt');
 receipt.result=report;receipt.status=report.status;
 await call('Browser.close',{},5000).catch(()=>{});
}catch(error){receipt.error=String(error?.stack??error)}
finally{
 ws?.close();server.close();
 const browserExited=()=>browser.exitCode!==null||browser.signalCode!==null;
 if(!browserExited()){
  await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(resolve,3000))]);
  if(!browserExited()){browser.kill();await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(resolve,5000))])}
 }
 receipt.browser_exit_code=browser.exitCode;
 receipt.browser_exit_signal=browser.signalCode;
 receipt.browser_process_reaped=browserExited();
 if(!receipt.browser_process_reaped){receipt.status='failed';receipt.cleanup_incomplete=true}
 receipt.finished=new Date().toISOString();
 fs.closeSync(browserLog);
 fs.writeFileSync(path.join(output,'acceptance.json'),JSON.stringify(receipt,null,2));
 console.log(JSON.stringify({status:receipt.status,error:receipt.error,cycles:receipt.result?.cycles?.length??0,output}));
 process.exitCode=receipt.status==='passed'?0:1;
}
