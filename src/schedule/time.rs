use chrono::{DateTime, Days, Local, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};

/// Given a timezone-less time (i.e. hours and minutes),
/// and the current time in the local timezone,
/// find the next occurence of the timezone-less time.
pub fn next_occurrence_local(time: NaiveTime, now: DateTime<Local>) -> DateTime<Local> {
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
