use std::time::{Duration, Instant};

use chrono::{DateTime, Days, Local, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};

use crate::volume::Percentage;

pub struct Schedule {
    targets: Vec<Target>,
}

#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub desired_sound: crate::volume::Percentage,
    pub time: chrono::NaiveTime,
}

#[derive(Debug, Clone, Copy)]
pub struct Invocation {
    pub desired_sound: crate::volume::Percentage,
    pub time: std::time::Instant,
}

impl Schedule {
    /// Creates an empty schedule.
    pub fn new() -> Schedule {
        Schedule { targets: vec![] }
    }

    /// Creates a schedule from string representations of times and desired volumes.
    pub fn from_raw(
        mut targets: Vec<(String, String)>,
    ) -> Result<Schedule, Box<dyn std::error::Error>> {
        let targets: Vec<_> = targets
            .into_iter()
            .map(|(t, v)| {
                let time = chrono::NaiveTime::parse_from_str(t.as_str(), "%H:%M").unwrap();
                let desired_sound: Percentage = v.as_str().parse().unwrap();

                Target {
                    desired_sound,
                    time,
                }
            })
            .collect();

        Self::from_targets(todo!())
    }

    pub fn from_targets(mut targets: Vec<Target>) -> Result<Schedule, Box<dyn std::error::Error>> {
        // TODO: don't make consumers have to construct the Target
        targets.sort_by_key(|t| t.time);
        Ok(Schedule { targets })
    }

    pub fn get_next(&mut self) -> Invocation {
        let now = chrono::Local::now();

        let (next_target, next_dt) = self
            .targets
            .iter()
            .map(|t| {
                let dt = next_occurrence_local(t.time, now);
                (t, dt)
            })
            .min_by_key(|(_, dt)| *dt)
            .expect("schedule has no targets");

        let delta = (next_dt - now).to_std().unwrap_or(Duration::from_secs(0));

        Invocation {
            desired_sound: next_target.desired_sound,
            time: Instant::now() + delta,
        }
    }
}

fn next_occurrence_local(time: NaiveTime, now: DateTime<Local>) -> DateTime<Local> {
    let today = now.date_naive();

    let today_ndt = NaiveDateTime::new(today, time);
    let today_local = resolve_local(today_ndt);

    if today_local > now {
        return today_local;
    }

    let tomorrow_ndt = today_ndt
        .checked_add_days(Days::new(1))
        .expect("finding tomorrow created an unrepresentable date");

    resolve_local(tomorrow_ndt)
}

fn resolve_local(ndt: NaiveDateTime) -> DateTime<Local> {
    match Local.from_local_datetime(&ndt) {
        chrono::offset::LocalResult::Single(dt) => dt,
        chrono::offset::LocalResult::Ambiguous(a, _) => a,
        chrono::offset::LocalResult::None => {
            let mut probe = ndt;

            loop {
                match Local.from_local_datetime(&probe) {
                    chrono::offset::LocalResult::Single(dt) => break dt,
                    chrono::offset::LocalResult::Ambiguous(a, _) => break a,
                    chrono::offset::LocalResult::None => {
                        probe = probe
                            .checked_add_signed(TimeDelta::minutes(1))
                            .expect("moving probe forward created invalid time");
                    }
                }
            }
        }
    }
}
