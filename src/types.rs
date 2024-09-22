use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
}

impl FromStr for Day {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Day::*;
        Ok(match s {
            "mon" => Mon,
            "tue" => Tue,
            "wed" => Wed,
            "thu" => Thu,
            "fri" => Fri,
            "sat" => Sat,
            "Monday" => Mon,
            "Tuesday" => Tue,
            "Wednesday" => Wed,
            "Thursday" => Thu,
            "Friday" => Fri,
            "Saturday" => Sat,
            _ => return Err(format!("bad day {s:?}")),
        })
    }
}

impl Day {
    pub fn long_name(self) -> &'static str {
        match self {
            Day::Mon => "Monday",
            Day::Tue => "Tuesday",
            Day::Wed => "Wednesday",
            Day::Thu => "Thursday",
            Day::Fri => "Friday",
            Day::Sat => "Saturday",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Day::Mon => "Mon",
            Day::Tue => "Tue",
            Day::Wed => "Wed",
            Day::Thu => "Thu",
            Day::Fri => "Fri",
            Day::Sat => "Sat",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WeekNum(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Venue {
    FaceToFace,
    Online,
}

impl FromStr for Venue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "f2f" => Venue::FaceToFace,
            "online" => Venue::Online,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: usize,
    pub day: Day,
    pub week: WeekNum,
    pub venue: Venue,
    pub time_24hr: u8,
    pub length_hours: u8,
    pub location: String,
    pub min_allocation: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Course {
    #[clap(name = "COMP1511")]
    Comp1511,
    #[clap(name = "COMP1521")]
    Comp1521,
    #[clap(name = "COMP2521")]
    Comp2521,
}

impl ToString for Course {
    fn to_string(&self) -> String {
        match self {
            Course::Comp1511 => "COMP1511",
            Course::Comp1521 => "COMP1521",
            Course::Comp2521 => "COMP2521",
        }
        .into()
    }
}

impl FromStr for Course {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "COMP1511" => Course::Comp1511,
            "COMP1521" => Course::Comp1521,
            "COMP2521" => Course::Comp2521,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Availability {
    Impossible,
    Dislike,
    Possible,
    Preferred,
}

#[derive(Debug, Clone)]
pub struct Applicant {
    pub id: u32,
    pub email: String,
    pub name: String,
    pub zid: String,
    pub course: Course,
    pub max_hours_per_week: u16,
    pub availabilities: Vec<Availability>,
    pub min_hours_per_week: Option<u16>,
}
