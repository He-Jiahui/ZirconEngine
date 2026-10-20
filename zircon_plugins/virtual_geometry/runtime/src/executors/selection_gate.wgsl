// Runs after indexed adaptation and before every mesh replay on the same encoder.
struct Words { values:array<u32> }
@group(0) @binding(0) var<storage,read_write> decision:Words;
@group(0) @binding(1) var<storage,read_write> indexed:Words;
@group(0) @binding(2) var<storage,read> sidecar:Words;
@group(0) @binding(3) var<storage,read> status:Words;
@group(0) @binding(4) var<storage,read> source:Words;
@group(0) @binding(5) var<storage,read> canonical_authority:Words;
@group(0) @binding(6) var<storage,read> authors:Words;
const NONE:u32=0xffffffffu;
fn accepted()->bool {
    if arrayLength(&decision.values)!=8u+4096u*33u ||
       decision.values[0]!=0x56475331u || decision.values[1]!=0u ||
       decision.values[2]>4096u || (decision.values[3]!=0u && decision.values[3]!=2u) ||
       decision.values[6]!=0u {return false;}
    let capacity=arrayLength(&indexed.values)/5u;
    if capacity!=65536u || arrayLength(&status.values)!=capacity ||
       arrayLength(&sidecar.values)!=capacity*32u || arrayLength(&authors.values)<1u ||
       authors.values[0]>(arrayLength(&authors.values)-1u)/32u {return false;}
    for(var slot=0u;slot<capacity;slot++){
        // Adapter rejects malformed authored slots before the decision gate. Any
        // rejection invalidates the whole frame, not only this one command.
        if status.values[slot]!=0u{return false;}
        let a=slot*5u;
        if indexed.values[a]==0u && indexed.values[a+1u]==0u {continue;}
        if indexed.values[a]==0u || indexed.values[a+1u]!=1u{return false;}
        let s=slot*32u;let row=sidecar.values[s+28u];
        if row>=authors.values[0] || row>=(arrayLength(&source.values)-1u)/4u{return false;}
        let author=1u+row*32u;let record=author+14u;
        // Exact original fourteen-word identity and every raw authority field.
        let projected=(authors.values[author+4u]&16u)!=0u;
        if (authors.values[author+4u]&0xffffffe0u)!=0u || row>=arrayLength(&canonical_authority.values)/14u{return false;}
        for(var k=0u;k<14u;k++){
            if projected && (k==6u || k==9u){continue;}
            if sidecar.values[s+k]!=canonical_authority.values[row*14u+k]{return false;}
        }
        for(var k=0u;k<14u;k++){
            var expected=authors.values[author+k];
            if k==4u{expected=expected&15u;}
            if projected && (k==1u || k==10u){continue;}
            if sidecar.values[s+14u+k]!=expected{return false;}
        }
        if sidecar.values[s]!=authors.values[record] ||
           sidecar.values[s+1u]!=authors.values[record+1u] ||
           sidecar.values[s+2u]!=authors.values[record+2u] ||
           sidecar.values[s+3u]!=authors.values[record+5u] ||
           sidecar.values[s+4u]!=authors.values[record+3u] ||
           sidecar.values[s+5u]!=authors.values[record+4u] ||
           sidecar.values[s+6u]!=sidecar.values[s+15u] ||
           sidecar.values[s+7u]!=authors.values[author] ||
           sidecar.values[s+8u]!=authors.values[record+7u] ||
           sidecar.values[s+9u]!=sidecar.values[s+24u] ||
           sidecar.values[s+10u]!=authors.values[record+16u] ||
           sidecar.values[s+11u]!=sidecar.values[s+29u] ||
           sidecar.values[s+12u]!=authors.values[record+13u] ||
           sidecar.values[s+13u]!=authors.values[record+14u] ||
           authors.values[record+10u]!=authors.values[author+11u] ||
           authors.values[record+11u]!=authors.values[author+12u] ||
           authors.values[record+17u]!=0u || sidecar.values[s+31u]!=1u {return false;}
        let token=(min(authors.values[author+9u],65535u)<<16u)|min(authors.values[author+10u],65535u);
        let old=1u+row*4u;
        if source.values[old]==0u || source.values[old+1u]!=1u ||
           source.values[old+3u]!=token {return false;}
        // Full canonical candidate/ordinal identity must be present in the
        // same frame's actual GPU decision; actual GPUScene first_instance is
        // deliberately not replaced by a visibility ordinal or debug token.
        var selected=false;
        for(var i=0u;i<decision.values[2u];i++){
            let d=8u+i*28u;
            if decision.values[d+27u]==sidecar.values[s+11u] &&
               decision.values[d]==sidecar.values[s] &&
               decision.values[d+1u]==sidecar.values[s+1u] &&
               decision.values[d+2u]==sidecar.values[s+2u] &&
               decision.values[d+3u]==sidecar.values[s+4u] &&
               decision.values[d+4u]==sidecar.values[s+5u] &&
               decision.values[d+5u]==sidecar.values[s+3u] &&
               decision.values[d+7u]==0u {selected=true;break;}
        }
        if !selected{return false;}
    }
    return true;
}
@compute @workgroup_size(1)
fn cs_main(){
    let valid=accepted();
    if !valid {
        // The real draw input is cleared before any raster command can read it.
        for(var i=0u;i<arrayLength(&indexed.values);i++){indexed.values[i]=0u;}
        decision.values[6]=2u;
        return;
    }
    // Count zero can be valid; status is independent of selected/draw counts.
    decision.values[6]=1u;
}
