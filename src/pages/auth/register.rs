use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::toast::ToastState;
use crate::icons::{AlertCircleIcon, AppLogo, CheckCircle2Icon, EyeIcon, EyeOffIcon, ShieldCheckIcon};
use crate::Route;

#[component]
pub fn Register() -> Element {
    let nav = use_navigator();
    let toast = try_use_context::<ToastState>();
    let auth_signal = try_use_context::<Signal<crate::auth::AuthState>>();

    let mut display_name = use_signal(String::new);
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut department = use_signal(|| "IT Support".to_string());
    let phone = use_signal(String::new);
    let mut selected_role = use_signal(|| "agent".to_string());
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut show_password = use_signal(|| false);

    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut is_submitting = use_signal(|| false);

    let is_first_user = !crate::auth::has_any_users();

    if !is_first_user {
        return rsx! {
            div { class: "flex flex-1 items-center justify-center p-4 min-h-[75vh] w-full max-w-md mx-auto my-auto animate-in fade-in-50 duration-300",
                Card { class: "w-full border-border/80 shadow-lg bg-card/90 backdrop-blur-md text-center",
                    CardHeader { class: "pb-3 pt-6 space-y-2",
                        div { class: "flex justify-center mb-1",
                            div { class: "flex size-14 items-center justify-center rounded-2xl bg-amber-500/10 text-amber-500 border border-amber-500/20 shadow-xs",
                                ShieldCheckIcon { class: "size-8" }
                            }
                        }
                        CardTitle { class: "text-xl font-bold tracking-tight text-foreground",
                            "Account Creation Restricted"
                        }
                        CardDescription { class: "text-xs text-muted-foreground leading-relaxed",
                            "A Super Admin account already exists on this installation. New user accounts can only be created by the Super Admin."
                        }
                    }
                    CardContent { class: "px-6 pb-6 pt-2",
                        Link {
                            to: Route::Login {},
                            class: "inline-flex w-full items-center justify-center gap-2 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground shadow-xs hover:bg-primary/90 transition-colors cursor-pointer",
                            "Back to Sign In"
                        }
                    }
                }
            }
        };
    }

    let mut handle_submit = move |_| {
        let name_val = display_name.read().trim().to_string();
        let uname_val = username.read().trim().to_lowercase();
        let email_val = email.read().trim().to_lowercase();
        let dept_val = department.read().trim().to_string();
        let phone_val = phone.read().trim().to_string();
        let pass_val = password.read().to_string();
        let confirm_val = confirm_password.read().to_string();
        let role_val = if is_first_user {
            "admin".to_string()
        } else {
            selected_role.read().clone()
        };

        if name_val.is_empty() {
            error_msg.set(Some("Please enter your full display name".to_string()));
            return;
        }
        if uname_val.len() < 3 {
            error_msg.set(Some("Username must be at least 3 characters long".to_string()));
            return;
        }
        if !email_val.contains('@') || !email_val.contains('.') {
            error_msg.set(Some("Please enter a valid email address".to_string()));
            return;
        }
        if pass_val.len() < 6 {
            error_msg.set(Some("Password must be at least 6 characters long".to_string()));
            return;
        }
        if pass_val != confirm_val {
            error_msg.set(Some("Passwords do not match".to_string()));
            return;
        }

        is_submitting.set(true);
        error_msg.set(None);

        let req = crate::auth::RegisterRequest {
            username: uname_val.clone(),
            email: email_val,
            display_name: name_val,
            password: pass_val.clone(),
            department: dept_val,
            phone: phone_val,
            requested_role: Some(role_val),
        };

        match crate::auth::register_user(req) {
            Ok(new_user) => {
                // Auto-login newly registered user
                if let Ok((logged_user, token)) = crate::auth::login(&uname_val, &pass_val) {
                    if let Some(mut state) = auth_signal {
                        state.set(crate::auth::AuthState {
                            user: Some(logged_user),
                            token: Some(token),
                        });
                    }
                }
                if let Some(t) = toast {
                    t.success(
                        "Account Created",
                        format!("Welcome to BPSL IT Support, {}!", new_user.display_name),
                    );
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

    // Password strength check
    let pass_len = password.read().len();
    let has_upper = password.read().chars().any(|c| c.is_uppercase());
    let has_digit = password.read().chars().any(|c| c.is_ascii_digit());
    let pass_score = if pass_len < 6 {
        0
    } else {
        1 + usize::from(pass_len >= 8) + usize::from(has_upper) + usize::from(has_digit)
    };

    rsx! {
        div { class: "flex flex-1 items-center justify-center p-4 min-h-[85vh] w-full max-w-lg mx-auto my-auto animate-in fade-in-50 duration-300",
            Card { class: "w-full border-border/80 shadow-lg bg-card/90 backdrop-blur-md my-6",
                CardHeader { class: "text-center pb-3 pt-6 space-y-2",
                    div { class: "flex justify-center mb-1",
                        div { class: "flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary border border-primary/20 shadow-xs",
                            AppLogo { class: "size-8" }
                        }
                    }
                    CardTitle { class: "text-2xl font-bold tracking-tight text-foreground",
                        "Setup Super Admin"
                    }
                    CardDescription { class: "text-xs text-muted-foreground",
                        "Create the primary Super Admin account for BPSL IT Support"
                    }
                }

                CardContent { class: "space-y-4 px-6 pt-2 pb-4",
                    div { class: "flex items-start gap-2.5 p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300 text-xs",
                        ShieldCheckIcon { class: "size-4.5 shrink-0 mt-0.5 text-amber-500" }
                        p { class: "leading-relaxed",
                            "This Super Admin account has full root privileges to create other user accounts, configure custom company email domains (e.g. .bhagwatipower.com, .gmail.com), and manage all tickets. Brackets (), [], {{}} are allowed in usernames."
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
                        class: "space-y-3",
                        onsubmit: move |e| {
                            e.prevent_default();
                            handle_submit(());
                        },

                        // Display Name & Username in 2 columns
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                            div { class: "space-y-1.5",
                                Label { r#for: "reg_name", class: "text-xs font-medium text-foreground",
                                    "Full Name *"
                                }
                                Input {
                                    id: "reg_name",
                                    r#type: "text".to_string(),
                                    class: "text-xs h-9 bg-background/50",
                                    placeholder: "e.g. Shreyash Patel".to_string(),
                                    value: display_name.read().clone(),
                                    oninput: move |e: FormEvent| display_name.set(e.value()),
                                }
                            }
                            div { class: "space-y-1.5",
                                Label { r#for: "reg_username", class: "text-xs font-medium text-foreground",
                                    "Username *"
                                }
                                Input {
                                    id: "reg_username",
                                    r#type: "text".to_string(),
                                    class: "text-xs h-9 bg-background/50",
                                    placeholder: "e.g. shreyash".to_string(),
                                    value: username.read().clone(),
                                    oninput: move |e: FormEvent| username.set(e.value()),
                                }
                            }
                        }

                        // Email Address
                        div { class: "space-y-1.5",
                            Label { r#for: "reg_email", class: "text-xs font-medium text-foreground",
                                "Corporate Email *"
                            }
                            Input {
                                id: "reg_email",
                                r#type: "email".to_string(),
                                class: "text-xs h-9 bg-background/50",
                                placeholder: "e.g. shreyash@bpsl.net".to_string(),
                                value: email.read().clone(),
                                oninput: move |e: FormEvent| email.set(e.value()),
                            }
                        }

                        // Department & Role in 2 columns
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                            div { class: "space-y-1.5",
                                Label { r#for: "reg_dept", class: "text-xs font-medium text-foreground",
                                    "Department"
                                }
                                select {
                                    id: "reg_dept",
                                    class: "w-full h-9 px-3 text-xs rounded-md border border-input bg-background/50 text-foreground focus:outline-none focus:ring-1 focus:ring-primary shadow-2xs",
                                    value: department.read().as_str(),
                                    onchange: move |e| department.set(e.value()),
                                    option { value: "IT Support", "IT Support" }
                                    option { value: "Electrical", "Electrical" }
                                    option { value: "Mechanical", "Mechanical" }
                                    option { value: "Power Plant", "Power Plant" }
                                    option { value: "SMS (Steel Melting Shop)", "SMS" }
                                    option { value: "Administration", "Administration" }
                                    option { value: "Human Resources", "Human Resources" }
                                    option { value: "Operations", "Operations" }
                                    option { value: "Quality Control", "Quality Control" }
                                }
                            }

                            div { class: "space-y-1.5",
                                Label { class: "text-xs font-medium text-foreground",
                                    "System Role"
                                }
                                if is_first_user {
                                    div { class: "flex items-center h-9 px-3 text-xs rounded-md border border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-400 font-semibold",
                                        "Super Admin (Root)"
                                    }
                                } else {
                                    select {
                                        class: "w-full h-9 px-3 text-xs rounded-md border border-input bg-background/50 text-foreground focus:outline-none focus:ring-1 focus:ring-primary shadow-2xs",
                                        value: selected_role.read().as_str(),
                                        onchange: move |e| selected_role.set(e.value()),
                                        option { value: "agent", "Support Agent" }
                                        option { value: "viewer", "Read-Only Viewer" }
                                        option { value: "admin", "Administrator" }
                                    }
                                }
                            }
                        }

                        // Passwords in 2 columns
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                            div { class: "space-y-1.5",
                                Label { r#for: "reg_password", class: "text-xs font-medium text-foreground",
                                    "Password *"
                                }
                                div { class: "relative",
                                    Input {
                                        id: "reg_password",
                                        r#type: if *show_password.read() { "text".to_string() } else { "password".to_string() },
                                        class: "pr-9 text-xs h-9 bg-background/50",
                                        placeholder: "At least 6 chars".to_string(),
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
                                        if *show_password.read() {
                                            EyeOffIcon { class: "size-3.5" }
                                        } else {
                                            EyeIcon { class: "size-3.5" }
                                        }
                                    }
                                }
                            }
                            div { class: "space-y-1.5",
                                Label { r#for: "reg_confirm", class: "text-xs font-medium text-foreground",
                                    "Confirm Password *"
                                }
                                Input {
                                    id: "reg_confirm",
                                    r#type: if *show_password.read() { "text".to_string() } else { "password".to_string() },
                                    class: "text-xs h-9 bg-background/50",
                                    placeholder: "Re-enter password".to_string(),
                                    value: confirm_password.read().clone(),
                                    oninput: move |e: FormEvent| confirm_password.set(e.value()),
                                }
                            }
                        }

                        // Password strength meter
                        if !password.read().is_empty() {
                            div { class: "space-y-1 pt-1",
                                div { class: "flex items-center justify-between text-[11px] text-muted-foreground",
                                    span { "Password Strength" }
                                    span {
                                        class: match pass_score {
                                            0 | 1 => "text-destructive font-medium",
                                            2 => "text-amber-500 font-medium",
                                            3 => "text-sky-500 font-medium",
                                            _ => "text-emerald-500 font-medium",
                                        },
                                        match pass_score {
                                            0 | 1 => "Weak",
                                            2 => "Fair",
                                            3 => "Good",
                                            _ => "Strong",
                                        }
                                    }
                                }
                                div { class: "h-1.5 w-full bg-muted rounded-full overflow-hidden flex gap-1",
                                    for i in 1..=4 {
                                        div {
                                            class: format!(
                                                "h-full flex-1 rounded-full transition-colors {}",
                                                if pass_score >= i {
                                                    match pass_score {
                                                        1 => "bg-destructive",
                                                        2 => "bg-amber-500",
                                                        3 => "bg-sky-500",
                                                        _ => "bg-emerald-500",
                                                    }
                                                } else {
                                                    "bg-border/60"
                                                }
                                            )
                                        }
                                    }
                                }
                            }
                        }

                        // Submit Button
                        Button {
                            r#type: "submit".to_string(),
                            class: "w-full h-9 text-xs font-semibold shadow-xs mt-3 cursor-pointer",
                            disabled: *is_submitting.read(),
                            onclick: move |_| handle_submit(()),
                            if *is_submitting.read() {
                                span { class: "inline-flex items-center gap-2",
                                    svg { class: "size-3.5 animate-spin", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                        circle { class: "opacity-25", cx: "12", cy: "12", r: "10" }
                                        path { class: "opacity-75", d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" }
                                    }
                                    "Creating Account…"
                                }
                            } else {
                                span { class: "inline-flex items-center gap-1.5",
                                    CheckCircle2Icon { class: "size-3.5" }
                                    "Create Super Admin Account"
                                }
                            }
                        }
                    }
                }

                CardFooter { class: "flex flex-col items-center justify-center border-t border-border/70 py-4 px-6 bg-muted/20 text-xs text-muted-foreground gap-1.5",
                    div { class: "flex items-center gap-1.5",
                        span { "Already have an account?" }
                        Link {
                            to: Route::Login {},
                            class: "font-semibold text-primary hover:underline cursor-pointer",
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}
