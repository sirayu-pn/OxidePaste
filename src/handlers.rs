use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use askama::Template;
use sqlx::SqlitePool;

use crate::models::{CreatePasteForm, Expiration, LoginForm, PasswordForm, Paste, RegisterForm, User};
use crate::utils::{generate_id, hash_password, verify_password};

const SESSION_COOKIE: &str = "oxide_session";

// =============================================================================
// Templates
// =============================================================================

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub languages: Vec<(&'static str, &'static str)>,
    pub user: Option<User>,
}

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewTemplate {
    pub paste: Paste,
    pub formatted_date: String,
    pub expires_in: Option<String>,
    pub user: Option<User>,
    pub is_owner: bool,
}

#[derive(Template)]
#[template(path = "password.html")]
pub struct PasswordTemplate {
    pub id: String,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "not_found.html")]
pub struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub user: User,
    pub pastes: Vec<Paste>,
}

#[derive(Template)]
#[template(path = "public.html")]
pub struct PublicTemplate {
    pub user: Option<User>,
    pub pastes: Vec<Paste>,
}

// =============================================================================
// Supported Languages
// =============================================================================

fn get_supported_languages() -> Vec<(&'static str, &'static str)> {
    vec![
        ("plaintext", "Plain Text"),
        ("rust", "Rust"),
        ("javascript", "JavaScript"),
        ("typescript", "TypeScript"),
        ("python", "Python"),
        ("go", "Go"),
        ("java", "Java"),
        ("c", "C"),
        ("cpp", "C++"),
        ("csharp", "C#"),
        ("php", "PHP"),
        ("ruby", "Ruby"),
        ("swift", "Swift"),
        ("kotlin", "Kotlin"),
        ("sql", "SQL"),
        ("html", "HTML"),
        ("css", "CSS"),
        ("json", "JSON"),
        ("yaml", "YAML"),
        ("markdown", "Markdown"),
        ("bash", "Bash"),
        ("dockerfile", "Dockerfile"),
    ]
}

// =============================================================================
// Auth Helpers
// =============================================================================

async fn get_current_user(pool: &SqlitePool, jar: &CookieJar) -> Option<User> {
    let session = jar.get(SESSION_COOKIE)?;
    let user_id: i64 = session.value().parse().ok()?;
    
    sqlx::query_as::<_, User>("SELECT id, username, password_hash, created_at FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()?
}

// =============================================================================
// Auth Handlers
// =============================================================================

pub async fn login_page() -> impl IntoResponse {
    Html(LoginTemplate { error: None }.render().unwrap())
}

pub async fn login(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?"
    )
    .bind(&form.username)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let user = match user {
        Some(u) if verify_password(&form.password, &u.password_hash) => u,
        _ => {
            let template = LoginTemplate {
                error: Some("Invalid username or password".to_string()),
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    let cookie = Cookie::build((SESSION_COOKIE, user.id.to_string()))
        .path("/")
        .http_only(true)
        .build();

    (jar.add(cookie), Redirect::to("/dashboard")).into_response()
}

pub async fn register_page() -> impl IntoResponse {
    Html(RegisterTemplate { error: None }.render().unwrap())
}

pub async fn register(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    // Validate input
    if form.username.len() < 3 {
        let template = RegisterTemplate {
            error: Some("Username must be at least 3 characters".to_string()),
        };
        return Html(template.render().unwrap()).into_response();
    }

    if form.password.len() < 6 {
        let template = RegisterTemplate {
            error: Some("Password must be at least 6 characters".to_string()),
        };
        return Html(template.render().unwrap()).into_response();
    }

    if form.password != form.confirm_password {
        let template = RegisterTemplate {
            error: Some("Passwords do not match".to_string()),
        };
        return Html(template.render().unwrap()).into_response();
    }

    // Check if username exists
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
        .bind(&form.username)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    if exists.is_some() {
        let template = RegisterTemplate {
            error: Some("Username already taken".to_string()),
        };
        return Html(template.render().unwrap()).into_response();
    }

    // Create user
    let password_hash = match hash_password(&form.password) {
        Ok(h) => h,
        Err(_) => {
            let template = RegisterTemplate {
                error: Some("Failed to create account".to_string()),
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&form.username)
        .bind(&password_hash)
        .execute(&pool)
        .await;

    match result {
        Ok(r) => {
            let user_id = r.last_insert_rowid();
            let cookie = Cookie::build((SESSION_COOKIE, user_id.to_string()))
                .path("/")
                .http_only(true)
                .build();
            (jar.add(cookie), Redirect::to("/dashboard")).into_response()
        }
        Err(_) => {
            let template = RegisterTemplate {
                error: Some("Failed to create account".to_string()),
            };
            Html(template.render().unwrap()).into_response()
        }
    }
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .build();
    (jar.remove(cookie), Redirect::to("/"))
}

pub async fn dashboard(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
) -> impl IntoResponse {
    let user = match get_current_user(&pool, &jar).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let pastes: Vec<Paste> = sqlx::query_as(
        "SELECT id, content, language, password_hash, expires_at, created_at, view_count, user_id 
         FROM pastes WHERE user_id = ? ORDER BY created_at DESC LIMIT 50"
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let template = DashboardTemplate { user, pastes };
    Html(template.render().unwrap()).into_response()
}

pub async fn public_pastes(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
) -> impl IntoResponse {
    let user = get_current_user(&pool, &jar).await;

    let pastes: Vec<Paste> = sqlx::query_as(
        "SELECT id, content, language, password_hash, expires_at, created_at, view_count, user_id 
         FROM pastes 
         WHERE password_hash IS NULL 
         AND (expires_at IS NULL OR expires_at > datetime('now'))
         ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let template = PublicTemplate { user, pastes };
    Html(template.render().unwrap()).into_response()
}

// =============================================================================
// Paste Handlers
// =============================================================================

pub async fn index(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
) -> impl IntoResponse {
    let user = get_current_user(&pool, &jar).await;
    let template = IndexTemplate {
        languages: get_supported_languages(),
        user,
    };
    Html(template.render().unwrap())
}

pub async fn create_paste(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Form(form): Form<CreatePasteForm>,
) -> impl IntoResponse {
    let user = get_current_user(&pool, &jar).await;
    let user_id = user.map(|u| u.id);
    
    let id = generate_id();
    let language = form.language.unwrap_or_else(|| "plaintext".to_string());
    
    let password_hash = match &form.password {
        Some(pw) if !pw.is_empty() => hash_password(pw).ok(),
        _ => None,
    };
    
    let expires_at = form.expiration
        .as_deref()
        .map(Expiration::from_str)
        .and_then(|exp| exp.to_datetime());

    let result = sqlx::query(
        "INSERT INTO pastes (id, content, language, password_hash, expires_at, user_id) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&form.content)
    .bind(&language)
    .bind(&password_hash)
    .bind(&expires_at)
    .bind(&user_id)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Redirect::to(&format!("/{}", id)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create paste").into_response(),
    }
}

pub async fn view_paste(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user = get_current_user(&pool, &jar).await;
    
    let paste: Option<Paste> = sqlx::query_as(
        "SELECT id, content, language, password_hash, expires_at, created_at, view_count, user_id 
         FROM pastes WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let paste = match paste {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, Html(NotFoundTemplate.render().unwrap())).into_response(),
    };

    // Check expiration
    if let Some(expires_at) = paste.expires_at {
        if expires_at < chrono::Utc::now().naive_utc() {
            let _ = sqlx::query("DELETE FROM pastes WHERE id = ?")
                .bind(&id)
                .execute(&pool)
                .await;
            return (StatusCode::NOT_FOUND, Html(NotFoundTemplate.render().unwrap())).into_response();
        }
    }

    // Password protected - check if owner
    let is_owner = user.as_ref().map(|u| Some(u.id) == paste.user_id).unwrap_or(false);
    
    if paste.password_hash.is_some() && !is_owner {
        let template = PasswordTemplate { id, error: None };
        return Html(template.render().unwrap()).into_response();
    }

    // Increment view count
    let _ = sqlx::query("UPDATE pastes SET view_count = view_count + 1 WHERE id = ?")
        .bind(&paste.id)
        .execute(&pool)
        .await;

    let formatted_date = paste.created_at.format("%Y-%m-%d %H:%M").to_string();
    let expires_in = calculate_expires_in(paste.expires_at);

    let template = ViewTemplate {
        paste,
        formatted_date,
        expires_in,
        user,
        is_owner,
    };
    Html(template.render().unwrap()).into_response()
}

pub async fn verify_paste_password(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<PasswordForm>,
) -> impl IntoResponse {
    let user = get_current_user(&pool, &jar).await;
    
    let paste: Option<Paste> = sqlx::query_as(
        "SELECT id, content, language, password_hash, expires_at, created_at, view_count, user_id 
         FROM pastes WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let paste = match paste {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, Html(NotFoundTemplate.render().unwrap())).into_response(),
    };

    let is_valid = paste.password_hash
        .as_ref()
        .map(|hash| verify_password(&form.password, hash))
        .unwrap_or(false);

    if !is_valid {
        let template = PasswordTemplate {
            id,
            error: Some("Incorrect password".to_string()),
        };
        return Html(template.render().unwrap()).into_response();
    }

    let _ = sqlx::query("UPDATE pastes SET view_count = view_count + 1 WHERE id = ?")
        .bind(&paste.id)
        .execute(&pool)
        .await;

    let formatted_date = paste.created_at.format("%Y-%m-%d %H:%M").to_string();
    let expires_in = calculate_expires_in(paste.expires_at);
    let is_owner = user.as_ref().map(|u| Some(u.id) == paste.user_id).unwrap_or(false);

    let template = ViewTemplate {
        paste,
        formatted_date,
        expires_in,
        user,
        is_owner,
    };
    Html(template.render().unwrap()).into_response()
}

pub async fn view_raw(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let paste: Option<Paste> = sqlx::query_as(
        "SELECT id, content, language, password_hash, expires_at, created_at, view_count, user_id 
         FROM pastes WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    match paste {
        Some(p) if p.password_hash.is_none() => {
            (StatusCode::OK, [("content-type", "text/plain; charset=utf-8")], p.content).into_response()
        }
        Some(_) => (StatusCode::FORBIDDEN, "This paste is password protected").into_response(),
        None => (StatusCode::NOT_FOUND, "Paste not found").into_response(),
    }
}

pub async fn delete_paste(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user = get_current_user(&pool, &jar).await;
    
    // Only allow deletion by owner
    let paste: Option<Paste> = sqlx::query_as(
        "SELECT id, content, language, password_hash, expires_at, created_at, view_count, user_id FROM pastes WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    if let Some(paste) = paste {
        let is_owner = user.as_ref().map(|u| Some(u.id) == paste.user_id).unwrap_or(false);
        let is_anonymous = paste.user_id.is_none();
        
        if is_owner || is_anonymous {
            let _ = sqlx::query("DELETE FROM pastes WHERE id = ?")
                .bind(&id)
                .execute(&pool)
                .await;
        }
    }

    Redirect::to("/").into_response()
}

// =============================================================================
// Helpers
// =============================================================================

fn calculate_expires_in(expires_at: Option<chrono::NaiveDateTime>) -> Option<String> {
    let expires = expires_at?;
    let now = chrono::Utc::now().naive_utc();
    let duration = expires.signed_duration_since(now);
    
    if duration.num_seconds() <= 0 {
        return Some("Expired".to_string());
    }
    
    if duration.num_days() > 0 {
        Some(format!("{} days", duration.num_days()))
    } else if duration.num_hours() > 0 {
        Some(format!("{} hours", duration.num_hours()))
    } else {
        Some(format!("{} minutes", duration.num_minutes()))
    }
}
