use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::toast::ToastState;
use crate::icons::{AlertCircleIcon, AppLogo, EyeIcon, EyeOffIcon, KeyIcon, LockIcon, ShieldCheckIcon, UserIcon};
use crate::Route;

#[component]
pub fn Login() -> Element {
    let nav = use_navigator();
    let toast = try_use_context::<ToastState>();
    let auth_signal = try_use_context::<Signal<crate::auth::AuthState>>();

    let mut identifier = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut show_password = use_signal(|| false);
    let mut remember_me = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut is_submitting = use_signal(|| false);

    let has_users = crate::auth::has_any_users();

    let mut handle_submit = move |_| {
        let id_val = identifier.read().trim().to_string();
        let pass_val = password.read().to_string();

        if id_val.is_empty() || pass_val.is_empty() {
            error_msg.set(Some("Please enter both username/email and password".to_string()));
            return;
        }

        is_submitting.set(true);
        error_msg.set(None);

        match crate::auth::login(&id_val, &pass_val) {
            Ok((user, token)) => {
                if let Some(mut state) = auth_signal {
                    state.set(crate::auth::AuthState {
                        user: Some(user.clone()),
                        token: Some(token),
                    });
                }
                if let Some(t) = toast {
                    t.success("Signed In", format!("Welcome back, {}!", user.display_name));
                }
                is_submitting.set(false);
                let _ = nav.replace(Route::Tickets {});
            }
            Err(e) => {
                error_msg.set(Some(e));
                is_submitting.set(false);
            }
        }
    };

    rsx! {
        div { class: "flex flex-1 items-center justify-center p-4 min-h-[75vh] w-full max-w-md mx-auto my-auto animate-in fade-in-50 duration-300",
            Card { class: "w-full border-border/80 shadow-lg bg-card/90 backdrop-blur-md",
                CardHeader { class: "text-center pb-4 pt-6 space-y-2",
                    div { class: "flex justify-center mb-1",
                        div { class: "flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary border border-primary/20 shadow-xs",
                            AppLogo { class: "size-8" }
                        }
                    }
                    CardTitle { class: "text-2xl font-bold tracking-tight text-foreground",
                        "Sign In"
                    }
                    CardDescription { class: "text-xs text-muted-foreground",
                        "Enter your credentials to access BPSL IT Support"
                    }
                }

                CardContent { class: "space-y-4 px-6 pt-2 pb-4",
                    // Initial setup notice if 0 users exist in the database
                    if !has_users {
                        div { class: "flex items-start gap-3 p-3.5 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300 text-xs",
                            ShieldCheckIcon { class: "size-4.5 shrink-0 mt-0.5 text-amber-500" }
                            div { class: "flex flex-col gap-1.5 flex-1",
                                span { class: "font-semibold text-foreground", "Fresh Install · Super Admin Setup" }
                                p { class: "leading-relaxed text-muted-foreground",
                                    "No accounts exist yet. Create your Super Admin account first—you can then create and manage all other user accounts and allowed email domains."
                                }
                                Link {
                                    to: Route::Register {},
                                    class: "inline-flex items-center gap-1.5 font-semibold text-primary hover:underline mt-0.5 text-xs",
                                    "Create Super Admin Account →"
                                }
                            }
                        }
                    }

                    // Error Alert
                    if let Some(err) = error_msg.read().as_ref() {
                        div { class: "flex items-center gap-2 p-3 rounded-lg border border-destructive/40 bg-destructive/10 text-destructive text-xs animate-in slide-in-from-top-1",
                            AlertCircleIcon { class: "size-4 shrink-0" }
                            span { class: "font-medium leading-tight", "{err}" }
                        }
                    }

                    form {
                        class: "space-y-3.5",
                        onsubmit: move |e| {
                            e.prevent_default();
                            handle_submit(());
                        },

                        // Username or Email Input
                        div { class: "space-y-1.5",
                            Label { r#for: "username_or_email", class: "text-xs font-medium text-foreground",
                                "Username or Email"
                            }
                            div { class: "relative",
                                div { class: "pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground",
                                    UserIcon { class: "size-4" }
                                }
                                Input {
                                    id: "username_or_email",
                                    r#type: "text".to_string(),
                                    class: "pl-9 text-xs h-9 bg-background/50",
                                    placeholder: "e.g. shreyash or it@bpsl.net".to_string(),
                                    value: identifier.read().clone(),
                                    oninput: move |e: FormEvent| identifier.set(e.value()),
                                }
                            }
                        }

                        // Password Input
                        div { class: "space-y-1.5",
                            div { class: "flex items-center justify-between",
                                Label { r#for: "password", class: "text-xs font-medium text-foreground",
                                    "Password"
                                }
                            }
                            div { class: "relative",
                                div { class: "pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground",
                                    LockIcon { class: "size-4" }
                                }
                                Input {
                                    id: "password",
                                    r#type: if *show_password.read() { "text".to_string() } else { "password".to_string() },
                                    class: "pl-9 pr-9 text-xs h-9 bg-background/50",
                                    placeholder: "Enter your password".to_string(),
                                    value: password.read().clone(),
                                    oninput: move |e: FormEvent| password.set(e.value()),
                                }
                                button {
                                    r#type: "button",
                                    class: "absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors p-1 cursor-pointer",
                                    onclick: move |_| {
                                        let cur = show_password();
                                        show_password.set(!cur);
                                    },
                                    title: if *show_password.read() { "Hide password" } else { "Show password" },
                                    if *show_password.read() {
                                        EyeOffIcon { class: "size-3.5" }
                                    } else {
                                        EyeIcon { class: "size-3.5" }
                                    }
                                }
                            }
                        }

                        // Remember Me Checkbox
                        div { class: "flex items-center justify-between pt-1",
                            label { class: "flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none",
                                Checkbox {
                                    checked: *remember_me.read(),
                                    on_checked_change: move |c| remember_me.set(c),
                                }
                                "Keep me signed in on this PC"
                            }
                        }

                        // Submit Button
                        Button {
                            r#type: "submit".to_string(),
                            class: "w-full h-9 text-xs font-semibold shadow-xs mt-2 cursor-pointer",
                            disabled: *is_submitting.read(),
                            onclick: move |_| handle_submit(()),
                            if *is_submitting.read() {
                                span { class: "inline-flex items-center gap-2",
                                    svg { class: "size-3.5 animate-spin", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                        circle { class: "opacity-25", cx: "12", cy: "12", r: "10" }
                                        path { class: "opacity-75", d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" }
                                    }
                                    "Signing in…"
                                }
                            } else {
                                span { class: "inline-flex items-center gap-1.5",
                                    KeyIcon { class: "size-3.5" }
                                    "Sign In"
                                }
                            }
                        }
                    }
                }

                CardFooter { class: "flex flex-col items-center justify-center border-t border-border/70 py-4 px-6 bg-muted/20 text-xs text-muted-foreground gap-1.5 text-center",
                    if !has_users {
                        div { class: "flex items-center gap-1.5",
                            span { "Fresh installation?" }
                            Link {
                                to: Route::Register {},
                                class: "font-semibold text-primary hover:underline cursor-pointer",
                                "Setup Super Admin"
                            }
                        }
                    } else {
                        span { "New accounts are created by your Super Admin from Profile → User Accounts." }
                    }
                }
            }
        }
    }
}
