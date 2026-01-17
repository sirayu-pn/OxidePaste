/// Generate a random ID for paste URLs
pub fn generate_id() -> String {
    nanoid::nanoid!(8)
}

/// Hash a password using bcrypt
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}