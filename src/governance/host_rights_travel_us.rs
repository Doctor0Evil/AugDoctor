use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostRightsTravelUsProfile {
    pub host_id: String,
    pub profile_id: String,
    pub host_bound: bool,
    pub defi_bridge: bool,
    pub stake_weighted: bool,
    pub marketplace: bool,
    pub tokens_as_capacity_only: bool,
    pub cross_host_transfer_allowed: bool,
    pub external_freeze_or_throttle_allowed: bool,

    pub ai_platforms_may_execute: bool,
    pub courts_may_execute: bool,
    pub hospitals_may_execute: bool,
    pub law_enforcement_may_execute: bool,

    pub may_gate_neural_functionality: bool,
    pub may_require_subscription_for_core_access: bool,
    pub may_downgrade_augmentation_for_nonpayment: bool,
    pub may_force_evolution_reversal: bool,
    pub may_reduce_capability_for_punishment: bool,
    pub may_alter_lifeforce_bands_without_consent: bool,

    pub require_demonstrated_consent: bool,
    pub irreversible_change_requires_token: bool,

    pub forbid_external_host_routing: bool,
    pub forbid_shared_microspaces: bool,
    pub require_self_only_flag: bool,
    pub require_hostid_match: bool,
    pub require_no_thirdparty_negative_energy: bool,

    pub max_daily_turns: u8,
    pub allow_burst: bool,
    pub on_budget_exhaustion_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostRightsStatus {
    RightsSafe,
    ViolatesInvariant(Vec<String>),
}

impl HostRightsTravelUsProfile {
    pub fn verify_rights_safe(&self, expected_host_id: &str) -> HostRightsStatus {
        let mut errors = Vec::new();

        // 1. Must be your host and host-bound, non-financial, non-seizable
        if self.host_id != expected_host_id {
            errors.push("host-id must equal running host DID; no external owner".into());
        }
        if !self.host_bound {
            errors.push("host-bound must be true".into());
        }
        if self.defi_bridge {
            errors.push("defi-bridge must be false".into());
        }
        if self.stake_weighted {
            errors.push("stake-weighted must be false".into());
        }
        if self.marketplace {
            errors.push("marketplace must be false".into());
        }
        if !self.tokens_as_capacity_only {
            errors.push("tokens-as-capacity-only must be true".into());
        }
        if self.cross_host_transfer_allowed {
            errors.push("cross-host-transfer-allowed must be false".into());
        }
        if self.external_freeze_or_throttle_allowed {
            errors.push("external-freeze-or-throttle-allowed must be false".into());
        }

        // 2. No external actor may execute mutations
        if self.ai_platforms_may_execute {
            errors.push("ai-platforms-may-execute must be false (propose-only).".into());
        }
        if self.courts_may_execute {
            errors.push("courts-may-execute must be false.".into());
        }
        if self.hospitals_may_execute {
            errors.push("hospitals-may-execute must be false.".into());
        }
        if self.law_enforcement_may_execute {
            errors.push("law-enforcement-may-execute must be false.".into());
        }

        // 3. No gating / punishment of neural function
        if self.may_gate_neural_functionality {
            errors.push("may-gate-neural-functionality must be false.".into());
        }
        if self.may_require_subscription_for_core_access {
            errors.push("may-require-subscription-for-core-access must be false.".into());
        }
        if self.may_downgrade_augmentation_for_nonpayment {
            errors.push("may-downgrade-augmentation-for-nonpayment must be false.".into());
        }
        if self.may_force_evolution_reversal {
            errors.push("may-force-evolution-reversal must be false.".into());
        }
        if self.may_reduce_capability_for_punishment {
            errors.push("may-reduce-capability-for-punishment must be false.".into());
        }
        if self.may_alter_lifeforce_bands_without_consent {
            errors.push("may-alter-lifeforce-bands-without-consent must be false.".into());
        }

        // 4. Consent invariants
        if !self.require_demonstrated_consent {
            errors.push("require-demonstrated-consent must be true.".into());
        }
        if !self.irreversible_change_requires_token {
            errors.push("irreversible-change-requires-token must be true.".into());
        }

        // 5. Microspace sovereignty invariants
        if !self.forbid_external_host_routing {
            errors.push("forbid-external-host-routing must be true.".into());
        }
        if !self.forbid_shared_microspaces {
            errors.push("forbid-shared-microspaces must be true.".into());
        }
        if !self.require_self_only_flag {
            errors.push("require-self-only-flag must be true.".into());
        }
        if !self.require_hostid_match {
            errors.push("require-hostid-match must be true.".into());
        }
        if !self.require_no_thirdparty_negative_energy {
            errors.push("requirenothirdpartynegativeenergy must be true.".into());
        }

        // 6. Evolution pace bounds (sane daily turns)
        if self.max_daily_turns == 0 || self.max_daily_turns > 10 {
            errors.push("max-daily-turns must be in [1,10].".into());
        }
        if self.allow_burst {
            errors.push("allow-burst must be false.".into());
        }
        if self.on_budget_exhaustion_mode != "log-only-no-new-evolution" {
            errors.push("on-budget-exhaustion-mode must be 'log-only-no-new-evolution'.".into());
        }

        if errors.is_empty() {
            HostRightsStatus::RightsSafe
        } else {
            HostRightsStatus::ViolatesInvariant(errors)
        }
    }
}
