pub struct Schedule {
    targets: Vec<Target>,
}

#[derive(Debug, Clone, Copy)]
struct Target {
    desired_sound: crate::volume::Percentage,
    time: chrono::NaiveTime,
}

#[derive(Debug, Clone, Copy)]
pub struct Invocation {
    pub desired_sound: crate::volume::Percentage,
    pub time: std::time::Instant,
}

impl Schedule {
    pub fn new() -> Schedule {
        todo!()
    }

    pub fn get_next(&mut self) -> Invocation {
        self.targets.sort_by(|a, b| a.time.cmp(&b.time));

        let next = self.targets[0];
        let today = chrono::Local::now().date_naive();
        let datetime = today.and_time(next.time);

        Invocation {
            time: todo!(),
            desired_sound: next.desired_sound,
        }
    }
}
