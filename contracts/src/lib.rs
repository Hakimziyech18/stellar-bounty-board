#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token::Client as TokenClient, Address, Env,
    String,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BountyStatus {
    Open,
    Reserved,
    Submitted,
    Released,
    Refunded,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounty {
    pub maintainer: Address,
    pub contributor: Option<Address>,
    pub token: Address,
    pub amount: i128,
    pub repo: String,
    pub issue_number: u32,
    pub title: String,
    pub deadline: u64,
    pub status: BountyStatus,
}

#[contracttype]
enum DataKey {
    NextBountyId,
    Bounty(u64),
    Config,
    PendingResolution(u64),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountyCreated {
    pub bounty_id: u64,
    pub maintainer: Address,
    pub amount: i128,
    pub repo: String,
    pub issue_number: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountyReserved {
    pub bounty_id: u64,
    pub contributor: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountySubmitted {
    pub bounty_id: u64,
    pub contributor: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountyReleased {
    pub bounty_id: u64,
    pub contributor: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountyRefunded {
    pub bounty_id: u64,
    pub maintainer: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub appeal_window: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisputeDecision {
    Release,
    Refund,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingResolution {
    pub decision: DisputeDecision,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisputeResolutionScheduled {
    pub bounty_id: u64,
    pub decision: DisputeDecision,
    pub resolve_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisputeAppealed {
    pub bounty_id: u64,
}

#[contract]
pub struct StellarBountyBoardContract;

#[contractimpl]
impl StellarBountyBoardContract {
    pub fn create_bounty(
        env: Env,
        maintainer: Address,
        token: Address,
        amount: i128,
        repo: String,
        issue_number: u32,
        title: String,
        deadline: u64,
    ) -> u64 {
        maintainer.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }
        if deadline <= env.ledger().timestamp() {
            panic!("deadline must be in the future");
        }

        let token_client = TokenClient::new(&env, &token);
        let contract_address = env.current_contract_address();
        token_client.transfer(&maintainer, &contract_address, &amount);

        let mut next_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextBountyId)
            .unwrap_or(0);
        next_id += 1;

        let bounty = Bounty {
            maintainer: maintainer.clone(),
            contributor: None,
            token,
            amount,
            repo: repo.clone(),
            issue_number,
            title,
            deadline,
            status: BountyStatus::Open,
        };

        env.storage()
            .persistent()
            .set(&DataKey::NextBountyId, &next_id);
        env.storage()
            .persistent()
            .set(&DataKey::Bounty(next_id), &bounty);

        env.events().publish(
            (symbol_short!("Bounty"), symbol_short!("Create")),
            BountyCreated {
                bounty_id: next_id,
                maintainer,
                amount,
                repo,
                issue_number,
            },
        );

        next_id
    }

    pub fn reserve_bounty(env: Env, bounty_id: u64, contributor: Address) {
        contributor.require_auth();
        let mut bounty = read_bounty(&env, bounty_id);
        expire_if_needed(&env, &mut bounty);

        if bounty.status != BountyStatus::Open {
            panic!("bounty is not open");
        }

        bounty.contributor = Some(contributor.clone());
        bounty.status = BountyStatus::Reserved;
        write_bounty(&env, bounty_id, &bounty);

        env.events().publish(
            (symbol_short!("Bounty"), symbol_short!("Reserv")),
            BountyReserved {
                bounty_id,
                contributor,
            },
        );
    }

    pub fn submit_bounty(env: Env, bounty_id: u64, contributor: Address) {
        contributor.require_auth();
        let mut bounty = read_bounty(&env, bounty_id);
        expire_if_needed(&env, &mut bounty);

        if bounty.status != BountyStatus::Reserved {
            panic!("bounty must be reserved");
        }
        if bounty.contributor != Some(contributor.clone()) {
            panic!("contributor mismatch");
        }

        bounty.status = BountyStatus::Submitted;
        write_bounty(&env, bounty_id, &bounty);

        env.events().publish(
            (symbol_short!("Bounty"), symbol_short!("Submit")),
            BountySubmitted {
                bounty_id,
                contributor,
            },
        );
    }

    pub fn release_bounty(env: Env, bounty_id: u64, maintainer: Address) {
        maintainer.require_auth();
        let mut bounty = read_bounty(&env, bounty_id);

        if bounty.maintainer != maintainer {
            panic!("maintainer mismatch");
        }
        if bounty.status != BountyStatus::Submitted {
            panic!("bounty must be submitted");
        }

        let contributor = bounty
            .contributor
            .clone()
            .unwrap_or_else(|| panic!("missing contributor"));
        let token_client = TokenClient::new(&env, &bounty.token);
        let contract_address = env.current_contract_address();
        token_client.transfer(&contract_address, &contributor, &bounty.amount);

        bounty.status = BountyStatus::Released;
        write_bounty(&env, bounty_id, &bounty);

        env.events().publish(
            (symbol_short!("Bounty"), symbol_short!("Releas")),
            BountyReleased {
                bounty_id,
                contributor,
                amount: bounty.amount,
            },
        );
    }

    pub fn refund_bounty(env: Env, bounty_id: u64, maintainer: Address) {
        maintainer.require_auth();
        let mut bounty = read_bounty(&env, bounty_id);
        
        if bounty.maintainer != maintainer {
            panic!("maintainer mismatch");
        }

        // Finalized bounties cannot be refunded
        if bounty.status == BountyStatus::Released || bounty.status == BountyStatus::Refunded {
            panic!("bounty already finalized");
        }

        // Active/Submitted bounties can ONLY be refunded if expired
        let now = env.ledger().timestamp();
        if now <= bounty.deadline && bounty.deadline != 0 {
            panic!("bounty not expired yet");
        }

        let token_client = TokenClient::new(&env, &bounty.token);
        let contract_address = env.current_contract_address();
        token_client.transfer(&contract_address, &maintainer, &bounty.amount);

        bounty.status = BountyStatus::Refunded;
        write_bounty(&env, bounty_id, &bounty);

        env.events().publish(
            (symbol_short!("Bounty"), symbol_short!("Refund")),
            BountyRefunded {
                bounty_id,
                maintainer,
                amount: bounty.amount,
            },
        );
    }

    pub fn get_bounty(env: Env, bounty_id: u64) -> Bounty {
        let mut bounty = read_bounty(&env, bounty_id);
        expire_if_needed(&env, &mut bounty);
        bounty
    }

    // ---------- New Functions ----------
    pub fn init(env: Env, appeal_window: u64) {
        // Only allow setting once
        if env.storage().persistent().has(&DataKey::Config) {
            panic!("config already set");
        }
        let cfg = Config { appeal_window };
        env.storage().persistent().set(&DataKey::Config, &cfg);
    }

    pub fn resolve_dispute(env: Env, bounty_id: u64, decision_u8: u8) {
        // For simplicity, any caller can resolve; in production enforce arbiter auth.
        let decision = match decision_u8 {
            0 => DisputeDecision::Release,
            1 => DisputeDecision::Refund,
            _ => panic!("invalid decision"),
        };
        let timestamp = env.ledger().timestamp();
        let pending = PendingResolution { decision, timestamp };
        env.storage()
            .persistent()
            .set(&DataKey::PendingResolution(bounty_id), &pending);
        env.events().publish(
            (symbol_short!("Dispute"), symbol_short!("Scheduled")),
            DisputeResolutionScheduled {
                bounty_id,
                decision,
                resolve_at: timestamp,
            },
        );
    }

    pub fn finalize_resolution(env: Env, bounty_id: u64) {
        // Load pending
        let pending_opt: Option<PendingResolution> = env
            .storage()
            .persistent()
            .get(&DataKey::PendingResolution(bounty_id));
        let pending = pending_opt.expect("no pending resolution");
        // Load config
        let cfg: Config = env.storage().persistent().get(&DataKey::Config).expect("config not set");
        let now = env.ledger().timestamp();
        if now < pending.timestamp + cfg.appeal_window {
            panic!("appeal window not elapsed");
        }
        // Load bounty
        let mut bounty = read_bounty(&env, bounty_id);
        // Resolve based on decision
        match pending.decision {
            DisputeDecision::Release => {
                // transfer to contributor
                let contributor = bounty
                    .contributor
                    .clone()
                    .unwrap_or_else(|| panic!("missing contributor"));
                let token_client = TokenClient::new(&env, &bounty.token);
                token_client.transfer(&env.current_contract_address(), &contributor, &bounty.amount);
                bounty.status = BountyStatus::Released;
                write_bounty(&env, bounty_id, &bounty);
                env.events().publish(
                    (symbol_short!("Bounty"), symbol_short!("Releas")),
                    BountyReleased {
                        bounty_id,
                        contributor,
                        amount: bounty.amount,
                    },
                );
            }
            DisputeDecision::Refund => {
                let maintainer = bounty.maintainer.clone();
                let token_client = TokenClient::new(&env, &bounty.token);
                token_client.transfer(&env.current_contract_address(), &maintainer, &bounty.amount);
                bounty.status = BountyStatus::Refunded;
                write_bounty(&env, bounty_id, &bounty);
                env.events().publish(
                    (symbol_short!("Bounty"), symbol_short!("Refund")),
                    BountyRefunded {
                        bounty_id,
                        maintainer,
                        amount: bounty.amount,
                    },
                );
            }
        }
        // Remove pending
        env.storage().persistent().remove(&DataKey::PendingResolution(bounty_id));
    }

    pub fn appeal(env: Env, bounty_id: u64) {
        let pending_opt: Option<PendingResolution> = env
            .storage()
            .persistent()
            .get(&DataKey::PendingResolution(bounty_id));
        let pending = pending_opt.expect("no pending resolution");
        let cfg: Config = env.storage().persistent().get(&DataKey::Config).expect("config not set");
        let now = env.ledger().timestamp();
        if now >= pending.timestamp + cfg.appeal_window {
            panic!("appeal window elapsed");
        }
        // Verify caller is losing party
        let bounty = read_bounty(&env, bounty_id);
        match pending.decision {
            DisputeDecision::Release => {
                // loser is maintainer
                bounty.maintainer.require_auth();
            }
            DisputeDecision::Refund => {
                // loser is contributor
                if let Some(contrib) = bounty.contributor.clone() {
                    contrib.require_auth();
                } else {
                    panic!("no contributor to appeal");
                }
            }
        }
        // Remove pending to block finalization until re-resolved
        env.storage().persistent().remove(&DataKey::PendingResolution(bounty_id));
        env.events().publish(
            (symbol_short!("Dispute"), symbol_short!("Appealed")),
            DisputeAppealed { bounty_id },
        );
    }

    pub fn get_next_bounty_id(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::NextBountyId)
            .unwrap_or(0)
    }
}

fn read_bounty(env: &Env, bounty_id: u64) -> Bounty {
    env.storage()
        .persistent()
        .get(&DataKey::Bounty(bounty_id))
        .unwrap_or_else(|| panic!("bounty not found"))
}

fn write_bounty(env: &Env, bounty_id: u64, bounty: &Bounty) {
    env.storage().persistent().set(&DataKey::Bounty(bounty_id), bounty);
}

fn expire_if_needed(env: &Env, bounty: &mut Bounty) {
    let now = env.ledger().timestamp();
    if now > bounty.deadline
        && (bounty.status == BountyStatus::Open || bounty.status == BountyStatus::Reserved)
    {
        bounty.status = BountyStatus::Expired;
    }
}

