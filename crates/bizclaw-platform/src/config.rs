//! Platform configuration.

use serde::{Deserialize, Serialize};

/// Multi-tenant platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Admin panel port.
    pub admin_port: u16,
    /// Base port for tenants (auto-increment).
    pub base_port: u16,
    /// Domain for subdomain routing.
    pub domain: String,
    /// JWT secret for admin auth.
    pub jwt_secret: String,
    /// Path to bizclaw binary.
    pub bizclaw_bin: String,
    /// Data directory for tenant files.
    pub data_dir: String,
    /// Database path.
    pub db_path: String,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            admin_port: 3000,
            base_port: 10001,
            domain: "bizclaw.vn".into(),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "bizclaw-platform-secret-change-me".into()),
            bizclaw_bin: "bizclaw".into(),
            data_dir: "~/.bizclaw/tenants".into(),
            db_path: "~/.bizclaw/platform.db".into(),
        }
    }
}
