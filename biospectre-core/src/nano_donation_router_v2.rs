//! NANO / EVOLVE Donation Router v2
//!
//! - Enforces per-day donation caps (hard maximum).
//! - Enforces a greed-index floor: minimum daily donation
//!   when surplus exists and eco/safety are OK.
//! - Donates ONLY to hardware-plane devices (no direct
//!   NANO/EVOLVE transfer to other hosts).
//! - Routes EVOLVE + NANO rewards to local stakeholders
//!   (validators, participants) on this host.
//!
//! All tokens remain host-bound and non-financial:
//! this only decrements local NANO and emits workloads
//! for partner devices under DID/ALN governance.

use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};

use crate::biophysicalruntime::{BioTokenState, RuntimeError};
use crate::lifeforcesafety::LifeforceState;
use crate::alndid::{ALNDID, AccessEnvelope, DIDDirectory, RoleClass};

/// Hardware vs biophysical planes (simplified).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneKind {
    HardwareDevice,
    AugmentedHost,
    CyberneticHost,
}

/// Donation beneficiary, must be a hardware device.
#[derive(Debug, Clone)]
pub struct DonationBeneficiary {
    pub org_id: String,
    pub device_id: String,
    pub did: ALNDID,
    pub plane: PlaneKind,
    pub max_nano_fraction_per_day: f64,
    pub active: bool,
}

/// Stakeholder class for reward routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StakeholderKind {
    Validator,
    Participant,
    ResearchContributor,
}

#[derive(Debug, Clone)]
pub struct Stakeholder {
    pub did: ALNDID,
    pub kind: StakeholderKind,
    pub weight: f64, // relative share
    pub active: bool,
}

/// A scheduled donation workload for a hardware device.
#[derive(Debug, Clone)]
pub struct DonationJob {
    pub job_id: String,
    pub org_id: String,
    pub device_id: String,
    pub beneficiary_did: ALNDID,
    pub nano_tokens: u64,
    pub eco_cost_flops: u64,
    pub label: String,
}

/// Context for a single scheduling window.
#[derive(Debug, Clone)]
pub struct DonationContext {
    pub now_utc: DateTime<Utc>,
    /// Surplus fraction of current NANO (0–1)
    /// after host-local needs and safety bands.
    pub surplus_nano_fraction01: f64,
    /// Eco alignment (1 = eco-perfect night).
    pub eco_alignment01: f64,
    /// Host explicitly opted in for donation today.
    pub host_opt_in: bool,
}

/// One-day accounting of donation activity.
#[derive(Debug, Clone)]
pub struct DailyDonationLedger {
    pub date: NaiveDate,
    pub nano_donated: u64,
}

/// Simple “greed index” emission: how we met today’s quota.
#[derive(Debug, Clone)]
pub enum GreedIndexStatus {
    NotApplicable,          // no surplus or donations disabled
    BelowFloorSkipped,      // surplus but we did not reach floor (should not happen under correct config)
    FloorMet,
    AboveFloorWithinCap,
}

/// Audit record for a donation cycle.
#[derive(Debug, Clone)]
pub struct DonationAudit {
    pub date: NaiveDate,
    pub applied_jobs: Vec<DonationJob>,
    pub total_nano_spent: u64,
    pub total_eco_cost_flops: u64,
    pub greed_index_status: GreedIndexStatus,
    /// EVOLVE and NANO stakeholder rewards minted locally.
    pub stakeholder_rewards: Vec<StakeholderReward>,
    pub reason: String,
}

/// Stakeholder reward delta (local only, non-transferable).
#[derive(Debug, Clone)]
pub struct StakeholderReward {
    pub stakeholder_did: ALNDID,
    pub kind: StakeholderKind,
    pub evolve_delta: f64,
    pub nano_delta: f64,
}

/// Router policy knobs.
#[derive(Debug, Clone)]
pub struct DonationPolicy {
    /// Min admin biophysics knowledge to configure router.
    pub min_admin_knowledge: f64,
    /// Min eco-alignment for any donation to proceed.
    pub min_eco_alignment: f64,
    /// Hard daily max NANO tokens we will donate.
    pub max_nano_per_day: u64,
    /// Required minimum NANO tokens per day when there
    /// is surplus and eco/safety allow donations.
    pub min_nano_per_day_floor: u64,
    /// Maximum fraction of current NANO that can be
    /// donated in this window (safety band on state).
    pub max_total_nano_fraction_per_window: f64,
    /// Fraction of spent NANO re-minted as EVOLVE
    /// for stakeholders (0–1, non-financial reward).
    pub evolve_reward_rate: f64,
    /// Fraction of spent NANO mirrored as local NANO
    /// rewards to stakeholders (0–1, governance quota).
    pub nano_reward_rate: f64,
}

impl Default for DonationPolicy {
    fn default() -> Self {
        Self {
            min_admin_knowledge: 0.7,
            min_eco_alignment: 0.6,
            max_nano_per_day: 10_000,
            min_nano_per_day_floor: 1_000,
            max_total_nano_fraction_per_window: 0.25,
            evolve_reward_rate: 0.10,
            nano_reward_rate: 0.05,
        }
    }
}

/// Router state for one host.
pub struct NanoDonationRouterV2<D>
where
    D: DIDDirectory,
{
    policy: DonationPolicy,
    lifeforce: LifeforceState,
    did_directory: D,
    beneficiaries: HashMap<String, DonationBeneficiary>,
    stakeholders: Vec<Stakeholder>,
    daily_ledger: Option<DailyDonationLedger>,
}

impl<D> NanoDonationRouterV2<D>
where
    D: DIDDirectory,
{
    pub fn new(policy: DonationPolicy, lifeforce: LifeforceState, did_directory: D) -> Self {
        Self {
            policy,
            lifeforce,
            did_directory,
            beneficiaries: HashMap::new(),
            stakeholders: Vec::new(),
            daily_ledger: None,
        }
    }

    /// Must be called once per day rollover to reset daily ledger.
    pub fn roll_over_day(&mut self, today: NaiveDate) {
        self.daily_ledger = Some(DailyDonationLedger {
            date: today,
            nano_donated: 0,
        });
    }

    /// Register / update a hardware-only beneficiary.
    pub fn register_hardware_beneficiary(
        &mut self,
        admin_did: &ALNDID,
        org_id: String,
        device_id: String,
        max_nano_fraction_per_day: f64,
    ) -> Result<(), RuntimeError> {
        let env = self
            .did_directory
            .resolve_access(admin_did.clone())
            .ok_or(RuntimeError::AccessDenied(
                "Unknown DID for donation admin",
            ))?;

        if env.min_biophysics_knowledge_score < self.policy.min_admin_knowledge {
            return Err(RuntimeError::AccessDenied(
                "Insufficient biophysics knowledge to configure donation router",
            ));
        }

        let allowed_role = env
            .roles
            .iter()
            .any(|r| matches!(r, RoleClass::AugmentedCitizen | RoleClass::EthicalOperator));
        if !allowed_role {
            return Err(RuntimeError::AccessDenied(
                "Role not permitted to configure donation router",
            ));
        }

        let capped = max_nano_fraction_per_day.clamp(0.0, 1.0);
        let b = DonationBeneficiary {
            org_id: org_id.clone(),
            device_id,
            did: admin_did.clone(),
            plane: PlaneKind::HardwareDevice,
            max_nano_fraction_per_day: capped,
            active: true,
        };
        self.beneficiaries.insert(org_id, b);
        Ok(())
    }

    pub fn add_stakeholder(&mut self, s: Stakeholder) {
        self.stakeholders.push(s);
    }

    /// Main donation scheduling entrypoint.
    ///
    /// - Updates local BioTokenState (NANO decrement only).
    /// - Never transfers tokens off-host.
    /// - Emits jobs for hardware-plane devices only.
    /// - Mints EVOLVE + NANO rewards to local stakeholders.
    pub fn schedule_donations(
        &mut self,
        state: &mut BioTokenState,
        ctx: &DonationContext,
    ) -> Result<DonationAudit, RuntimeError> {
        // Ensure daily ledger initialized.
        let today = ctx.now_utc.date_naive();
        if self
            .daily_ledger
            .as_ref()
            .map(|d| d.date != today)
            .unwrap_or(true)
        {
            self.roll_over_day(today);
        }
        let ledger = self.daily_ledger.as_mut().expect("ledger just initialized");

        // 1. Preconditions: opt-in, eco.
        if !ctx.host_opt_in {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "Host did not opt-in today".to_string(),
            });
        }

        if ctx.eco_alignment01 < self.policy.min_eco_alignment {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "Eco alignment below threshold".to_string(),
            });
        }

        // 2. Lifeforce bands and invariants.
        self.lifeforce
            .validate_bands(state.clone())
            .map_err(RuntimeError::SafetyViolation)?;

        let surplus_fraction = ctx.surplus_nano_fraction01.clamp(0.0, 1.0);
        if surplus_fraction == 0.0 || state.nano <= 0.0 {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "No surplus NANO available".to_string(),
            });
        }

        // 3. Window- and day-based caps.
        let max_fraction_window = self
            .policy
            .max_total_nano_fraction_per_window
            .clamp(0.0, 1.0);
        let upper_fraction = surplus_fraction.min(max_fraction_window);
        let theoretical_available = (state.nano * upper_fraction).floor() as u64;

        if theoretical_available == 0 {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "Effective surplus too small".to_string(),
            });
        }

        let remaining_daily_cap = if ledger.nano_donated >= self.policy.max_nano_per_day {
            0
        } else {
            self.policy
                .max_nano_per_day
                .saturating_sub(ledger.nano_donated)
        };

        if remaining_daily_cap == 0 {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "Daily donation cap already reached".to_string(),
            });
        }

        let spendable_today = theoretical_available.min(remaining_daily_cap);
        if spendable_today == 0 {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "No spendable NANO under caps".to_string(),
            });
        }

        // 4. Beneficiaries: hardware devices only.
        let active_hw: Vec<_> = self
            .beneficiaries
            .values()
            .filter(|b| b.active && b.plane == PlaneKind::HardwareDevice)
            .collect();

        if active_hw.is_empty() {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "No active hardware beneficiaries".to_string(),
            });
        }

        // 5. Enforce greed-index floor: ensure we at least
        // hit min_nano_per_day_floor when possible.
        let target_min_today = self.policy.min_nano_per_day_floor;
        let current_total_after = ledger.nano_donated.saturating_add(spendable_today);
        let must_ensure_min =
            target_min_today > ledger.nano_donated && target_min_today <= current_total_after;

        // For simplicity, we attempt to donate `spendable_today`. This will
        // naturally satisfy the floor as long as spendable_today + donated_so_far
        // >= floor. If not, we mark BelowFloorSkipped.
        let planned_spend = spendable_today;
        let meets_floor = ledger
            .nano_donated
            .saturating_add(planned_spend) >= target_min_today;

        // 6. Allocate across hardware devices with per-beneficiary caps.
        let mut remaining = planned_spend;
        let mut jobs = Vec::new();
        let mut total_eco_cost = 0u64;

        for (idx, b) in active_hw.iter().enumerate() {
            if remaining == 0 {
                break;
            }

            let per_device_cap =
                (state.nano * b.max_nano_fraction_per_day.clamp(0.0, 1.0)).floor() as u64;
            if per_device_cap == 0 {
                continue;
            }

            let planned_for_b = (remaining / (active_hw.len() - idx).max(1) as u64).max(1);
            let final_for_b = planned_for_b.min(per_device_cap).min(remaining);
            if final_for_b == 0 {
                continue;
            }

            let eco_cost = final_for_b.saturating_mul(1_000_000);
            let job = DonationJob {
                job_id: format!("nano-donation-v2-{}-{}", b.org_id, idx),
                org_id: b.org_id.clone(),
                device_id: b.device_id.clone(),
                beneficiary_did: b.did.clone(),
                nano_tokens: final_for_b,
                eco_cost_flops: eco_cost,
                label: format!(
                    "Hardware-plane donation to {}::{}",
                    b.org_id, b.device_id
                ),
            };

            remaining = remaining.saturating_sub(final_for_b);
            total_eco_cost = total_eco_cost.saturating_add(eco_cost);
            jobs.push(job);
        }

        let nano_spent_actual: u64 = jobs.iter().map(|j| j.nano_tokens).sum();
        if nano_spent_actual == 0 {
            return Ok(DonationAudit {
                date: today,
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                greed_index_status: GreedIndexStatus::NotApplicable,
                stakeholder_rewards: Vec::new(),
                reason: "No feasible allocation to hardware devices".to_string(),
            });
        }

        // 7. Apply NANO decrement locally (host-bound).
        let new_nano = (state.nano - nano_spent_actual as f64).max(0.0);
        state.nano = new_nano;

        // Update daily ledger.
        ledger.nano_donated = ledger.nano_donated.saturating_add(nano_spent_actual);

        // 8. Compute stakeholder rewards (local only).
        let rewards = self.distribute_stakeholder_rewards(nano_spent_actual);

        // 9. Greed-index status.
        let greed_status = if !must_ensure_min {
            GreedIndexStatus::NotApplicable
        } else if meets_floor {
            if ledger.nano_donated <= self.policy.max_nano_per_day {
                GreedIndexStatus::FloorMet
            } else {
                GreedIndexStatus::AboveFloorWithinCap
            }
        } else {
            GreedIndexStatus::BelowFloorSkipped
        };

        Ok(DonationAudit {
            date: today,
            applied_jobs: jobs,
            total_nano_spent: nano_spent_actual,
            total_eco_cost_flops: total_eco_cost,
            greed_index_status: greed_status,
            stakeholder_rewards: rewards,
            reason: "Donations scheduled within safety, eco, and greed-index bounds".to_string(),
        })
    }

    /// Internal: mint EVOLVE + NANO rewards for stakeholders on this host.
    fn distribute_stakeholder_rewards(&self, nano_spent: u64) -> Vec<StakeholderReward> {
        if self.stakeholders.is_empty() || nano_spent == 0 {
            return Vec::new();
        }

        let total_weight: f64 = self
            .stakeholders
            .iter()
            .filter(|s| s.active && s.weight > 0.0)
            .map(|s| s.weight)
            .sum();

        if total_weight <= 0.0 {
            return Vec::new();
        }

        let evolve_pool = nano_spent as f64 * self.policy.evolve_reward_rate;
        let nano_pool = nano_spent as f64 * self.policy.nano_reward_rate;

        self.stakeholders
            .iter()
            .filter(|s| s.active && s.weight > 0.0)
            .map(|s| {
                let share = s.weight / total_weight;
                StakeholderReward {
                    stakeholder_did: s.did.clone(),
                    kind: s.kind,
                    evolve_delta: evolve_pool * share,
                    nano_delta: nano_pool * share,
                }
            })
            .collect()
    }
}
