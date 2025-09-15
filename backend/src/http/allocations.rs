use std::cmp;

use anyhow::{Context, anyhow};
use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::get, Extension};
use serde::{Deserialize, Serialize};
use sqlx::Transaction;
use uuid::Uuid;
use super::{
    candidates::Candidate, circuits::Circuit, examiners::Examiner, runs::{Run, RunTime}, slots::Slot, stations::Station, users::{AccessClaims, User}, AppState, SomethingID};
use crate::{
    allocation_algo::{allocate_by_slot, allocate_by_time, SlotAllocation, TimeAllocation}, error::AppError
};
use tracing::trace;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/generate", get(gen_new))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Allocation {
    pub id: Uuid,
    pub slot_id: Uuid,
    pub circuit_id: Uuid,
    pub station_id: Uuid,
    pub candidate_1: Uuid,
    pub candidate_2: Uuid,
    pub examiner: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub modified_at: time::OffsetDateTime
}

pub struct AllocationPayload {
    pub slot_id: Uuid,
    pub circuit_id: Uuid,
    pub station_id: Uuid,
    pub candidate_1: Uuid,
    pub candidate_2: Uuid,
    pub examiner: Uuid
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllocationHistory {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub slot_id: Uuid,
    pub circuit_id: Uuid,
    pub station_id: Uuid,
    pub candidate_1: Uuid,
    pub candidate_2: Uuid,
    pub examiner: Uuid,
    pub modified_by: Uuid,
    pub auto_gen: bool,
    #[serde(with = "time::serde::iso8601")]
    pub modified_at: time::OffsetDateTime
}

#[derive(Debug, Serialize)]
pub struct Availability {
    pub am: bool,
    pub pm: bool,
}

#[derive(Debug)]
pub enum PeopleType {
    Candidate,
    Examiner,
}

// Examiners will just leave AM / PM regardless of overlapping slots in AM&PM
async fn fill_by_time( // AM & PM EXAMINERS
    session_id: &Uuid,
    pool: &sqlx::PgPool,
    total_am_examiners: usize, // including all-day + AM
    total_pm_examiners: usize, // including all-day + PM
    total_am_cap: usize,
    total_pm_cap: usize,
    female: bool,
) -> Result<(Vec<Examiner>, Vec<Examiner>, Vec<Examiner>), AppError> {
    let mut am_examiner_diff = total_am_cap as isize - total_am_examiners as isize;
    let mut pm_examiner_diff = total_pm_cap as isize - total_pm_examiners as isize;

    // for AM / PM examiners fulfillment, examiners should include all-day examiners + AM/PM-only examiners
    // all-day examiners are the only ones who can make it to "ANY" slots too
    let mut new_full_examiners = Vec::new();
    let mut new_am_examiners = Vec::new();
    let mut new_pm_examiners = Vec::new();
    if am_examiner_diff > 0 && pm_examiner_diff > 0 { // get minimum number of examiners, fill with full-day first
        let mut common_diff = am_examiner_diff - pm_examiner_diff;
        if common_diff > 0 || common_diff < 0 {
            common_diff = cmp::min(am_examiner_diff, pm_examiner_diff);
        } else {
            common_diff = cmp::max(am_examiner_diff, pm_examiner_diff);
        }
        for _ in 0..common_diff {
            let new_examiner = super::examiners::create_fill(session_id.clone(), pool, None, female).await?;
            new_full_examiners.push(new_examiner);
        }
        am_examiner_diff -= common_diff;
        pm_examiner_diff -= common_diff;
    }
    if am_examiner_diff > 0 { // any standalone AM examiners
        for _ in 0..am_examiner_diff {
            let new_examiner = super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: false }), female).await?;
            new_am_examiners.push(new_examiner);
        }
    }
    if pm_examiner_diff > 0 { // any standalone PM examiners
        for _ in 0..pm_examiner_diff {
            let new_examiner = super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: false, pm: true }), female).await?;
            new_pm_examiners.push(new_examiner);
        }
    }
    Ok((new_full_examiners, new_am_examiners, new_pm_examiners))
}

async fn fill_by_slot(  // EXAMINERS ONLY
    session_id: &Uuid,
    pool: &sqlx::PgPool,
    total_examiners: usize, // includes all examiners (female + not)
    total_female_examiners: usize,
    exam_circuit_cap: usize,
    exam_female_circuit_cap: usize,
    am: bool, // fill with slot's timing. E.g: AM-only: am=true, pm=false
    pm: bool,
 ) -> Result<usize, AppError> {
    // female examiners
    let mut total_examiners = total_examiners.clone();
    let female_examiner_diff = exam_female_circuit_cap as isize - total_female_examiners as isize;
    let examiner_diff = exam_circuit_cap as isize - total_examiners as isize;
    if total_female_examiners > exam_female_circuit_cap {
        trace!("There are more female examiners than capacity. {} over from {}", total_female_examiners, exam_female_circuit_cap);
        return Ok(total_female_examiners - exam_female_circuit_cap);
    } else if total_female_examiners < exam_female_circuit_cap {
        for _ in 0..female_examiner_diff {
            let _ = super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am, pm, }), true).await?;
            total_examiners += 1;
        }
    }

    // all examiners
    if total_examiners > exam_circuit_cap {
        trace!("There are more examiners than capacity. {} over from {}", total_examiners, exam_circuit_cap);
        return Ok(total_examiners - exam_circuit_cap);
    } else if total_examiners < exam_circuit_cap {
        for _ in 0..examiner_diff {
            let _ = super::examiners::create_fill(session_id.clone(), &pool, Some(Availability { am, pm }), false).await?;
        }
    }
    Ok(0)
}

async fn fill_slot_fixed_time(  // CANDIDATES ONLY. for sessions with 1 slot with runs in either AM or PM
    session_id: &Uuid,
    pool: &sqlx::PgPool,
    total_candidates: usize,
    total_female_candidates: usize,
    can_circuit_cap: usize,
    can_female_circuit_cap: usize,
    am: bool,
 ) -> Result<(), AppError> {
    // female candidates
    let mut total_candidates = total_candidates.clone();
    if total_female_candidates > can_female_circuit_cap {
        return Err(AppError::from(anyhow!("There are more AM/PM female candidates than capacity. {} over from {}", total_female_candidates, can_female_circuit_cap)));
    } else if total_female_candidates < can_female_circuit_cap {
        if total_female_candidates % 2 != 0 {
            let _ = super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: am, pm: !am }), true).await?;
            total_candidates += 1;
        }
    }

    // all candidates
    if total_candidates > can_circuit_cap {
        return Err(AppError::from(anyhow!("There are more candidates than capacity. {} over from {}", total_candidates, can_circuit_cap)));
    } else if total_candidates < can_circuit_cap {
        if total_candidates % 2 != 0 {
            let _ = super::candidates::create_fill(session_id.clone(), &pool, Some(Availability { am: am, pm: !am }), false).await?;
        }
    }
    Ok(())
}

fn is_odd(n: usize) -> bool {
    n & 1 != 0 // even: LSB = 0, odd: LSB = 1
}

/// x_split, y_split, z_split, x_diff, y_diff, z_diff
fn even_split(
    total_count: usize,
    mut x_fix_count: usize, // am
    mut y_fix_count: usize, // pm
    mut z_flex_count: usize, // any
    z_fix_cap: usize, // any-cap
) -> Result<(usize, usize, usize, usize, usize, usize), AppError> {
    x_fix_count += x_fix_count & 1;
    y_fix_count += y_fix_count & 1;
    z_flex_count = z_flex_count.saturating_sub(x_fix_count & 1 + y_fix_count & 1);

    if z_fix_cap == 0 {
        // 2 sets
        let target_split = total_count / 2;
        let mut x_split = target_split;
        let mut y_split = target_split;

        if x_split & 1 != x_fix_count & 1 {
            if x_split > 0 {
                x_split -= 1;
                y_split += 1;
            } else {
                x_split += 1;
                y_split -= 1;
            }
        }
        
        if y_split & 1 != y_fix_count & 1 {
            if y_split > 0 {
                y_split -= 1;
                x_split += 1;
            } else {
                return Err(AppError::from(anyhow!("y_split = 0")));
            }
        }
        
        // find the z-flex split into each x and y respectively
        let x_diff = x_split.checked_sub(x_fix_count).ok_or_else(|| anyhow!("Underflow: x_split = {}, x_fix_count = {}", x_split, x_fix_count))?;
        let y_diff = y_split.checked_sub(y_fix_count).ok_or_else(|| anyhow!("Underflow: y_split = {}, y_fix_count = {}", y_split, y_fix_count))?;
        
        let all_sum = x_diff.checked_add(y_diff).ok_or_else(|| anyhow!("Overflow in all_sum"))?;
        if all_sum != z_flex_count || x_split & 1 != 0 || y_split & 1 != 0 {
            trace!("Invalid split: all_sum = {}, z_flex_count = {}, x_split = {}, y_split = {}", all_sum, z_flex_count, x_split, y_split);
            if all_sum < z_flex_count {
                let diff = z_flex_count - all_sum;
                if x_split > y_split {
                    x_split += diff;
                } else {
                    y_split += diff;
                }
            }
        }
        Ok((x_split, y_split, z_fix_cap, x_diff, y_diff, z_fix_cap))
    } else {
        // 3 sets
        if z_flex_count > z_fix_cap {
            return Err(AppError::from(anyhow!(
                "z_flex_count = {} exceeds z_fix_cap = {}",
                z_flex_count,
                z_fix_cap
            )));
        }

        let base_split = (total_count / 3) & !1; // round down to nearest even number
        let mut remainder = total_count - base_split * 3; // 0, 2, or 4

        let mut x_split = base_split;
        let mut y_split = base_split;
        let mut z_split = base_split;

        if remainder >= 2 {
            x_split += 2;
            remainder -= 2;
        }
        if remainder >= 2 {
            y_split += 2;
            remainder -= 2;
        }

        let x_diff_needed = x_split.checked_sub(x_fix_count)
            .ok_or_else(|| anyhow!("Underflow: x_split = {}, x_fix_count = {}", x_split, x_fix_count))?;
        let y_diff_needed = y_split.checked_sub(y_fix_count)
            .ok_or_else(|| anyhow!("Underflow: y_split = {}, y_fix_count = {}", y_split, y_fix_count))?;
        let mut x_diff = x_diff_needed;
        let mut y_diff = y_diff_needed;
        let mut z_diff = z_flex_count.saturating_sub(x_diff + y_diff);
        z_split = z_diff;

        x_split = x_fix_count.checked_add(x_diff)
            .ok_or_else(|| anyhow!("Overflow in x_split"))?;
        y_split = y_fix_count.checked_add(y_diff)
            .ok_or_else(|| anyhow!("Overflow in y_split"))?;

        // total assigned so far
        let mut total_assigned = x_diff.checked_add(y_diff)
            .and_then(|sum| sum.checked_add(z_diff))
            .ok_or_else(|| anyhow!("Overflow in total_assigned"))?;

        // Minimising differences between x, y, z splits
        while total_assigned != z_flex_count || x_split & 1 != 0 || y_split & 1 != 0 || z_split & 1 != 0 {
            if total_assigned > z_flex_count {
                return Err(AppError::from(anyhow!("Overassigned z_flex_count")));
            }

            let max_split = x_split.max(y_split).max(z_split);
            let min_split = x_split.min(y_split).min(z_split);

            if max_split - min_split > 2 && total_assigned < z_flex_count {
                let available = z_flex_count - total_assigned;
                if available >= 2 {
                    if max_split == x_split && x_diff > 0 {
                        x_split -= 2;
                        x_diff -= 2;
                        if y_split == min_split {
                            y_split += 2;
                            y_diff += 2;
                        } else {
                            z_split += 2;
                            z_diff += 2;
                        }
                    } else if max_split == y_split && y_diff > 0 {
                        y_split -= 2;
                        y_diff -= 2;
                        if x_split == min_split {
                            x_split += 2;
                            x_diff += 2;
                        } else {
                            z_split += 2;
                            z_diff += 2;
                        }
                    } else if max_split == z_split && z_diff > 0 {
                        z_split -= 2;
                        z_diff -= 2;
                        if x_split == min_split {
                            x_split += 2;
                            x_diff += 2;
                        } else {
                            y_split += 2;
                            y_diff += 2;
                        }
                    }
                }
            } else if total_assigned < z_flex_count {
                let remaining = z_flex_count - total_assigned;
                if remaining >= 2 {
                    if x_split == min_split {
                        x_split += 2;
                        x_diff += 2;
                    } else if y_split == min_split {
                        y_split += 2;
                        y_diff += 2;
                    } else {
                        z_split += 2;
                        z_diff += 2;
                    }
                } else {
                    // Can't add more without exceeding
                    break;
                }
            } else {
                // All assigned, check evenness
                break;
            }

            total_assigned = x_diff.checked_add(y_diff)
                .and_then(|sum| sum.checked_add(z_diff))
                .ok_or_else(|| anyhow!("Overflow in total_assigned"))?;
        }

        // Final validation
        if total_assigned != z_flex_count || x_split & 1 != 0 || y_split & 1 != 0 || z_split & 1 != 0 {
            trace!("Invalid 3-way split: total_assigned = {}, z_flex_count = {}, x_split = {}, y_split = {}, z_split = {}",
                total_assigned, z_flex_count, x_split, y_split, z_split);
            if total_assigned < z_flex_count {
                let diff = z_flex_count - total_assigned;
                let min_split = cmp::min(x_split, cmp::min(y_split, z_split));
                if x_split == min_split {
                    x_split += diff;
                } else if y_split == min_split {
                    y_split += diff;
                } else {
                    z_split += diff;
                }
            }
        }

        Ok((x_split, y_split, z_split, x_diff, y_diff, z_diff))
    }
    
}

async fn fill_any_priority(
    session_id: &Uuid,
    pool: &sqlx::PgPool,
    ppl_type: PeopleType, 
    total_count: usize,
    x_fix_count: usize, // am_only
    x_cap: usize, // am_cap
    y_fix_count: usize, // pm_only
    y_cap: usize, //pm_cap
    z_flex_count: usize, // any
    z_cap: usize, // any_cap
) -> Result<(), AppError> { // sorry this part is cooked af frfr, defo need that refactoring
    if is_odd(total_count) {
        let odd_count = is_odd(x_fix_count) as isize + is_odd(y_fix_count) as isize + is_odd(z_flex_count) as isize;
        if z_cap == 0 {
            // total_count = x_fix_count + y_fix_count + z_flex_count
            let (x_split, y_split, ..) = even_split(total_count, x_fix_count, y_fix_count, z_flex_count, z_cap)?;
            // either all 3 are odd, or 1 of them is odd
            if odd_count == 1 {
                if is_odd(x_fix_count) {
                    if x_fix_count > x_cap {
                        return Err(AppError::from(anyhow!("There are more x_fix_count than capacity. {} over from {}", x_fix_count, x_cap)));
                    } else if x_fix_count < x_cap {
                        match ppl_type {
                            PeopleType::Candidate => {
                                super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: false }), true).await?;
                            },
                            PeopleType::Examiner => {
                                super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: false }), true).await?;
                            },
                        };
                    }
                } else if is_odd(y_fix_count) {
                    if y_fix_count > y_cap {
                        return Err(AppError::from(anyhow!("There are more y_fix_count than capacity. {} over from {}", y_fix_count, y_cap)));
                    } else if y_fix_count < y_cap {
                        match ppl_type {
                            PeopleType::Candidate => {
                                super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: false, pm: true }), true).await?;
                            },
                            PeopleType::Examiner => {
                                super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: false, pm: true }), true).await?;
                            }
                        }
                    }
                } else if is_odd(z_flex_count) {
                    if x_split < x_cap || y_split < y_cap {
                        match ppl_type {
                            PeopleType::Candidate => {
                                super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                            },
                            PeopleType::Examiner => {
                                super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                            }
                        }
                    } else if x_split > x_cap || y_split > y_cap {
                        return Err(AppError::from(anyhow!("There are more total_count than capacity. x: {} over from {} | y: {} over from {}", x_split, x_cap, y_split, y_cap)));
                    }
                }
            }
            if odd_count == 3 { // only need 1 addition
                if x_split < x_cap || y_split < y_cap {
                    match ppl_type {
                        PeopleType::Candidate => {
                            super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                        },
                        PeopleType::Examiner => {
                            super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                        }
                    }
                } else if x_split > x_cap || y_split > y_cap {
                    return Err(AppError::from(anyhow!("There are more total_count than capacity. x: {} over from {} | y: {} over from {}", x_split, x_cap, y_split, y_cap)));
                }
            }
        } else {
            if odd_count == 1 {
                if is_odd(x_fix_count) {
                    if x_fix_count > x_cap {
                        return Err(AppError::from(anyhow!("There are more x_fix_count than capacity. {} over from {}", x_fix_count, x_cap)));
                    } else if x_fix_count < x_cap {
                        if z_flex_count > 0 { // any_count must be 0 or even
                            match ppl_type {
                                PeopleType::Candidate => {
                                    super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                                },
                                PeopleType::Examiner => {
                                    super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                                }
                            }
                        } else {
                            match ppl_type {
                                PeopleType::Candidate => {
                                    super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: false }), true).await?;
                                },
                                PeopleType::Examiner => {
                                    super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: false }), true).await?;
                                },
                            };
                        }
                    }
                } else if is_odd(y_fix_count) {
                    if y_fix_count > y_cap {
                        return Err(AppError::from(anyhow!("There are more y_fix_count than capacity. {} over from {}", y_fix_count, y_cap)));
                    } else if y_fix_count < y_cap {
                        if z_flex_count > 0 { // any_count must be 0 or even
                            match ppl_type {
                                PeopleType::Candidate => {
                                    super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                                },
                                PeopleType::Examiner => {
                                    super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                                }
                            }
                        } else {
                            match ppl_type {
                                PeopleType::Candidate => {
                                    super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: false, pm: true }), true).await?;
                                },
                                PeopleType::Examiner => {
                                    super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: false, pm: true }), true).await?;
                                }
                            }
                        }
                    }
                } else if is_odd(z_flex_count) {
                    // currently prefer filling with ANY (more flexability, less hard-coded logic)
                    match ppl_type {
                        PeopleType::Candidate => {
                            super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                        },
                        PeopleType::Examiner => {
                            super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                        }
                    }
                }
            }
            if odd_count == 3 {
                if x_fix_count < x_cap && y_fix_count < y_cap {
                    match ppl_type {
                        PeopleType::Candidate => {
                            super::candidates::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                        },
                        PeopleType::Examiner => {
                            super::examiners::create_fill(session_id.clone(), pool, Some(Availability { am: true, pm: true }), true).await?;
                        }
                    }
                } else if x_fix_count > x_cap {
                    return Err(AppError::from(anyhow!("There are more x_fix_count than capacity. {} over from {}", x_fix_count, x_cap)));
                } else if y_fix_count > y_cap {
                    return Err(AppError::from(anyhow!("There are more y_fix_count than capacity. {} over from {}", y_fix_count, y_cap)));
                }
            }
        }
    }
    Ok(())
}

async fn gen_new( // for static/initial allocation
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    session: Query<SomethingID> // session id
) -> Result<impl IntoResponse, AppError> {
    if !User::is_admin(&pool, &claim.id).await? {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    let session_id = session.0.id;

    let slots = Slot::get_all_by_session(&pool, &session_id).await?;
    let stations = Station::get_by_session(&pool, &session_id).await?;
    let stations_not_rest = Station::get_not_rest_by_session(&pool, &session_id).await?;

    let batch_id = Uuid::new_v4();
    let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

    // how to manage overflow examiners? go ahead?
    match slots.len() {
        1 => { // as only 1 slot, check if any runs are in AM or PM, as all candidates need to be there for its full duration
            trace!("1 Slot Only");
            let cur_slot = &slots[0];
            let (circuit_result, female_circuit_result, am_runs_result, pm_runs_result) = tokio::join!(
                Circuit::get_by_slot(&pool, &cur_slot.id),
                Circuit::get_female_slot(&pool, &cur_slot.id),
                Run::get_by_time(&pool, &cur_slot.id, RunTime::AM),
                Run::get_by_time(&pool, &cur_slot.id, RunTime::PM)
            );
            let circuits = circuit_result?;
            let female_circuits = female_circuit_result?; 
            let am_runs = am_runs_result?;
            let pm_runs = pm_runs_result?;

            let am_runs_count = am_runs.len();
            let pm_runs_count = pm_runs.len();
            let mut examiners = Vec::new();
            let mut candidates = Vec::new();
            let exam_circuit_cap = circuits.len() * stations_not_rest.len();
            let can_circuit_cap = circuits.len() * stations.len() * 2;
            let exam_female_circuit_cap = female_circuits.len() * stations_not_rest.len();
            let can_female_circuit_cap = female_circuits.len() * stations.len() * 2;

            if am_runs_count > 0 && pm_runs_count == 0 { // Slot is AM runs only
                trace!("AM runs only");
                // candidates
                let (candidate_result, female_candidate_result) = tokio::join!(
                    Candidate::get_all_by_time(&pool, &session_id, RunTime::AM),
                    Candidate::get_all_female_by_time(&pool, &session_id, RunTime::AM)
                );
                candidates = candidate_result?;
                let female_candidates = female_candidate_result?;
                fill_slot_fixed_time(&session_id, &pool, candidates.len(), female_candidates.len(), can_circuit_cap, can_female_circuit_cap, true).await?;
                candidates = Candidate::get_all_by_time(&pool, &session_id, RunTime::AM).await?;

                // female examiners
                let female_examiners = Examiner::get_all_female_by_time(&pool, &session_id, RunTime::AM).await?;
                let total_female_examiners = female_examiners.len();
                if total_female_examiners > exam_female_circuit_cap {
                    trace!("There are more AM female examiners than capacity. Checking if there's space in non-female circuits");
                    let am_examiners = Examiner::get_all_by_time(&pool, &session_id, RunTime::AM).await?;
                    if am_examiners.len() >= exam_circuit_cap { // if there's no way to move them
                        return Err(AppError::from(anyhow!("There are more AM examiners than capacity. {} over from {}", am_examiners.len(), exam_circuit_cap)));
                    }
                } else if total_female_examiners < exam_female_circuit_cap {
                    if total_female_examiners % 2 != 0 {
                        let _ = super::examiners::create_fill(session_id.clone(), &pool, Some(Availability { am: true, pm: false }), true).await?;
                    }
                }

                // all examiners
                examiners = Examiner::get_all_by_time(&pool, &session_id, RunTime::AM).await?;
                let total_examiners = examiners.len();
                if total_examiners > exam_circuit_cap {
                    return Err(AppError::from(anyhow!("There are more examiners than capacity. {} over from {}", total_examiners, exam_circuit_cap)));
                } else if total_examiners < exam_circuit_cap {
                    if total_examiners % 2 != 0 {
                        let new_examiner = super::examiners::create_fill(session_id.clone(), &pool, Some(Availability { am: true, pm: false }), false).await?;
                        examiners.push(new_examiner);
                    }
                }
            } else if am_runs_count == 0 && pm_runs_count > 0 { // Slot is PM runs only
                trace!("PM runs only");
                // candidates
                let (candidate_result, female_candidate_result) = tokio::join!(
                    Candidate::get_all_by_time(&pool, &session_id, RunTime::PM),
                    Candidate::get_all_female_by_time(&pool, &session_id, RunTime::PM)
                );
                candidates = candidate_result?;
                let female_candidates = female_candidate_result?;
                fill_slot_fixed_time(&session_id, &pool, candidates.len(), female_candidates.len(), can_circuit_cap, can_female_circuit_cap, false).await?;
                candidates = Candidate::get_all_by_time(&pool, &session_id, RunTime::PM).await?;

                // female examiners
                let female_examiners = Examiner::get_all_female_by_time(&pool, &session_id, RunTime::PM).await?;
                let total_female_examiners = female_examiners.len();
                if total_female_examiners > exam_female_circuit_cap {
                    trace!("There are more PM female examiners than capacity. Checking if there's space in non-female circuits");
                    let pm_examiners = Examiner::get_all_by_time(&pool, &session_id, RunTime::PM).await?;
                    if pm_examiners.len() >= exam_circuit_cap { // if there's no way to move them
                        return Err(AppError::from(anyhow!("There are more PM examiners than capacity. {} over from {}", pm_examiners.len(), exam_circuit_cap)));
                    }
                } else if total_female_examiners < exam_female_circuit_cap {
                    if total_female_examiners % 2 != 0 {
                        let _ = super::examiners::create_fill(session_id.clone(), &pool, Some(Availability { am: false, pm: true }), true).await?;
                    }
                }

                // all examiners
                examiners = Examiner::get_all_by_time(&pool, &session_id, RunTime::PM).await?;
                let total_examiners = examiners.len();
                if total_examiners > exam_circuit_cap {
                    return Err(AppError::from(anyhow!("There are more examiners than capacity. {} over from {}", total_examiners, exam_circuit_cap)));
                } else if total_examiners < exam_circuit_cap {
                    if total_examiners % 2 != 0 {
                        let new_examiner = super::examiners::create_fill(session_id.clone(), &pool, Some(Availability { am: false, pm: true }), false).await?;
                        examiners.push(new_examiner);
                    }
                }
            } else if am_runs_count > 0 && pm_runs_count > 0 { // Slot has runs in both AM and PM
                trace!("AM and PM runs");
                // female candidates
                let female_candidates = Candidate::get_female_ava_all(&pool, &session_id, Availability { am: true, pm: true }).await?;
                let total_female_candidates = female_candidates.len();
                if total_female_candidates > can_female_circuit_cap {
                    return Err(AppError::from(anyhow!("There are more female candidates than capacity. {} over from {}", total_female_candidates, can_female_circuit_cap)));
                } else if total_female_candidates < can_female_circuit_cap {
                    if total_female_candidates % 2 != 0 { // just need to make it even, don't need to fill it all, only examiners need to be filled to MAX
                        let _ = super::candidates::create_fill(session_id.clone(), &pool, Some(Availability { am: true, pm: true }), true).await?;
                    }
                }

                // all candidates
                candidates = Candidate::get_ava_all(&pool, &session_id, Availability { am: true, pm: true }).await?;
                let total_candidates = candidates.len();
                if total_candidates > can_circuit_cap {
                    return Err(AppError::from(anyhow!("There are more candidates than capacity. {} over from {}", total_candidates, can_circuit_cap)));
                } else if total_candidates < can_circuit_cap {
                    if total_candidates % 2 != 0 {
                        let new_candidate = super::candidates::create_fill(session_id.clone(), &pool, Some(Availability { am: true, pm: true }), false).await?;
                        candidates.push(new_candidate);
                    }
                }

                // female examiners
                let (am_result, pm_result) = tokio::join!(
                    Examiner::get_all_female_by_time(&pool, &session_id, RunTime::AM),
                    Examiner::get_all_female_by_time(&pool, &session_id, RunTime::PM)
                );
                let am_female_examiners = am_result?;
                let pm_female_examiners = pm_result?;
                
                let am_female_examiner_diff = female_circuits.len() as isize - am_female_examiners.len() as isize;
                let pm_female_examiner_diff = female_circuits.len() as isize - pm_female_examiners.len() as isize;
                if am_female_examiner_diff < 0 {
                    trace!("There are more AM female examiners than capacity. Checking if there's space in non-female circuits");
                    let am_examiners = Examiner::get_all_by_time(&pool, &session_id, RunTime::AM).await?;
                    if am_examiners.len() >= exam_circuit_cap { // if there's no way to move them
                        return Err(AppError::from(anyhow!("There are more AM examiners than capacity. {} over from {}", am_examiners.len(), exam_circuit_cap)));
                    }
                }
                if pm_female_examiner_diff < 0 {
                    trace!("There are more PM female examiners than capacity. Checking if there's space in non-female circuits");
                    let pm_examiners = Examiner::get_all_by_time(&pool, &session_id, RunTime::PM).await?;
                    if pm_examiners.len() >= exam_circuit_cap { // if there's no way to move them
                        return Err(AppError::from(anyhow!("There are more PM examiners than capacity. {} over from {}", pm_examiners.len(), exam_circuit_cap)));
                    }
                }

                let _ = fill_by_time(
                    &session_id,
                    &pool,
                    am_female_examiners.len(),
                    pm_female_examiners.len(),
                    exam_female_circuit_cap,
                    exam_female_circuit_cap,
                    true
                ).await?;

                // all examiners
                let (am_result, pm_result) = tokio::join!(
                    Examiner::get_all_by_time(&pool, &session_id, RunTime::AM),
                    Examiner::get_all_by_time(&pool, &session_id, RunTime::PM),
                );
                let mut am_examiners = am_result?;
                let mut pm_examiners = pm_result?;
                
                let (mut new_full_examiners, mut new_am_examiners, mut new_pm_examiners) = fill_by_time(
                    &session_id,
                    &pool,
                    am_examiners.len(),
                    pm_examiners.len(),
                    exam_circuit_cap,
                    exam_circuit_cap,
                    false
                ).await?;

                am_examiners.append(&mut new_full_examiners);
                pm_examiners.append(&mut new_full_examiners);
                am_examiners.append(&mut new_am_examiners);
                pm_examiners.append(&mut new_pm_examiners);

                trace!("Total Candidates: {} = Total Examiners: AM: {}, PM: {}", candidates.len(), am_examiners.len(), pm_examiners.len());
                // allocation needs to take account that if theres more female examiners than capacity, move them into non-female circuits
                // let candidate_allocations = allocate_by_slot(&circuits, &stations, &candidates)?;
                // let am_examiner_allocations = allocate_by_time(&circuits, &stations, &am_examiners)?;
                // let pm_examiner_allocations = allocate_by_time(&circuits, &stations, &pm_examiners)?;

                // Allocation::add_by_slot(&mut transaction, candidate_allocations, &batch_id, &cur_slot.id, &claim.id).await?;
                // Allocation::add_by_time(&mut transaction, am_examiner_allocations, &batch_id, &cur_slot.id, &claim.id).await?;
                // Allocation::add_by_time(&mut transaction, pm_examiner_allocations, &batch_id, &cur_slot.id, &claim.id).await?;

                // transaction.commit().await.map_err(|e| AppError::from(anyhow!("Failed to commit transaction: {}", e)))?;

                return Ok(StatusCode::OK.into_response());
            } else {
                return Err(AppError::from(anyhow!("Slot has no runs")));
            }

            trace!("Total Candidates: {} = Total Examiners: {}", candidates.len(), examiners.len());
            // allocations (for either AM or PM runs)
            // let candidates_allocation = allocate_by_slot(&circuits, &stations, &candidates)?;
            // let examiners_allocations = allocate_by_time(&circuits, &stations, &examiners)?;

            // Allocation::add_by_slot(&mut transaction, candidates_allocation, &batch_id, &cur_slot.id, &claim.id).await?;
            // Allocation::add_by_time(&mut transaction, examiners_allocations, &batch_id, &cur_slot.id, &claim.id).await?;

            // transaction.commit().await.map_err(|e| AppError::from(anyhow!("Failed to commit transaction: {}", e)))?;
        },
        _ => { // slots > 1
            trace!("Multiple Slots");
            // sort slots into AM and PM (before 12:00 start = AM, else = PM)
            let mut am_only_slots: Vec<Slot> = Vec::new();
            let mut pm_only_slots: Vec<Slot> = Vec::new();
            let mut any_slots: Vec<Slot> = Vec::new();
            for cur_slot in slots {
                let (am_runs_result, pm_runs_result) = tokio::join!(
                    Run::get_by_time(&pool, &cur_slot.id, RunTime::AM),
                    Run::get_by_time(&pool, &cur_slot.id, RunTime::PM)
                );
                let am_runs = am_runs_result?;
                let pm_runs = pm_runs_result?;
                if am_runs.len() > 0 && pm_runs.len() == 0 { // AM runs only
                    am_only_slots.push(cur_slot);
                } else if am_runs.len() == 0 && pm_runs.len() > 0 { // PM runs only
                    pm_only_slots.push(cur_slot);
                } else if am_runs.len() > 0 && pm_runs.len() > 0 { // AM & PM runs
                    any_slots.push(cur_slot);
                } else {
                    return Err(AppError::from(anyhow!("Slot has no runs")));
                }
            }

            trace!("Slots: AM: {}, PM: {}, Both: {}", am_only_slots.len(), pm_only_slots.len(), any_slots.len());
            // CANDIDATES
            let (can_am_result, can_pm_result) = tokio::join!(
                Candidate::get_ava_all(&pool, &session_id, Availability { am: true, pm: false }),
                Candidate::get_ava_all(&pool, &session_id, Availability { am: false, pm: true }),
            );
            let candidates_am_only = can_am_result?; // candidates that can ONLY do AM
            let candidates_pm_only = can_pm_result?; // candidates that can ONLY do PM
            if candidates_am_only.len() != 0 && am_only_slots.len() == 0 {
                return Err(AppError::from(anyhow!("No AM slots but have {} candidates who can only make it to AM", candidates_am_only.len())));
            } else if candidates_pm_only.len() != 0 && pm_only_slots.len() == 0 {
                return Err(AppError::from(anyhow!("No PM slots but have {} candidates who can only make it to PM", candidates_pm_only.len())));
            }

            // split candidates evenly into each slot (and make exceptions for candidates with specific availability)
            let (can_am_fem_result, can_pm_fem_result, can_any_fem_result, can_fem_result) = tokio::join!(
                Candidate::get_female_ava_all(&pool, &session_id, Availability { am: true, pm: false }),
                Candidate::get_female_ava_all(&pool, &session_id, Availability { am: false, pm: true }),
                Candidate::get_female_ava_all(&pool, &session_id, Availability { am: true, pm: true }),
                Candidate::get_female_all(&pool, &session_id)
            );
            let candidates_female_am_only = can_am_fem_result?;
            let can_female_am_only_count = candidates_female_am_only.len();

            let mut am_can_cap = 0;
            let mut am_can_female_cap = 0;
            for am_slot in &am_only_slots {
                let circuits = Circuit::get_by_slot(&pool, &am_slot.id).await?;
                let female_circuits = Circuit::get_female_slot(&pool, &am_slot.id).await?;
                am_can_cap += circuits.len() * stations.len() * 2;
                am_can_female_cap += female_circuits.len() * stations.len() * 2;
            }

            let candidates_female_pm_only = can_pm_fem_result?;
            let can_female_pm_only_count = candidates_female_pm_only.len();

            let mut pm_can_cap = 0;
            let mut pm_can_female_cap = 0;
            for pm_slot in &pm_only_slots {
                let circuits = Circuit::get_by_slot(&pool, &pm_slot.id).await?;
                let female_circuits = Circuit::get_female_slot(&pool, &pm_slot.id).await?;
                pm_can_cap += circuits.len() * stations.len() * 2;
                pm_can_female_cap += female_circuits.len() * stations.len() * 2;
            }

            let candidates_female_any = can_any_fem_result?;
            let can_female_any_count = candidates_female_any.len();

            let mut any_can_cap = 0;
            let mut any_can_female_cap = 0;
            for any_slot in &any_slots {
                let circuits = Circuit::get_by_slot(&pool, &any_slot.id).await?;
                let female_circuits = Circuit::get_female_slot(&pool, &any_slot.id).await?;
                any_can_cap += circuits.len() * stations.len() * 2;
                any_can_female_cap += female_circuits.len() * stations.len() * 2;
            }

            let candidates_female = can_fem_result?; // all female candidates
            let can_female_count = candidates_female.len();

            trace!("Female Candidates: AM-only = {}, PM-only = {}, Any = {}, TOTAL: {}", can_female_am_only_count, can_female_pm_only_count, can_female_any_count, can_female_count);

            //// FILLING ODD PAIRS BASED ON AM + PM + ANY
            
            // female_candidates
            fill_any_priority(&session_id, &pool, PeopleType::Candidate,
                can_female_count,
                can_female_am_only_count,
                am_can_female_cap,
                can_female_pm_only_count,
                pm_can_female_cap,
                can_female_any_count,
                any_can_female_cap
            ).await?;

            let (can_am_result, can_pm_result, can_any_result, can_result) = tokio::join!(
                Candidate::get_ava_all(&pool, &session_id, Availability { am: true, pm: false }),
                Candidate::get_ava_all(&pool, &session_id, Availability { am: false, pm: true }),
                Candidate::get_ava_all(&pool, &session_id, Availability { am: true, pm: true }),
                Candidate::get_all_by_session(&pool, &session_id)
            );
            let candidates_am_only = can_am_result?; // candidates that can ONLY do AM
            let candidates_pm_only = can_pm_result?; // candidates that can ONLY do PM
            let candidates_any = can_any_result?; // candidates that can do AM or PM
            let candidates = can_result?; // all candidates

            let can_am_only_count = candidates_am_only.len();
            let can_pm_only_count = candidates_pm_only.len();
            let can_any_count = candidates_any.len();
            let can_count = candidates.len();

            trace!("Candidates: AM-only = {}, PM-only = {}, Any = {}, TOTAL: {}", can_am_only_count, can_pm_only_count, can_any_count, can_count);

            // all candidates
            fill_any_priority(&session_id, &pool, PeopleType::Candidate,
                can_count,
                can_am_only_count,
                am_can_cap,
                can_pm_only_count,
                pm_can_cap,
                can_any_count,
                any_can_cap
            ).await?;

            //// SPLITTING INTO AM + PM + ANY
            
            // female candidates
            let (can_am_fem_result, can_pm_fem_result, can_any_fem_result, can_fem_result) = tokio::join!(
                Candidate::get_female_ava_all(&pool, &session_id, Availability { am: true, pm: false }),
                Candidate::get_female_ava_all(&pool, &session_id, Availability { am: false, pm: true }),
                Candidate::get_female_ava_all(&pool, &session_id, Availability { am: true, pm: true }),
                Candidate::get_female_all(&pool, &session_id)
            );
            let can_female_am_only_count = can_am_fem_result?.len();
            let can_female_pm_only_count = can_pm_fem_result?.len();
            let can_female_any_count = can_any_fem_result?.len();
            let can_female_count = can_fem_result?.len();

            // purpose is to split can_female_any equally among am, pm (and any if exists), whilst giving priority to people who can only make it to AM/PM
            let (
                can_am_female_split,
                can_pm_female_split,
                can_any_female_split,
                can_am_female_diff,
                can_pm_female_diff,
                can_any_female_diff
            ) = even_split(
                can_female_count,
                can_female_am_only_count,
                can_female_pm_only_count,
                can_female_any_count,
                any_can_female_cap,
            )?;

            // all candidates
            let (can_am_result, can_pm_result, can_any_result, can_result) = tokio::join!(
                Candidate::get_ava_all(&pool, &session_id, Availability { am: true, pm: false }),
                Candidate::get_ava_all(&pool, &session_id, Availability { am: false, pm: true }),
                Candidate::get_ava_all(&pool, &session_id, Availability { am: true, pm: true }),
                Candidate::get_all_by_session(&pool, &session_id)
            );
            
            // female is distributed independent from the rest, this could create inbalance?
            let can_am_only_count = can_am_result?.len() - can_female_am_only_count;
            let can_pm_only_count = can_pm_result?.len() - can_female_pm_only_count;
            let can_any_count = can_any_result?.len() - can_female_any_count;
            let can_count = can_result?.len() - can_female_count;

            let (
                can_am_split,
                can_pm_split,
                can_any_split,
                can_am_diff,
                can_pm_diff,
                can_any_diff,
            ) = even_split(
                can_count,
                can_am_only_count,
                can_pm_only_count,
                can_any_count,
                any_can_cap,
            )?;

            let total_can_am = can_am_female_split + can_am_split;
            let total_can_pm = can_pm_female_split + can_pm_split;
            let total_can_any = can_any_female_split + can_any_split;

            trace!("Total Candidates split: AM: {}, PM: {}, BOTH: {} ==> Total Female Candidates split: AM: {}, PM: {}, BOTH: {}", total_can_am, total_can_pm, total_can_any, can_am_female_split, can_pm_female_split, can_any_female_split);

            // EXAMINERS
            let (exam_am_result, exam_pm_result) = tokio::join!(
                Examiner::get_ava_all(&pool, &session_id, Availability { am: true, pm: false }),
                Examiner::get_ava_all(&pool, &session_id, Availability { am: false, pm: true }),
            );
            let examiners_am_only = exam_am_result?;
            let examiners_pm_only = exam_pm_result?;
            if examiners_am_only.len() != 0 && am_only_slots.len() == 0 {
                return Err(AppError::from(anyhow!("No AM slots but have {} examiners who can only make it to AM", examiners_am_only.len())));
            } else if examiners_pm_only.len() != 0 && pm_only_slots.len() == 0 {
                return Err(AppError::from(anyhow!("No PM slots but have {} examiners who can only make it to PM", examiners_pm_only.len())));
            }
            
            // creating fills for examiners
            // havent implemented finding common missing examiners in both AM & PM and merging them into a both examiner insertion

            let mut am_excess_exam: usize = 0;
            for am_slot in &am_only_slots {
                let (circuit_result, fem_circuit_result, am_fem_result, am_result) = tokio::join!(
                    Circuit::get_by_slot(&pool, &am_slot.id),
                    Circuit::get_female_slot(&pool, &am_slot.id),
                    Examiner::get_all_female_by_time(&pool, &session_id, RunTime::AM),
                    Examiner::get_all_by_time(&pool, &session_id, RunTime::AM)
                );
                let circuits = circuit_result?;
                let female_circuits = fem_circuit_result?;
                let am_female_examiners = am_fem_result?;
                let am_examiners = am_result?;

                let temp_excess = fill_by_slot(
                    &session_id,
                    &pool,
                    am_examiners.len(),
                    am_female_examiners.len(),
                    circuits.len() * stations_not_rest.len(),
                    female_circuits.len() * stations_not_rest.len(),
                    true, // am
                    false // pm
                ).await?;
                if am_excess_exam == 0 {
                    am_excess_exam = temp_excess; 
                }
                else if temp_excess < am_excess_exam {
                    am_excess_exam = temp_excess;
                }
            }

            let mut pm_excess_exam: usize = 0;
            for pm_slot in &pm_only_slots {
                let (circuit_result, fem_circuit_result, pm_fem_result, pm_result) = tokio::join!(
                    Circuit::get_by_slot(&pool, &pm_slot.id),
                    Circuit::get_female_slot(&pool, &pm_slot.id),
                    Examiner::get_all_female_by_time(&pool, &session_id, RunTime::PM),
                    Examiner::get_all_by_time(&pool, &session_id, RunTime::PM)
                );
                let circuits = circuit_result?;
                let female_circuits = fem_circuit_result?;
                let pm_female_examiners = pm_fem_result?;
                let pm_examiners = pm_result?;

                let temp_excess = fill_by_slot(
                    &session_id,
                    &pool,
                    pm_examiners.len(),
                    pm_female_examiners.len(),
                    circuits.len() * stations_not_rest.len(),
                    female_circuits.len() * stations_not_rest.len(),
                    false, // am
                    true // pm
                ).await?;
                if pm_excess_exam == 0 {
                    pm_excess_exam = temp_excess; 
                }
                else if temp_excess < pm_excess_exam {
                    pm_excess_exam = temp_excess;
                }
            }

            let mut any_excess_exam: usize = 0;
            for any_slot in &any_slots { // for slots with runs in AM & PM, only examiners with full-day availability are taken account of
                let (circuit_result, fem_circuit_result, fem_result, any_result) = tokio::join!(
                    Circuit::get_by_slot(&pool, &any_slot.id),
                    Circuit::get_female_slot(&pool, &any_slot.id),
                    Examiner::get_female_ava_all(&pool, &session_id, Availability { am: true, pm: true }),
                    Examiner::get_ava_all(&pool, &session_id, Availability { am: true, pm: true })
                );
                let circuits = circuit_result?;
                let female_circuits = fem_circuit_result?;
                let any_female_examiners = fem_result?;
                let any_examiners = any_result?;

                let temp_excess = fill_by_slot(
                    &session_id,
                    &pool,
                    any_examiners.len(),
                    any_female_examiners.len(),
                    circuits.len() * stations_not_rest.len(),
                    female_circuits.len() * stations_not_rest.len(),
                    true, // am
                    true // pm
                ).await?;
                if any_excess_exam == 0 {
                    any_excess_exam = temp_excess; 
                }
                else if temp_excess < any_excess_exam {
                    any_excess_exam = temp_excess;
                }
            }

            // ALLOCATION
            let (am_examiner, pm_examiner, both_examiner, am_fem_examiner, pm_fem_examiner, both_fem_examiner) = tokio::join!(
                Examiner::get_all_by_time(&pool, &session_id, RunTime::AM),
                Examiner::get_all_by_time(&pool, &session_id, RunTime::PM),
                Examiner::get_ava_all(&pool, &session_id, Availability { am: true, pm: true }),
                Examiner::get_all_female_by_time(&pool, &session_id, RunTime::AM),
                Examiner::get_all_female_by_time(&pool, &session_id, RunTime::PM),
                Examiner::get_female_ava_all(&pool, &session_id, Availability { am: true, pm: true })
            );
            let total_am_examiners = am_examiner?;
            let total_pm_examiners = pm_examiner?;
            let total_both_examiners = both_examiner?;
            let total_am_fem_examiners = am_fem_examiner?;
            let total_pm_fem_examiners = pm_fem_examiner?;
            let total_both_fem_examiners = both_fem_examiner?;

            trace!("Total Examiners split: AM: {}, PM: {}, BOTH: {} ==> Total Female Examiners split: AM: {}, PM: {}, BOTH: {}",
                total_am_examiners.len(), total_pm_examiners.len(), total_both_examiners.len(), total_am_fem_examiners.len(), total_pm_fem_examiners.len(), total_both_fem_examiners.len());
            trace!("Excess Examiners: AM: {}, PM: {}, BOTH: {}", am_excess_exam, pm_excess_exam, any_excess_exam);
            // let total_can_am = can_am_female_split + can_am_split;
            // let total_can_pm = can_pm_female_split + can_pm_split;
            // let total_can_any = can_any_female_split + can_any_split;

            // let candidates_allocation = allocate_by_slot(&circuits, &stations, &candidates)?;

            // let examiners_allocations = allocate_by_slot(&circuits, &stations, &examiners)?;

            // Allocation::add_by_slot(&mut transaction, candidates_allocation, &batch_id, &cur_slot.id, &claim.id).await?;
            // Allocation::add_by_slot(&mut transaction, examiners_allocations, &batch_id, &cur_slot.id, &claim.id).await?;

            // transaction.commit().await.map_err(|e| AppError::from(anyhow!("Failed to commit transaction: {}", e)))?;
        },
    };

    Ok(StatusCode::OK.into_response())
}

impl Allocation {
    pub async fn add_by_slot(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        allocations: Vec<SlotAllocation>,
        batch_id: &Uuid,
        slot_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> { // for candidates ONLY
        sqlx::query!(
            r#"
            DELETE FROM records.allocations
            WHERE slot_id = $1
            "#,
            slot_id
        )
        .execute(&mut **tx)
        .await
        .with_context(|| "Failed to delete existing allocations")?;

        for allocation in &allocations {
            sqlx::query!(
                r#"
                INSERT INTO records.allocations (
                    slot_id, circuit_id, station_id, candidate_1, candidate_2
                ) VALUES ($1, $2, $3, $4, $5)
                "#,
                slot_id,
                allocation.circuit_id,
                allocation.station_id,
                allocation.candidate_1,
                allocation.candidate_2,
            )
            .execute(&mut **tx)
            .await
            .with_context(|| "Failed to insert allocation")?;

            sqlx::query!(
                r#"
                INSERT INTO records.allocations_history (
                    batch_id, slot_id, circuit_id, station_id, candidate_1, candidate_2, modified_by, auto_gen
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                batch_id,
                slot_id,
                allocation.circuit_id,
                allocation.station_id,
                allocation.candidate_1,
                allocation.candidate_2,
                user_id,
                true
            )
            .execute(&mut **tx)
            .await
            .with_context(|| "Failed to insert allocation history")?;
        }
        Ok(())
    }

    pub async fn add_by_time(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        allocations: Vec<TimeAllocation>,
        batch_id: &Uuid,
        slot_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> { // for examiners ONLY
        Ok(())
    }
}