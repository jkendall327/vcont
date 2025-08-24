use std::time::{Duration, Instant};

use chrono::{DateTime, Days, Local, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};

use crate::{
    config::ScheduleItem,
    volume::{self, Percentage},
};

pub struct Schedule {
    targets: Vec<Target>,
    ramp_duration: Duration,
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
    pub ramp_duration: Duration,
}

impl Invocation {
    pub fn get_start(&self) -> std::time::Instant {
        self.time
            .checked_sub(self.ramp_duration)
            .expect("subtracting a minute created invalid Instant")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ScheduleError {
    #[error("could not be parse time portion")]
    UnparsedTime(#[from] chrono::format::ParseError),
    #[error("could not parse percentage portion")]
    UnparsedVolume(#[from] volume::PercentageError),
}

impl Schedule {
    /// Creates an empty schedule.
    pub fn new() -> Schedule {
        Schedule {
            targets: vec![],
            ramp_duration: Duration::from_secs(60),
        }
    }

    /// Creates a schedule from string representations of times and desired volumes.
    pub fn from_schedule_items(
        mut targets: Vec<ScheduleItem>,
        ramp_duration_seconds: u64,
    ) -> Result<Schedule, ScheduleError> {
        let targets: Result<Vec<_>, ScheduleError> = targets
            .into_iter()
            .map(|item| {
                let time = chrono::NaiveTime::parse_from_str(item.time.as_str(), "%H:%M")?;
                let desired_sound: Percentage = item.volume.try_into()?;

                Ok(Target {
                    desired_sound,
                    time,
                })
            })
            .collect();

        let mut targets = targets?;

        targets.sort_by_key(|t| t.time);

        Ok(Schedule {
            targets,
            ramp_duration: Duration::from_secs(ramp_duration_seconds),
        })
    }

    pub fn get_next(&self) -> Option<Invocation> {
        if self.targets.is_empty() {
            return None;
        }

        let now = chrono::Local::now();

        let (next_target, next_dt) = self
            .targets
            .iter()
            .map(|t| {
                let dt = next_occurrence_local(t.time, now);
                (t, dt)
            })
            .min_by_key(|(_, dt)| *dt)?;

        let delta = (next_dt - now).to_std().unwrap_or(Duration::from_secs(0));

        Some(Invocation {
            desired_sound: next_target.desired_sound,
            time: Instant::now() + delta,
            ramp_duration: self.ramp_duration,
        })
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
