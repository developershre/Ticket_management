//! Authentication & User Management module for BPSL IT Support.
//!
//! Provides secure password hashing (SHA-256 + cryptographic salt),
//! SQLite-backed user credentials and active session management,
//! role-based authorization (admin, agent, viewer), and profile management.

use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// An authenticated user profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: String, // "super_admin" | "admin" | "agent" | "viewer"
    pub department: String,
    pub phone: String,
    pub avatar_color: String,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

/// An allowed corporate/custom email domain configured by the Super Admin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailDomain {
    pub id: i64,
    pub domain: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: String,
}

impl AuthUser {
    pub fn is_super_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("super_admin") || self.id == 1
    }

    pub fn is_admin(&self) -> bool {
        self.is_super_admin() || self.role.eq_ignore_ascii_case("admin")
    }

    pub fn is_agent(&self) -> bool {
        self.is_admin() || self.role.eq_ignore_ascii_case("agent")
    }

    pub fn role_label(&self) -> &'static str {
        if self.is_super_admin() {
            return "Super Admin";
        }
        match self.role.to_lowercase().as_str() {
            "super_admin" => "Super Admin",
            "admin" => "Administrator",
            "agent" => "Support Agent",
            "viewer" => "Read-Only Viewer",
            _ => "User",
        }
    }

    pub fn role_badge_color(&self) -> &'static str {
        if self.is_super_admin() {
            return "bg-amber-500/15 text-amber-700 dark:text-amber-400 border-amber-500/30";
        }
        match self.role.to_lowercase().as_str() {
            "super_admin" => "bg-amber-500/15 text-amber-700 dark:text-amber-400 border-amber-500/30",
            "admin" => "bg-indigo-500/15 text-indigo-700 dark:text-indigo-400 border-indigo-500/30",
            "agent" => "bg-sky-500/15 text-sky-700 dark:text-sky-400 border-sky-500/30",
            "viewer" => "bg-zinc-500/15 text-zinc-700 dark:text-zinc-400 border-zinc-500/30",
            _ => "bg-muted text-muted-foreground border-border",
        }
    }

    pub fn initials(&self) -> String {
        let name = self.display_name.trim();
        if name.is_empty() {
            let u = self.username.trim();
            if u.is_empty() {
                return "?".to_string();
            }
            return u.chars().take(2).collect::<String>().to_uppercase();
        }
        let parts: Vec<&str> = name.split_whitespace().collect();
        if parts.len() >= 2 {
            let first = parts[0].chars().next().unwrap_or('?');
            let last = parts[parts.len() - 1].chars().next().unwrap_or('?');
            format!("{}{}", first, last).to_uppercase()
        } else {
            name.chars().take(2).collect::<String>().to_uppercase()
        }
    }
}

/// Request payload to register a new user account.
#[derive(Debug, Clone)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub department: String,
    pub phone: String,
    pub requested_role: Option<String>,
}

/// Global reactive authentication state passed through Dioxus context.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthState {
    pub user: Option<AuthUser>,
    pub token: Option<String>,
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    pub fn current_user(&self) -> Option<&AuthUser> {
        self.user.as_ref()
    }

    pub fn is_admin(&self) -> bool {
        self.user.as_ref().map(|u| u.is_admin()).unwrap_or(false)
    }
}

// ============================================================
// Cryptographic Helpers
// ============================================================

/// Generates a random cryptographic salt using UUID v4.
pub fn generate_salt() -> String {
    Uuid::new_v4().simple().to_string()
}

/// Computes SHA-256(salt + ":bpsl_auth_v1:" + password).
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":bpsl_auth_v1:");
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verifies whether the provided raw password matches the stored salt and hash.
pub fn verify_password(password: &str, salt: &str, expected_hash: &str) -> bool {
    let computed = hash_password(password, salt);
    computed.eq_ignore_ascii_case(expected_hash)
}

// ============================================================
// Database Schema Initialization
// ============================================================

/// Ensures `auth_users` and `auth_sessions` exist in SQLite.
pub fn init_auth_tables(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS auth_users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL COLLATE NOCASE,
            email TEXT UNIQUE NOT NULL COLLATE NOCASE,
            display_name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'agent',
            department TEXT DEFAULT '',
            phone TEXT DEFAULT '',
            avatar_color TEXT DEFAULT '',
            is_active BOOLEAN DEFAULT 1,
            created_at TEXT NOT NULL,
            last_login_at TEXT
        );

        CREATE TABLE IF NOT EXISTS auth_sessions (
            token TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT,
            FOREIGN KEY(user_id) REFERENCES auth_users(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS allowed_email_domains (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT UNIQUE NOT NULL COLLATE NOCASE,
            description TEXT DEFAULT '',
            is_active BOOLEAN DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_auth_users_username ON auth_users(username);
        CREATE INDEX IF NOT EXISTS idx_auth_users_email ON auth_users(email);
        CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_id ON auth_sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_allowed_email_domains ON allowed_email_domains(domain);

        -- Ensure the first registered account (id=1 or lowest id) is promoted to super_admin
        UPDATE auth_users
        SET role = 'super_admin'
        WHERE id = (SELECT id FROM auth_users ORDER BY id ASC LIMIT 1)
          AND LOWER(role) != 'super_admin';
        "#,
    )?;
    Ok(())
}

fn now_iso() -> String {
    chrono_like_now()
}

fn chrono_like_now() -> String {
    crate::state::format_utc(crate::sync::now_epoch())
}

fn avatar_colors() -> &'static [&'static str] {
    &[
        "bg-blue-600",
        "bg-indigo-600",
        "bg-violet-600",
        "bg-emerald-600",
        "bg-amber-600",
        "bg-rose-600",
        "bg-cyan-600",
    ]
}

fn pick_avatar_color(username: &str) -> String {
    let colors = avatar_colors();
    let sum: usize = username.bytes().map(|b| b as usize).sum();
    colors[sum % colors.len()].to_string()
}

// ============================================================
// Auth Queries & Operations
// ============================================================

/// Checks whether any user has been registered in the database yet.
pub fn has_any_users() -> bool {
    let Ok(conn) = crate::db::get_connection() else {
        return false;
    };
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM auth_users", [], |r| r.get(0))
        .unwrap_or(0);
    count > 0
}

/// Validates username: minimum 3 characters, allows alphanumeric, `_`, `.`, `-`, and brackets `()`, `[]`, `{}`.
pub fn validate_username(raw: &str) -> Result<String, String> {
    let username = raw.trim().to_lowercase();
    if username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }
    let valid = username.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, '_' | '.' | '-' | '(' | ')' | '[' | ']' | '{' | '}')
    });
    if !valid {
        return Err(
            "Username may only contain letters, numbers, dots, underscores, hyphens, and brackets ((), [], {})"
                .to_string(),
        );
    }
    Ok(username)
}

/// Normalizes a domain string (e.g. `.gmail.com`, `@bhagwatipower.com`, `bhagwatipower.com` -> `bhagwatipower.com`).
pub fn normalize_domain(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .trim_start_matches('@')
        .trim_start_matches('.')
        .to_string()
}

/// Checks whether an email matches basic syntax and any active Super Admin domain restrictions.
/// When no active domains are configured in `allowed_email_domains`, any valid email is permitted.
pub fn validate_email_against_domains(email: &str) -> Result<String, String> {
    let email = email.trim().to_lowercase();
    let Some((local, domain)) = email.split_once('@') else {
        return Err("Please enter a valid email address".to_string());
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return Err("Please enter a valid email address".to_string());
    }

    let active_domains: Vec<String> = list_email_domains()
        .into_iter()
        .filter(|d| d.is_active)
        .map(|d| d.domain)
        .collect();

    if active_domains.is_empty() {
        return Ok(email);
    }

    let matched = active_domains
        .iter()
        .any(|allowed| domain == allowed || domain.ends_with(&format!(".{allowed}")));

    if !matched {
        let formatted_list: Vec<String> = active_domains.iter().map(|d| format!("@{d}")).collect();
        return Err(format!(
            "Email domain '@{domain}' is not allowed. Permitted domains: {}",
            formatted_list.join(", ")
        ));
    }

    Ok(email)
}

fn insert_user_internal(req: RegisterRequest, forced_role: &str) -> Result<AuthUser, String> {
    let username = validate_username(&req.username)?;
    let email = validate_email_against_domains(&req.email)?;
    let display_name = req.display_name.trim().to_string();
    let password = req.password;

    if display_name.is_empty() {
        return Err("Display name cannot be empty".to_string());
    }
    if password.len() < 6 {
        return Err("Password must be at least 6 characters long".to_string());
    }

    let conn = crate::db::get_connection().map_err(|e| format!("Database connection error: {e}"))?;

    // Check username collision
    let user_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM auth_users WHERE username = ?1)",
            params![username],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if user_exists {
        return Err(format!("Username '{username}' is already taken"));
    }

    // Check email collision
    let email_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM auth_users WHERE email = ?1)",
            params![email],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if email_exists {
        return Err(format!("An account with email '{email}' already exists"));
    }

    let salt = generate_salt();
    let hash = hash_password(&password, &salt);
    let created_at = now_iso();
    let avatar_color = pick_avatar_color(&username);

    conn.execute(
        r#"
        INSERT INTO auth_users (
            username, email, display_name, password_hash, salt,
            role, department, phone, avatar_color, is_active, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10)
        "#,
        params![
            username,
            email,
            display_name,
            hash,
            salt,
            forced_role,
            req.department.trim(),
            req.phone.trim(),
            avatar_color,
            created_at,
        ],
    )
    .map_err(|e| format!("Failed to create user account: {e}"))?;

    let new_id = conn.last_insert_rowid();
    get_user_by_id(new_id).ok_or_else(|| "Failed to retrieve registered user".to_string())
}

/// Registers the initial Super Admin account on a fresh installation.
/// Once a Super Admin exists, public registration is locked and new accounts must be
/// created by the Super Admin.
pub fn register_user(req: RegisterRequest) -> Result<AuthUser, String> {
    if has_any_users() {
        return Err(
            "An administrator account already exists. New accounts must be created by the Super Admin."
                .to_string(),
        );
    }
    insert_user_internal(req, "super_admin")
}

/// Creates a new user account on behalf of the Super Admin.
pub fn create_user_by_admin(requesting_user_id: i64, req: RegisterRequest) -> Result<AuthUser, String> {
    let Some(requester) = get_user_by_id(requesting_user_id) else {
        return Err("Requester account not found".to_string());
    };
    if !requester.is_super_admin() {
        return Err("Only the Super Admin can create new user accounts".to_string());
    }

    let role = req
        .requested_role
        .as_deref()
        .unwrap_or("agent")
        .trim()
        .to_lowercase();
    let valid_roles = ["admin", "agent", "viewer"];
    if !valid_roles.contains(&role.as_str()) {
        return Err(format!("Invalid role '{role}'. Choose admin, agent, or viewer."));
    }

    insert_user_internal(req, &role)
}

/// Authenticates a user by username or email and raw password.
/// Returns the authenticated `AuthUser` and a new session token.
pub fn login(username_or_email: &str, password: &str) -> Result<(AuthUser, String), String> {
    let identifier = username_or_email.trim();
    if identifier.is_empty() || password.is_empty() {
        return Err("Please enter both username/email and password".to_string());
    }

    let conn = crate::db::get_connection().map_err(|e| format!("Database connection error: {e}"))?;

    let row = conn
        .query_row(
            r#"
            SELECT id, username, email, display_name, password_hash, salt,
                   role, department, phone, avatar_color, is_active, created_at, last_login_at
            FROM auth_users
            WHERE username = ?1 OR email = ?1
            "#,
            params![identifier],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, String>(9)?,
                    r.get::<_, bool>(10)?,
                    r.get::<_, String>(11)?,
                    r.get::<_, Option<String>>(12)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("Database query error: {e}"))?;

    let Some((
        id,
        uname,
        em,
        dname,
        stored_hash,
        salt,
        role,
        dept,
        phone,
        avatar_col,
        is_active,
        created,
        _,
    )) = row
    else {
        return Err("Invalid username or password".to_string());
    };

    if !is_active {
        return Err("Your account has been deactivated. Please contact an administrator.".to_string());
    }

    if !verify_password(password, &salt, &stored_hash) {
        return Err("Invalid username or password".to_string());
    }

    let now = now_iso();
    let _ = conn.execute(
        "UPDATE auth_users SET last_login_at = ?1 WHERE id = ?2",
        params![now, id],
    );

    // Create session token
    let token = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO auth_sessions (token, user_id, created_at) VALUES (?1, ?2, ?3)",
        params![token, id, now],
    );

    // Persist active session token in sync_meta
    let _ = crate::db::sync_meta_set("auth_active_token", &token);

    let user = AuthUser {
        id,
        username: uname,
        email: em,
        display_name: dname,
        role,
        department: dept,
        phone,
        avatar_color: avatar_col,
        is_active,
        created_at: created,
        last_login_at: Some(now),
    };

    Ok((user, token))
}

/// Logs out the current session and clears the active session token.
pub fn logout() -> Result<(), String> {
    if let Some(token) = crate::db::sync_meta_get("auth_active_token") {
        if let Ok(conn) = crate::db::get_connection() {
            let _ = conn.execute("DELETE FROM auth_sessions WHERE token = ?1", params![token]);
        }
    }
    let _ = crate::db::sync_meta_set("auth_active_token", "");
    Ok(())
}

/// Retrieves the user for a specific session token.
pub fn get_user_by_token(token: &str) -> Option<AuthUser> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    let conn = crate::db::get_connection().ok()?;
    conn.query_row(
        r#"
        SELECT u.id, u.username, u.email, u.display_name, u.role,
               u.department, u.phone, u.avatar_color, u.is_active,
               u.created_at, u.last_login_at
        FROM auth_sessions s
        JOIN auth_users u ON s.user_id = u.id
        WHERE s.token = ?1 AND u.is_active = 1
        "#,
        params![token],
        |r| {
            Ok(AuthUser {
                id: r.get(0)?,
                username: r.get(1)?,
                email: r.get(2)?,
                display_name: r.get(3)?,
                role: r.get(4)?,
                department: r.get(5)?,
                phone: r.get(6)?,
                avatar_color: r.get(7)?,
                is_active: r.get(8)?,
                created_at: r.get(9)?,
                last_login_at: r.get(10)?,
            })
        },
    )
    .optional()
    .ok()
    .flatten()
}

/// Retrieves a user by their primary key id.
pub fn get_user_by_id(id: i64) -> Option<AuthUser> {
    let conn = crate::db::get_connection().ok()?;
    conn.query_row(
        r#"
        SELECT id, username, email, display_name, role,
               department, phone, avatar_color, is_active,
               created_at, last_login_at
        FROM auth_users
        WHERE id = ?1
        "#,
        params![id],
        |r| {
            Ok(AuthUser {
                id: r.get(0)?,
                username: r.get(1)?,
                email: r.get(2)?,
                display_name: r.get(3)?,
                role: r.get(4)?,
                department: r.get(5)?,
                phone: r.get(6)?,
                avatar_color: r.get(7)?,
                is_active: r.get(8)?,
                created_at: r.get(9)?,
                last_login_at: r.get(10)?,
            })
        },
    )
    .optional()
    .ok()
    .flatten()
}

/// Retrieves the currently authenticated user from the active session.
pub fn current_active_user() -> Option<AuthUser> {
    let token = crate::db::sync_meta_get("auth_active_token")?;
    get_user_by_token(&token)
}

/// Updates an existing user's profile details.
pub fn update_profile(
    user_id: i64,
    display_name: &str,
    email: &str,
    department: &str,
    phone: &str,
) -> Result<AuthUser, String> {
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err("Display name cannot be empty".to_string());
    }
    let email = validate_email_against_domains(email)?;

    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;

    // Check email uniqueness excluding current user
    let email_taken: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM auth_users WHERE email = ?1 AND id != ?2)",
            params![email, user_id],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if email_taken {
        return Err(format!("The email '{email}' is already in use by another account"));
    }

    conn.execute(
        r#"
        UPDATE auth_users
        SET display_name = ?1, email = ?2, department = ?3, phone = ?4
        WHERE id = ?5
        "#,
        params![display_name, email, department.trim(), phone.trim(), user_id],
    )
    .map_err(|e| format!("Failed to update profile: {e}"))?;

    get_user_by_id(user_id).ok_or_else(|| "Failed to reload updated profile".to_string())
}

/// Updates a user's avatar color theme.
pub fn update_avatar_color(user_id: i64, color: &str) -> Result<AuthUser, String> {
    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;
    conn.execute(
        "UPDATE auth_users SET avatar_color = ?1 WHERE id = ?2",
        params![color.trim(), user_id],
    )
    .map_err(|e| format!("Failed to update avatar color: {e}"))?;

    get_user_by_id(user_id).ok_or_else(|| "Failed to reload updated profile".to_string())
}

/// Changes a user's password after verifying the old password.
pub fn change_password(user_id: i64, current_password: &str, new_password: &str) -> Result<(), String> {
    if new_password.len() < 6 {
        return Err("New password must be at least 6 characters long".to_string());
    }

    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;

    let (stored_hash, salt): (String, String) = conn
        .query_row(
            "SELECT password_hash, salt FROM auth_users WHERE id = ?1",
            params![user_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "User not found".to_string())?;

    if !verify_password(current_password, &salt, &stored_hash) {
        return Err("Current password is incorrect".to_string());
    }

    let new_salt = generate_salt();
    let new_hash = hash_password(new_password, &new_salt);

    conn.execute(
        "UPDATE auth_users SET password_hash = ?1, salt = ?2 WHERE id = ?3",
        params![new_hash, new_salt, user_id],
    )
    .map_err(|e| format!("Failed to update password: {e}"))?;

    Ok(())
}

/// Lists all registered user accounts (for Admin view).
pub fn list_all_users() -> Vec<AuthUser> {
    let Ok(conn) = crate::db::get_connection() else {
        return Vec::new();
    };
    let mut stmt = match conn.prepare(
        r#"
        SELECT id, username, email, display_name, role,
               department, phone, avatar_color, is_active,
               created_at, last_login_at
        FROM auth_users
        ORDER BY id ASC
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |r| {
        Ok(AuthUser {
            id: r.get(0)?,
            username: r.get(1)?,
            email: r.get(2)?,
            display_name: r.get(3)?,
            role: r.get(4)?,
            department: r.get(5)?,
            phone: r.get(6)?,
            avatar_color: r.get(7)?,
            is_active: r.get(8)?,
            created_at: r.get(9)?,
            last_login_at: r.get(10)?,
        })
    });

    match rows {
        Ok(iter) => iter.flatten().collect(),
        Err(_) => Vec::new(),
    }
}

/// Changes the role of a user (Super Admin operation). Cannot demote a Super Admin.
pub fn update_user_role(target_id: i64, new_role: &str) -> Result<(), String> {
    let valid_roles = ["admin", "agent", "viewer"];
    let role_lower = new_role.to_lowercase();
    if !valid_roles.contains(&role_lower.as_str()) {
        return Err(format!("Invalid role '{new_role}'. Must be admin, agent, or viewer."));
    }

    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;
    let current_role: String = conn
        .query_row(
            "SELECT role FROM auth_users WHERE id = ?1",
            params![target_id],
            |r| r.get(0),
        )
        .map_err(|_| "User not found".to_string())?;
    if current_role.eq_ignore_ascii_case("super_admin") {
        return Err("The Super Admin role cannot be changed".to_string());
    }

    conn.execute(
        "UPDATE auth_users SET role = ?1 WHERE id = ?2",
        params![role_lower, target_id],
    )
    .map_err(|e| format!("Failed to update user role: {e}"))?;

    Ok(())
}

/// Toggles active status of an account (Admin/Super Admin). Cannot deactivate oneself or a Super Admin.
pub fn toggle_user_active(target_id: i64, current_admin_id: i64) -> Result<bool, String> {
    if target_id == current_admin_id {
        return Err("You cannot deactivate your own account".to_string());
    }
    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;
    let (is_active, role): (bool, String) = conn
        .query_row(
            "SELECT is_active, role FROM auth_users WHERE id = ?1",
            params![target_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "User not found".to_string())?;

    if role.eq_ignore_ascii_case("super_admin") {
        return Err("The Super Admin account cannot be deactivated".to_string());
    }

    let new_state = !is_active;
    conn.execute(
        "UPDATE auth_users SET is_active = ?1 WHERE id = ?2",
        params![new_state, target_id],
    )
    .map_err(|e| format!("Failed to update status: {e}"))?;

    if !new_state {
        // Invalidate any active sessions for the deactivated user
        let _ = conn.execute("DELETE FROM auth_sessions WHERE user_id = ?1", params![target_id]);
    }

    Ok(new_state)
}

// ============================================================
// Allowed Email Domains (Super Admin Domain Management)
// ============================================================

/// Lists all configured allowed email domains.
pub fn list_email_domains() -> Vec<EmailDomain> {
    let Ok(conn) = crate::db::get_connection() else {
        return Vec::new();
    };
    let mut stmt = match conn.prepare(
        r#"
        SELECT id, domain, description, is_active, created_at
        FROM allowed_email_domains
        ORDER BY id ASC
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |r| {
        Ok(EmailDomain {
            id: r.get(0)?,
            domain: r.get(1)?,
            description: r.get(2)?,
            is_active: r.get(3)?,
            created_at: r.get(4)?,
        })
    });

    match rows {
        Ok(iter) => iter.flatten().collect(),
        Err(_) => Vec::new(),
    }
}

/// Adds a new allowed email domain (e.g. `.gmail.com`, `.bhagwatipower.com`, `company.com`).
pub fn add_email_domain(raw_domain: &str, description: &str) -> Result<EmailDomain, String> {
    let domain = normalize_domain(raw_domain);
    if domain.is_empty() || !domain.contains('.') || domain.ends_with('.') || domain.contains(' ') || domain.contains('@') {
        return Err("Please enter a valid domain such as .bhagwatipower.com or gmail.com".to_string());
    }

    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM allowed_email_domains WHERE domain = ?1)",
            params![domain],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if exists {
        return Err(format!("Domain '@{domain}' is already configured"));
    }

    let created_at = now_iso();
    conn.execute(
        "INSERT INTO allowed_email_domains (domain, description, is_active, created_at) VALUES (?1, ?2, 1, ?3)",
        params![domain, description.trim(), created_at],
    )
    .map_err(|e| format!("Failed to add email domain: {e}"))?;

    let id = conn.last_insert_rowid();
    Ok(EmailDomain {
        id,
        domain,
        description: description.trim().to_string(),
        is_active: true,
        created_at,
    })
}

/// Updates an existing allowed email domain and its description.
pub fn update_email_domain(id: i64, raw_domain: &str, description: &str) -> Result<(), String> {
    let domain = normalize_domain(raw_domain);
    if domain.is_empty() || !domain.contains('.') || domain.ends_with('.') || domain.contains(' ') || domain.contains('@') {
        return Err("Please enter a valid domain such as .bhagwatipower.com or gmail.com".to_string());
    }

    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;
    let duplicate: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM allowed_email_domains WHERE domain = ?1 AND id != ?2)",
            params![domain, id],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if duplicate {
        return Err(format!("Domain '@{domain}' already exists"));
    }

    conn.execute(
        "UPDATE allowed_email_domains SET domain = ?1, description = ?2 WHERE id = ?3",
        params![domain, description.trim(), id],
    )
    .map_err(|e| format!("Failed to update email domain: {e}"))?;
    Ok(())
}

/// Toggles active status of an allowed email domain.
pub fn toggle_email_domain_active(id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;
    let is_active: bool = conn
        .query_row(
            "SELECT is_active FROM allowed_email_domains WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| "Domain not found".to_string())?;
    let next = !is_active;
    conn.execute(
        "UPDATE allowed_email_domains SET is_active = ?1 WHERE id = ?2",
        params![next, id],
    )
    .map_err(|e| format!("Failed to toggle domain: {e}"))?;
    Ok(next)
}

/// Deletes an allowed email domain by ID.
pub fn delete_email_domain(id: i64) -> Result<(), String> {
    let conn = crate::db::get_connection().map_err(|e| format!("Database error: {e}"))?;
    conn.execute("DELETE FROM allowed_email_domains WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete domain: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let pass = "P@ssw0rd123!";
        let salt = generate_salt();
        let hash = hash_password(pass, &salt);
        assert!(verify_password(pass, &salt, &hash));
        assert!(!verify_password("wrong_password", &salt, &hash));
    }

    #[test]
    fn test_auth_user_initials() {
        let u = AuthUser {
            id: 1,
            username: "jdoe".to_string(),
            email: "jdoe@example.com".to_string(),
            display_name: "John Doe".to_string(),
            role: "agent".to_string(),
            department: "IT".to_string(),
            phone: "".to_string(),
            avatar_color: "bg-blue-600".to_string(),
            is_active: true,
            created_at: "2026-01-01".to_string(),
            last_login_at: None,
        };
        assert_eq!(u.initials(), "JD");

        let single = AuthUser {
            display_name: "Shreyash".to_string(),
            ..u.clone()
        };
        assert_eq!(single.initials(), "SH");
    }

    #[test]
    fn test_username_brackets_allowed() {
        assert_eq!(validate_username("shreyash(it)").unwrap(), "shreyash(it)");
        assert_eq!(validate_username("admin[bpsl]").unwrap(), "admin[bpsl]");
        assert_eq!(validate_username("ops{01}").unwrap(), "ops{01}");
        assert!(validate_username("bad@name").is_err());
        assert!(validate_username("ab").is_err());
    }

    #[test]
    fn test_domain_normalization() {
        assert_eq!(normalize_domain(".gmail.com"), "gmail.com");
        assert_eq!(normalize_domain("@bhagwatipower.com"), "bhagwatipower.com");
        assert_eq!(normalize_domain("  .BPSL.CO.IN "), "bpsl.co.in");
    }
}
