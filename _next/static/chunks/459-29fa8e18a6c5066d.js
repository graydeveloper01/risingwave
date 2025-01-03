"use strict";(self.webpackChunk_N_E=self.webpackChunk_N_E||[]).push([[459],{23924:function(t,e,r){r.d(e,{Jv:function(){return s},b4:function(){return o},k:function(){return n},qC:function(){return i}});var a=r(52189),l=r(98032);function n(t){let e=[a.rS.colors.green["100"],a.rS.colors.green["300"],a.rS.colors.yellow["400"],a.rS.colors.orange["500"],a.rS.colors.red["700"]].map(t=>(0,l.H)(t)),r=(t=Math.min(t=Math.max(t,0),100))/100*(e.length-1),n=Math.floor(r);return(0,l.H)(e[n]).mix((0,l.H)(e[Math.ceil(r)]),(r-n)*100).toHexString()}function o(t,e){return(t=Math.min(t=Math.max(t,0),100))/100*e+2}function s(t){return 16172352e5+t/65536}function i(t,e){let r=[e,a.rS.colors.yellow["200"],a.rS.colors.orange["300"],a.rS.colors.red["400"]].map(t=>(0,l.H)(t));if(t<=1e4)return e;if(t>=3e5)return a.rS.colors.red["400"];let n=Math.log(1e4),o=(Math.log(t)-n)/(Math.log(3e5)-n)*(r.length-1),s=Math.floor(o);return(0,l.H)(r[s]).mix((0,l.H)(r[Math.ceil(o)]),(o-s)*100).toHexString()}},66459:function(t,e,r){r.r(e),r.d(e,{BackPressureSnapshot:function(){return T},default:function(){return F}});var a=r(85893),l=r(40639),n=r(83234),o=r(20979),s=r(57026),i=r(36696),c=r(47741),d=r(49379),u=r(97098),p=r(96486),h=r.n(p),m=r(9008),f=r.n(m),g=r(95100),x=r(67294),y=r(52189),v=r(50361),j=r.n(v);function b(t){let{fragmentDependency:e,svgWidth:r,selectedId:l,onSelectedIdChange:n}=t,o=(0,x.useRef)(null),[s,i]=(0,x.useState)("0px"),c=(0,x.useCallback)(()=>{let t=(0,u.zx)().nodeSize([10,34,5]),r=j()(e),{width:a,height:l}=t(r);return{width:a,height:l,dag:r}},[e])();return(0,x.useEffect)(()=>{let{width:t,height:e,dag:a}=c,s=o.current,u=d.Ys(s),p=d.ak_,h=d.jvg().curve(p).x(t=>{let{x:e}=t;return e+10}).y(t=>{let{y:e}=t;return e}),m=t=>t.data.id===l,f=u.select(".edges").selectAll(".edge").data(a.links()),g=t=>t.attr("d",t=>{let{points:e}=t;return h(e)}).attr("fill","none").attr("stroke-width",t=>m(t.source)||m(t.target)?2:1).attr("stroke",t=>m(t.source)||m(t.target)?y.rS.colors.blue["500"]:y.rS.colors.gray["300"]);f.exit().remove(),f.enter().call(t=>t.append("path").attr("class","edge").call(g)),f.call(g);let x=u.select(".nodes").selectAll(".node").data(a.descendants()),v=t=>t.attr("transform",t=>"translate(".concat(t.x+10,", ").concat(t.y,")")).attr("fill",t=>m(t)?y.rS.colors.blue["500"]:y.rS.colors.gray["500"]);x.exit().remove(),x.enter().call(t=>t.append("circle").attr("class","node").attr("r",5).call(v)),x.call(v);let j=u.select(".labels").selectAll(".label").data(a.descendants()),b=t=>t.text(t=>t.data.name).attr("x",r-10).attr("font-family","inherit").attr("text-anchor","end").attr("alignment-baseline","middle").attr("y",t=>t.y).attr("fill",t=>m(t)?y.rS.colors.black["500"]:y.rS.colors.gray["500"]).attr("font-weight","600");j.exit().remove(),j.enter().call(t=>t.append("text").attr("class","label").call(b)),j.call(b);let S=u.select(".overlays").selectAll(".overlay").data(a.descendants()),I=t=>t.attr("x",3).attr("height",24).attr("width",r-6).attr("y",t=>t.y-5-12+2+3).attr("rx",5).attr("fill",y.rS.colors.gray["500"]).attr("opacity",0).style("cursor","pointer");S.exit().remove(),S.enter().call(t=>t.append("rect").attr("class","overlay").call(I).on("mouseover",function(t,e){d.Ys(this).transition().duration(parseInt(y.rS.transition.duration.normal)).attr("opacity",".10")}).on("mouseout",function(t,e){d.Ys(this).transition().duration(parseInt(y.rS.transition.duration.normal)).attr("opacity","0")}).on("mousedown",function(t,e){d.Ys(this).transition().duration(parseInt(y.rS.transition.duration.normal)).attr("opacity",".20")}).on("mouseup",function(t,e){d.Ys(this).transition().duration(parseInt(y.rS.transition.duration.normal)).attr("opacity",".10")}).on("click",function(t,e){n&&n(e.data.id)})),S.call(I),i("".concat(e,"px"))},[e,l,r,n,c]),(0,a.jsxs)("svg",{ref:o,width:"".concat(r,"px"),height:s,children:[(0,a.jsx)("g",{className:"edges"}),(0,a.jsx)("g",{className:"nodes"}),(0,a.jsx)("g",{className:"labels"}),(0,a.jsx)("g",{className:"overlays"})]})}var S=r(39653),I=r(79351),k=r(63679),w=r(70681),C=r(23924);let E=(0,k.ZP)(()=>r.e(171).then(r.t.bind(r,55171,23)));function N(t){let{planNodeDependencies:e,fragmentDependency:r,selectedFragmentId:l,backPressures:n,fragmentStats:o}=t,s=(0,x.useRef)(null),{isOpen:i,onOpen:u,onClose:p}=(0,S.qY)(),[h,m]=(0,x.useState)(),f=(0,x.useCallback)(t=>{m(t),u()},[u,m]),{svgWidth:g,svgHeight:v,edges:b,layoutResult:k,includedFragmentIds:N}=(0,x.useCallback)(()=>{let t=j()(e),a=j()(r),l=new Map,n=new Set;for(let[e,r]of t){var o;let t=function(t,e){let{dx:r,dy:a}=e,l=d.G_s().nodeSize([a,r])(t);return l.each(t=>([t.x,t.y]=[t.y,t.x])),l.each(t=>t.x=-t.x),l}(r,{dx:72,dy:48}),{width:a,height:s}=function(t,e){let{margin:{top:r,bottom:a,left:l,right:n}}=e,o=1/0,s=-1/0,i=1/0,c=-1/0;return t.each(t=>s=t.x>s?t.x:s),t.each(t=>o=t.x<o?t.x:o),t.each(t=>c=t.y>c?t.y:c),t.each(t=>i=t.y<i?t.y:i),o-=l,s+=n,i-=r,c+=a,t.each(t=>t.x=t.x-o),t.each(t=>t.y=t.y-i),{width:s-o,height:c-i}}(t,{margin:{left:48,right:48,top:36,bottom:48}});l.set(e,{layoutRoot:t,width:a,height:s,actorIds:null!==(o=r.data.actorIds)&&void 0!==o?o:[]}),n.add(e)}let s=new w.graphlib.Graph;s.setGraph({rankdir:"LR",nodesep:48,ranksep:60,marginx:24,marginy:24}),s.setDefaultEdgeLabel(()=>({})),a.forEach(t=>{let{id:e,parentIds:r}=t,a=l.get(e);s.setNode(e,a)}),a.forEach(t=>{let{id:e,parentIds:r}=t;null==r||r.forEach(t=>{s.setEdge(t,e)})}),w.layout(s);let i=s.nodes().map(t=>{let e=s.node(t);return{id:t,x:e.x-e.width/2,y:e.y-e.height/2,width:e.width,height:e.height,layoutRoot:e.layoutRoot,actorIds:e.actorIds}}),c=s.edges().map(t=>{let e=s.edge(t);return{source:t.v,target:t.w,points:e.points||[]}}),u=0,p=0;return i.forEach(t=>{let{x:e,y:r,width:a,height:l}=t;u=Math.max(u,e+a+50),p=Math.max(p,r+l+50)}),{layoutResult:i,svgWidth:u,svgHeight:p,edges:c,includedFragmentIds:n}},[e,r])();return(0,x.useEffect)(()=>{if(k){let t=Date.now(),e=s.current,r=d.Ys(e),a=d.h5h().x(t=>t.x).y(t=>t.y),i=t=>t===l,c=e=>{e.attr("transform",t=>{let{x:e,y:r}=t;return"translate(".concat(e,", ").concat(r,")")});let r=e.select(".text-frag-id");r.empty()&&(r=e.append("text").attr("class","text-frag-id")),r.attr("fill","black").text(t=>{let{id:e}=t;return"Fragment ".concat(e)}).attr("font-family","inherit").attr("text-anchor","end").attr("dy",t=>{let{height:e}=t;return e+12}).attr("dx",t=>{let{width:e}=t;return e}).attr("fill","black").attr("font-size",12);let l=e.select(".text-actor-id");l.empty()&&(l=e.append("text").attr("class","text-actor-id")),l.attr("fill","black").text(t=>{let{actorIds:e}=t;return"Actor ".concat(e.join(", "))}).attr("font-family","inherit").attr("text-anchor","end").attr("dy",t=>{let{height:e}=t;return e+24}).attr("dx",t=>{let{width:e}=t;return e}).attr("fill","black").attr("font-size",12);let n=e.select(".bounding-box");n.empty()&&(n=e.append("rect").attr("class","bounding-box")),n.attr("width",t=>{let{width:e}=t;return e}).attr("height",t=>{let{height:e}=t;return e}).attr("x",0).attr("y",0).attr("fill",o?e=>{let{id:r}=e,a=parseInt(r);if(isNaN(a)||!o[a])return"white";let l=(0,C.Jv)(o[a].currentEpoch);return(0,C.qC)(t-l,"white")}:"white").attr("stroke-width",t=>{let{id:e}=t;return i(e)?3:1}).attr("rx",5).attr("stroke",t=>{let{id:e}=t;return i(e)?y.rS.colors.blue[500]:y.rS.colors.gray[500]});let s=e=>{var r;let a=parseInt(e),l=null==o?void 0:o[a],n=l?((t-(0,C.Jv)(l.currentEpoch))/1e3).toFixed(2):"N/A",s=null!==(r=null==l?void 0:l.currentEpoch)&&void 0!==r?r:"N/A";return"<b>Fragment ".concat(a,"</b><br>Epoch: ").concat(s,"<br>Latency: ").concat(n," seconds")};n.on("mouseover",(t,e)=>{let{id:r}=e;d.td_(".tooltip").remove(),d.Ys("body").append("div").attr("class","tooltip").style("position","absolute").style("background","white").style("padding","10px").style("border","1px solid #ddd").style("border-radius","4px").style("pointer-events","none").style("left",t.pageX+10+"px").style("top",t.pageY+10+"px").style("font-size","12px").html(s(r))}).on("mousemove",t=>{d.Ys(".tooltip").style("left",t.pageX+10+"px").style("top",t.pageY+10+"px")}).on("mouseout",()=>{d.td_(".tooltip").remove()});let c=e.select(".edges");c.empty()&&(c=e.append("g").attr("class","edges"));let u=t=>t.attr("d",a),p=c.selectAll("path").data(t=>{let{layoutRoot:e}=t;return e.links()});p.enter().call(t=>(t.append("path").attr("fill","none").attr("stroke",y.rS.colors.gray[700]).attr("stroke-width",1.5).call(u),t)),p.call(u),p.exit().remove();let h=e.select(".nodes");h.empty()&&(h=e.append("g").attr("class","nodes"));let m=t=>{t.attr("transform",t=>"translate(".concat(t.x,",").concat(t.y,")"));let e=t.select("circle");e.empty()&&(e=t.append("circle")),e.attr("fill",y.rS.colors.blue[500]).attr("r",12).style("cursor","pointer").on("click",(t,e)=>f(e.data));let r=t.select("text");r.empty()&&(r=t.append("text")),r.attr("fill","black").text(t=>t.data.name).attr("font-family","inherit").attr("text-anchor","middle").attr("dy",21.6).attr("fill","black").attr("font-size",12).attr("transform","rotate(-8)");let a=t.select("title");return a.empty()&&(a=t.append("title")),a.text(t=>{var e;return null!==(e=t.data.node.identity)&&void 0!==e?e:t.data.name}),t},g=h.selectAll(".stream-node").data(t=>{let{layoutRoot:e}=t;return e.descendants()});g.exit().remove(),g.enter().call(t=>t.append("g").attr("class","stream-node").call(m)),g.call(m)},u=r.select(".fragments").selectAll(".fragment").data(k);u.enter().call(t=>t.append("g").attr("class","fragment").call(c)),u.call(c),u.exit().remove();let p=r.select(".fragment-edges").selectAll(".fragment-edge").data(b),h=d.$0Z,m=d.jvg().curve(h).x(t=>{let{x:e}=t;return e}).y(t=>{let{y:e}=t;return e}),g=t=>{let e=t.select("path");e.empty()&&(e=t.append("path"));let r=t=>i(t.source)||i(t.target);return e.attr("d",t=>{let{points:e}=t;return m(e)}).attr("fill","none").attr("stroke-width",t=>{if(n){let e=n.get("".concat(t.source,"_").concat(t.target));if(e)return(0,C.b4)(e,30)}return r(t)?4:2}).attr("stroke",t=>{if(n){let e=n.get("".concat(t.source,"_").concat(t.target));if(e)return(0,C.k)(e)}return r(t)?y.rS.colors.blue["500"]:y.rS.colors.gray["300"]}),e.on("mouseover",(t,e)=>{d.td_(".tooltip").remove();let r=null==n?void 0:n.get("".concat(e.source,"_").concat(e.target)),a="<b>Fragment ".concat(e.source," → ").concat(e.target,"</b><br>Backpressure: ").concat(null!=r?"".concat((100*r).toFixed(2),"%"):"N/A");d.Ys("body").append("div").attr("class","tooltip").style("position","absolute").style("background","white").style("padding","10px").style("border","1px solid #ddd").style("border-radius","4px").style("pointer-events","none").style("left",t.pageX+10+"px").style("top",t.pageY+10+"px").style("font-size","12px").html(a)}).on("mousemove",t=>{d.Ys(".tooltip").style("left",t.pageX+10+"px").style("top",t.pageY+10+"px")}).on("mouseout",()=>{d.td_(".tooltip").remove()}),t};p.enter().call(t=>t.append("g").attr("class","fragment-edge").call(g)),p.call(g),p.exit().remove()}},[k,b,n,l,f]),(0,a.jsxs)(x.Fragment,{children:[(0,a.jsxs)(I.u_,{isOpen:i,onClose:p,size:"5xl",children:[(0,a.jsx)(I.ZA,{}),(0,a.jsxs)(I.hz,{children:[(0,a.jsxs)(I.xB,{children:[null==h?void 0:h.operatorId," - ",null==h?void 0:h.name]}),(0,a.jsx)(I.ol,{}),(0,a.jsx)(I.fe,{children:i&&(null==h?void 0:h.node)&&(0,a.jsx)(E,{shouldCollapse:t=>{let{name:e}=t;return"input"===e||"fields"===e||"streamKey"===e},src:h.node,collapsed:3,name:null,displayDataTypes:!1})}),(0,a.jsx)(I.mz,{children:(0,a.jsx)(c.zx,{colorScheme:"blue",mr:3,onClick:p,children:"Close"})})]})]}),(0,a.jsxs)("svg",{ref:s,width:"".concat(g,"px"),height:"".concat(v,"px"),children:[(0,a.jsx)("g",{className:"fragment-edges"}),(0,a.jsx)("g",{className:"fragments"})]})]})}var A=r(56103),D=r(51388),M=r(29286),Y=r(3047),_=r(55992);class T{static fromResponse(t){let e=new Map;for(let[r,a]of Object.entries(t))e.set(r,a.value/a.actorCount);return new T(e,Date.now())}getRate(t){let e=new Map;for(let[r,a]of this.result){let l=t.result.get(r);l&&e.set(r,(a-l)/(this.time-t.time)/1e6)}return e}constructor(t,e){this.result=t,this.time=e}}function F(){var t,e,r;let{response:p}=(0,Y.Z)(_.gG),{response:m}=(0,Y.Z)(_.KY),[y,v]=(0,g.v1)("id",g.U),[j,S]=(0,x.useState)(),[I,k]=(0,x.useState)(),w=(0,x.useMemo)(()=>null==p?void 0:p.find(t=>t.id===y),[p,y]),C=(0,D.Z)();(0,x.useEffect)(()=>{y&&(k(void 0),(0,_.Dx)(y).then(t=>{k(t)}))},[y]);let E=(0,x.useCallback)(()=>{if(I){let t=function(t){let e=[],r=new Map;for(let e in t.fragments)for(let a of t.fragments[e].actors)r.set(a.actorId,a.fragmentId);for(let a in t.fragments){let l=t.fragments[a],n=new Set,o=new Set;for(let t of l.actors)for(let e of t.upstreamActorId){let a=r.get(e);if(a)n.add(a);else for(let e of function(t){let e=new Set,r=t=>{var a;for(let l of((null===(a=t.nodeBody)||void 0===a?void 0:a.$case)==="merge"&&e.add(t.nodeBody.merge),t.input||[]))r(l)};return r(t),Array.from(e)}(t.nodes))o.add(e.upstreamFragmentId)}e.push({id:l.fragmentId.toString(),name:"Fragment ".concat(l.fragmentId),parentIds:Array.from(n).map(t=>t.toString()),externalParentIds:Array.from(o).map(t=>t.toString()),width:0,height:0,order:l.fragmentId,fragment:l})}return e}(I);return{fragments:I,fragmentDep:t,fragmentDepDag:(0,u.lu)()(t)}}},[I]);(0,x.useEffect)(()=>{p&&!y&&p.length>0&&v(p[0].id)},[y,p,v]);let F=null===(t=E())||void 0===t?void 0:t.fragmentDep,z=null===(e=E())||void 0===e?void 0:e.fragmentDepDag,P=null===(r=E())||void 0===r?void 0:r.fragments,R=(0,x.useCallback)(()=>{let t=null==P?void 0:P.fragments;if(t){let e=new Map;for(let r in t){let a=function(t){let e;let r=t.actors[0],a=t=>{var e,r;return{name:(null===(r=t.nodeBody)||void 0===r?void 0:null===(e=r.$case)||void 0===e?void 0:e.toString())||"unknown",children:(t.input||[]).map(a),operatorId:t.operatorId,node:t}};if(r.dispatcher.length>0){let t=h().camelCase(r.dispatcher[0].type.replace(/^DISPATCHER_TYPE_/,""));e=r.dispatcher.length>1?r.dispatcher.every(t=>t.type===r.dispatcher[0].type)?"".concat(t,"Dispatchers"):"multipleDispatchers":"".concat(t,"Dispatcher")}else e="noDispatcher";let l=t.actors.reduce((t,e)=>(t[e.actorId]=e.dispatcher,t),{});return l.fragment={...t,actors:[]},d.bT9({name:e,actorIds:t.actors.map(t=>t.actorId.toString()),children:r.nodes?[a(r.nodes)]:[],operatorId:"dispatcher",node:l})}(t[r]);e.set(r,a)}return e}},[null==P?void 0:P.fragments])(),[G,H]=(0,x.useState)(""),[X,Z]=(0,x.useState)(""),W=()=>{let t=parseInt(X);if(m){let e=m.map;for(let r in e){let a=e[r].map;for(let e in a)if(parseInt(e)==t){v(parseInt(r)),S(t);return}}}C(Error("Fragment ".concat(t," not found")))},B=()=>{let t=parseInt(G);if(m){let e=m.map;for(let r in e){let a=e[r].map;for(let e in a)if(a[e].ids.includes(t)){v(parseInt(r)),S(parseInt(e));return}}}C(Error("Actor ".concat(t," not found")))},[J,L]=(0,x.useState)(),[q,U]=(0,x.useState)();(0,x.useEffect)(()=>{let t;function e(){M.ZP.get("/metrics/fragment/embedded_back_pressures").then(e=>{let r=T.fromResponse(e.channelStats);t?L(r.getRate(t)):t=r,U(e.fragmentStats)},t=>{console.error(t),C(t,"error")})}e();let r=setInterval(e,5e3);return()=>{clearInterval(r)}},[C]);let $=(0,a.jsxs)(l.kC,{p:3,height:"calc(100vh - 20px)",flexDirection:"column",children:[(0,a.jsx)(A.Z,{children:"Fragment Graph"}),(0,a.jsxs)(l.kC,{flexDirection:"row",height:"full",width:"full",children:[(0,a.jsxs)(l.gC,{mr:3,spacing:3,alignItems:"flex-start",width:225,height:"full",children:[(0,a.jsxs)(n.NI,{children:[(0,a.jsx)(n.lX,{children:"Streaming Jobs"}),(0,a.jsx)(o.II,{list:"relationList",spellCheck:!1,onChange:t=>{var e;let r=null==p?void 0:null===(e=p.find(e=>e.name==t.target.value))||void 0===e?void 0:e.id;r&&v(r)},placeholder:"Search...",mb:2}),(0,a.jsx)("datalist",{id:"relationList",children:p&&p.map(t=>(0,a.jsxs)("option",{value:t.name,children:["(",t.id,") ",t.name]},t.id))}),(0,a.jsx)(s.Ph,{value:null!=y?y:void 0,onChange:t=>v(parseInt(t.target.value)),children:p&&p.map(t=>(0,a.jsxs)("option",{value:t.id,children:["(",t.id,") ",t.name]},t.name))})]}),w&&(0,a.jsxs)(n.NI,{children:[(0,a.jsx)(n.lX,{children:"Information"}),(0,a.jsx)(i.xJ,{children:(0,a.jsx)(i.iA,{size:"sm",children:(0,a.jsxs)(i.p3,{children:[(0,a.jsxs)(i.Tr,{children:[(0,a.jsx)(i.Td,{fontWeight:"medium",children:"Type"}),(0,a.jsx)(i.Td,{isNumeric:!0,children:w.type})]}),(0,a.jsxs)(i.Tr,{children:[(0,a.jsx)(i.Td,{fontWeight:"medium",children:"Status"}),(0,a.jsx)(i.Td,{isNumeric:!0,children:w.jobStatus})]}),(0,a.jsxs)(i.Tr,{children:[(0,a.jsx)(i.Td,{fontWeight:"medium",children:"Parallelism"}),(0,a.jsx)(i.Td,{isNumeric:!0,children:w.parallelism})]}),(0,a.jsxs)(i.Tr,{children:[(0,a.jsx)(i.Td,{fontWeight:"medium",paddingEnd:0,children:"Max Parallelism"}),(0,a.jsx)(i.Td,{isNumeric:!0,children:w.maxParallelism})]})]})})})]}),(0,a.jsxs)(n.NI,{children:[(0,a.jsx)(n.lX,{children:"Goto"}),(0,a.jsxs)(l.gC,{spacing:2,children:[(0,a.jsxs)(l.Ug,{children:[(0,a.jsx)(o.II,{placeholder:"Fragment Id",value:X,onChange:t=>Z(t.target.value)}),(0,a.jsx)(c.zx,{onClick:t=>W(),children:"Go"})]}),(0,a.jsxs)(l.Ug,{children:[(0,a.jsx)(o.II,{placeholder:"Actor Id",value:G,onChange:t=>H(t.target.value)}),(0,a.jsx)(c.zx,{onClick:t=>B(),children:"Go"})]})]})]}),(0,a.jsxs)(l.kC,{height:"full",width:"full",flexDirection:"column",children:[(0,a.jsx)(l.xv,{fontWeight:"semibold",children:"Fragments"}),z&&(0,a.jsx)(l.xu,{flex:"1",overflowY:"scroll",children:(0,a.jsx)(b,{svgWidth:225,fragmentDependency:z,onSelectedIdChange:t=>S(parseInt(t)),selectedId:null==j?void 0:j.toString()})})]})]}),(0,a.jsxs)(l.xu,{flex:1,height:"full",ml:3,overflowX:"scroll",overflowY:"scroll",children:[(0,a.jsx)(l.xv,{fontWeight:"semibold",children:"Fragment Graph"}),R&&F&&(0,a.jsx)(N,{selectedFragmentId:null==j?void 0:j.toString(),fragmentDependency:F,planNodeDependencies:R,backPressures:J,fragmentStats:q})]})]})]});return(0,a.jsxs)(x.Fragment,{children:[(0,a.jsx)(f(),{children:(0,a.jsx)("title",{children:"Streaming Fragments"})}),$]})}}}]);