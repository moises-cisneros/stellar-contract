#![no_std]
use soroban_sdk::{
    contract,
    contractimpl,
    contracttype,
    log,
    token::Client as TokenClient,
    Address,
    Env,
};

#[contracttype]
pub enum DataKey {
    Admin,
    TokenA,
    TokenB,
    Fee,
    Initialized,
}

#[contract]
pub struct StellarPayAmm;

#[contractimpl]
impl StellarPayAmm {
    pub fn initialize(
        env: Env,
        admin: Address,
        token_a_address: Address,
        token_b_address: Address,
        fee_bps: u32,
    ) {
        if env.storage().persistent().has(&DataKey::Initialized) {
            panic!("Contract already initialized");
        }

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::TokenA, &token_a_address);
        env.storage()
            .persistent()
            .set(&DataKey::TokenB, &token_b_address);
        env.storage().persistent().set(&DataKey::Fee, &fee_bps);
        env.storage().persistent().set(&DataKey::Initialized, &true);

        log!(
            &env,
            "AMM initialized. Admin: {:?}, TokenA: {:?}, TokenB: {:?}, Fee: {}",
            admin,
            token_a_address,
            token_b_address,
            fee_bps
        );
    }

    pub fn deposit(env: Env, to: Address, amount_a: i128, amount_b: i128) {
        to.require_auth();

        let token_a_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenA)
            .expect("TokenA not set");
        let token_b_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenB)
            .expect("TokenB not set");

        let token_a = TokenClient::new(&env, &token_a_address);
        let token_b = TokenClient::new(&env, &token_b_address);

        let balance_a_before = token_a.balance(&env.current_contract_address());
        let balance_b_before = token_b.balance(&env.current_contract_address());

        token_a.transfer(&to, &env.current_contract_address(), &amount_a);
        token_b.transfer(&to, &env.current_contract_address(), &amount_b);

        let balance_a_after = token_a.balance(&env.current_contract_address());
        let balance_b_after = token_b.balance(&env.current_contract_address());

        log!(&env, "Deposit by {:?}: TokenA {:?} (balance before: {}, after: {}), TokenB {:?} (balance before: {}, after: {})",
             to, amount_a, balance_a_before, balance_a_after, amount_b, balance_b_before, balance_b_after);
    }

    pub fn swap(
        env: Env,
        from: Address,
        token_in_address: Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> i128 {
        from.require_auth();

        let token_a_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenA)
            .expect("TokenA not set");
        let token_b_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenB)
            .expect("TokenB not set");
        let fee_bps: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Fee)
            .expect("Fee not set");

        let is_token_in_a = token_in_address == token_a_address;
        let (token_in_client, token_out_client, initial_reserve_in, initial_reserve_out) =
            if is_token_in_a {
                (
                    TokenClient::new(&env, &token_a_address),
                    TokenClient::new(&env, &token_b_address),
                    TokenClient::new(&env, &token_a_address)
                        .balance(&env.current_contract_address()),
                    TokenClient::new(&env, &token_b_address)
                        .balance(&env.current_contract_address()),
                )
            } else {
                (
                    TokenClient::new(&env, &token_b_address),
                    TokenClient::new(&env, &token_a_address),
                    TokenClient::new(&env, &token_b_address)
                        .balance(&env.current_contract_address()),
                    TokenClient::new(&env, &token_a_address)
                        .balance(&env.current_contract_address()),
                )
            };

        token_in_client.transfer(&from, &env.current_contract_address(), &amount_in);

        let amount_in_after_fee = amount_in - (amount_in * (fee_bps as i128) / 10_000);

        if initial_reserve_in == 0 {
            panic!("Insufficient liquidity for swap");
        }
        let amount_out = (initial_reserve_out * amount_in_after_fee)
            / (initial_reserve_in + amount_in_after_fee);

        if amount_out < min_amount_out {
            panic!("Slippage too high or min_amount_out not met");
        }

        token_out_client.transfer(&env.current_contract_address(), &from, &amount_out);

        log!(
            &env,
            "Swap by {:?}: In {:?} amount {}, Out amount {}",
            from,
            token_in_address,
            amount_in,
            amount_out
        );

        amount_out
    }

    pub fn get_reserves(env: Env) -> (i128, i128) {
        let token_a_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenA)
            .expect("TokenA not set");
        let token_b_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenB)
            .expect("TokenB not set");

        let token_a = TokenClient::new(&env, &token_a_address);
        let token_b = TokenClient::new(&env, &token_b_address);

        (
            token_a.balance(&env.current_contract_address()),
            token_b.balance(&env.current_contract_address()),
        )
    }

    pub fn set_fee(env: Env, admin: Address, new_fee_bps: u32) {
        admin.require_auth();

        let current_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        if admin != current_admin {
            panic!("Unauthorized: Only admin can set fee");
        }
        env.storage().persistent().set(&DataKey::Fee, &new_fee_bps);
        log!(&env, "Fee updated to {}", new_fee_bps);
    }

    pub fn get_contract_info(env: Env) -> (Address, Address, Address, u32) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        let token_a: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenA)
            .expect("TokenA not set");
        let token_b: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenB)
            .expect("TokenB not set");
        let fee: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Fee)
            .expect("Fee not set");

        (admin, token_a, token_b, fee)
    }

    pub fn quote_swap(env: Env, token_in_address: Address, amount_in: i128) -> i128 {
        let token_a_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenA)
            .expect("TokenA not set");
        let token_b_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TokenB)
            .expect("TokenB not set");
        let fee_bps: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Fee)
            .expect("Fee not set");

        let is_token_in_a = token_in_address == token_a_address;
        let (reserve_in, reserve_out) = if is_token_in_a {
            let token_a = TokenClient::new(&env, &token_a_address);
            let token_b = TokenClient::new(&env, &token_b_address);
            (
                token_a.balance(&env.current_contract_address()),
                token_b.balance(&env.current_contract_address()),
            )
        } else {
            let token_a = TokenClient::new(&env, &token_a_address);
            let token_b = TokenClient::new(&env, &token_b_address);
            (
                token_b.balance(&env.current_contract_address()),
                token_a.balance(&env.current_contract_address()),
            )
        };

        if reserve_in == 0 {
            return 0;
        }

        let amount_in_after_fee = amount_in - (amount_in * (fee_bps as i128) / 10_000);
        (reserve_out * amount_in_after_fee) / (reserve_in + amount_in_after_fee)
    }
}

mod test;
