use std::fs;

use clap::Parser;
use read_responses::extract_applicants_from_tsv;
use read_sessions::{
    expand_sequence_specification, extract_desired_hours, read_sessions_from_string,
};

use solution_output::tabule_solution_info;
use solver::solve_many_times;
use tsv::Tsv;
use types::Course;

mod read_responses;
mod read_sessions;
mod solution_output;
mod solver;
mod tsv;
mod types;

#[derive(clap::Parser, Debug)]
struct Args {
    course: Course,
    seed: String,
}

fn main() {
    let args = Args::parse();
    let course = args.course;

    println!("{}", "=".repeat(80));
    println!("{:?}", args);
    println!("{}", "-".repeat(80));

    let sessions = read_sessions_from_string(&fs::read_to_string("sessions.txt").unwrap());
    println!("{} sessions to schedule", sessions.len());

    let responses = Tsv::from_string(&fs::read_to_string("responses.tsv").unwrap());
    println!("{} form responses", responses.num_rows());

    let desired_hours_tsv = Tsv::from_string(&fs::read_to_string("desired_hours.tsv").unwrap());
    let desired_hours = extract_desired_hours(desired_hours_tsv, course);

    let applicants = extract_applicants_from_tsv(responses, &sessions);

    let solution = solve_many_times(
        expand_sequence_specification(&args.seed)
            .into_iter()
            .map(|seed| seed as u64)
            .collect(),
        course,
        &applicants,
        &sessions,
        &desired_hours,
    );

    let solution_info = tabule_solution_info(solution);

    fs::write("solution.tsv", solution_info).unwrap();
}
