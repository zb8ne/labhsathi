//! GitHub issue #1: concurrent jobs finish out of order (a fast job can
//! complete before a slower one received earlier on the same partition),
//! but Kafka offsets must be committed contiguously -- storing a later
//! offset before an earlier one has completed would let librdkafka's
//! auto-commit advance past the still-in-flight earlier message. If the
//! worker then crashed, that earlier message is gone: the committed offset
//! says it was handled, and it never redelivers.
//!
//! This tracks, per partition, the highest *contiguous* run of completed
//! offsets starting from the next one due, and only ever returns offsets
//! from that contiguous prefix as safe to store. A message that never
//! completes (e.g. its `document.jobs.completed` publish keeps failing)
//! blocks every later offset on that partition from committing too --
//! that's intentional: on restart, everything from that point redelivers.
//! Some already-successful jobs may get reprocessed, which is a normal
//! at-least-once tradeoff and strictly safer than the alternative (silent
//! loss of the stuck one).

use std::collections::{BTreeSet, HashMap};
use std::sync::Mutex;

struct PartitionState {
    next_to_commit: i64,
    completed_out_of_order: BTreeSet<i64>,
}

pub struct OffsetTracker {
    partitions: Mutex<HashMap<i32, PartitionState>>,
}

impl OffsetTracker {
    pub fn new() -> Self {
        Self {
            partitions: Mutex::new(HashMap::new()),
        }
    }

    /// Must be called synchronously, in receipt order, for every message
    /// *before* it's handed off to a spawned task -- Kafka guarantees
    /// in-order delivery to a single consumer within one partition, so
    /// calling this directly in the recv loop (not from inside a spawned
    /// task, where ordering isn't guaranteed) is what lets this correctly
    /// seed `next_to_commit` even if the first *completion* the tracker
    /// ever observes turns out to be for a later offset than the first
    /// *receipt* was.
    pub fn register_received(&self, partition: i32, offset: i64) {
        let mut partitions = self.partitions.lock().expect("offset tracker mutex poisoned");
        partitions.entry(partition).or_insert_with(|| PartitionState {
            next_to_commit: offset,
            completed_out_of_order: BTreeSet::new(),
        });
        // If this partition was already known, next_to_commit was already
        // correctly seeded from an earlier (lower) offset -- nothing to do.
    }

    /// Records that `offset` on `partition` has reached a terminal,
    /// safe-to-commit state. Returns the offsets (in ascending order) that
    /// are now part of a contiguous completed run and should be passed to
    /// `store_offset` -- empty if `offset` extended the out-of-order set
    /// but didn't close a gap.
    ///
    /// Panics if called before `register_received` for the same
    /// (partition, offset) -- that would be a bug in the caller, not a
    /// runtime condition to handle gracefully.
    pub fn complete(&self, partition: i32, offset: i64) -> Vec<i64> {
        let mut partitions = self.partitions.lock().expect("offset tracker mutex poisoned");
        let state = partitions
            .get_mut(&partition)
            .expect("complete() called for a partition with no register_received() call");

        state.completed_out_of_order.insert(offset);

        let mut ready = Vec::new();
        while state.completed_out_of_order.remove(&state.next_to_commit) {
            ready.push(state.next_to_commit);
            state.next_to_commit += 1;
        }
        ready
    }
}

impl Default for OffsetTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_order_completion_commits_immediately() {
        let t = OffsetTracker::new();
        t.register_received(0, 5);
        assert_eq!(t.complete(0, 5), vec![5]);
        t.register_received(0, 6);
        assert_eq!(t.complete(0, 6), vec![6]);
        t.register_received(0, 7);
        assert_eq!(t.complete(0, 7), vec![7]);
    }

    #[test]
    fn out_of_order_completion_holds_until_gap_closes() {
        let t = OffsetTracker::new();
        // Received in order (as Kafka guarantees), completed out of order.
        t.register_received(0, 5);
        t.register_received(0, 6);
        t.register_received(0, 7);

        assert_eq!(t.complete(0, 7), Vec::<i64>::new(), "7 finishes first but 5/6 aren't done, nothing ready yet");
        assert_eq!(t.complete(0, 6), Vec::<i64>::new(), "6 finishes, still waiting on 5");
        assert_eq!(t.complete(0, 5), vec![5, 6, 7], "5 finishes last, closes the gap -- 5,6,7 all ready in order");
    }

    #[test]
    fn a_stuck_offset_blocks_everything_after_it_on_the_same_partition() {
        let t = OffsetTracker::new();
        t.register_received(0, 10);
        assert_eq!(t.complete(0, 10), vec![10]);
        t.register_received(0, 11);
        t.register_received(0, 12);
        t.register_received(0, 13);
        assert_eq!(t.complete(0, 12), Vec::<i64>::new());
        assert_eq!(t.complete(0, 13), Vec::<i64>::new());
        // offset 11 never completes -- 12 and 13 stay pending forever in this run
    }

    #[test]
    fn partitions_are_tracked_independently() {
        let t = OffsetTracker::new();
        t.register_received(0, 100);
        assert_eq!(t.complete(0, 100), vec![100]);
        t.register_received(1, 50);
        assert_eq!(t.complete(1, 50), vec![50], "partition 1 starting at 50 is unaffected by partition 0's state");
    }

    #[test]
    fn duplicate_completion_of_the_same_offset_is_idempotent() {
        let t = OffsetTracker::new();
        t.register_received(0, 5);
        assert_eq!(t.complete(0, 5), vec![5]);
        // 5 already committed and next_to_commit moved past it; a
        // redelivery-driven duplicate completion of the same offset must
        // not panic or corrupt state -- it goes into the out-of-order set
        // and simply never closes a gap since next_to_commit is already 6.
        assert_eq!(t.complete(0, 5), Vec::<i64>::new());
        t.register_received(0, 6);
        assert_eq!(t.complete(0, 6), vec![6]);
    }

    #[test]
    #[should_panic(expected = "register_received")]
    fn completing_an_unregistered_offset_panics() {
        let t = OffsetTracker::new();
        t.complete(0, 5);
    }
}
