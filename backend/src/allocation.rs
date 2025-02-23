use anyhow::{Context, anyhow};
use good_lp::{solvers::highs::highs, variable, variables, Expression, IntoAffineExpression, Solution, SolverModel, Variable, Constraint};
use uuid::Uuid;
use crate::error::AppError;

use super::http::{
    candidates::Candidate,
    examiners::Examiner,
    stations::Station,
    circuits::Circuit,
};

#[derive(Debug)]
pub struct StationAllocation {
    pub circuit_id: Uuid,
    pub station_id: Uuid,
    pub candidate_1: Uuid,
    pub candidate_2: Uuid,
    pub examiner: Uuid,
}

pub fn allocate_stations(
    circuits: &[Circuit],
    stations: &[Station], // shared stations
    candidates: &[Candidate],
    examiners: &[Examiner],
) -> Result<Vec<StationAllocation>, AppError> {
    let mut vars = variables!();
    let num_circuits = circuits.len();
    let num_stations = stations.len();

    // candidate_vars[c][i][s]: candidate i assigned to station s in circuit c
    let candidate_vars: Vec<Vec<Vec<Variable>>> = (0..num_circuits)
    .map(|_| {
        (0..candidates.len())
            .map(|_| (0..num_stations).map(|_| vars.add(variable().binary())).collect())
            .collect()
    })
    .collect();

    // examiner_vars[c][k][s]: examiner k assigned to station s in circuit c
    let examiner_vars: Vec<Vec<Vec<Variable>>> = (0..num_circuits)
        .map(|_| {
            (0..examiners.len())
                .map(|_| (0..num_stations).map(|_| vars.add(variable().binary())).collect())
                .collect()
        })
        .collect();

    // pair_vars[c][(i,j)][s]: candidates i and j paired at station s in circuit c
    let mut pair_vars = Vec::new();
    for c in 0..num_circuits {
        for i in 0..candidates.len() {
            for j in (i + 1)..candidates.len() {
                for s in 0..num_stations {
                    pair_vars.push(((c, i, j, s), vars.add(variable().binary())));
                }
            }
        }
    }

    // Constraint: Each station must have exactly 2 candidates.
    let st_can_constraints: Vec<Constraint> = (0..num_circuits)
        .flat_map(|c| {
            let val_vars = candidate_vars.clone();
            (0..num_stations).map(move |s| {
                let candidate_sum: Expression = (0..candidates.len())
                    .map(|i| val_vars[c][i][s])
                    .sum();
                candidate_sum.eq(2)
            })
        })
        .collect();

    // 2. Each station in each circuit must have exactly 1 examiner
    let st_ex_constraints: Vec<Constraint> = (0..num_circuits)
        .flat_map(|c| {
            let val_vars = examiner_vars.clone();
            (0..num_stations).map(move |s| {
                let examiner_sum: Expression = (0..examiners.len())
                    .map(|k| val_vars[c][k][s])
                    .sum();
                examiner_sum.eq(1)
            })
        })
        .collect();

    // 3. Each candidate can be assigned to at most one station across all circuits
    let can_st_constraints: Vec<Constraint> = (0..candidates.len())
        .map(|i| {
            let station_sum: Expression = (0..num_circuits)
                .flat_map(|c| {
                    let val_vars = candidate_vars.clone();
                    (0..num_stations).map(move |s| val_vars[c][i][s])
                })
                .sum();
            station_sum.leq(1)
        })
        .collect();

    // 4. Each examiner can be assigned to at most one station across all circuits
    let ex_st_constraints: Vec<Constraint> = (0..examiners.len())
        .map(|k| {
            let station_sum: Expression = (0..num_circuits)
                .flat_map(|c| {
                    let val_vars = examiner_vars.clone();
                    (0..num_stations).map(move |s| val_vars[c][k][s])
                })
                .sum();
            station_sum.leq(1)
        })
        .collect();

    // Constraint: Force non-female candidates/examiners to 0 for female-only stations
    let female_only_constraints: Vec<Constraint> = (0..num_circuits)
        .flat_map(|c| {
            let mut circuit_constraints = Vec::new();
            if circuits[c].female_only {
                for (s, _station) in stations.iter().enumerate() {
                    for (i, candidate) in candidates.iter().enumerate() {
                        if !candidate.female_only {
                            circuit_constraints.push(candidate_vars[c][i][s].into_expression().eq(0));
                        }
                    }
                    for (k, examiner) in examiners.iter().enumerate() {
                        if !examiner.female {
                            circuit_constraints.push(examiner_vars[c][k][s].into_expression().eq(0));
                        }
                    }
                }
            }
            circuit_constraints.into_iter() // Return an iterator of constraints
        })
        .collect();

    // Constraint 6: Link candidate assignment variables to pairing variables.
    // For each pair variable x(i,j,s) we add:
    //   x(i,j,s) <= candidate_vars[i][s]
    //   x(i,j,s) <= candidate_vars[j][s]
    //   x(i,j,s) >= candidate_vars[i][s] + candidate_vars[j][s] - 1
    let pairing_constraints: Vec<Constraint> = pair_vars
    .iter()
    .flat_map(|&((c, i, j, s), x_var)| {
        vec![
            x_var.into_expression().leq(candidate_vars[c][i][s]),
            x_var.into_expression().leq(candidate_vars[c][j][s]),
            x_var.into_expression().geq(candidate_vars[c][i][s] + candidate_vars[c][j][s] - 1),
        ]
    })
    .collect();

    let all_constraints: Vec<Constraint> = vec![
        st_can_constraints,
        st_ex_constraints,
        can_st_constraints,
        ex_st_constraints,
        female_only_constraints,
        pairing_constraints,
    ]
    .into_iter()
    .flatten()
    .collect();


    // Objective: maximize candidate preference satisfaction.
    // For each pair (i, j, s), if candidate i prefers candidate j or vice versa, add a bonus.
    let mut objective = Expression::from(0.0);
    for &((c, i, j, s), x_var) in &pair_vars {
        if let Some(i_pref) = &candidates[i].partner_pref {
            if i_pref == &candidates[j].shortcode {
                objective = objective + 1.0 * x_var;
            }
        }
        if let Some(j_pref) = &candidates[j].partner_pref {
            if j_pref == &candidates[i].shortcode {
                objective = objective + 1.0 * x_var;
            }
        }
    }

    let solution = vars
        .maximise(objective)
        .using(highs)
        .with_all(all_constraints)
        .solve()
        .with_context(|| "Error when solving problem")?;
    
    let mut allocations = Vec::new();
    for c in 0..num_circuits {
        let circuit_id = circuits[c].id;
        for (s, station) in stations.iter().enumerate() {
            let assigned_candidates: Vec<Uuid> = (0..candidates.len())
                .filter(|&i| solution.value(candidate_vars[c][i][s]) > 0.5)
                .map(|i| candidates[i].id)
                .collect();
            if assigned_candidates.len() != 2 {
                return Err(AppError::from(anyhow!("Each station must have exactly 2 candidates")));
            }
            let candidate_1 = assigned_candidates[0];
            let candidate_2 = assigned_candidates[1];

            let assigned_examiner: Uuid = (0..examiners.len())
                .find(|&k| solution.value(examiner_vars[c][k][s]) > 0.5)
                .map(|k| examiners[k].id)
                .ok_or(anyhow!("Each station must have exactly 1 examiner"))?;

            allocations.push(StationAllocation {
                circuit_id,
                station_id: station.id,
                candidate_1,
                candidate_2,
                examiner: assigned_examiner,
            });
        }
    }

    Ok(allocations)
}