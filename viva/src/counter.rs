use std::sync::{OnceLock, atomic::{AtomicU64, Ordering}};

static COUNTER: OnceLock<AtomicU64> = OnceLock::new();

fn counter() -> &'static AtomicU64 {
    COUNTER.get_or_init(|| AtomicU64::new(0))
}

pub fn next_id() -> u64 {
    counter().fetch_add(1, Ordering::SeqCst) + 1
}
