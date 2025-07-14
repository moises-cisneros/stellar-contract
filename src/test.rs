#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env
};

// Función helper para crear un token de prueba
fn create_token_contract<'a>(env: &Env, admin: &Address) -> (Address, TokenClient<'a>, StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone()).address();
    (
        contract_address.clone(),
        TokenClient::new(env, &contract_address),
        StellarAssetClient::new(env, &contract_address),
    )
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(StellarPayAmm, ());
    let client = StellarPayAmmClient::new(&env, &contract_id);

    // Crear tokens de prueba (simulando USDC y BOB)
    let (token_a_id, _token_a_client, _token_a_stellar) = create_token_contract(&env, &admin);
    let (token_b_id, _token_b_client, _token_b_stellar) = create_token_contract(&env, &admin);

    // Inicializar el contrato AMM
    client.initialize(&admin, &token_a_id, &token_b_id, &30); // 0.3% fee

    // Verificar que la información del contrato se guardó correctamente
    let (stored_admin, stored_token_a, stored_token_b, stored_fee) = client.get_contract_info();
    
    assert_eq!(stored_admin, admin);
    assert_eq!(stored_token_a, token_a_id);
    assert_eq!(stored_token_b, token_b_id);
    assert_eq!(stored_fee, 30);
}

#[test]
fn test_deposit_and_get_reserves() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = env.register(StellarPayAmm, ());
    let client = StellarPayAmmClient::new(&env, &contract_id);

    // Crear tokens de prueba
    let (token_a_id, token_a_client, token_a_stellar) = create_token_contract(&env, &admin);
    let (token_b_id, token_b_client, token_b_stellar) = create_token_contract(&env, &admin);

    // Inicializar el contrato AMM
    client.initialize(&admin, &token_a_id, &token_b_id, &30);

    // Mint tokens para el usuario
    token_a_stellar.mint(&user, &1000);
    token_b_stellar.mint(&user, &2000);

    // Depositar liquidez
    client.deposit(&user, &500, &1000);

    // Verificar reservas
    let (reserve_a, reserve_b) = client.get_reserves();
    assert_eq!(reserve_a, 500);
    assert_eq!(reserve_b, 1000);
}

#[test]
fn test_swap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = env.register(StellarPayAmm, ());
    let client = StellarPayAmmClient::new(&env, &contract_id);

    // Crear tokens de prueba
    let (token_a_id, token_a_client, token_a_stellar) = create_token_contract(&env, &admin);
    let (token_b_id, token_b_client, token_b_stellar) = create_token_contract(&env, &admin);

    // Inicializar el contrato AMM
    client.initialize(&admin, &token_a_id, &token_b_id, &30);

    // Mint tokens iniciales para el usuario
    token_a_stellar.mint(&user, &2000);
    token_b_stellar.mint(&user, &2000);

    // Depositar liquidez inicial (para crear el pool)
    client.deposit(&user, &1000, &1000);

    // Balances iniciales del usuario antes del swap
    let user_balance_a_before = token_a_client.balance(&user);
    let user_balance_b_before = token_b_client.balance(&user);

    // Realizar un swap: intercambiar 100 de token A por token B
    let amount_out = client.swap(&user, &token_a_id, &100, &90); // min_amount_out = 90

    // Verificar que se recibió algún token B
    assert!(amount_out > 0);
    assert!(amount_out >= 90); // Cumple con el mínimo esperado

    // Verificar balances después del swap
    let user_balance_a_after = token_a_client.balance(&user);
    let user_balance_b_after = token_b_client.balance(&user);

    // El usuario debe tener 100 tokens A menos
    assert_eq!(user_balance_a_after, user_balance_a_before - 100);
    // El usuario debe tener más tokens B
    assert_eq!(user_balance_b_after, user_balance_b_before + amount_out);
}

#[test]
fn test_quote_swap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = env.register(StellarPayAmm, ());
    let client = StellarPayAmmClient::new(&env, &contract_id);

    // Crear tokens de prueba
    let (token_a_id, token_a_client, token_a_stellar) = create_token_contract(&env, &admin);
    let (token_b_id, token_b_client, token_b_stellar) = create_token_contract(&env, &admin);

    // Inicializar el contrato AMM
    client.initialize(&admin, &token_a_id, &token_b_id, &30);

    // Mint tokens y depositar liquidez
    token_a_stellar.mint(&user, &1000);
    token_b_stellar.mint(&user, &1000);
    client.deposit(&user, &1000, &1000);

    // Cotizar un swap sin ejecutarlo
    let quoted_amount = client.quote_swap(&token_a_id, &100);
    
    // La cotización debe ser mayor que 0 y menor que la cantidad de entrada
    assert!(quoted_amount > 0);
    assert!(quoted_amount < 100); // Debido a las comisiones y la fórmula AMM
}

#[test]
fn test_set_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(StellarPayAmm, ());
    let client = StellarPayAmmClient::new(&env, &contract_id);

    // Crear tokens de prueba
    let (token_a_id, _token_a_client, _token_a_stellar) = create_token_contract(&env, &admin);
    let (token_b_id, _token_b_client, _token_b_stellar) = create_token_contract(&env, &admin);

    // Inicializar el contrato AMM
    client.initialize(&admin, &token_a_id, &token_b_id, &30);

    // Cambiar la comisión
    client.set_fee(&admin, &50); // Cambiar a 0.5%

    // Verificar que la comisión se actualizó
    let (_, _, _, fee) = client.get_contract_info();
    assert_eq!(fee, 50);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(StellarPayAmm, ());
    let client = StellarPayAmmClient::new(&env, &contract_id);

    // Crear tokens de prueba
    let (token_a_id, _token_a_client, _token_a_stellar) = create_token_contract(&env, &admin);
    let (token_b_id, _token_b_client, _token_b_stellar) = create_token_contract(&env, &admin);

    // Inicializar el contrato AMM por primera vez
    client.initialize(&admin, &token_a_id, &token_b_id, &30);

    // Intentar inicializar de nuevo (debe fallar)
    client.initialize(&admin, &token_a_id, &token_b_id, &50);
}
