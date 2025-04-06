use crate::error::{Result, ZinitError};
use futures::Future;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Strategy for retrying operations
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    /// Maximum number of retry attempts
    max_retries: usize,
    /// Base delay between retries
    base_delay: Duration,
    /// Maximum delay between retries
    max_delay: Duration,
    /// Whether to add jitter to retry delays
    jitter: bool,
}

impl RetryStrategy {
    /// Create a new retry strategy
    pub fn new(
        max_retries: usize,
        base_delay: Duration,
        max_delay: Duration,
        jitter: bool,
    ) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            jitter,
        }
    }

    /// Execute an operation with retries
    pub async fn retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempt = 0;

        loop {
            attempt += 1;
            debug!("Attempt {}/{}", attempt, self.max_retries + 1);

            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    // Don't retry certain errors
                    match &err {
                        ZinitError::UnknownService(_)
                        | ZinitError::ServiceAlreadyMonitored(_)
                        | ZinitError::ServiceIsUp(_)
                        | ZinitError::ServiceIsDown(_)
                        | ZinitError::InvalidSignal(_)
                        | ZinitError::ShuttingDown => return Err(err),
                        _ => {
                            warn!("Attempt {} failed: {}", attempt, err);
                        }
                    }
                }
            }

            if attempt > self.max_retries {
                return Err(ZinitError::RetryLimitReached(self.max_retries));
            }

            let delay = self.calculate_delay(attempt);
            debug!("Retrying after {:?}", delay);
            sleep(delay).await;
        }
    }

    /// Calculate the delay for a retry attempt
    fn calculate_delay(&self, attempt: usize) -> Duration {
        // Exponential backoff: base_delay * 2^(attempt-1)
        let exp_backoff = self.base_delay.as_millis() * 2u128.pow((attempt - 1) as u32);

        // Cap at max_delay
        let capped_delay = std::cmp::min(exp_backoff, self.max_delay.as_millis());

        // Add jitter if enabled (±20%)
        let delay_ms = if self.jitter {
            let jitter_factor = rand::thread_rng().gen_range(0.8..1.2);
            (capped_delay as f64 * jitter_factor) as u64
        } else {
            capped_delay as u64
        };

        Duration::from_millis(delay_ms)
    }
}

/// Default retry strategy
impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            jitter: true,
        }
    }
}
