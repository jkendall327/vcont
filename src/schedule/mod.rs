use nonempty::{NonEmpty, nonempty};
use tokio::time::Instant;

pub fn build_schedule() -> NonEmpty<Invocation> {
    let default = Invocation {
        awakening: Instant::from_std(std::time::Instant::now()),
        end: Instant::from_std(std::time::Instant::now()),
    };

    nonempty![default]
}

#[derive(Debug, Clone, Copy)]
pub struct Invocation {
    pub awakening: Instant,
    pub end: Instant,
}

struct Schedule {
    targets: NonEmpty<Target>,
}

struct Target {
    desired_sound: crate::volume::Percentage,
    time: chrono::NaiveTime,
}
