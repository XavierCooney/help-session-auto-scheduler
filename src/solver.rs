use std::{
    cmp::{max, min},
    collections::HashSet,
};

use smallvec::SmallVec;

use crate::{
    solution_output::SolvedSession,
    types::{Applicant, Availability, Course, Session, WeekNum},
};

const MAX_TUTORS_PER_SESSION: usize = 5;
type ApplicantId = u16;
type HourCount = u16;
type Cost = u64;

#[derive(Debug, Clone)]
struct SessionAllocation {
    assigned: SmallVec<[ApplicantId; MAX_TUTORS_PER_SESSION]>,
}

struct Week {
    desired_total_hours: HourCount,
    session_indexes: Vec<usize>,
}

struct Solver<'a> {
    sessions: &'a [Session],
    applicants: &'a [Applicant],
    weeks: Vec<Week>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Mutation {
    AddToSession { session: usize, applicant: u16 },
    RemoveFromSession { session: usize, applicant: u16 },
}

impl<'a> Solver<'a> {
    fn eval_allocation(&self, allocations: &[SessionAllocation]) -> Option<Cost> {
        let mut total_cost: Cost = 0;

        let mut applicant_overall_total: Vec<HourCount> = [0].repeat(self.applicants.len());

        for week in &self.weeks {
            let mut effective_hours_this_week = 0;
            let mut applicant_weekly_total: Vec<HourCount> = [0].repeat(self.applicants.len());

            let mut min_size_this_week = MAX_TUTORS_PER_SESSION;
            let mut max_size_this_week = 0;

            for session_index in week.session_indexes.iter().copied() {
                let allocation = &allocations[session_index];
                let session = &self.sessions[session_index];

                let session_length = session.length_hours as HourCount;
                let effective_hours = session_length * (allocation.assigned.len() as HourCount);

                effective_hours_this_week += effective_hours;

                for applicant_index in allocation.assigned.iter().copied() {
                    let availability =
                        self.applicants[applicant_index as usize].availabilities[session_index];
                    total_cost += match availability {
                        Availability::Impossible => return None,
                        Availability::Dislike => 100,
                        Availability::Possible => 5,
                        Availability::Preferred => 0,
                    };

                    applicant_weekly_total[applicant_index as usize] += session_length;
                    applicant_overall_total[applicant_index as usize] += session_length;
                }

                let num_tutors = allocation.assigned.len();
                if num_tutors > 0 {
                    min_size_this_week = min(min_size_this_week, num_tutors);
                    max_size_this_week = max(max_size_this_week, num_tutors);
                }
            }

            for (applicant_total, applicant) in applicant_weekly_total.iter().zip(self.applicants) {
                if *applicant_total > applicant.max_hours_per_week {
                    return None;
                }
            }

            if effective_hours_this_week < week.desired_total_hours {
                total_cost +=
                    20 * ((week.desired_total_hours - effective_hours_this_week) as Cost).pow(2);
            } else {
                let diff = (effective_hours_this_week - week.desired_total_hours) as Cost;
                total_cost += 200 * diff;
            }

            if max_size_this_week > min_size_this_week + 1 {
                total_cost += 50 * ((max_size_this_week - min_size_this_week) as Cost);
            }
        }

        // TOOD: disincentive not giving many hours to tutors who requested many

        Some(total_cost)
    }

    fn mutate_allocation(&self, allocations: &mut [SessionAllocation]) -> Option<Mutation> {
        let session_index = fastrand::usize(..allocations.len());

        let action = fastrand::u8(0..=1);

        let assigned = &mut allocations[session_index].assigned;

        match action {
            0 => {
                if assigned.len() == MAX_TUTORS_PER_SESSION {
                    // full!
                    return None;
                }

                // add a random applicant
                let all_possible_applicants = self
                    .applicants
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, applicant)| {
                        (!matches!(
                            applicant.availabilities[session_index],
                            Availability::Impossible
                        ) && !assigned.contains(&(idx as _)))
                        .then_some(idx)
                    })
                    .collect::<Vec<_>>();

                if all_possible_applicants.is_empty() {
                    // no possible applicants
                    return None;
                }

                let applicant =
                    all_possible_applicants[fastrand::usize(0..all_possible_applicants.len())] as _;

                assigned.push(applicant);

                Some(Mutation::AddToSession {
                    session: session_index,
                    applicant,
                })
            }
            1 => {
                // remove a random applicant
                if assigned.is_empty() {
                    return None;
                }

                let applicant_index = fastrand::usize(0..assigned.len());
                let applicant = assigned[applicant_index];
                assigned.remove(applicant_index);

                Some(Mutation::RemoveFromSession {
                    session: session_index,
                    applicant,
                })
            }
            _ => panic!(),
        }
    }
}

fn solve(
    applicants: &[Applicant],
    sessions: &[Session],
    desired_hours: &[(WeekNum, HourCount)],
) -> (Cost, Vec<SessionAllocation>) {
    let weeks = desired_hours
        .iter()
        .map(|(week_num, desired_total)| Week {
            desired_total_hours: *desired_total,
            session_indexes: sessions
                .iter()
                .enumerate()
                .filter_map(|(idx, session)| (session.week == *week_num).then_some(idx))
                .collect(),
        })
        .collect::<Vec<_>>();

    assert!(
        weeks.len()
            == sessions
                .iter()
                .map(|session| session.week)
                .collect::<HashSet<_>>()
                .len()
    );

    let solver = Solver {
        sessions,
        applicants,
        weeks,
    };

    let mut allocation = (0..sessions.len())
        .map(|_| SessionAllocation {
            assigned: Default::default(),
        })
        .collect::<Vec<_>>();

    let mut old_cost = solver.eval_allocation(&allocation).unwrap();
    // println!("initial cost: {old_cost}");
    let mut old_allocation = allocation.clone();

    let total_steps = 40000;
    let temp_multiplier = 5.0;

    for i in 0..total_steps {
        if let Some(_mutation) = solver.mutate_allocation(&mut allocation) {
            let new_cost = solver.eval_allocation(&allocation);

            let improved = match new_cost {
                Some(new_cost) => {
                    if new_cost <= old_cost {
                        true
                    } else {
                        // possibly allow a bump, depending on temp
                        let cost_increase = (new_cost - old_cost) as f32;
                        let temperature =
                            temp_multiplier * (total_steps as f32) / ((i as f32) + 1.0);

                        let accept_prob = (-cost_increase / temperature).exp();
                        fastrand::f32() < accept_prob
                    }
                }
                None => false,
            };

            if improved {
                let unwrapped_new_cost = new_cost.unwrap();
                old_allocation = allocation.clone();
                // println!(
                //     "{i}: improved by {} with {:?}, current cost: {}",
                //     (old_cost as i64) - (unwrapped_new_cost as i64),
                //     mutation,
                //     unwrapped_new_cost
                // );
                old_cost = unwrapped_new_cost;
            } else {
                allocation = old_allocation.clone();
            }
        }
    }

    (old_cost, old_allocation)
}

pub fn solve_many_times(
    seeds: Vec<u64>,
    course: Course,
    applicants: &[Applicant],
    sessions: &[Session],
    desired_hours: &[(WeekNum, HourCount)],
) -> Vec<SolvedSession> {
    let applicants = &applicants
        .iter()
        .filter(|applicant| applicant.course == course)
        .cloned()
        .collect::<Vec<_>>();

    let (best_cost, best_seed) = seeds
        .into_iter()
        .map(|seed| {
            fastrand::seed(seed);
            let (cost, _) = solve(applicants, sessions, desired_hours);
            println!("seed = {seed}, cost = {cost}");
            (cost, seed)
        })
        .min()
        .unwrap();

    fastrand::seed(best_seed);
    let (_, solution) = solve(applicants, sessions, desired_hours);

    println!("best_cost = {best_cost:?}");
    // println!("solution = {solution:?}");

    solution
        .into_iter()
        .enumerate()
        .map(|(session_index, allocation)| SolvedSession {
            session: sessions[session_index].clone(),
            applicants: allocation
                .assigned
                .into_iter()
                .map(|applicant_index| applicants[applicant_index as usize].clone())
                .collect(),
        })
        .collect::<Vec<_>>()
}
