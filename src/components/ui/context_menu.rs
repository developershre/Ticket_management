#![allow(dead_code)]

use dioxus::prelude::*;

// ============================================================
// Shared context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
struct ContextMenuState {
    open: Signal<bool>,
    x: Signal<f64>,
    y: Signal<f64>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuProps {
    children: Element,
}

#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    let open = use_signal(|| false);
    let x = use_signal(|| 0.0);
    let y = use_signal(|| 0.0);
    use_context_provider(|| ContextMenuState { open, x, y });

    rsx! {
        div {
            "data-slot": "context-menu",
            class: "contents",
            {props.children}
        }
    }
}

// ============================================================
// Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuTriggerProps {
    children: Element,
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuTrigger(props: ContextMenuTriggerProps) -> Element {
    let mut state = use_context::<ContextMenuState>();

    rsx! {
        div {
            "data-slot": "context-menu-trigger",
            class: "select-none {props.class}",
            oncontextmenu: move |event| {
                event.prevent_default();

                let coordinates = event.client_coordinates();
                state.x.set(coordinates.x);
                state.y.set(coordinates.y);
                state.open.set(true);
            },
            {props.children}
        }
    }
}

// ============================================================
// Portal
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuPortalProps {
    children: Element,
}

#[component]
pub fn ContextMenuPortal(props: ContextMenuPortalProps) -> Element {
    // Inline rendering. This is not a separate DOM portal.
    rsx! {
        {props.children}
    }
}

// ============================================================
// Content
// ============================================================

/// On open: clamp the panel into the viewport, focus the first item, and
/// attach document-level Escape / arrow-key / outside-scroll listeners.
const CONTENT_OPEN_JS: &str = r#"
(function(){
var uid="__UID__";
var m=document.querySelector('[data-popup="'+uid+'"]');
if(!m)return;
var x=parseFloat(m.style.left), y=parseFloat(m.style.top);
if(!isFinite(x))x=0;
if(!isFinite(y))y=0;
var w=m.offsetWidth, h=m.offsetHeight, pad=8;
var vw=window.innerWidth, vh=window.innerHeight;
if(x+w>vw-pad)x=Math.max(pad, vw-w-pad);
if(y+h>vh-pad)y=Math.max(pad, vh-h-pad);
if(x<pad)x=pad;
if(y<pad)y=pad;
m.style.left=x+"px";
m.style.top=y+"px";
var first=m.querySelector('[role="menuitem"]');
if(first)first.focus();
if(window["__ctx_kb_"+uid])return;
function close(){
  var b=document.getElementById("__CLOSE__");
  if(b)b.click();
}
function current(){return document.querySelector('[data-popup="'+uid+'"]');}
function onKey(e){
  var cm=current();
  if(!cm)return;
  if(e.key==="Escape"){e.preventDefault();e.stopPropagation();close();return;}
  var els=[].slice.call(cm.querySelectorAll('[role="menuitem"],[role="menuitemcheckbox"],[role="menuitemradio"]')).filter(function(el){return el.getAttribute("aria-disabled")!=="true";});
  if(!els.length)return;
  var i=els.indexOf(document.activeElement);
  if(e.key==="ArrowDown"){e.preventDefault();els[(i+1)%els.length].focus();}
  else if(e.key==="ArrowUp"){e.preventDefault();els[i<=0?els.length-1:i-1].focus();}
  else if(e.key==="Home"){e.preventDefault();els[0].focus();}
  else if(e.key==="End"){e.preventDefault();els[els.length-1].focus();}
  else if(e.key==="Enter"||e.key===" "){
    if(cm.contains(document.activeElement)){e.preventDefault();document.activeElement.click();}
  }
}
function onScroll(e){
  var cm=current();
  if(cm&&e.target&&cm.contains(e.target))return;
  close();
}
window["__ctx_kb_"+uid]=true;
window["__ctx_key_"+uid]=onKey;
window["__ctx_scroll_"+uid]=onScroll;
document.addEventListener("keydown",onKey,true);
document.addEventListener("scroll",onScroll,true);
})();
"#;

const CONTENT_CLOSE_JS: &str = r#"
(function(){
var uid="__UID__";
if(window["__ctx_key_"+uid]){document.removeEventListener("keydown",window["__ctx_key_"+uid],true);delete window["__ctx_key_"+uid];}
if(window["__ctx_scroll_"+uid]){document.removeEventListener("scroll",window["__ctx_scroll_"+uid],true);delete window["__ctx_scroll_"+uid];}
delete window["__ctx_kb_"+uid];
})();
"#;

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuContentProps {
    children: Element,
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuContent(props: ContextMenuContentProps) -> Element {
    let mut state = use_context::<ContextMenuState>();
    let uid = use_hook(super::popup::next_popup_id);
    let open = state.open;
    let x = state.x;
    let y = state.y;
    let effect_uid = uid.clone();
    let listener_uid = uid.clone();
    let close_id = format!("__ctx_close_{}", uid);
    let effect_close_id = close_id.clone();

    // Position, focus, and keyboard/scroll wiring — torn down when closed.
    use_effect(move || {
        let _ = (x.read(), y.read());
        if open() {
            let js = CONTENT_OPEN_JS
                .replace("__UID__", &effect_uid)
                .replace("__CLOSE__", &effect_close_id);
            let _ = document::eval(&js);
        } else {
            let js = CONTENT_CLOSE_JS.replace("__UID__", &listener_uid);
            let _ = document::eval(&js);
        }
    });

    if !(state.open)() {
        return rsx! {};
    }

    let style = format!(
        "position: fixed; left: {}px; top: {}px; z-index: 9999;",
        (state.x)(),
        (state.y)()
    );

    rsx! {
        div { class: "contents",
            // Hidden close button driven by the document-level listeners.
            button {
                id: "{close_id}",
                class: "hidden",
                onclick: move |_| state.open.set(false),
            }

            // Dismiss the popup when clicking or right-clicking outside it.
            div {
                "data-slot": "context-menu-dismiss",
                class: "fixed inset-0",
                style: "z-index: 9998;",
                onclick: move |_| state.open.set(false),
                oncontextmenu: move |event| {
                    event.prevent_default();
                    state.open.set(false);
                }
            }

            div {
                "data-slot": "context-menu-content",
                "data-popup": uid,
                role: "menu",
                style: "{style}",
                class: "min-w-36 max-h-[var(--available-height,80vh)] overflow-x-hidden overflow-y-auto rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10 outline-none animate-in fade-in-0 zoom-in-95 {props.class}",
                onclick: move |event| event.stop_propagation(),
                oncontextmenu: move |event| event.prevent_default(),
                {props.children}
            }
        }
    }
}

// ============================================================
// Group
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuGroupProps {
    children: Element,
}

#[component]
pub fn ContextMenuGroup(props: ContextMenuGroupProps) -> Element {
    rsx! {
        div {
            "data-slot": "context-menu-group",
            role: "group",
            {props.children}
        }
    }
}

// ============================================================
// Label
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuLabelProps {
    children: Element,
    #[props(default)]
    inset: bool,
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuLabel(props: ContextMenuLabelProps) -> Element {
    let inset_class = if props.inset { "pl-7" } else { "" };

    rsx! {
        div {
            "data-slot": "context-menu-label",
            class: "px-1.5 py-1 text-xs font-medium text-muted-foreground {inset_class} {props.class}",
            {props.children}
        }
    }
}

// ============================================================
// Item
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuItemProps {
    children: Element,
    #[props(default)]
    class: String,
    #[props(default)]
    inset: bool,
    #[props(default)]
    destructive: bool,
    #[props(default)]
    disabled: bool,
    #[props(default)]
    on_select: EventHandler<()>,
}

#[component]
pub fn ContextMenuItem(props: ContextMenuItemProps) -> Element {
    let mut state = use_context::<ContextMenuState>();

    let inset_class = if props.inset { "pl-7" } else { "" };
    let variant_class = if props.destructive {
        "hover:bg-destructive/10 hover:text-destructive focus:bg-destructive/10 focus:text-destructive"
    } else {
        "hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
    };
    let disabled_class = if props.disabled {
        "pointer-events-none opacity-50"
    } else {
        "cursor-default"
    };

    rsx! {
        div {
            "data-slot": "context-menu-item",
            "data-inset": "{props.inset}",
            "data-variant": if props.destructive { "destructive" } else { "default" },
            role: "menuitem",
            tabindex: if props.disabled { "-1" } else { "0" },
            "aria-disabled": "{props.disabled}",
            class: "relative flex items-center gap-1.5 rounded-md px-1.5 py-1 text-sm outline-none select-none {inset_class} {variant_class} {props.class} {disabled_class}",
            onclick: move |_| {
                if !props.disabled {
                    props.on_select.call(());
                    state.open.set(false);
                }
            },
            {props.children}
        }
    }
}

// ============================================================
// Separator
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuSeparatorProps {
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuSeparator(props: ContextMenuSeparatorProps) -> Element {
    rsx! {
        div {
            "data-slot": "context-menu-separator",
            role: "separator",
            class: "-mx-1 my-1 h-px bg-border {props.class}",
        }
    }
}

// ============================================================
// Shortcut
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuShortcutProps {
    children: Element,
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuShortcut(props: ContextMenuShortcutProps) -> Element {
    rsx! {
        span {
            "data-slot": "context-menu-shortcut",
            class: "ml-auto text-xs tracking-widest text-muted-foreground {props.class}",
            {props.children}
        }
    }
}

// ============================================================
// Checkbox item (stays open, matching Radix behavior)
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuCheckboxItemProps {
    children: Element,
    checked: bool,
    #[props(default)]
    inset: bool,
    #[props(default)]
    disabled: bool,
    #[props(default)]
    class: String,
    on_checked_change: EventHandler<bool>,
}

#[component]
pub fn ContextMenuCheckboxItem(props: ContextMenuCheckboxItemProps) -> Element {
    let inset_class = if props.inset { "pl-7" } else { "" };
    let disabled_class = if props.disabled {
        "pointer-events-none opacity-50"
    } else {
        "cursor-default"
    };

    rsx! {
        div {
            "data-slot": "context-menu-checkbox-item",
            role: "menuitemcheckbox",
            "aria-checked": "{props.checked}",
            "aria-disabled": "{props.disabled}",
            tabindex: "0",
            class: "relative flex items-center gap-1.5 rounded-md py-1 pr-8 pl-1.5 text-sm outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground {inset_class} {disabled_class} {props.class}",
            onclick: move |_| {
                if !props.disabled {
                    props.on_checked_change.call(!props.checked);
                }
            },
            span {
                class: "pointer-events-none absolute right-2 flex h-4 w-4 items-center justify-center",
                if props.checked {
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "m5 12 4 4L19 6" }
                    }
                }
            }
            {props.children}
        }
    }
}

// ============================================================
// Radio group and radio item (group shares the caller's signal,
// mirroring DropdownMenuRadioGroup)
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuRadioGroupProps {
    children: Element,
    value: Signal<String>,
}

#[component]
pub fn ContextMenuRadioGroup(props: ContextMenuRadioGroupProps) -> Element {
    use_context_provider(|| props.value);

    rsx! {
        div {
            "data-slot": "context-menu-radio-group",
            role: "group",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuRadioItemProps {
    children: Element,
    value: String,
    #[props(default)]
    disabled: bool,
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuRadioItem(props: ContextMenuRadioItemProps) -> Element {
    let mut state = use_context::<ContextMenuState>();
    let mut group_value = use_context::<Signal<String>>();

    let selected = group_value() == props.value;
    let disabled_class = if props.disabled {
        "pointer-events-none opacity-50"
    } else {
        "cursor-default"
    };

    rsx! {
        div {
            "data-slot": "context-menu-radio-item",
            role: "menuitemradio",
            "aria-checked": "{selected}",
            "aria-disabled": "{props.disabled}",
            tabindex: "0",
            class: "relative flex items-center gap-1.5 rounded-md py-1 pr-8 pl-1.5 text-sm outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground {disabled_class} {props.class}",
            onclick: move |_| {
                if !props.disabled {
                    group_value.set(props.value.clone());
                    state.open.set(false);
                }
            },
            span {
                class: "pointer-events-none absolute right-2 flex h-4 w-4 items-center justify-center",
                if selected {
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        circle { cx: "12", cy: "12", r: "10" }
                        circle { cx: "12", cy: "12", r: "3", fill: "currentColor", stroke: "none" }
                    }
                }
            }
            {props.children}
        }
    }
}

// ============================================================
// Submenu
// ============================================================

#[derive(Clone, Copy, PartialEq)]
struct ContextMenuSubState {
    open: Signal<bool>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuSubProps {
    children: Element,
}

#[component]
pub fn ContextMenuSub(props: ContextMenuSubProps) -> Element {
    let mut open = use_signal(|| false);
    use_context_provider(|| ContextMenuSubState { open });

    rsx! {
        div {
            "data-slot": "context-menu-sub",
            onmouseleave: move |_| open.set(false),
            {props.children}
        }
    }
}

/// Place the sub panel flush against its trigger (flipping to the other side
/// near the viewport edge) and reveal it.
const SUB_OPEN_JS: &str = r#"
(function(){
var uid="__UID__";
var m=document.querySelector('[data-popup="'+uid+'"]');
if(!m)return;
var wrap=m.closest('[data-slot="context-menu-sub"]');
var t=wrap?wrap.querySelector('[data-slot="context-menu-sub-trigger"]'):null;
if(!t){m.style.visibility="visible";return;}
var tr=t.getBoundingClientRect();
var w=m.offsetWidth, h=m.offsetHeight, pad=8;
var x=tr.right, y=tr.top;
var vw=window.innerWidth, vh=window.innerHeight;
if(x+w>vw-pad)x=Math.max(pad, tr.left-w);
if(y+h>vh-pad)y=Math.max(pad, vh-h-pad);
if(y<pad)y=pad;
m.style.left=x+"px";
m.style.top=y+"px";
m.style.visibility="visible";
})();
"#;

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuSubTriggerProps {
    children: Element,
    #[props(default)]
    inset: bool,
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuSubTrigger(props: ContextMenuSubTriggerProps) -> Element {
    let mut sub = use_context::<ContextMenuSubState>();

    let inset_class = if props.inset { "pl-7" } else { "" };

    rsx! {
        div {
            "data-slot": "context-menu-sub-trigger",
            role: "menuitem",
            tabindex: "0",
            "aria-expanded": if (sub.open)() { "true" } else { "false" },
            class: "flex w-full cursor-default items-center gap-1.5 rounded-md px-1.5 py-1 text-sm outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground {inset_class} {props.class}",
            onclick: move |_| {
                let now = (sub.open)();
                sub.open.set(!now);
            },
            {props.children}
            svg {
                class: "ml-auto",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                path { d: "m9 18 6-6-6-6" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuSubContentProps {
    children: Element,
    #[props(default)]
    class: String,
}

#[component]
pub fn ContextMenuSubContent(props: ContextMenuSubContentProps) -> Element {
    let sub = use_context::<ContextMenuSubState>();
    let uid = use_hook(super::popup::next_popup_id);
    let open = sub.open;
    let effect_uid = uid.clone();

    use_effect(move || {
        if open() {
            let js = SUB_OPEN_JS.replace("__UID__", &effect_uid);
            let _ = document::eval(&js);
        }
    });

    if !(sub.open)() {
        return rsx! {};
    }

    rsx! {
        div {
            "data-slot": "context-menu-sub-content",
            "data-popup": uid,
            role: "menu",
            style: "position: fixed; left: 0px; top: 0px; visibility: hidden; z-index: 10001;",
            class: "min-w-36 max-h-72 overflow-y-auto rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10 outline-none animate-in fade-in-0 zoom-in-95 {props.class}",
            onclick: move |event| event.stop_propagation(),
            oncontextmenu: move |event| event.prevent_default(),
            {props.children}
        }
    }
}
