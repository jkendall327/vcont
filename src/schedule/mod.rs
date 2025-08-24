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
        targets: Vec<ScheduleItem>,
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

// -- Tests --

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduleItem;
    use chrono::{Local, NaiveTime, Timelike};
    use std::time::Duration;

    /// Helper to create a schedule from a simple list of "HH:MM" strings.
    fn schedule_from_str(times: &[&str], ramp_duration_seconds: u64) -> Schedule {
        let items = times
            .iter()
            .map(|s| ScheduleItem {
                time: s.to_string(),
                volume: 50, // Volume doesn't matter for these tests
            })
            .collect();
        Schedule::from_schedule_items(items, ramp_duration_seconds).unwrap()
    }

    // ===================================================================
    // 1. Tests for Schedule Construction
    // ===================================================================

    #[test]
    fn test_from_schedule_items_sorts_targets() {
        // Arrange: targets are intentionally out of order
        let items = vec![
            ScheduleItem {
                time: "14:00".to_string(),
                volume: 50,
            },
            ScheduleItem {
                time: "08:00".to_string(),
                volume: 20,
            },
            ScheduleItem {
                time: "09:30".to_string(),
                volume: 30,
            },
        ];

        // Act
        let schedule = Schedule::from_schedule_items(items, 60).unwrap();

        // Assert
        assert_eq!(
            schedule.targets[0].time,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            schedule.targets[1].time,
            NaiveTime::from_hms_opt(9, 30, 0).unwrap()
        );
        assert_eq!(
            schedule.targets[2].time,
            NaiveTime::from_hms_opt(14, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_from_schedule_items_fails_on_bad_time() {
        // Arrange
        let items = vec![ScheduleItem {
            time: "25:00".to_string(),
            volume: 50,
        }];

        // Act
        let result = Schedule::from_schedule_items(items, 60);

        // Assert
        assert!(matches!(result, Err(ScheduleError::UnparsedTime(_))));
    }

    #[test]
    fn test_from_schedule_items_fails_on_bad_volume() {
        // Arrange
        let items = vec![ScheduleItem {
            time: "08:00".to_string(),
            volume: 101,
        }];

        // Act
        let result = Schedule::from_schedule_items(items, 60);

        // Assert
        assert!(matches!(result, Err(ScheduleError::UnparsedVolume(_))));
    }

    // ===================================================================
    // 2. Tests for Next Invocation Logic (`get_next`)
    // ===================================================================

    #[test]
    fn get_next_finds_upcoming_event_today() {
        // Arrange
        let schedule = schedule_from_str(&["08:00", "17:00"], 60);
        // Simulate "now" being 10:00 AM
        let now = Local::now().with_hour(10).unwrap().with_minute(0).unwrap();

        // Act
        let next_invocation = schedule.get_next_at(now).unwrap();

        // Assert: The next event should be at 17:00 today
        let expected_time = now.with_hour(17).unwrap().with_minute(0).unwrap();
        let expected_delta = (expected_time - now).to_std().unwrap();
        let actual_delta = next_invocation.time.duration_since(Instant::now());

        // We check that the deltas are very close, allowing for a small margin
        // for the time it takes to run the test code.
        assert!((expected_delta.as_secs_f64() - actual_delta.as_secs_f64()).abs() < 0.1);
        assert_eq!(next_invocation.desired_sound.value(), 50);
    }

    #[test]
    fn get_next_wraps_around_to_tomorrow() {
        // Arrange
        let schedule = schedule_from_str(&["08:00", "17:00"], 60);
        // Simulate "now" being 18:00 PM
        let now = Local::now().with_hour(18).unwrap().with_minute(0).unwrap();

        // Act
        let next_invocation = schedule.get_next_at(now).unwrap();

        // Assert: The next event should be at 08:00 tomorrow
        let tomorrow = now.checked_add_days(Days::new(1)).unwrap();
        let expected_time = tomorrow.with_hour(8).unwrap().with_minute(0).unwrap();
        let expected_delta = (expected_time - now).to_std().unwrap(); // roughly 14 hours
        let actual_delta = next_invocation.time.duration_since(Instant::now());

        assert!((expected_delta.as_secs_f64() - actual_delta.as_secs_f64()).abs() < 0.1);
    }

    #[test]
    fn get_next_returns_none_for_empty_schedule() {
        // Arrange
        let schedule = Schedule::new();

        // Act
        let next_invocation = schedule.get_next();

        // Assert
        assert!(next_invocation.is_none());
    }

    // To make the tests above possible, we can add a helper to `Schedule`
    // that accepts a mocked "now" time.
    impl Schedule {
        #[cfg(test)]
        pub fn get_next_at(&self, now: DateTime<Local>) -> Option<Invocation> {
            // This is a direct copy of the original get_next, but using the passed `now`.
            if self.targets.is_empty() {
                return None;
            }

            let (next_target, next_dt) = self
                .targets
                .iter()
                .map(|t| (t, next_occurrence_local(t.time, now)))
                .min_by_key(|(_, dt)| *dt)?;

            let delta = (next_dt - now).to_std().unwrap_or(Duration::from_secs(0));

            Some(Invocation {
                desired_sound: next_target.desired_sound,
                time: Instant::now() + delta,
                ramp_duration: self.ramp_duration,
            })
        }
    }

    // ===================================================================
    // 3. Tests for Time Calculation (`next_occurrence_local`)
    // ===================================================================

    #[test]
    fn next_occurrence_is_later_today() {
        let now = Local::now().with_hour(10).unwrap();
        let target_time = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        let next = next_occurrence_local(target_time, now);

        assert_eq!(next.date_naive(), now.date_naive());
        assert_eq!(next.hour(), 14);
    }

    #[test]
    fn next_occurrence_is_tomorrow() {
        let now = Local::now().with_hour(10).unwrap();
        let target_time = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let next = next_occurrence_local(target_time, now);

        let tomorrow = now.date_naive().checked_add_days(Days::new(1)).unwrap();
        assert_eq!(next.date_naive(), tomorrow);
        assert_eq!(next.hour(), 8);
    }

    #[test]
    fn next_occurrence_handles_exact_time_as_tomorrow() {
        // Your current logic pushes an exact match to the next day. This test documents that behavior.
        let now = Local::now()
            .with_hour(10)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        let target_time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let next = next_occurrence_local(target_time, now);

        let tomorrow = now.date_naive().checked_add_days(Days::new(1)).unwrap();
        assert_eq!(next.date_naive(), tomorrow);
        assert_eq!(next.hour(), 10);
    }
}
