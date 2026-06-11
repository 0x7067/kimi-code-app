//! Bounded outbound message queue used while the agent is disconnected
//! (F-001.8). Queued lines are flushed in order on reconnect. Also hosts
//! [`TurnQueue`], which serializes `session/prompt` turns per session, since
//! kimi rejects a second prompt while a turn is active.

use std::collections::{HashMap, HashSet, VecDeque};

pub struct MessageQueue {
    items: VecDeque<String>,
    max: usize,
}

impl MessageQueue {
    pub fn new(max: usize) -> Self {
        Self {
            items: VecDeque::new(),
            max,
        }
    }

    /// Queue a serialized message. If the queue is full, the oldest entry is
    /// dropped and returned so callers can log the loss.
    pub fn push(&mut self, line: String) -> Option<String> {
        let dropped = if self.items.len() >= self.max {
            self.items.pop_front()
        } else {
            None
        };
        self.items.push_back(line);
        dropped
    }

    /// Remove and return all queued messages in FIFO order.
    pub fn drain(&mut self) -> Vec<String> {
        self.items.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Per-session turn serializer. Only one `session/prompt` may be in flight
/// per session; later prompts park a waiter token of type `T` (in practice a
/// oneshot sender) that is released FIFO when the active turn ends.
#[derive(Default)]
pub struct TurnQueue<T> {
    active: HashSet<String>,
    waiting: HashMap<String, VecDeque<T>>,
}

impl<T> TurnQueue<T> {
    pub fn new() -> Self {
        Self {
            active: HashSet::new(),
            waiting: HashMap::new(),
        }
    }

    /// Try to start a turn. Returns `true` if the session was idle and the
    /// turn is now active; `false` if another turn is already in flight.
    pub fn try_begin(&mut self, session_id: &str) -> bool {
        self.active.insert(session_id.to_string())
    }

    /// Park a waiter to be released when the current turn ends.
    pub fn enqueue_waiter(&mut self, session_id: &str, waiter: T) {
        self.waiting
            .entry(session_id.to_string())
            .or_default()
            .push_back(waiter);
    }

    /// End the active turn. Returns the next waiter (the session stays
    /// active for it), or `None` once the session is idle.
    pub fn end_turn(&mut self, session_id: &str) -> Option<T> {
        if let Some(next) = self.waiting.get_mut(session_id).and_then(VecDeque::pop_front) {
            Some(next)
        } else {
            self.active.remove(session_id);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_turns_within_a_session_fifo() {
        let mut t = TurnQueue::new();
        assert!(t.try_begin("s"));
        assert!(!t.try_begin("s")); // second prompt must wait
        t.enqueue_waiter("s", 1u32);
        t.enqueue_waiter("s", 2u32);
        assert_eq!(t.end_turn("s"), Some(1)); // still active for waiter 1
        assert!(!t.try_begin("s"));
        assert_eq!(t.end_turn("s"), Some(2));
        assert_eq!(t.end_turn("s"), None); // now idle
        assert!(t.try_begin("s"));
    }

    #[test]
    fn sessions_do_not_block_each_other() {
        let mut t: TurnQueue<u32> = TurnQueue::new();
        assert!(t.try_begin("a"));
        assert!(t.try_begin("b"));
        assert_eq!(t.end_turn("a"), None);
        assert!(!t.try_begin("b"));
    }

    #[test]
    fn queues_and_drains_in_fifo_order() {
        let mut q = MessageQueue::new(8);
        assert!(q.push("a".into()).is_none());
        assert!(q.push("b".into()).is_none());
        assert_eq!(q.len(), 2);
        assert_eq!(q.drain(), vec!["a".to_string(), "b".to_string()]);
        assert!(q.is_empty());
    }

    #[test]
    fn drops_oldest_when_full() {
        let mut q = MessageQueue::new(2);
        assert!(q.push("a".into()).is_none());
        assert!(q.push("b".into()).is_none());
        assert_eq!(q.push("c".into()), Some("a".to_string()));
        assert_eq!(q.drain(), vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn drain_on_empty_returns_nothing() {
        let mut q = MessageQueue::new(2);
        assert!(q.drain().is_empty());
    }
}
