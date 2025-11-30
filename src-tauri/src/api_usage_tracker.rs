use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUsage {
    pub hourly_limit: Option<u32>,
    pub hourly_remaining: Option<u32>,
    pub hourly_reset: Option<DateTime<Utc>>,
    pub daily_limit: Option<u32>,
    pub daily_remaining: Option<u32>,
    pub daily_reset: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl Default for ApiUsage {
    fn default() -> Self {
        Self {
            hourly_limit: None,
            hourly_remaining: None,
            hourly_reset: None,
            daily_limit: None,
            daily_remaining: None,
            daily_reset: None,
            last_updated: None,
        }
    }
}

pub struct ApiUsageTracker {
    usage: Arc<Mutex<ApiUsage>>,
}

impl ApiUsageTracker {
    pub fn new() -> Self {
        Self {
            usage: Arc::new(Mutex::new(ApiUsage::default())),
        }
    }

    /// Update usage from Nexus API response headers
    pub async fn update_from_headers(&self, headers: &reqwest::header::HeaderMap) {
        let mut usage = self.usage.lock().await;

        // Parse hourly limits
        if let Some(hourly_limit) = headers.get("x-rl-hourly-limit") {
            if let Ok(s) = hourly_limit.to_str() {
                usage.hourly_limit = s.parse().ok();
            }
        }

        if let Some(hourly_remaining) = headers.get("x-rl-hourly-remaining") {
            if let Ok(s) = hourly_remaining.to_str() {
                usage.hourly_remaining = s.parse().ok();
            }
        }

        if let Some(hourly_reset) = headers.get("x-rl-hourly-reset") {
            if let Ok(s) = hourly_reset.to_str() {
                if let Ok(timestamp) = s.parse::<i64>() {
                    usage.hourly_reset = DateTime::from_timestamp(timestamp, 0);
                }
            }
        }

        // Parse daily limits
        if let Some(daily_limit) = headers.get("x-rl-daily-limit") {
            if let Ok(s) = daily_limit.to_str() {
                usage.daily_limit = s.parse().ok();
            }
        }

        if let Some(daily_remaining) = headers.get("x-rl-daily-remaining") {
            if let Ok(s) = daily_remaining.to_str() {
                usage.daily_remaining = s.parse().ok();
            }
        }

        if let Some(daily_reset) = headers.get("x-rl-daily-reset") {
            if let Ok(s) = daily_reset.to_str() {
                if let Ok(timestamp) = s.parse::<i64>() {
                    usage.daily_reset = DateTime::from_timestamp(timestamp, 0);
                }
            }
        }

        usage.last_updated = Some(Utc::now());

        println!("📊 API Usage Updated:");
        println!("   Hourly: {}/{}",
            usage.hourly_remaining.unwrap_or(0),
            usage.hourly_limit.unwrap_or(0)
        );
        println!("   Daily: {}/{}",
            usage.daily_remaining.unwrap_or(0),
            usage.daily_limit.unwrap_or(0)
        );
    }

    /// Get current usage stats
    pub async fn get_usage(&self) -> ApiUsage {
        self.usage.lock().await.clone()
    }
}
