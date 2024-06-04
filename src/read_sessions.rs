use core::panic;

use itertools::Itertools;

use crate::{
    tsv::Tsv,
    types::{Course, Session, Venue, WeekNum},
};

// "1-3,5" ==> [1, 2, 3, 5]
pub fn expand_sequence_specification(spec: &str) -> Vec<i32> {
    spec.split(',')
        .flat_map(|range| match range.split_once('-') {
            Some((start, end)) => {
                let start_num = start.parse().unwrap();
                let end_num = end.parse().unwrap();
                start_num..=end_num
            }
            None => {
                let num = range.parse().unwrap();
                num..=num // single num
            }
        })
        .collect()
}

fn twelve_hour_to_twentfour_hour(s: &str) -> Option<u8> {
    if let Some(am) = s.strip_suffix("am") {
        am.parse().ok()
    } else {
        s.strip_suffix("pm")?.parse().ok().map(|hr: u8| hr + 12)
    }
}

fn sessions_from_specification_line(line: &str, id: &mut usize) -> Vec<Session> {
    let without_comment = line
        .split_once('#')
        .map(|(before, _)| before)
        .unwrap_or(line)
        .trim();

    if without_comment.is_empty() {
        return vec![];
    }

    let (day, time, length, venue, weeks) = without_comment
        .split_whitespace()
        .collect_tuple()
        .unwrap_or_else(|| panic!("bad session line: {line:?}"));

    let day = day.parse().unwrap_or_else(|err| panic!("{err}: {line:?}"));
    let time = twelve_hour_to_twentfour_hour(time)
        .unwrap_or_else(|| panic!("bad time {time:?} on line {line:?}"));
    let length = length
        .strip_suffix("hrs")
        .and_then(|hrs| hrs.parse().ok())
        .unwrap_or_else(|| panic!("bad time length {length:?} (on line {line:?})"));
    let venue = match venue {
        "f2f" => Venue::FaceToFace,
        "online" => Venue::Online,
        _ => panic!("bad venue {venue:?} (on line {line:?})"),
    };

    expand_sequence_specification(weeks)
        .into_iter()
        .map(|week| Session {
            id: {
                *id += 1;
                *id - 1
            },
            day,
            week: WeekNum(week as _),
            venue,
            time_24hr: time,
            length_hours: length,
        })
        .collect::<Vec<_>>()
}

pub fn read_sessions_from_string(input: &str) -> Vec<Session> {
    let mut id = 0;

    input
        .lines()
        .flat_map(|line| sessions_from_specification_line(line, &mut id))
        .collect()
}

pub fn extract_desired_hours(tsv: Tsv, course: Course) -> Vec<(WeekNum, u16)> {
    tsv.into_iter()
        .map(|row| {
            let week = row.get("Week").parse().unwrap();
            let hours = row
                .get(&format!("Desired {} hours", course.to_string()))
                .parse()
                .unwrap();
            (WeekNum(week), hours)
        })
        .collect()
}
