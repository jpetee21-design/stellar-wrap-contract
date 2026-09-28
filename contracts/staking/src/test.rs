//! Tests for the staking contract.
//!
//! Paused behaviour (issue #874):
//! - `stake` is blocked while paused: opening new positions during a pause is
//!   defensible because it is a new commitment, not a user's existing funds.
//! - `unstake` is blocked while paused: it mutates the active position and
//!   starts the unbonding clock, so it is treated like `stake`.
//! - `withdraw_stake` is allowed while paused: the stake is already unbonded
//!   and the user is only reclaiming their own funds. Blocking it would trap
//!   user funds for the duration of the pause, which is not justifiable.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, StakingContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StakingContract);
    let client = StakingContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn stake_is_blocked_while_paused() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.pause(&admin);

    let result = client.try_stake(&user, &1_000);
    assert_eq!(
        result,
        Err(Ok(ContractError::ContractPaused)),
        "stake must be blocked while paused"
    );
}

#[test]
fn unstake_is_blocked_while_paused() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.stake(&user, &1_000);
    client.pause(&admin);

    let result = client.try_unstake(&user, &1_000);
    assert_eq!(
        result,
        Err(Ok(ContractError::ContractPaused)),
        "unstake must be blocked while paused"
    );
}

#[test]
fn withdraw_stake_is_allowed_while_paused() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.stake(&user, &1_000);
    client.unstake(&user, &1_000);
    client.pause(&admin);

    // Already-unbonded stake must remain withdrawable during a pause so user
    // funds are not trapped for the duration of the pause.
    client.withdraw_stake(&user);

    assert_eq!(client.get_stake(&user), 0);
}
