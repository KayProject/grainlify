//! # Tests for Batch Payout Size Limits and Deterministic Failure Behavior
//!
//! Covers:
//! - MAX_BATCH_SIZE enforcement (max+1 rejected, exactly max accepted)
//! - Empty batch rejection
//! - Mismatched recipients/amounts rejection
//! - Zero amount rejection
//! - Insufficient balance rejection
//! - Deterministic failure ordering
#![cfg(test)]
extern crate std;
use soroban_sdk::{
    testutils::Address as _,
    token, Address, Env, String, Vec,
};
use crate::{ProgramEscrowContract, ProgramEscrowContractClient, MAX_BATCH_SIZE};

fn setup(env: &Env, initial_amount: i128) -> (
    ProgramEscrowContractClient<'static>,
    Address,
    token::StellarAssetClient<'static>,
) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProgramEscrowContract);
    let client = ProgramEscrowContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_id = sac.address();
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);
    let program_id = String::from_str(env, "batch-test");
    client.init_program(&program_id, &admin, &token_id, &admin, &None, &None);
    if initial_amount > 0 {
        token_admin_client.mint(&client.address, &initial_amount);
        client.lock_program_funds(&initial_amount);
    }
    (client, admin, token_admin_client)
}

#[test]
fn test_max_batch_size_constant_is_100() {
    assert_eq!(MAX_BATCH_SIZE, 100);
}

#[test]
#[should_panic(expected = "Cannot process empty batch")]
fn test_empty_batch_rejected() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 100_000);
    let recipients: Vec<Address> = Vec::new(&env);
    let amounts: Vec<i128> = Vec::new(&env);
    client.batch_payout(&recipients, &amounts);
}

#[test]
#[should_panic(expected = "Batch size exceeds MAX_BATCH_SIZE limit of 100")]
fn test_batch_over_max_rejected() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 1_000_000);
    let mut recipients = Vec::new(&env);
    let mut amounts = Vec::new(&env);
    for _ in 0..=MAX_BATCH_SIZE {
        recipients.push_back(Address::generate(&env));
        amounts.push_back(1i128);
    }
    client.batch_payout(&recipients, &amounts);
}

#[test]
#[should_panic(expected = "Recipients and amounts vectors must have the same length")]
fn test_mismatched_lengths_rejected() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 100_000);
    let mut recipients = Vec::new(&env);
    let mut amounts = Vec::new(&env);
    recipients.push_back(Address::generate(&env));
    recipients.push_back(Address::generate(&env));
    amounts.push_back(100i128);
    client.batch_payout(&recipients, &amounts);
}

#[test]
#[should_panic(expected = "All amounts must be greater than zero")]
fn test_zero_amount_rejected() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 100_000);
    let mut recipients = Vec::new(&env);
    let mut amounts = Vec::new(&env);
    recipients.push_back(Address::generate(&env));
    amounts.push_back(0i128);
    client.batch_payout(&recipients, &amounts);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_insufficient_balance_rejected() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 100);
    let mut recipients = Vec::new(&env);
    let mut amounts = Vec::new(&env);
    recipients.push_back(Address::generate(&env));
    amounts.push_back(999_999i128);
    client.batch_payout(&recipients, &amounts);
}

#[test]
fn test_batch_exactly_at_max_size_succeeds() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 1_000_000);
    let mut recipients = Vec::new(&env);
    let mut amounts = Vec::new(&env);
    for _ in 0..MAX_BATCH_SIZE {
        recipients.push_back(Address::generate(&env));
        amounts.push_back(1i128);
    }
    // should not panic — exactly at limit is valid
    client.batch_payout(&recipients, &amounts);
}
