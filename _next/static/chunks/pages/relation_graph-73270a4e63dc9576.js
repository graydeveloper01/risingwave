(self.webpackChunk_N_E=self.webpackChunk_N_E||[]).push([[799],{31351:function(e){var t=Array.prototype.reverse;e.exports=function(e){return null==e?e:t.call(e)}},18956:function(e,t,n){(window.__NEXT_P=window.__NEXT_P||[]).push(["/relation_graph",function(){return n(51597)}])},60469:function(e,t,n){"use strict";n.d(t,{D:function(){return h},r:function(){return u}});var r=n(85893),l=n(79351),a=n(47741),o=n(41664),i=n.n(o),s=n(95100),c=n(55992),d=n(96402);function u(e){let[t,n]=(0,s.v1)("modalId",s.U);return[null==e?void 0:e.find(e=>e.id===t),n]}function h(e){let{modalData:t,onClose:n}=e;return(0,r.jsxs)(l.u_,{isOpen:void 0!==t,onClose:n,size:"3xl",children:[(0,r.jsx)(l.ZA,{}),(0,r.jsxs)(l.hz,{children:[(0,r.jsxs)(l.xB,{children:["Catalog of ",t&&(0,c.ks)(t)," ",null==t?void 0:t.id," - ",null==t?void 0:t.name]}),(0,r.jsx)(l.ol,{}),(0,r.jsx)(l.fe,{children:t&&(0,r.jsx)(d.Rm,{src:t,collapsed:1,name:null,displayDataTypes:!1})}),(0,r.jsxs)(l.mz,{children:[t&&(0,c.vx)(t)&&(0,r.jsx)(a.zx,{colorScheme:"blue",mr:3,children:(0,r.jsx)(i(),{href:"/fragment_graph/?id=".concat(t.id),children:"View Fragments"})}),(0,r.jsx)(a.zx,{mr:3,onClick:n,children:"Close"})]})]})]})}},96402:function(e,t,n){"use strict";n.d(t,{Rm:function(){return g},KB:function(){return _},Kf:function(){return b},gU:function(){return w},vk:function(){return y},sW:function(){return k},v6:function(){return j}});var r=n(85893),l=n(47741),a=n(40639),o=n(36696),i=n(63679),s=n(9008),c=n.n(s),d=n(41664),u=n.n(d),h=n(67294),p=n(56103),x=n(3047),m=n(55992);function f(e){var t,n,r,l;return"columnDesc"in e?"".concat(null===(t=e.columnDesc)||void 0===t?void 0:t.name," (").concat(null===(r=e.columnDesc)||void 0===r?void 0:null===(n=r.columnType)||void 0===n?void 0:n.typeName,")"):"".concat(e.name," (").concat(null===(l=e.dataType)||void 0===l?void 0:l.typeName,")")}var v=n(60469);let g=(0,i.ZP)(()=>n.e(171).then(n.t.bind(n,55171,23))),y={name:"Depends",width:1,content:e=>(0,r.jsx)(u(),{href:"/relation_graph/?id=".concat(e.id),children:(0,r.jsx)(l.zx,{size:"sm","aria-label":"view dependents",colorScheme:"blue",variant:"link",children:"D"})})},j=[{name:"Primary Key",width:1,content:e=>e.pk.map(e=>e.columnIndex).map(t=>e.columns[t]).map(e=>f(e)).join(", ")},{name:"Vnode Count",width:1,content:e=>{var t;return null!==(t=e.maybeVnodeCount)&&void 0!==t?t:"?"}}],w={name:"Connector",width:3,content:e=>{var t;return null!==(t=e.withProperties.connector)&&void 0!==t?t:"unknown"}},b={name:"Connector",width:3,content:e=>{var t;return null!==(t=e.properties.connector)&&void 0!==t?t:"unknown"}},k=[y,{name:"Fragments",width:1,content:e=>(0,r.jsx)(u(),{href:"/fragment_graph/?id=".concat(e.id),children:(0,r.jsx)(l.zx,{size:"sm","aria-label":"view fragments",colorScheme:"blue",variant:"link",children:"F"})})}];function _(e,t,n){let{response:i}=(0,x.Z)(async()=>{let e=await t(),n=await (0,m.Rf)(),r=await (0,m.Cp)(),l=await (0,m.jW)();return e.map(e=>{let t=n.find(t=>t.id===e.owner),a=null==t?void 0:t.name,o=l.find(t=>t.id===e.schemaId),i=null==o?void 0:o.name,s=r.find(t=>t.id===e.databaseId),c=null==s?void 0:s.name;return{...e,ownerName:a,schemaName:i,databaseName:c}})}),[s,d]=(0,v.r)(i),u=(0,r.jsx)(v.D,{modalData:s,onClose:()=>d(null)}),g=(0,r.jsxs)(a.xu,{p:3,children:[(0,r.jsx)(p.Z,{children:e}),(0,r.jsx)(o.xJ,{children:(0,r.jsxs)(o.iA,{variant:"simple",size:"sm",maxWidth:"full",children:[(0,r.jsx)(o.hr,{children:(0,r.jsxs)(o.Tr,{children:[(0,r.jsx)(o.Th,{width:3,children:"Id"}),(0,r.jsx)(o.Th,{width:5,children:"Database"}),(0,r.jsx)(o.Th,{width:5,children:"Schema"}),(0,r.jsx)(o.Th,{width:5,children:"Name"}),(0,r.jsx)(o.Th,{width:3,children:"Owner"}),n.map(e=>(0,r.jsx)(o.Th,{width:e.width,children:e.name},e.name)),(0,r.jsx)(o.Th,{children:"Visible Columns"})]})}),(0,r.jsx)(o.p3,{children:null==i?void 0:i.map(e=>(0,r.jsxs)(o.Tr,{children:[(0,r.jsx)(o.Td,{children:(0,r.jsx)(l.zx,{size:"sm","aria-label":"view catalog",colorScheme:"blue",variant:"link",onClick:()=>d(e.id),children:e.id})}),(0,r.jsx)(o.Td,{children:e.databaseName}),(0,r.jsx)(o.Td,{children:e.schemaName}),(0,r.jsx)(o.Td,{children:e.name}),(0,r.jsx)(o.Td,{children:e.ownerName}),n.map(t=>(0,r.jsx)(o.Td,{children:t.content(e)},t.name)),e.columns&&e.columns.length>0&&(0,r.jsx)(o.Td,{overflowWrap:"normal",children:e.columns.filter(e=>!("isHidden"in e)||!e.isHidden).map(e=>f(e)).join(", ")})]},e.id))})]})})]});return(0,r.jsxs)(h.Fragment,{children:[(0,r.jsx)(c(),{children:(0,r.jsx)("title",{children:e})}),u,g]})}},51597:function(e,t,n){"use strict";n.r(t),n.d(t,{default:function(){return N}});var r=n(85893),l=n(40639),a=n(47741),o=n(31351),i=n.n(o),s=n(89734),c=n.n(s),d=n(9008),u=n.n(d),h=n(95100),p=n(67294),x=n(52189),m=n(49379),f=n(70681),v=n(55992),g=n(60469),y=n(23924);function j(e){let{nodes:t,selectedId:n,setSelectedId:l,backPressures:a,relationStats:o}=e,[i,s]=(0,g.r)(t.map(e=>e.relation)),c=(0,p.useRef)(null),{layoutMap:d,links:u,width:h,height:j}=(0,p.useCallback)(()=>{let e=new f.graphlib.Graph;e.setGraph({rankdir:"LR",nodesep:30,ranksep:80,marginx:30,marginy:30}),e.setDefaultEdgeLabel(()=>({})),t.forEach(t=>{e.setNode(t.id,t)}),t.forEach(t=>{var n;null===(n=t.parentIds)||void 0===n||n.forEach(n=>{e.setEdge(n,t.id)})}),f.layout(e);let n=e.nodes().map(t=>{let n=e.node(t);return{...n,x:n.x-75,y:n.y-22.5}}),r=e.edges().map(t=>{let n=e.edge(t);return{source:t.v,target:t.w,points:n.points||[]}}),{width:l,height:a}=function(e){let t=0,n=0;for(let{x:r,y:l}of e)t=Math.max(t,r+150),n=Math.max(n,l+45);return{width:t,height:n}}(n);return{layoutMap:n,links:r,width:l,height:a}},[t])();return(0,p.useEffect)(()=>{let e=Date.now(),t=c.current,r=m.Ys(t),i=m.$0Z,h=m.jvg().curve(i).x(e=>{let{x:t}=e;return t}).y(e=>{let{y:t}=e;return t}),p=r.select(".edges").selectAll(".edge").data(u),f=e=>e===n,g=e=>(e.attr("d",e=>{let{points:t}=e;return h(t)}).attr("fill","none").attr("stroke-width",e=>{if(a){let t=a.get("".concat(e.source,"_").concat(e.target));if(t)return(0,y.b4)(t,15)}return 2}).attr("stroke",e=>{if(a){let t=a.get("".concat(e.source,"_").concat(e.target));if(t)return(0,y.k)(t)}return x.rS.colors.gray["300"]}).attr("opacity",e=>f(e.source)||f(e.target)?1:.5),e.on("mouseover",(e,t)=>{m.td_(".tooltip").remove();let n=null==a?void 0:a.get("".concat(t.source,"_").concat(t.target)),r="<b>Relation ".concat(t.source," → ").concat(t.target,"</b><br>Backpressure: ").concat(null!=n?"".concat((100*n).toFixed(2),"%"):"N/A");m.Ys("body").append("div").attr("class","tooltip").style("position","absolute").style("background","white").style("padding","10px").style("border","1px solid #ddd").style("border-radius","4px").style("pointer-events","none").style("left",e.pageX+10+"px").style("top",e.pageY+10+"px").style("font-size","12px").html(r)}).on("mousemove",e=>{m.Ys(".tooltip").style("left",e.pageX+10+"px").style("top",e.pageY+10+"px")}).on("mouseout",()=>{m.td_(".tooltip").remove()}),e);p.exit().remove(),p.enter().call(e=>e.append("path").attr("class","edge").call(g)),p.call(g);let j=t=>{t.attr("transform",e=>{let{x:t,y:n}=e;return"translate(".concat(t,",").concat(n,")")});let n=t.select("rect");n.empty()&&(n=t.append("rect")),n.attr("width",150).attr("height",45).attr("rx",6).attr("ry",6).attr("fill","white").attr("stroke",e=>{let{id:t}=e;return f(t)?x.rS.colors.blue["500"]:x.rS.colors.gray["200"]}).attr("stroke-width",2);let r=t.select("circle");r.empty()&&(r=t.append("circle")),r.attr("cx",22).attr("cy",22.5).attr("r",12).attr("fill",t=>{let{id:n,relation:r}=t,l=(0,v.vx)(r)?"500":"400",a=f(n)?x.rS.colors.blue[l]:x.rS.colors.gray[l];if(o){let t=parseInt(n);if(!isNaN(t)&&o[t]){let n=(0,y.Jv)(o[t].currentEpoch);return(0,y.qC)(e-n,a)}}return a});let a=t.select(".type");a.empty()&&(a=t.append("text").attr("class","type"));let i=t.select(".clip-path");i.empty()&&(i=t.append("clipPath").attr("class","clip-path").attr("id",e=>"clip-".concat(e.id))).append("rect"),i.select("rect").attr("width",106).attr("height",45).attr("x",39).attr("y",0),a.attr("fill","white").text(e=>{let{relation:t}=e;return"".concat(function(e){let t=(0,v.Mk)(e);return"SINK"===t?"K":t.charAt(0)}(t))}).attr("font-family","inherit").attr("text-anchor","middle").attr("x",22).attr("y",22.5).attr("dy","0.35em").attr("font-size",16).attr("font-weight","bold");let c=t.select(".text");c.empty()&&(c=t.append("text").attr("class","text")),c.attr("fill","black").text(e=>{let{name:t}=e;return t}).attr("font-family","inherit").attr("x",39).attr("y",22.5).attr("dy","0.35em").attr("font-size",14).attr("clip-path",e=>"url(#clip-".concat(e.id,")"));let d=(e,t)=>{var n;let r=parseInt(t),l=null==o?void 0:o[r],a=l?((Date.now()-(0,y.Jv)(l.currentEpoch))/1e3).toFixed(2):"N/A",i=null!==(n=null==l?void 0:l.currentEpoch)&&void 0!==n?n:"N/A";return"<b>".concat(e.name," (").concat((0,v.ks)(e),")</b><br>Epoch: ").concat(i,"<br>Latency: ").concat(a," seconds")};return t.on("mouseover",(e,t)=>{let{relation:n,id:r}=t;m.td_(".tooltip").remove(),m.Ys("body").append("div").attr("class","tooltip").style("position","absolute").style("background","white").style("padding","10px").style("border","1px solid #ddd").style("border-radius","4px").style("pointer-events","none").style("left",e.pageX+10+"px").style("top",e.pageY+10+"px").style("font-size","12px").html(d(n,r))}).on("mousemove",e=>{m.Ys(".tooltip").style("left",e.pageX+10+"px").style("top",e.pageY+10+"px")}).on("mouseout",()=>{m.td_(".tooltip").remove()}),t.style("cursor","pointer").on("click",(e,t)=>{let{relation:n,id:r}=t;l(r),s(n.id)}),t},w=r.select(".boxes").selectAll(".node").data(d);w.enter().call(e=>e.append("g").attr("class","node").call(j)),w.call(j),w.exit().remove()},[d,u,n,s,l,a,o]),(0,r.jsxs)(r.Fragment,{children:[(0,r.jsxs)("svg",{ref:c,width:"".concat(h,"px"),height:"".concat(j,"px"),children:[(0,r.jsx)("g",{className:"edges"}),(0,r.jsx)("g",{className:"boxes"})]}),(0,r.jsx)(g.D,{modalData:i,onClose:()=>s(null)})]})}var w=n(56103),b=n(51388),k=n(29286),_=n(3047),C=n(66459);let S="200px";function N(){let{response:e}=(0,_.Z)(v.H4),{response:t}=(0,_.Z)(v.rs),[n,o]=(0,h.v1)("id",h.U),{response:s}=(0,_.Z)(v.yp),[d,x]=(0,p.useState)(!1),m=()=>{x(e=>!e)},f=(0,b.Z)(),g=(0,p.useCallback)(()=>e&&t?function(e,t){let n=[],r=new Set(e.map(e=>e.id));for(let a of i()(c()(e,"id"))){var l;n.push({id:a.id.toString(),name:a.name,parentIds:(0,v.vx)(a)&&t.has(a.id)?null===(l=t.get(a.id))||void 0===l?void 0:l.filter(e=>r.has(e)).map(e=>e.toString()):[],order:a.id,width:150,height:45,relation:a})}return n}(e,t):void 0,[e,t])(),[y,N]=(0,p.useState)(),[T,z]=(0,p.useState)();(0,p.useEffect)(()=>{let e;function t(){k.ZP.get("/metrics/fragment/embedded_back_pressures").then(t=>{let n=C.BackPressureSnapshot.fromResponse(t.channelStats);e?N(n.getRate(e)):e=n,z(t.relationStats)},e=>{console.error(e),f(e,"error")})}d&&(N(void 0),m()),t();let n=setInterval(t,5e3);return()=>{clearInterval(n)}},[f,d]);let D=(0,p.useMemo)(()=>{if(!s)return new Map;let e=s.inMap,t=s.outMap;if(y){let n=new Map;for(let[r,l]of y){let[a,o]=r.split("_").map(Number);if(t[a]&&e[o]){let r=t[a],i=e[o],s="".concat(r,"_").concat(i);n.set(s,l)}}return n}},[y,s]),E=(0,r.jsxs)(l.kC,{p:3,height:"calc(100vh - 20px)",flexDirection:"column",children:[(0,r.jsx)(w.Z,{children:"Relation Graph"}),(0,r.jsxs)(l.kC,{flexDirection:"row",height:"full",children:[(0,r.jsx)(l.kC,{width:S,height:"full",maxHeight:"full",mr:3,alignItems:"flex-start",flexDirection:"column",children:(0,r.jsx)(l.xu,{flex:1,overflowY:"scroll",children:(0,r.jsxs)(l.gC,{width:S,align:"start",spacing:1,children:[(0,r.jsx)(a.zx,{onClick:e=>m(),children:"Reset Back Pressures"}),(0,r.jsx)(l.xv,{fontWeight:"semibold",mb:3,children:"Relations"}),null==e?void 0:e.map(e=>{let t=n===e.id;return(0,r.jsx)(a.zx,{colorScheme:t?"blue":"gray",color:t?"blue.600":"gray.500",variant:t?"outline":"ghost",py:0,height:8,justifyContent:"flex-start",onClick:()=>o(e.id),children:e.name},e.id)})]})})}),(0,r.jsxs)(l.xu,{flex:1,height:"full",ml:3,overflowX:"scroll",overflowY:"scroll",children:[(0,r.jsx)(l.xv,{fontWeight:"semibold",children:"Relation Graph"}),g&&(0,r.jsx)(j,{nodes:g,selectedId:null==n?void 0:n.toString(),setSelectedId:e=>o(parseInt(e)),backPressures:D,relationStats:T})]})]})]});return(0,r.jsxs)(p.Fragment,{children:[(0,r.jsx)(u(),{children:(0,r.jsx)("title",{children:"Relation Graph"})}),E]})}}},function(e){e.O(0,[662,876,679,184,278,48,240,273,851,441,459,888,774,179],function(){return e(e.s=18956)}),_N_E=e.O()}]);