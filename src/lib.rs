#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Address, Env, token};

#[contracttype]
#[derive(Clone)]
pub struct Grant {
    pub admin: Address,
    pub grantee: Address,
    pub flow_rate: i128,
    pub balance: i128,
    pub last_claim_time: u64,
    pub is_paused: bool,
    pub token: Address,
}

#[contracttype]
pub enum DataKey {
    Grant(u64),
    Count,
}

#[contract]
pub struct GrantContract;

#[contractimpl]
impl GrantContract {
    pub fn create_grant(env: Env, admin: Address, grantee: Address, deposit: i128, flow_rate: i128, token: Address) -> u64 {
        admin.require_auth();
        let mut count: u64 = env.storage().instance().get(&DataKey::Count).unwrap_or(0);
        count += 1;
        let client = token::Client::new(&env, &token);
        client.transfer(&admin, &env.current_contract_address(), &deposit);
        let grant = Grant { admin, grantee, flow_rate, balance: deposit, last_claim_time: env.ledger().timestamp(), is_paused: false, token };
        env.storage().instance().set(&DataKey::Grant(count), &grant);
        env.storage().instance().set(&DataKey::Count, &count);
        count
    }

    pub fn withdraw(env: Env, grant_id: u64) {
        let mut grant: Grant = env.storage().instance().get(&DataKey::Grant(grant_id)).unwrap();
        grant.grantee.require_auth();
        if grant.is_paused { panic!("Grant PAUSED by admin"); }
        let current_time = env.ledger().timestamp();
        let seconds_passed = current_time - grant.last_claim_time;
        let amount_due = grant.flow_rate * seconds_passed as i128;
        let payout = if grant.balance >= amount_due { amount_due } else { grant.balance };
        if payout > 0 {
            let client = token::Client::new(&env, &grant.token);
            client.transfer(&env.current_contract_address(), &grant.grantee, &payout);
            grant.balance -= payout;
            grant.last_claim_time = current_time;
            env.storage().instance().set(&DataKey::Grant(grant_id), &grant);
        }
    }

    pub fn set_pause(env: Env, grant_id: u64, pause_state: bool) {
        let mut grant: Grant = env.storage().instance().get(&DataKey::Grant(grant_id)).unwrap();
        grant.admin.require_auth();
        grant.is_paused = pause_state;
        env.storage().instance().set(&DataKey::Grant(grant_id), &grant);
    }
}
