#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};
pub mod asset;
#[contract]
pub struct GrainlifyContract;
#[contractimpl]
impl GrainlifyContract {
    pub fn init_admin(_e: Env, _admin: Address) {}
    pub fn verify_storage_layout(_e: Env) -> bool {
        true
    }
}
pub const STORAGE_SCHEMA_VERSION: u32 = 1;
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    RegEntry(Symbol),
    Admin,
}
#[contract]
pub struct GrainlifyRegistry;
#[contractimpl]
impl GrainlifyRegistry {
    pub fn init(e: Env, admin: Address) {
        if e.storage().instance().has(&DataKey::Admin) {
            panic!("Init");
        }
        e.storage().instance().set(&DataKey::Admin, &admin);
    }
    pub fn set_addr(e: Env, n: Symbol, a: Address) {
        let adm: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        adm.require_auth();
        e.storage().persistent().set(&DataKey::RegEntry(n), &a);
    }
    pub fn get_addr(e: Env, n: Symbol) -> Address {
        e.storage()
            .persistent()
            .get(&DataKey::RegEntry(n))
            .expect("NF")
    }
}
mod test;
