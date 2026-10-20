// The graph owns every output. Only invocation zero publishes this bounded baseline stream.
struct Words { values: array<u32> }
@group(0) @binding(0) var<storage,read> resident_pages: Words;
@group(0) @binding(1) var<storage,read> source: Words;
@group(0) @binding(2) var<storage,read_write> visible_clusters: Words;
@group(0) @binding(3) var<storage,read_write> visible_candidates: Words;
@group(0) @binding(4) var<storage,read_write> requested_pages: Words;
@group(0) @binding(5) var<storage,read_write> visbuffer64: Words;
@group(0) @binding(6) var<storage,read> startup: Words;
@group(0) @binding(7) var<storage,read_write> decisions: Words;

const CAP: u32 = 4096u;
const STRIDE: u32 = 28u;
const NONE: u32 = 0xffffffffu;
const MAGIC: u32 = 0x56475331u;
const WORK: u32 = 8u + CAP * STRIDE;
const QUEUE: u32 = WORK + CAP * 2u;
fn raw(i:u32)->u32 { return source.values[22u] + i*12u; }
fn author(i:u32)->u32 { return source.values[23u] + i*10u; }
fn rec(i:u32)->u32 { return 8u+i*STRIDE; }
fn same_entity(a:u32,b:u32)->bool {
    return source.values[a]==source.values[b] && source.values[a+1u]==source.values[b+1u];
}
fn state(page:u32)->u32 {
    var result=2u;
    for(var i=0u;i<source.values[20u];i++){
        let p=source.values[26u]+i*2u;
        if source.values[p]==page { result=select(1u,0u,source.values[p+1u]!=0u); }
    }
    return result;
}
fn candidate(row:u32,instance:u32)->u32 {
    for(var i=0u;i<source.values[17u];i++){
        let a=author(i);
        if source.values[a]==instance && source.values[a+8u]==row { return i; }
    }
    return NONE;
}
fn work_raw(i:u32)->u32 { return decisions.values[WORK+i*2u+1u]; }
fn normal_ordinal(row:u32)->u32 {
    let r=raw(row);var found=false;var result=0u;
    for(var i=0u;i<decisions.values[4u];i++){
        let p=raw(work_raw(i));
        if !same_entity(r,p){continue;}
        if source.values[p+2u]==source.values[r+2u]{found=true;}
        if source.values[p+2u]>=source.values[r+2u]{continue;}
        var duplicate=false;
        for(var j=0u;j<i;j++){let q=raw(work_raw(j));if same_entity(p,q) && source.values[p+2u]==source.values[q+2u]{duplicate=true;break;}}
        if !duplicate{result++;}
    }
    return select(NONE,result,found);
}
fn normal_total(row:u32)->u32 {
    let r=raw(row);var result=0u;
    for(var i=0u;i<decisions.values[4u];i++){
        let p=raw(work_raw(i));if !same_entity(r,p){continue;}
        var duplicate=false;
        for(var j=0u;j<i;j++){let q=raw(work_raw(j));if same_entity(p,q) && source.values[p+2u]==source.values[q+2u]{duplicate=true;break;}}
        if !duplicate{result++;}
    }
    return max(result,1u);
}
fn belongs(row:u32,instance:u32)->bool {
    if row>=source.values[16u] || instance>=source.values[21u]{return false;}
    let r=raw(row);let w=instance*10u;
    if source.values[r]!=startup.values[w+1u] || source.values[r+1u]!=startup.values[w+2u]{return false;}
    for(var other=0u;other<source.values[21u];other++){
        if other==instance{continue;}
        let p=other*10u;let start=startup.values[p+3u];let count=startup.values[p+4u];
        if row>=start && row-start<count{return false;}
    }
    return true;
}
fn parent(row:u32,instance:u32)->u32 {
    let r=raw(row); let id=source.values[r+5u];
    if id==NONE{return NONE;}
    // A parent belongs to this source entity; never follow an unrelated ID collision.
    for(var i=0u;i<source.values[16u];i++){
        if !belongs(i,instance){continue;}
        let p=raw(i);
        if same_entity(r,p) && source.values[p+2u]==id {return i;}
    }
    return NONE;
}
fn intersects(row:u32)->bool {
    let r=raw(row); let radius=bitcast<f32>(source.values[r+10u]);
    if radius<=0.0{return true;}
    let m=mat4x4<f32>(
        bitcast<vec4<f32>>(vec4<u32>(source.values[35],source.values[36],source.values[37],source.values[38])),
        bitcast<vec4<f32>>(vec4<u32>(source.values[39],source.values[40],source.values[41],source.values[42])),
        bitcast<vec4<f32>>(vec4<u32>(source.values[43],source.values[44],source.values[45],source.values[46])),
        bitcast<vec4<f32>>(vec4<u32>(source.values[47],source.values[48],source.values[49],source.values[50])));
    let v=(m*vec4<f32>(bitcast<f32>(source.values[r+7u]),bitcast<f32>(source.values[r+8u]),bitcast<f32>(source.values[r+9u]),1.0)).xyz;
    let near=bitcast<f32>(source.values[29u]); let far=bitcast<f32>(source.values[30u]);
    let depth=-v.z;
    if depth+radius<near || depth-radius>far{return false;}
    var h=bitcast<f32>(source.values[33u]);
    if source.values[31u]!=0u {h=max(depth,near)*bitcast<f32>(source.values[32u]);}
    return abs(v.y)<=h+radius && abs(v.x)<=h*bitcast<f32>(source.values[34u])+radius;
}
fn node(instance:u32,id:u32)->u32 {
    for(var i=0u;i<source.values[18u];i++){
        let n=source.values[24u]+i*6u;
        if source.values[n]==instance && source.values[n+1u]==id{return i;}
    }
    return NONE;
}
fn push(original:u32,instance:u32)->bool {
    let o=raw(original);
    if source.values[11u]!=NONE && source.values[o+4u]!=source.values[11u]{return true;}
    if state(source.values[o+3u])!=0u {request(source.values[o+3u]);}
    var selected=original; var current=parent(original,instance); var depth=0u;
    var search=source.values[11u]==NONE && state(source.values[o+3u])!=0u;
    for(var step=0u;step<source.values[16u] && current!=NONE;step++){
        // Detect the first repeated parent without an unbounded private array.
        var prior=parent(original,instance); var repeated=false;
        for(var k=0u;k<step && prior!=NONE;k++){if prior==current{repeated=true;break;}prior=parent(prior,instance);}
        if repeated{return false;}
        depth++;
        let p=raw(current);
        if search && current!=original && same_entity(o,p) && state(source.values[p+3u])==0u {selected=current;search=false;}
        current=parent(current,instance);
    }
    if current!=NONE{return false;}
    let c=candidate(selected,instance);
    if c==NONE{return false;}
    let r=raw(selected); var index=decisions.values[2u];
    for(var i=0u;i<decisions.values[2u];i++){
        let p=rec(i);
        if decisions.values[p+1u]==source.values[r] &&
           decisions.values[p+2u]==source.values[r+1u] && decisions.values[p+3u]==source.values[r+2u] {index=i;break;}
    }
    if index==CAP{return false;}
    if index==decisions.values[2u]{decisions.values[2u]++;}
    let b=rec(index);var original_ordinal=normal_ordinal(original);
    if original_ordinal==NONE{return false;}
    let original_total=normal_total(original);
    let resolved_ordinal=normal_ordinal(selected);
    let selected_ordinal=select(resolved_ordinal,original_ordinal,resolved_ordinal==NONE);
    decisions.values[b]=instance; decisions.values[b+1u]=source.values[r]; decisions.values[b+2u]=source.values[r+1u];
    decisions.values[b+3u]=source.values[r+2u]; decisions.values[b+4u]=selected_ordinal;
    decisions.values[b+5u]=source.values[r+3u]; decisions.values[b+6u]=source.values[r+4u]; decisions.values[b+7u]=state(source.values[r+3u]);
    // Startup geometry is the resolved selected cluster; submission range/page retain the original child.
    for(var k=0u;k<7u;k++){decisions.values[b+8u+k]=decisions.values[b+k];}
    decisions.values[b+15u]=instance; decisions.values[b+16u]=source.values[o+3u]; decisions.values[b+17u]=source.values[o+4u];
    decisions.values[b+18u]=original_ordinal; decisions.values[b+19u]=1u; decisions.values[b+20u]=original_total;
    decisions.values[b+21u]=depth; decisions.values[b+22u]=0u; decisions.values[b+23u]=NONE; decisions.values[b+24u]=NONE;
    decisions.values[b+25u]=state(source.values[r+3u]); decisions.values[b+26u]=original; decisions.values[b+27u]=c;
    return true;
}
fn less(a:u32,b:u32)->bool {
    var fields=array<u32,8>(0u,2u,1u,4u,3u,5u,6u,15u);
    for(var i=0u;i<8u;i++){let x=decisions.values[rec(a)+fields[i]];let y=decisions.values[rec(b)+fields[i]];if x!=y{return x<y;}}
    return false;
}
fn request(page:u32) {
    let budget=source.values[1u]-min(source.values[1u],source.values[8u]);
    for(var i=0u;i<requested_pages.values[0];i++){if requested_pages.values[1u+i]==page{return;}}
    let n=requested_pages.values[0];
    if n<budget && n+1u<arrayLength(&requested_pages.values){requested_pages.values[n+1u]=page;requested_pages.values[0]=n+1u;}
}
@compute @workgroup_size(64,1,1)
fn cs_main(@builtin(global_invocation_id) id:vec3<u32>) {
    if id.x!=0u{return;}
    visible_clusters.values[0]=0u; visible_candidates.values[0]=0u; requested_pages.values[0]=0u;visbuffer64.values[0]=0u;
    decisions.values[0]=MAGIC;decisions.values[1]=1u;decisions.values[2]=0u;decisions.values[3]=source.values[15u];
    decisions.values[4]=0u;decisions.values[5]=0u;decisions.values[6]=0u;decisions.values[7]=0u;
    if source.values[14u]!=MAGIC || source.values[16u]>CAP || source.values[17u]>CAP || source.values[18u]>CAP ||
       source.values[21u]>CAP || arrayLength(&decisions.values)<QUEUE+CAP*3u{return;}
    if source.values[15u]==2u {
        if arrayLength(&startup.values)<source.values[21u]*10u{return;}
        // Rebuild the exact producer work domain from actual GPU startup output,
        // applying global cluster_count while creating work items, before selection.
        for(var seed=0u;seed<source.values[21u];seed++){
            let w=seed*10u;let start=startup.values[w+3u];let count=startup.values[w+4u];
            if startup.values[w]!=seed || start>source.values[16u] || count>source.values[16u]-start{return;}
            for(var local=0u;local<count && decisions.values[4u]<source.values[3u];local++){
                let row=start+local;if !belongs(row,seed){return;}
                let n=decisions.values[4u];if n==CAP{return;}
                decisions.values[WORK+n*2u]=seed;decisions.values[WORK+n*2u+1u]=row;
                decisions.values[4u]=n+1u;
            }
        }
        for(var seed=0u;seed<source.values[21u];seed++){
            let w=seed*10u;let instance=startup.values[w];let start=startup.values[w+3u];let count=startup.values[w+4u];
            if instance!=seed || startup.values[w+7u]!=source.values[0u] ||
               startup.values[w+8u]!=source.values[1u] || startup.values[w+9u]!=source.values[11u] ||
               start>source.values[16u] || count>source.values[16u]-start{return;}
            var queue_count=0u;
            for(var work=0u;work<decisions.values[4u];work++){
                if decisions.values[WORK+work*2u]!=instance{continue;}
                let row=work_raw(work);let r=raw(row);
                if source.values[r]!=startup.values[w+1u] || source.values[r+1u]!=startup.values[w+2u]{return;}
                if !push(row,instance){return;}
                let n=node(instance,source.values[r+6u]);
                if n==NONE {continue;}
                var exists=false;for(var q=0u;q<queue_count;q++){if decisions.values[QUEUE+q*3u]==n{exists=true;break;}}
                if !exists {if queue_count==CAP{return;}decisions.values[QUEUE+queue_count*3u]=n;decisions.values[QUEUE+queue_count*3u+1u]=0u;decisions.values[QUEUE+queue_count*3u+2u]=NONE;queue_count++;}
            }
            for(var q=0u;q<queue_count;q++){
                let n=source.values[24u]+decisions.values[QUEUE+q*3u]*6u;let level=decisions.values[QUEUE+q*3u+1u];
                let first=source.values[n+4u];let length=source.values[n+5u];
                if first>source.values[16u] || length>source.values[16u]-first{return;}
                for(var row=first;row<first+length;row++){if !belongs(row,instance){return;}}
                var visible=source.values[27u]==0u || length==0u;var sse=false;
                for(var row=first;row<first+length;row++){visible=visible || intersects(row);sse=sse || bitcast<f32>(source.values[raw(row)+11u])>bitcast<f32>(source.values[28u]);}
                if !visible{continue;}
                if source.values[n+3u]>0u && (length>source.values[0u] || sse) {
                    let first_child=source.values[n+2u];let children=source.values[n+3u];
                    if first_child>source.values[19u] || children>source.values[19u]-first_child{return;}
                    for(var child=0u;child<children;child++){
                        let child_node=node(instance,source.values[source.values[25u]+first_child+child]);
                        if child_node==NONE{return;}
                        var ancestor=q;
                        for(var steps=0u;steps<CAP && ancestor!=NONE;steps++){
                            if decisions.values[QUEUE+ancestor*3u]==child_node{return;}
                            ancestor=decisions.values[QUEUE+ancestor*3u+2u];
                        }
                        var exists=false;for(var prior=0u;prior<queue_count;prior++){if decisions.values[QUEUE+prior*3u]==child_node{exists=true;break;}}
                        if !exists{if queue_count==CAP{return;}decisions.values[QUEUE+queue_count*3u]=child_node;decisions.values[QUEUE+queue_count*3u+1u]=level+1u;decisions.values[QUEUE+queue_count*3u+2u]=q;queue_count++;}
                    }
                }else{
                    for(var row=first;row<first+length;row++){
                        if (source.values[27u]==0u || intersects(row)) &&
                           (source.values[11u]==NONE || source.values[raw(row)+4u]==source.values[11u]) &&
                           state(source.values[raw(row)+3u])!=0u {
                            request(source.values[raw(row)+3u]);
                        }
                    }
                }
            }
        }
        // Shell sort retains deterministic lexicographic identity ordering before budget truncation.
        var gap=decisions.values[2u]/2u;
        while gap>0u{
            for(var i=gap;i<decisions.values[2u];i++){
                var j=i;
                while j>=gap && less(j,j-gap){
                    for(var k=0u;k<STRIDE;k++){let tmp=decisions.values[rec(j)+k];decisions.values[rec(j)+k]=decisions.values[rec(j-gap)+k];decisions.values[rec(j-gap)+k]=tmp;}
                    j-=gap;
                }
            }
            gap/=2u;
        }
        decisions.values[2u]=min(decisions.values[2u],source.values[0u]);
        // Frontier ranks are assigned AFTER the final stable order/budget projection.
        var ranks=0u;
        for(var i=0u;i<decisions.values[2u];i++){
            let b=rec(i);
            if state(source.values[raw(decisions.values[b+26u])+3u])!=0u{
                var rank=ranks;var seen=false;
                for(var j=0u;j<i;j++){let p=rec(j);if state(source.values[raw(decisions.values[p+26u])+3u])!=0u && decisions.values[p+16u]==decisions.values[b+16u]{rank=decisions.values[p+22u];seen=true;break;}}
                if !seen{ranks++;}
                decisions.values[b+22u]=rank;
            }
        }
    }else{
        for(var i=0u;i<source.values[17u];i++){
            let a=author(i);var resident=false;
            for(var p=0u;p<min(resident_pages.values[0],arrayLength(&resident_pages.values)-1u);p++){resident=resident || resident_pages.values[1u+p]==source.values[a+5u];}
            if !resident{request(source.values[a+5u]);continue;}
            let b=rec(decisions.values[2u]);for(var k=0u;k<8u;k++){decisions.values[b+k]=source.values[a+k];}
            decisions.values[b+27u]=i;decisions.values[2u]++;
        }
    }
    let count=decisions.values[2u];
    if count+1u>arrayLength(&visible_candidates.values) || count+1u>arrayLength(&visible_clusters.values) || count*2u+1u>arrayLength(&visbuffer64.values){return;}
    for(var i=0u;i<count;i++){
        let b=rec(i);let cluster=decisions.values[b+3u];let page=decisions.values[b+5u];let instance=decisions.values[b];let lod=decisions.values[b+6u];let s=decisions.values[b+7u];
        if cluster>0xfffffu || page>0xfffffu || (instance!=NONE && instance>0xffffu) || lod>0x3fu || s>2u{return;}
        visible_clusters.values[1u+i]=cluster;visible_candidates.values[1u+i]=decisions.values[b+27u];
        visbuffer64.values[1u+i*2u]=(cluster&0xfffffu)|((page&0xfffu)<<20u);
        visbuffer64.values[2u+i*2u]=((page>>12u)&0xffu)|((instance&0xffffu)<<8u)|(lod<<24u)|(s<<30u);
        if s!=0u{request(page);}
    }
    decisions.values[1u]=0u;
    visible_clusters.values[0]=count;visible_candidates.values[0]=count;visbuffer64.values[0]=count;
}
