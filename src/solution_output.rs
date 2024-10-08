use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use serde::Serialize;
use serde_json::json;

use crate::{
    solver::Seed,
    types::{Applicant, Availability, Course, Session, Venue, WeekNum},
};

#[derive(Debug, Clone)]
pub struct SolvedSession {
    pub session: Session,
    pub applicants: Vec<Applicant>,
}

pub fn tabulate_solution_info(mut solution: Vec<SolvedSession>) -> String {
    println!("Solved for {} sessions", solution.len());

    solution.sort_by_key(|assignment| (assignment.session.week.0));

    let mut hours_by_week: HashMap<WeekNum, u32> = HashMap::new();
    let mut preference_totals: HashMap<Availability, u32> = HashMap::new();

    let mut output = String::new();

    for assignment in solution {
        let session = &assignment.session;

        let count_pref = |pref| {
            assignment
                .applicants
                .iter()
                .filter(move |applicant| applicant.availabilities[session.id] == pref)
                .count()
        };

        let fields = [
            session.week.0.to_string(),
            session.day.long_name().to_string(),
            session.time_24hr.to_string(),
            session.length_hours.to_string(),
            match session.venue {
                Venue::FaceToFace => "f2f",
                Venue::Online => "online",
            }
            .to_string(),
            assignment.applicants.len().to_string(),
            count_pref(Availability::Preferred).to_string(),
            count_pref(Availability::Possible).to_string(),
            count_pref(Availability::Dislike).to_string(),
            (assignment.applicants.len() * (session.length_hours as usize)).to_string(),
            assignment
                .applicants
                .iter()
                .map(|person| &person.name)
                .join(", "),
        ];
        output.push_str(&fields.iter().join("\t"));
        output.push('\n');

        *hours_by_week.entry(session.week).or_default() +=
            (session.length_hours as u32) * (assignment.applicants.len() as u32);

        for applicant in &assignment.applicants {
            *preference_totals
                .entry(applicant.availabilities[session.id])
                .or_default() += 1;
        }
    }

    println!("{}", output);

    for (week, hours) in hours_by_week.iter().sorted() {
        println!("week {}: {hours} hours", week.0)
    }
    println!();

    for (availability, count) in preference_totals.iter().sorted() {
        println!("{availability:?}: {count}")
    }

    output
}

pub fn tabulate_hours_by_tutor(solution: Vec<SolvedSession>) -> String {
    let mut totals: HashMap<String, HashMap<WeekNum, u32>> = HashMap::new();
    let mut zid_to_applicant: HashMap<String, Applicant> = HashMap::new();

    for assignment in solution {
        let session = &assignment.session;
        for applicant in assignment.applicants {
            zid_to_applicant
                .entry(applicant.zid.clone())
                .or_insert_with(|| applicant.clone());

            *totals
                .entry(applicant.zid)
                .or_default()
                .entry(session.week)
                .or_default() += session.length_hours as u32;
        }
    }
    let all_weeks = totals
        .iter()
        .flat_map(|(_, map_by_week)| map_by_week.keys())
        .copied()
        .collect::<HashSet<_>>()
        .iter()
        .copied()
        .sorted()
        .collect::<Vec<_>>();

    let mut result = String::new();

    result.push_str("Name\tzid\tMax hours\tMin hours");
    for week in &all_weeks {
        result.push_str(&format!("\tWeek {}", week.0));
    }
    result.push('\n');

    for (zid, hours_by_week) in totals.into_iter().sorted_by_key(|(s, _)| s.clone()) {
        let applicant = &zid_to_applicant[&zid];

        result.push_str(&format!(
            "{}\t{}\t{}\t{}",
            &applicant.name,
            &applicant.zid,
            &applicant.max_hours_per_week,
            &applicant.min_hours_per_week.unwrap_or_default()
        ));

        for week in &all_weeks {
            result.push_str(&format!(
                "\t{}",
                hours_by_week.get(week).copied().unwrap_or_default()
            ));
        }

        result.push('\n');
    }

    result
}

pub fn output_to_atci_toml(mut solution: Vec<SolvedSession>, seed: Seed) -> String {
    let mut result = String::new();

    let github_url = "https://github.com/XavierCooney/help-session-auto-scheduler/";
    result.push_str(&format!("# Generated by {github_url}\n"));
    result.push_str(&format!("# using the result seed {}\n\n", seed));

    // # [[class.consult]]
    // # instructors = []
    // # weeks = 'N-N,N-N'
    // # day = 'DDD'
    // # start = '12:00'
    // # end = '12:00'
    // # mode = 'online'

    solution.sort_by_key(|assignment| {
        let session = &assignment.session;
        (session.week, session.day, session.time_24hr)
    });

    for assignment in solution {
        if assignment.applicants.is_empty() {
            continue;
        }

        let session = &assignment.session;

        result.push_str("[[class.consult]]\n");

        result.push_str("instructors = [");
        result.push_str(
            &assignment
                .applicants
                .iter()
                .map(|applicant| format!("'{}'", &applicant.zid))
                .join(", "),
        );
        result.push_str("] # ");
        result.push_str(
            &assignment
                .applicants
                .iter()
                .map(|applicant| &applicant.name)
                .join(", "),
        );
        result.push('\n');

        result.push_str(&format!("weeks       = '{}'\n", session.week.0));
        result.push_str(&format!("day         = '{}'\n", session.day.short_name()));

        result.push_str(&format!("start       = '{}:00'\n", session.time_24hr));
        result.push_str(&format!(
            "end         = '{}:00'\n",
            session.time_24hr + session.length_hours
        ));

        result.push_str(&format!(
            "mode        = '{}'\n",
            match session.venue {
                Venue::FaceToFace => "f2f",
                Venue::Online => "online",
            }
        ));

        if let Venue::FaceToFace = session.venue {
            result.push_str(&format!("location    = '{}'\n", session.location));
        }

        result.push('\n');
    }

    result
}

#[derive(Serialize)]
struct SerialisedSession {
    instructor_zids: Vec<String>,
    week: u8,
    day: &'static str,
    start_time_24hrs: u8,
    duration_hours: u8,
    mode: &'static str,
    location: String,
}

impl From<&SolvedSession> for SerialisedSession {
    fn from(assignment: &SolvedSession) -> Self {
        let session = &assignment.session;

        SerialisedSession {
            instructor_zids: assignment
                .applicants
                .iter()
                .map(|applicant| &applicant.zid)
                .cloned()
                .collect(),
            week: session.week.0,
            day: session.day.long_name(),
            start_time_24hrs: session.time_24hr,
            duration_hours: session.length_hours,
            mode: match session.venue {
                Venue::FaceToFace => "f2f",
                Venue::Online => "online",
            },
            location: session.location.clone(),
        }
    }
}

pub fn convert_to_json_output(
    mut solution: Vec<SolvedSession>,
    seed: Seed,
    course: Course,
) -> String {
    solution.sort_by_key(|assignment| {
        let session = &assignment.session;
        (session.week, session.day, session.time_24hr)
    });

    let session_array = serde_json::to_value(
        solution
            .iter()
            .filter(|session| !session.applicants.is_empty())
            .map(SerialisedSession::from)
            .collect::<Vec<_>>(),
    )
    .unwrap();

    json!({
        course.to_string(): {
            "seed": seed,
            "sessions": session_array
        }
    })
    .to_string()
}
