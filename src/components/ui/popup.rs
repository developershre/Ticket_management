use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;

static NEXT_POPUP_ID: AtomicUsize = AtomicUsize::new(0);

pub fn next_popup_id() -> String {
    format!("popup-{}", NEXT_POPUP_ID.fetch_add(1, Ordering::Relaxed))
}

const POSITION_JS: &str = r#"
(function(){
var w=document.querySelector('[data-popup="__UID__"]');
if(!w)return;
w.style.left='0px';
w.style.top='0px';
var root=w.closest('__ANCHOR__');
var t=root?root.querySelector('__TRIGGER__'):null;
if(!t){w.style.visibility='visible';return;}
var p=w.firstElementChild;
var cap=p?parseFloat(getComputedStyle(p).maxHeight):NaN;
if(!isFinite(cap))cap=Infinity;
var cr=w.getBoundingClientRect();
var tr=t.getBoundingClientRect();
var pw=w.offsetWidth;
var ph=Math.min(w.offsetHeight,cap);
var side='__SIDE__',align='__ALIGN__',off=__OFF__;
var x,y;
if(side==='left'){x=tr.left-off-pw;y=tr.top;}
else if(side==='right'){x=tr.right+off;y=tr.top;}
else{
y=(side==='top')?tr.top-off-ph:tr.bottom+off;
if(align==='center')x=tr.left+(tr.width-pw)/2;
else if(align==='end')x=tr.right-pw;
else x=tr.left;
}
if(side==='left'||side==='right'){
if(align==='center')y=tr.top+(tr.height-ph)/2;
else if(align==='end')y=tr.bottom-ph;
else y=tr.top;
}
var vw=window.innerWidth,vh=window.innerHeight,m=4;
if(y+ph>vh-m){var fy=(side==='top')?tr.bottom+off:tr.top-off-ph;if(fy>=m)y=fy;}
if(x+pw>vw-m)x=vw-m-pw;
if(x<m)x=m;
if(y<m)y=m;
if(y+ph>vh-m)y=Math.max(m,vh-m-ph);
w.style.left=(x-cr.left)+'px';
w.style.top=(y-cr.top)+'px';
if(__MATCHW__)w.style.width=tr.width+'px';
if(p)p.style.maxHeight=Math.max(0,Math.min(cap,vh-y-m))+'px';
w.style.visibility='visible';
})();
"#;

pub fn position_popup(
    uid: &str,
    anchor: &str,
    trigger: &str,
    side: &str,
    align: &str,
    offset: i32,
    match_trigger_width: bool,
) {
    let js = POSITION_JS
        .replace("__UID__", uid)
        .replace("__ANCHOR__", anchor)
        .replace("__TRIGGER__", trigger)
        .replace("__SIDE__", side)
        .replace("__ALIGN__", align)
        .replace("__OFF__", &offset.to_string())
        .replace(
            "__MATCHW__",
            if match_trigger_width { "true" } else { "false" },
        );
    let _ = document::eval(&js);
}
