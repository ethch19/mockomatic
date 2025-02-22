use good_lp::solvers::highs::HighsProblem;
use good_lp::{variables, Expression, Solution, Variable};

#[derive(PartialEq)]
enum Gender {
    Male,
    Female,
}

struct Candidate {
    id: usize,
    gender: Gender,
    preferred_partner: Option<usize>,
}

struct Examiner {
    id: usize,
    gender: Gender,
}

fn allocate_stations(
    candidates: &[Candidate],
    examiners: &[Examiner],
    stations: &[Station],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a variable container.
    let mut vars = variables!();

    // Create binary variables for candidate assignments:
    // candidate_vars[i][s] == 1 if candidate i is assigned to station s.
    let candidate_vars: Vec<Vec<Variable>> = (0..candidates.len())
        .map(|_| {
            (0..stations.len())
                .map(|_| vars.add(variable().binary()))
                .collect()
        })
        .collect();

    // Create binary variables for examiner assignments:
    // examiner_vars[k][s] == 1 if examiner k is assigned to station s.
    let examiner_vars: Vec<Vec<Variable>> = (0..examiners.len())
        .map(|_| {
            (0..stations.len())
                .map(|_| vars.add(variable().binary()))
                .collect()
        })
        .collect();

    // Create binary variables for candidate pairings:
    // For each unordered candidate pair (i,j) (with i < j) and station s,
    // pair_vars[(i,j,s)] == 1 if both candidates i and j are assigned together at station s.
    let mut pair_vars = vec![];
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            for s in 0..stations.len() {
                pair_vars.push(((i, j, s), vars.add(variable().binary())));
            }
        }
    }

    // We now build the ILP model. (Note that many LP libraries only support minimization;
    // so if we need to maximize an objective, we multiply by -1.)
    let mut problem = vars.minimize(0.0);

    // Constraint 1: Each station must have exactly 2 candidates.
    for s in 0..stations.len() {
        let candidate_sum: Expression = (0..candidates.len())
            .map(|i| candidate_vars[i][s])
            .sum();
        problem = problem.with(candidate_sum.eq(2));
    }

    // Constraint 2: Each station must have exactly 1 examiner.
    for s in 0..stations.len() {
        let examiner_sum: Expression = (0..examiners.len())
            .map(|k| examiner_vars[k][s])
            .sum();
        problem = problem.with(examiner_sum.eq(1));
    }

    // Constraint 3: Each candidate can be assigned to at most one station.
    for i in 0..candidates.len() {
        let station_sum: Expression = (0..stations.len())
            .map(|s| candidate_vars[i][s])
            .sum();
        problem = problem.with(station_sum.leq(1));
    }

    // Constraint 4: Each examiner can be assigned to at most one station.
    for k in 0..examiners.len() {
        let station_sum: Expression = (0..stations.len())
            .map(|s| examiner_vars[k][s])
            .sum();
        problem = problem.with(station_sum.leq(1));
    }

    // Constraint 5: For stations designated as "female only," force non-female candidates/examiners to 0.
    for (s, station) in stations.iter().enumerate() {
        if station.female_only {
            for (i, candidate) in candidates.iter().enumerate() {
                if candidate.gender != Gender::Female {
                    problem = problem.with(candidate_vars[i][s].eq(0));
                }
            }
            for (k, examiner) in examiners.iter().enumerate() {
                if examiner.gender != Gender::Female {
                    problem = problem.with(examiner_vars[k][s].eq(0));
                }
            }
        }
    }

    // Constraint 6: Link candidate assignment variables to pairing variables.
    // For each pair variable x(i,j,s) we add:
    //   x(i,j,s) <= candidate_vars[i][s]
    //   x(i,j,s) <= candidate_vars[j][s]
    //   x(i,j,s) >= candidate_vars[i][s] + candidate_vars[j][s] - 1
    for &((i, j, s), x_var) in &pair_vars {
        problem = problem.with(x_var.leq(candidate_vars[i][s]));
        problem = problem.with(x_var.leq(candidate_vars[j][s]));
        problem = problem.with(x_var.geq(candidate_vars[i][s] + candidate_vars[j][s] - 1));
    }

    // Objective: maximize candidate preference satisfaction.
    // For each pair (i, j, s), if candidate i prefers candidate j or vice versa, add a bonus.
    // (Weights can be tuned; here we simply add 1 per satisfied preference.)
    let mut objective = Expression::from(0.0);
    for &((i, j, s), x_var) in &pair_vars {
        if let Some(pref) = candidates[i].preferred_partner {
            if pref == j {
                objective = objective + 1.0 * x_var;
            }
        }
        if let Some(pref) = candidates[j].preferred_partner {
            if pref == i {
                objective = objective + 1.0 * x_var;
            }
        }
    }
    // Since our solver minimizes, we minimize the negative of our objective.
    problem = problem.with(objective.clone().negate());

    // Solve the problem using the CBC solver.
    let solution = problem.solve(CbcSolver::new())?;

    // Process the solution: print out assignments for each station.
    for (s, station) in stations.iter().enumerate() {
        let assigned_candidates: Vec<usize> = (0..candidates.len())
            .filter(|&i| solution.value(candidate_vars[i][s]) > 0.5)
            .map(|i| candidates[i].id)
            .collect();
        let assigned_examiner: Option<usize> = (0..examiners.len())
            .find(|&k| solution.value(examiner_vars[k][s]) > 0.5)
            .map(|k| examiners[k].id);
        println!(
            "Station {} (female_only: {}): candidates {:?}, examiner {:?}",
            station.id, station.female_only, assigned_candidates, assigned_examiner
        );
    }

    Ok(())
}