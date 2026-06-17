// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 core/pal_work_constants.py (TICK_*)
// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 services/work_tick_service.py (DEFAULT_MAX_CATCHUP_TICKS)
// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/ws_exit_codes.py (WORKER_LOCK_CONFLICT_EXIT_CODE)
// No Python parity: DEFAULT_BATCH_SIZE, DEFAULT_POLLING_INTERVAL_SECS, DEFAULT_QUEUE_SIZE are new Rust-only

pub const TICK_INTERVAL_MINUTES: i64 = 10;
pub const TICKS_PER_HOUR: i64 = 6;
pub const DEFAULT_MAX_CATCHUP_TICKS: i64 = 144;
pub const DEFAULT_BATCH_SIZE: usize = 100;
pub const DEFAULT_POLLING_INTERVAL_SECS: u64 = 5;
pub const DEFAULT_QUEUE_SIZE: usize = 5000;
pub const WORKER_LOCK_CONFLICT_EXIT_CODE: i32 = 75;
