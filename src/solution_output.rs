use std::collections::HashMap;

use itertools::Itertools;

use crate::types::{Applicant, Availability, Session, Venue, WeekNum};

#[derive(Debug)]
pub struct SolvedSession {
    pub session: Session,
    pub applicants: Vec<Applicant>,
}

pub fn tabule_solution_info(mut solution: Vec<SolvedSession>) -> String {
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
