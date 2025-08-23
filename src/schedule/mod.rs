pub struct Schedule {
    targets: Vec<Target>,
}

#[derive(Debug, Clone, Copy)]
pub struct Target {
    desired_sound: crate::volume::Percentage,
    time: chrono::NaiveTime,
}

#[derive(Debug, Clone, Copy)]
pub struct Invocation {
    pub desired_sound: crate::volume::Percentage,
    pub time: std::time::Instant,
}

impl Schedule {
    pub fn new(mut targets: Vec<Target>) -> Schedule {
        // TODO: don't make consumers have to construct the Target
        targets.sort_by_key(|t| t.time);
        Schedule { targets }
    }

    pub fn get_next(&mut self) -> Invocation {
        let next = self.targets[0];
        let today = chrono::Local::now().date_naive();
        let datetime = today.and_time(next.time);

        Invocation {
            time: todo!(),
            desired_sound: next.desired_sound,
        }
    }
}
