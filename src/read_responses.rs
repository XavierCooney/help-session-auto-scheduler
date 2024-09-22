use std::str::FromStr;

use crate::{
    tsv::Tsv,
    types::{Applicant, Availability, Session, Venue},
};

impl FromStr for Availability {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Impossible" => Availability::Impossible,
            "Dislike" => Availability::Dislike,
            "Possible" => Availability::Possible,
            "Preferred" => Availability::Preferred,
            _ => return Err(()),
        })
    }
}

fn twentfour_hour_to_twelve_hour(time: u8) -> String {
    #[allow(clippy::comparison_chain)]
    if time == 12 {
        String::from("12pm")
    } else if time < 12 {
        format!("{time}am")
    } else {
        format!("{}pm", time - 12)
    }
}

pub fn extract_applicants_from_tsv(tsv: Tsv, sessions: &[Session]) -> Vec<Applicant> {
    (&tsv)
        .into_iter()
        .enumerate()
        .map(|(idx, row)| {
            let email = row.get("Email");
            let zid = email.strip_suffix("@ad.unsw.edu.au").unwrap();
            let name = row.get("Name");
            let course_raw = row.get("Which course are you primarily teaching?");
            let course = course_raw.parse().unwrap();
            let raw_hours_request =
                row.get("Around how many hours would you like to work on help sessions, per week?");
            let max_hours_per_week = match raw_hours_request {
                "1-5" => 5,
                "6-10" => 10,
                ">10" => 14,
                _ => panic!("bad max hours {raw_hours_request:?}"),
            };
            let raw_min_hours = row.get("Min hours");
            let min_hours = if raw_min_hours.is_empty() {
                None
            } else {
                Some(raw_min_hours.parse().unwrap())
            };
            let cant_do_weeks = row
                .get("Are then any weeks you specifically are not available?")
                .split(';')
                .filter(|s| !s.is_empty())
                .map(|week| {
                    week.strip_prefix("Week ")
                        .unwrap_or_else(|| panic!("Bad week {week:?}"))
                        .parse()
                        .unwrap()
                })
                .collect::<Vec<u8>>();

            let availabilities = sessions
                .iter()
                .map(|session| {
                    if cant_do_weeks.contains(&session.week.0) {
                        return Availability::Impossible;
                    }

                    let column_name = format!(
                        "{}{} {}-{}",
                        match session.venue {
                            Venue::FaceToFace => "",
                            Venue::Online => "Online ",
                        },
                        session.day.long_name(),
                        twentfour_hour_to_twelve_hour(session.time_24hr),
                        twentfour_hour_to_twelve_hour(session.time_24hr + session.length_hours),
                    );

                    let raw_availability = row.get(&column_name);
                    raw_availability
                        .parse()
                        .unwrap_or_else(|()| panic!("bad availability {raw_availability:?}"))
                })
                .collect();

            Applicant {
                id: idx as _,
                email: email.into(),
                zid: zid.into(),
                name: name.into(),
                course,
                max_hours_per_week,
                availabilities,
                min_hours_per_week: min_hours,
            }
        })
        .collect()
}
