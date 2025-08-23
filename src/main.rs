use crate::volume::{VolumeChange, VolumeSetter};

mod schedule;
mod volume;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let times = vec!["08:00", "11:32"];

    let targets: Vec<_> = times
        .into_iter()
        .map(|hhmm| {
            let t = chrono::NaiveTime::parse_from_str(hhmm, "%H:%M").unwrap();
            schedule::Target {
                desired_sound: crate::volume::Percentage::new(50).unwrap(),
                time: t,
            }
        })
        .collect();

    let mut schedule = schedule::Schedule::new(targets);

    let next = schedule.get_next();

    tokio::time::sleep_until(next.time.into()).await;

    loop {
        println!("Invoking...!");

        let changer = volume::system_volume();

        changer.change_volume(VolumeChange::Up(next.desired_sound))?;

        let next = schedule.get_next();

        tokio::time::sleep_until(next.time.into()).await;
    }
}
