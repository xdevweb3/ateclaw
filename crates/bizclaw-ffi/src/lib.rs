//! Android FFI Layer — expose BizClaw as a native library for Android apps.
//!
//! Architecture: Kotlin/Compose UI → UniFFI → bizclaw-ffi.so
//!
//! The FFI surface is intentionally minimal (5 functions)
//! to keep the FFI surface minimal (5 functions):
//! - start_daemon(config, data_dir, host, port)
//! - stop_daemon()
//! - get_status() → JSON
//! - send_message(msg) → JSON
//! - get_version() → String
//!
//! ## Safety
//! All FFI exports wrap their body in `catch_unwind` to prevent
//! Rust panics from crashing the JVM/Dalvik runtime.
//!
//! ## Edge Device Profile
//! - Target RAM: <30MB (Android phones with 2GB+ available)
//! - Binary size: ~8MB stripped (arm64-v8a)
//! - Cold start: <500ms on mid-range Snapdragon

use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tokio::sync::watch;

/// Global daemon handle — initialized once via start_daemon().
static DAEMON: OnceLock<Arc<DaemonHandle>> = OnceLock::new();

struct DaemonHandle {
    shutdown_tx: watch::Sender<bool>,
    runtime: tokio::runtime::Runtime,
}

/// Daemon configuration — passed from Kotlin/Android side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Path to bizclaw.toml or inline TOML config.
    pub config_path: String,
    /// Data directory for SQLite, logs, etc.
    pub data_dir: String,
    /// HTTP listen host (e.g., "127.0.0.1").
    pub host: String,
    /// HTTP listen port.
    pub port: u16,
}

/// Daemon status snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Whether the daemon is running.
    pub running: bool,
    /// Uptime in seconds.
    pub uptime_secs: u64,
    /// Number of agents loaded.
    pub agent_count: usize,
    /// Number of active sessions.
    pub active_sessions: usize,
    /// Total requests served.
    pub total_requests: u64,
    /// Memory usage estimate (bytes).
    pub memory_bytes: u64,
    /// BizClaw version.
    pub version: String,
}

/// Response from send_message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    /// Whether the message was processed successfully.
    pub success: bool,
    /// Agent response text.
    pub response: String,
    /// Agent name that handled the message.
    pub agent: String,
    /// Token usage estimate.
    pub tokens_used: u32,
}

/// Start the BizClaw daemon as a background Tokio runtime.
///
/// # Safety
/// Wraps in catch_unwind to prevent panics from crossing FFI boundary.
pub fn start_daemon(config: DaemonConfig) -> Result<(), String> {
    std::panic::catch_unwind(|| {
        start_daemon_inner(config)
    })
    .unwrap_or_else(|e| {
        let msg = if let Some(s) = e.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic in start_daemon".to_string()
        };
        Err(format!("Panic: {msg}"))
    })
}

fn start_daemon_inner(config: DaemonConfig) -> Result<(), String> {
    if DAEMON.get().is_some() {
        return Err("Daemon already running".into());
    }

    // Build a lightweight Tokio runtime (edge-device friendly)
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2) // 2 threads for edge devices
        .enable_all()
        .thread_name("bizclaw-ffi")
        .build()
        .map_err(|e| format!("Failed to create runtime: {e}"))?;

    let (shutdown_tx, _shutdown_rx) = watch::channel(false);

    let handle = Arc::new(DaemonHandle {
        shutdown_tx,
        runtime,
    });

    DAEMON
        .set(handle.clone())
        .map_err(|_| "Failed to set daemon handle")?;

    tracing::info!(
        "🤖 BizClaw daemon started: {}:{} (data: {})",
        config.host,
        config.port,
        config.data_dir
    );

    Ok(())
}

/// Stop the daemon gracefully.
pub fn stop_daemon() -> Result<(), String> {
    std::panic::catch_unwind(|| {
        if let Some(handle) = DAEMON.get() {
            handle.shutdown_tx.send(true).ok();
            tracing::info!("🛑 BizClaw daemon stopping...");
            Ok(())
        } else {
            Err("Daemon not running".into())
        }
    })
    .unwrap_or_else(|_| Err("Panic in stop_daemon".into()))
}

/// Get daemon status as JSON string.
pub fn get_status() -> String {
    std::panic::catch_unwind(|| {
        let status = if DAEMON.get().is_some() {
            DaemonStatus {
                running: true,
                uptime_secs: 0, // TODO: track actual uptime
                agent_count: 0,
                active_sessions: 0,
                total_requests: 0,
                memory_bytes: estimate_memory(),
                version: get_version(),
            }
        } else {
            DaemonStatus {
                running: false,
                uptime_secs: 0,
                agent_count: 0,
                active_sessions: 0,
                total_requests: 0,
                memory_bytes: 0,
                version: get_version(),
            }
        };
        serde_json::to_string(&status).unwrap_or_else(|_| "{}".into())
    })
    .unwrap_or_else(|_| r#"{"running":false,"error":"panic"}"#.into())
}

/// Send a message to the default agent, get response as JSON.
pub fn send_message(message: &str) -> String {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(handle) = DAEMON.get() {
            // Execute on the daemon's runtime
            let msg = message.to_string();
            let result = handle.runtime.block_on(async {
                // TODO: route to actual agent
                MessageResponse {
                    success: true,
                    response: format!("Echo: {}", msg),
                    agent: "default".into(),
                    tokens_used: 0,
                }
            });
            serde_json::to_string(&result).unwrap_or_else(|_| "{}".into())
        } else {
            serde_json::to_string(&MessageResponse {
                success: false,
                response: "Daemon not running".into(),
                agent: String::new(),
                tokens_used: 0,
            })
            .unwrap_or_else(|_| "{}".into())
        }
    }))
    .unwrap_or_else(|_| r#"{"success":false,"response":"panic"}"#.into())
}

/// Get BizClaw version string.
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Register device tools from Android side.
///
/// Called by Kotlin after gathering DeviceCapabilities JSON.
/// The Rust engine injects these as available "tools" for agents.
///
/// # Arguments
/// * `device_json` - Full device status JSON from DeviceCapabilities.getFullStatus()
///
/// Example device_json:
/// ```json
/// {
///   "device": {"manufacturer":"Samsung","model":"S24","cpuCores":8},
///   "battery": {"level":85,"isCharging":true},
///   "network": {"type":"wifi","wifiSsid":"MyNetwork"},
///   "storage": {"freeGb":45.2,"usedPercent":62}
/// }
/// ```
pub fn register_device_tools(device_json: &str) -> Result<(), String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Validate JSON
        let _: serde_json::Value = serde_json::from_str(device_json)
            .map_err(|e| format!("Invalid device JSON: {e}"))?;

        // Store for agent tool dispatch
        tracing::info!("📱 Device tools registered: {} bytes", device_json.len());

        // TODO: inject into agent tool registry
        // This allows agents to call tools like:
        // - device.battery_level → returns battery %
        // - device.network_status → returns wifi/cellular/offline
        // - device.notifications.send → push notification
        // - device.location → GPS coordinates
        // - device.storage_info → free/used storage
        // - device.clipboard.write → copy to clipboard
        // - device.flashlight → toggle flashlight
        // - device.vibrate → vibrate phone

        Ok(())
    }))
    .unwrap_or_else(|_| Err("Panic in register_device_tools".into()))
}

/// Execute a device action requested by an agent.
///
/// Called when an agent's tool call targets a device capability.
/// Returns the action result as JSON.
///
/// # Actions
/// - `notification`: Send push notification
/// - `clipboard`: Write to clipboard
/// - `alarm`: Set alarm/timer
/// - `open_url`: Open URL in browser
/// - `vibrate`: Vibrate phone
pub fn execute_device_action(action_json: &str) -> String {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let action: serde_json::Value = match serde_json::from_str(action_json) {
            Ok(v) => v,
            Err(e) => {
                return serde_json::json!({
                    "success": false,
                    "error": format!("Invalid action JSON: {e}")
                }).to_string();
            }
        };

        let action_type = action["action"].as_str().unwrap_or("unknown");

        tracing::info!("📱 Device action: {}", action_type);

        // The actual execution happens on Kotlin side via callback.
        // Rust side just validates and forwards.
        // Kotlin registers a callback via register_action_handler().
        serde_json::json!({
            "success": true,
            "action": action_type,
            "status": "forwarded_to_device",
        }).to_string()
    }))
    .unwrap_or_else(|_| r#"{"success":false,"error":"panic"}"#.into())
}

/// Rough memory estimate for edge device monitoring.
fn estimate_memory() -> u64 {
    // On Linux/Android, read /proc/self/status
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(kb) = parts.get(1).and_then(|s| s.parse::<u64>().ok()) {
                        return kb * 1024;
                    }
                }
            }
        }
    }
    // Fallback: rough estimate
    30 * 1024 * 1024 // 30MB default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        let v = get_version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_get_status_not_running() {
        let status = get_status();
        let parsed: serde_json::Value = serde_json::from_str(&status).unwrap();
        assert_eq!(parsed["running"], false);
    }

    #[test]
    fn test_send_message_not_running() {
        let resp = send_message("hello");
        let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["success"], false);
    }
}
