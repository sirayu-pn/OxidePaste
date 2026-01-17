use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

// =============================================================================
// User Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

// =============================================================================
// Paste Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Paste {
    pub id: String,
    pub content: String,
    pub language: Option<String>,
    pub password_hash: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub view_count: i32,
    pub user_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePasteForm {
    pub content: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub expiration: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordForm {
    pub password: String,
}

// =============================================================================
// Expiration
// =============================================================================

pub enum Expiration {
    Never,
    Minutes(i64),
    Hours(i64),
    Days(i64),
}

impl Expiration {
    pub fn from_str(s: &str) -> Self {
        if s.is_empty() || s == "never" {
            return Self::Never;
        }
        
        let len = s.len();
        if len < 2 {
            return Self::Never;
        }
        
        let (num_str, unit) = s.split_at(len - 1);
        let num: i64 = num_str.parse().unwrap_or(0);
        
        match unit {
            "m" => Self::Minutes(num),
            "h" => Self::Hours(num),
            "d" => Self::Days(num),
            _ => Self::Never,
        }
    }
    
    pub fn to_datetime(&self) -> Option<NaiveDateTime> {
        use chrono::{Duration, Utc};
        
        let duration = match self {
            Self::Never => return None,
            Self::Minutes(n) => Duration::minutes(*n),
            Self::Hours(n) => Duration::hours(*n),
            Self::Days(n) => Duration::days(*n),
        };
        
        Some((Utc::now() + duration).naive_utc())
    }
}