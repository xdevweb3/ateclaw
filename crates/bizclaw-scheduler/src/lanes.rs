//! Lane-based Task Scheduler — 4 priority lanes for fair scheduling.
//!
//! Prevents agent floods: high-priority lanes (main, cron) always process first,
//! lower-priority lanes (subagent, delegate) throttle under load.
//! RAM: ~200 bytes per lane.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Scheduling lane — determines execution priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Lane {
    /// User-facing direct messages — highest priority.
    Main,
    /// Cron/scheduled tasks — high priority.
    Cron,
    /// Sub-agent spawns — medium priority.
    Subagent,
    /// Inter-agent delegation — lower priority.
    Delegate,
}

impl Lane {
    /// Priority order (lower = higher priority).
    pub fn priority(&self) -> u8 {
        match self {
            Lane::Main => 0,
            Lane::Cron => 1,
            Lane::Subagent => 2,
            Lane::Delegate => 3,
        }
    }

    /// Max concurrent tasks per lane (edge-device friendly limits).
    pub fn max_concurrent(&self) -> usize {
        match self {
            Lane::Main => 4,      // Direct user requests
            Lane::Cron => 2,      // Scheduled tasks
            Lane::Subagent => 3,  // Spawned sub-agents
            Lane::Delegate => 2,  // Delegated tasks
        }
    }
}

impl std::fmt::Display for Lane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lane::Main => write!(f, "main"),
            Lane::Cron => write!(f, "cron"),
            Lane::Subagent => write!(f, "subagent"),
            Lane::Delegate => write!(f, "delegate"),
        }
    }
}

/// A task queued for execution.
#[derive(Debug, Clone)]
pub struct LaneTask {
    /// Unique task ID.
    pub id: String,
    /// Which lane this task is in.
    pub lane: Lane,
    /// Agent name to execute.
    pub agent_name: String,
    /// Input/prompt for the agent.
    pub input: String,
    /// Session ID for context.
    pub session_id: String,
    /// When this task was queued.
    pub queued_at: chrono::DateTime<chrono::Utc>,
}

/// Per-lane state.
struct LaneState {
    queue: VecDeque<LaneTask>,
    active: usize,
    max_concurrent: usize,
    total_processed: u64,
}

impl LaneState {
    fn new(max_concurrent: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            active: 0,
            max_concurrent,
            total_processed: 0,
        }
    }

    fn can_run(&self) -> bool {
        self.active < self.max_concurrent && !self.queue.is_empty()
    }

    fn enqueue(&mut self, task: LaneTask) {
        self.queue.push_back(task);
    }

    fn dequeue(&mut self) -> Option<LaneTask> {
        if self.active < self.max_concurrent {
            let task = self.queue.pop_front()?;
            self.active += 1;
            Some(task)
        } else {
            None
        }
    }

    fn complete(&mut self) {
        self.active = self.active.saturating_sub(1);
        self.total_processed += 1;
    }
}

/// Lane Scheduler — fair priority-based task scheduling.
pub struct LaneScheduler {
    lanes: [Arc<Mutex<LaneState>>; 4],
}

impl LaneScheduler {
    /// Create a new lane scheduler with default concurrency limits.
    pub fn new() -> Self {
        Self {
            lanes: [
                Arc::new(Mutex::new(LaneState::new(Lane::Main.max_concurrent()))),
                Arc::new(Mutex::new(LaneState::new(Lane::Cron.max_concurrent()))),
                Arc::new(Mutex::new(LaneState::new(Lane::Subagent.max_concurrent()))),
                Arc::new(Mutex::new(LaneState::new(Lane::Delegate.max_concurrent()))),
            ],
        }
    }

    /// Submit a task to the appropriate lane.
    pub async fn submit(&self, task: LaneTask) {
        let idx = task.lane.priority() as usize;
        let mut lane = self.lanes[idx].lock().await;
        tracing::debug!(
            "📥 Lane[{}] enqueue: {} (queue: {}, active: {})",
            task.lane,
            task.id,
            lane.queue.len(),
            lane.active
        );
        lane.enqueue(task);
    }

    /// Pop the next task to execute, respecting lane priorities.
    /// Returns None if no tasks are available or all lanes are at capacity.
    pub async fn next(&self) -> Option<LaneTask> {
        // Check lanes in priority order
        for lane in &self.lanes {
            let mut state = lane.lock().await;
            if state.can_run() {
                return state.dequeue();
            }
        }
        None
    }

    /// Mark a lane task as complete (frees a concurrency slot).
    pub async fn complete(&self, lane: Lane) {
        let idx = lane.priority() as usize;
        let mut state = self.lanes[idx].lock().await;
        state.complete();
    }

    /// Get statistics for all lanes.
    pub async fn stats(&self) -> Vec<LaneStats> {
        let mut result = Vec::with_capacity(4);
        let lane_names = [Lane::Main, Lane::Cron, Lane::Subagent, Lane::Delegate];
        for (i, lane_name) in lane_names.iter().enumerate() {
            let state = self.lanes[i].lock().await;
            result.push(LaneStats {
                lane: *lane_name,
                queued: state.queue.len(),
                active: state.active,
                max_concurrent: state.max_concurrent,
                total_processed: state.total_processed,
            });
        }
        result
    }

    /// Total pending tasks across all lanes.
    pub async fn total_pending(&self) -> usize {
        let mut total = 0;
        for lane in &self.lanes {
            let state = lane.lock().await;
            total += state.queue.len() + state.active;
        }
        total
    }
}

impl Default for LaneScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a single lane.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LaneStats {
    pub lane: Lane,
    pub queued: usize,
    pub active: usize,
    pub max_concurrent: usize,
    pub total_processed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(lane: Lane, id: &str) -> LaneTask {
        LaneTask {
            id: id.to_string(),
            lane,
            agent_name: "test".into(),
            input: "hello".into(),
            session_id: "s1".into(),
            queued_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let sched = LaneScheduler::new();

        // Submit delegate first, then main
        sched.submit(make_task(Lane::Delegate, "d1")).await;
        sched.submit(make_task(Lane::Main, "m1")).await;

        // Should dequeue main first (higher priority)
        let next = sched.next().await.unwrap();
        assert_eq!(next.id, "m1");
        sched.complete(Lane::Main).await;

        let next = sched.next().await.unwrap();
        assert_eq!(next.id, "d1");
        sched.complete(Lane::Delegate).await;
    }

    #[tokio::test]
    async fn test_concurrency_limits() {
        let sched = LaneScheduler::new();

        // Fill delegate lane to max (2)
        sched.submit(make_task(Lane::Delegate, "d1")).await;
        sched.submit(make_task(Lane::Delegate, "d2")).await;
        sched.submit(make_task(Lane::Delegate, "d3")).await;

        // First two should dequeue
        assert!(sched.next().await.is_some()); // d1
        assert!(sched.next().await.is_some()); // d2
        // Third should NOT dequeue (at capacity)
        assert!(sched.next().await.is_none());

        // Complete one, now d3 can dequeue
        sched.complete(Lane::Delegate).await;
        assert!(sched.next().await.is_some()); // d3
    }

    #[tokio::test]
    async fn test_stats() {
        let sched = LaneScheduler::new();
        sched.submit(make_task(Lane::Main, "m1")).await;
        sched.submit(make_task(Lane::Cron, "c1")).await;

        let stats = sched.stats().await;
        assert_eq!(stats.len(), 4);
        assert_eq!(stats[0].queued, 1); // Main
        assert_eq!(stats[1].queued, 1); // Cron
        assert_eq!(stats[2].queued, 0); // Subagent
        assert_eq!(stats[3].queued, 0); // Delegate
    }
}
