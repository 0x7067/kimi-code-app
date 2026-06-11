//! Connection health monitoring and crash recovery policy (F-001.6,
//! F-001.10). Pure state machine over an injectable [`ProcessHandle`] so the
//! restart/backoff logic is testable without the real `kimi` binary.

/// Abstraction over the agent subprocess so tests can use a fake.
pub trait ProcessHandle {
    fn is_alive(&mut self) -> bool;
    fn restart(&mut self) -> Result<(), String>;
}

/// Outcome of one supervision tick.
#[derive(Debug, Clone, PartialEq)]
pub enum SupervisorAction {
    Healthy,
    /// Process was restarted; the listed sessions should be replayed.
    Restarted {
        attempt: u32,
        backoff_ms: u64,
        replay_sessions: Vec<String>,
    },
    /// Restart attempt failed; will retry on a later tick.
    RestartFailed {
        attempt: u32,
        backoff_ms: u64,
    },
    /// Exceeded the restart budget.
    GaveUp,
}

pub struct Supervisor {
    max_restarts: u32,
    attempts: u32,
    base_backoff_ms: u64,
    sessions: Vec<String>,
}

impl Supervisor {
    pub fn new(max_restarts: u32, base_backoff_ms: u64) -> Self {
        Self {
            max_restarts,
            attempts: 0,
            base_backoff_ms,
            sessions: Vec::new(),
        }
    }

    /// Remember a session id for context replay after a crash. Idempotent.
    pub fn record_session(&mut self, id: &str) {
        if !self.sessions.iter().any(|s| s == id) {
            self.sessions.push(id.to_string());
        }
    }

    pub fn replay_sessions(&self) -> &[String] {
        &self.sessions
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

    /// One health-check tick: restart the process if it died.
    pub fn check<P: ProcessHandle>(&mut self, process: &mut P) -> SupervisorAction {
        if process.is_alive() {
            self.reset();
            return SupervisorAction::Healthy;
        }
        let Some(backoff_ms) = self.next_backoff() else {
            return SupervisorAction::GaveUp;
        };
        let attempt = self.attempts;
        match process.restart() {
            Ok(()) => SupervisorAction::Restarted {
                attempt,
                backoff_ms,
                replay_sessions: self.sessions.clone(),
            },
            Err(_) => SupervisorAction::RestartFailed { attempt, backoff_ms },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeProcess {
        alive: bool,
        restart_ok: bool,
        restarts: u32,
    }

    impl FakeProcess {
        fn new(alive: bool, restart_ok: bool) -> Self {
            Self {
                alive,
                restart_ok,
                restarts: 0,
            }
        }
    }

    impl ProcessHandle for FakeProcess {
        fn is_alive(&mut self) -> bool {
            self.alive
        }
        fn restart(&mut self) -> Result<(), String> {
            self.restarts += 1;
            if self.restart_ok {
                self.alive = true;
                Ok(())
            } else {
                Err("spawn failed".into())
            }
        }
    }

    #[test]
    fn healthy_process_needs_no_action() {
        let mut sup = Supervisor::new(3, 100);
        let mut p = FakeProcess::new(true, true);
        assert_eq!(sup.check(&mut p), SupervisorAction::Healthy);
        assert_eq!(p.restarts, 0);
    }

    #[test]
    fn restarts_dead_process_and_replays_sessions() {
        let mut sup = Supervisor::new(3, 100);
        sup.record_session("s1");
        sup.record_session("s2");
        sup.record_session("s1"); // dedup
        let mut p = FakeProcess::new(false, true);
        assert_eq!(
            sup.check(&mut p),
            SupervisorAction::Restarted {
                attempt: 1,
                backoff_ms: 100,
                replay_sessions: vec!["s1".into(), "s2".into()],
            }
        );
        assert_eq!(p.restarts, 1);
    }

    #[test]
    fn backoff_grows_exponentially_and_is_capped() {
        let mut sup = Supervisor::new(10, 100);
        let mut p = FakeProcess::new(false, false);
        let mut backoffs = vec![];
        for _ in 0..3 {
            match sup.check(&mut p) {
                SupervisorAction::RestartFailed { backoff_ms, .. } => backoffs.push(backoff_ms),
                other => panic!("unexpected: {other:?}"),
            }
        }
        assert_eq!(backoffs, vec![100, 200, 400]);
        let mut sup = Supervisor::new(100, 30_000);
        assert!(sup.next_backoff().unwrap() <= 30_000);
        assert!(sup.next_backoff().unwrap() <= 30_000);
    }

    #[test]
    fn gives_up_after_max_restarts() {
        let mut sup = Supervisor::new(2, 10);
        let mut p = FakeProcess::new(false, false);
        assert!(matches!(
            sup.check(&mut p),
            SupervisorAction::RestartFailed { .. }
        ));
        assert!(matches!(
            sup.check(&mut p),
            SupervisorAction::RestartFailed { .. }
        ));
        assert_eq!(sup.check(&mut p), SupervisorAction::GaveUp);
        assert_eq!(p.restarts, 2);
    }

    #[test]
    fn healthy_tick_resets_attempt_budget() {
        let mut sup = Supervisor::new(2, 10);
        let mut dead = FakeProcess::new(false, false);
        assert!(matches!(
            sup.check(&mut dead),
            SupervisorAction::RestartFailed { .. }
        ));
        let mut alive = FakeProcess::new(true, true);
        assert_eq!(sup.check(&mut alive), SupervisorAction::Healthy);
        // budget restored: two more failed attempts allowed before giving up
        assert!(matches!(
            sup.check(&mut dead),
            SupervisorAction::RestartFailed { attempt: 1, .. }
        ));
    }
}
