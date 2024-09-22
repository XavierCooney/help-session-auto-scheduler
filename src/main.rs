use std::fs;

use clap::Parser;
use read_responses::extract_applicants_from_tsv;
use read_sessions::{
    apply_priorities, expand_sequence_specification, extract_desired_hours,
    read_sessions_from_string,
};

use solution_output::{
    convert_to_json_output, output_to_atci_toml, tabulate_hours_by_tutor, tabulate_solution_info,
};
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
    #[arg(long)]
    no_write: bool,
    #[arg(long)]
    quick: bool,
}

fn main() {
    let args = Args::parse();
    let course = args.course;

    println!("{}", "=".repeat(80));
    println!("{:?}", args);
    println!("{}", "-".repeat(80));

    let sessions = {
        let mut sessions = read_sessions_from_string(&fs::read_to_string("sessions.txt").unwrap());
        let priorities = Tsv::from_string(&fs::read_to_string("priorities.tsv").unwrap());
        apply_priorities(args.course, &priorities, &mut sessions);
        sessions
    };

    println!("{} sessions to schedule", sessions.len());

    let responses = Tsv::from_string(&fs::read_to_string("responses.tsv").unwrap());
    println!("{} form responses", responses.num_rows());

    let desired_hours_tsv = Tsv::from_string(&fs::read_to_string("desired_hours.tsv").unwrap());
    let desired_hours = extract_desired_hours(desired_hours_tsv, course);

    let applicants = extract_applicants_from_tsv(responses, &sessions);

    let (solution, best_seed) = solve_many_times(
        expand_sequence_specification(&args.seed)
            .into_iter()
            .map(|seed| seed as u64)
            .collect(),
        course,
        &applicants,
        &sessions,
        &desired_hours,
        args.quick,
    );

    let solution_info = tabulate_solution_info(solution.clone());

    if !args.no_write {
        fs::write(
            format!("solution.{}.tsv", course.to_string()),
            solution_info,
        )
        .unwrap();

        fs::write(
            format!("hours.{}.tsv", course.to_string()),
            tabulate_hours_by_tutor(solution.clone()),
        )
        .unwrap();

        fs::write(
            format!("help_sessions.{}.toml", course.to_string()),
            output_to_atci_toml(solution.clone(), best_seed),
        )
        .unwrap();

        fs::write(
            format!("help_sessions.{}.json", course.to_string()),
            convert_to_json_output(solution.clone(), best_seed, course),
        )
        .unwrap();
    }
}
