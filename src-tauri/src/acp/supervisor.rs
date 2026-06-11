//! Connection health monitoring and crash recovery policy (F-001.6,
//! F-001.10). Tracks the restart-attempt budget and exponential backoff used
//! by the ACP client when the `kimi` subprocess dies.

pub struct Supervisor {
    max_restarts: u32,
    attempts: u32,
    base_backoff_ms: u64,
}

impl Supervisor {
    pub fn new(max_restarts: u32, base_backoff_ms: u64) -> Self {
        Self {
            max_restarts,
            attempts: 0,
            base_backoff_ms,
        }
    }

    /// Exponential backoff for the *next* restart attempt, capped at 30s.
    pub fn backoff_ms(&self) -> u64 {
        self.base_backoff_ms
            .saturating_mul(1u64 << self.attempts.min(16))
            .min(30_000)
    }

    /// Consume one restart attempt; `None` when the budget is exhausted.
    pub fn next_backoff(&mut self) -> Option<u64> {
        if self.attempts >= self.max_restarts {
            return None;
        }
        let backoff = self.backoff_ms();
        self.attempts += 1;
        Some(backoff)
    }

    /// Reset the attempt counter after a confirmed-healthy connection.
    pub fn reset(&mut self) {
        self.attempts = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_exponentially_and_is_capped() {
        let mut sup = Supervisor::new(10, 100);
        assert_eq!(sup.next_backoff(), Some(100));
        assert_eq!(sup.next_backoff(), Some(200));
        assert_eq!(sup.next_backoff(), Some(400));

        let mut sup = Supervisor::new(100, 30_000);
        assert!(sup.next_backoff().unwrap() <= 30_000);
        assert!(sup.next_backoff().unwrap() <= 30_000);
    }

    #[test]
    fn gives_up_after_max_restarts() {
        let mut sup = Supervisor::new(2, 10);
        assert!(sup.next_backoff().is_some());
        assert!(sup.next_backoff().is_some());
        assert_eq!(sup.next_backoff(), None);
    }

    #[test]
    fn reset_restores_the_attempt_budget() {
        let mut sup = Supervisor::new(2, 10);
        assert!(sup.next_backoff().is_some());
        assert!(sup.next_backoff().is_some());
        assert_eq!(sup.next_backoff(), None);

        sup.reset();
        // Budget restored: backoff starts again from the base.
        assert_eq!(sup.next_backoff(), Some(10));
    }
}
