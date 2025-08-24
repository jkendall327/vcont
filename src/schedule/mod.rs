use std::time::{Duration, Instant};

use crate::{
    config::ScheduleItem,
    volume::{self, Percentage},
};

mod time;

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
    pub fn new() -> Schedule {
        Schedule {
            targets: vec![],
            ramp_duration: Duration::from_secs(60),
        }
    }

    /// Creates a schedule from string representations of times and desired volumes.
    pub fn from_schedule_items(
        targets: &[ScheduleItem],
        ramp_duration_seconds: u64,
    ) -> Result<Schedule, ScheduleError> {
        let mut mapped = targets
            .iter()
            .map(Self::from_schedule_item)
            .collect::<Result<Vec<_>, _>>()?;

        mapped.sort_unstable_by_key(|t| t.time);

        Ok(Schedule {
            targets: mapped,
            ramp_duration: Duration::from_secs(ramp_duration_seconds),
        })
    }

    fn from_schedule_item(item: &ScheduleItem) -> Result<Target, ScheduleError> {
        let time = chrono::NaiveTime::parse_from_str(item.time.as_str(), "%H:%M")?;
        let desired_sound: Percentage = item.volume.try_into()?;

        Ok(Target {
            desired_sound,
            time,
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
                let dt = time::next_occurrence_local(t.time, now);
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

// -- Tests --

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduleItem;
    use chrono::NaiveTime;

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
        let schedule = Schedule::from_schedule_items(&items, 60).unwrap();

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
        let result = Schedule::from_schedule_items(&items, 60);

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
        let result = Schedule::from_schedule_items(&items, 60);

        // Assert
        assert!(matches!(result, Err(ScheduleError::UnparsedVolume(_))));
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
}
