use std::sync::Arc;

use crate::{config::RateLimitConfig, error::CoreError};
use dashmap::DashMap;

#[derive(Clone, Default)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, TokenBucket>>,
}
impl RateLimiter {
    pub fn check(&self, key: &str, config: &RateLimitConfig) -> Result<(), CoreError> {
        let bucket = TokenBucket::default()
            .with_bucket_size(config.capacity)
            .with_refill_rate(config.capacity)
            .with_duration(config.duration_minutes);

        let mut bucket = self.buckets.entry(key.to_string()).or_insert(bucket);
        bucket.run()
    }
}

#[derive(Clone, Debug)]
pub struct TokenBucket {
    capacity: u32,
    bucket_size: u32,
    refill_rate: u32,
    duration: chrono::Duration,
    last_refill: chrono::DateTime<chrono::Utc>,
}

impl Default for TokenBucket {
    fn default() -> Self {
        Self {
            capacity: 20,
            bucket_size: 20,
            refill_rate: 20,
            duration: chrono::Duration::minutes(15),
            last_refill: chrono::Utc::now(),
        }
    }
}

impl TokenBucket {
    pub fn with_bucket_size(mut self, bucket_size: u32) -> Self {
        self.bucket_size = bucket_size;
        self.capacity = bucket_size;
        self
    }
    pub fn with_refill_rate(mut self, refill_rate: u32) -> Self {
        self.refill_rate = refill_rate;
        self
    }
    pub fn with_duration(mut self, minutes: u32) -> Self {
        self.duration = chrono::Duration::minutes(minutes as i64);
        self
    }
}
impl TokenBucket {
    pub fn run(&mut self) -> Result<(), CoreError> {
        let time_since_refill = chrono::Utc::now() - self.last_refill;
        if time_since_refill >= self.duration {
            self.bucket_size = self.refill_rate;
            self.last_refill = chrono::Utc::now();
        }

        if self.bucket_size == 0 {
            return Err(CoreError::RateLimitExceeded {
                limit: self.capacity,
                window: self.duration,
            });
        }

        self.bucket_size -= 1;
        Ok(())
    }
}
