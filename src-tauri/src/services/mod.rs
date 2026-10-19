mod game_watcher;
mod timer_service;

pub use game_watcher::{start_game_watcher, stop_game_watcher, watcher_status, WatcherStatus};
pub use timer_service::{
    measure_timer_resolution_us, set_timer_policy, start_timer_service, stop_timer_service,
    TimerPolicy,
};
