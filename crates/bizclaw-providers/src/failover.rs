//! Provider Failover — automatic fallback when primary provider fails.
//!
//! Lightweight failover chain: try primary → fallback₁ → fallback₂.
//! No heavyweight circuit breaker, no thread pools.
//! RAM: ~100 bytes per provider entry.

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::provider::{GenerateParams, Provider};
use bizclaw_core::types::{Message, ModelInfo, ProviderResponse, ToolDefinition};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Per-provider health tracking (64 bytes).
struct ProviderSlot {
    provider: Box<dyn Provider>,
    /// Consecutive failure count.
    failures: AtomicU32,
    /// Timestamp of last failure (unix secs, 0 = never failed).
    last_failure: AtomicU64,
    /// Max failures before skip (default: 3).
    max_failures: u32,
    /// Cool-down period in seconds before retrying a failed provider.
    cooldown_secs: u64,
}

impl ProviderSlot {
    fn new(provider: Box<dyn Provider>) -> Self {
        Self {
            provider,
            failures: AtomicU32::new(0),
            last_failure: AtomicU64::new(0),
            max_failures: 3,
            cooldown_secs: 60,
        }
    }

    /// Check if this provider is healthy (below failure threshold or cooldown expired).
    fn is_healthy(&self) -> bool {
        let fails = self.failures.load(Ordering::Relaxed);
        if fails < self.max_failures {
            return true;
        }
        // Check cooldown
        let last = self.last_failure.load(Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(last) > self.cooldown_secs
    }

    fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_failure.store(now, Ordering::Relaxed);
    }
}

/// Failover provider — tries providers in order, skipping unhealthy ones.
pub struct FailoverProvider {
    slots: Vec<ProviderSlot>,
}

impl FailoverProvider {
    /// Create a failover chain from a list of providers.
    /// First provider is primary, rest are fallbacks.
    pub fn new(providers: Vec<Box<dyn Provider>>) -> Self {
        assert!(!providers.is_empty(), "Need at least one provider");
        Self {
            slots: providers.into_iter().map(ProviderSlot::new).collect(),
        }
    }

    /// Create from a primary + single fallback.
    pub fn with_fallback(primary: Box<dyn Provider>, fallback: Box<dyn Provider>) -> Self {
        Self::new(vec![primary, fallback])
    }

    /// Number of providers in the chain.
    pub fn chain_len(&self) -> usize {
        self.slots.len()
    }

    /// Get health status of all providers.
    pub fn health_status(&self) -> Vec<(&str, bool, u32)> {
        self.slots
            .iter()
            .map(|s| {
                (
                    s.provider.name(),
                    s.is_healthy(),
                    s.failures.load(Ordering::Relaxed),
                )
            })
            .collect()
    }
}

#[async_trait]
impl Provider for FailoverProvider {
    fn name(&self) -> &str {
        // Return primary provider name
        self.slots
            .first()
            .map(|s| s.provider.name())
            .unwrap_or("failover")
    }

    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        params: &GenerateParams,
    ) -> Result<ProviderResponse> {
        let mut last_error = None;

        for (idx, slot) in self.slots.iter().enumerate() {
            if !slot.is_healthy() {
                tracing::debug!(
                    "⏭️ Skipping unhealthy provider: {} ({} failures)",
                    slot.provider.name(),
                    slot.failures.load(Ordering::Relaxed)
                );
                continue;
            }

            match slot.provider.chat(messages, tools, params).await {
                Ok(response) => {
                    if idx > 0 {
                        tracing::info!(
                            "🔄 Failover: {} → {} (success)",
                            self.slots[0].provider.name(),
                            slot.provider.name()
                        );
                    }
                    slot.record_success();
                    return Ok(response);
                }
                Err(e) => {
                    slot.record_failure();
                    tracing::warn!(
                        "⚠️ Provider {} failed (attempt {}): {}",
                        slot.provider.name(),
                        slot.failures.load(Ordering::Relaxed),
                        e
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            BizClawError::Provider("All providers unhealthy".into())
        }))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Aggregate models from all healthy providers
        let mut all = Vec::new();
        for slot in &self.slots {
            if slot.is_healthy()
                && let Ok(models) = slot.provider.list_models().await {
                    all.extend(models);
                }
        }
        Ok(all)
    }

    async fn health_check(&self) -> Result<bool> {
        // Healthy if at least one provider is healthy
        for slot in &self.slots {
            if slot.is_healthy()
                && let Ok(true) = slot.provider.health_check().await {
                    return Ok(true);
                }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_tracking() {
        let failures = AtomicU32::new(0);
        let last_failure = AtomicU64::new(0);
        let max_failures = 3u32;
        let cooldown_secs = 60u64;

        // Helper to check health
        let is_healthy = || {
            let fails = failures.load(Ordering::Relaxed);
            if fails < max_failures {
                return true;
            }
            let last = last_failure.load(Ordering::Relaxed);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now.saturating_sub(last) > cooldown_secs
        };

        assert!(is_healthy()); // 0 failures
        failures.fetch_add(1, Ordering::Relaxed);
        assert!(is_healthy()); // 1 < 3
        failures.fetch_add(1, Ordering::Relaxed);
        failures.fetch_add(1, Ordering::Relaxed);
        // Set last_failure to NOW so cooldown hasn't expired
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        last_failure.store(now, Ordering::Relaxed);
        assert!(!is_healthy()); // 3 >= 3, cooldown active
        failures.store(0, Ordering::Relaxed); // success reset
        assert!(is_healthy()); // back to 0
    }
}
