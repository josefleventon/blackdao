#![cfg(test)]

use cosmwasm_std::{coins, Addr, Binary, Empty, Uint128};
use cw20::{Cw20Coin, Cw20ExecuteMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::{DepositInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

pub fn contract_swap() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

const RED: &str = "contract0";
const BLUE: &str = "contract1";
const BLACK: &str = "contract2";
const SWAP: &str = "contract3";

const ADMIN: &str = "admin";
const USER: &str = "user";

// Initial contract setup
fn setup_contract() -> App {
    let admin = Addr::unchecked(ADMIN);
    let user = Addr::unchecked(USER);

    let init_funds = coins(2000, "ujuno");

    let mut router = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &admin, init_funds)
            .unwrap();
    });

    let cw20_id = router.store_code(contract_cw20());

    // set up $RED cw20 contract with some tokens
    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Red"),
        symbol: String::from("RED"),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: user.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let red_addr = router
        .instantiate_contract(cw20_id, admin.clone(), &msg, &[], "RED", None)
        .unwrap();

    // set up $BLUE cw20 contract with some tokens
    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Blue"),
        symbol: String::from("BLUE"),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: user.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let blue_addr = router
        .instantiate_contract(cw20_id, admin.clone(), &msg, &[], "BLUE", None)
        .unwrap();

    // set up $BLACK cw20 contract with some tokens
    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Black"),
        symbol: String::from("BLACK"),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: admin.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let black_addr = router
        .instantiate_contract(cw20_id, admin.clone(), &msg, &[], "BLACK", None)
        .unwrap();

    // set up swap contract
    let swap_id = router.store_code(contract_swap());
    let swap_addr = router
        .instantiate_contract(
            swap_id,
            admin.clone(),
            &InstantiateMsg {
                red_token_address: red_addr,
                blue_token_address: blue_addr,
                token_address: black_addr.clone(),
                total_supply: Uint128::new(5000),
            },
            &[],
            "SWAP",
            None,
        )
        .unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: swap_addr.into(),
        amount: Uint128::new(5000),
        msg: Binary::default(),
    };

    let res = router.execute_contract(admin.clone(), black_addr, &send_msg, &[]);
    assert!(res.is_ok());

    router
}

#[test]
fn proper_initialization() {
    setup_contract();
}

#[test]
fn try_deposit() {
    let mut router = setup_contract();

    let user = Addr::unchecked(USER);

    // Send 5000 $RED
    let send_msg = Cw20ExecuteMsg::Send {
        contract: SWAP.into(),
        amount: Uint128::new(5000),
        msg: Binary::default(),
    };

    let res = router.execute_contract(user.clone(), Addr::unchecked(RED), &send_msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn try_claim() {
    let mut router = setup_contract();

    let user = Addr::unchecked(USER);

    // Send 5000 $RED
    let send_msg = Cw20ExecuteMsg::Send {
        contract: SWAP.into(),
        amount: Uint128::new(5000),
        msg: Binary::default(),
    };

    let res = router.execute_contract(user.clone(), Addr::unchecked(RED), &send_msg, &[]);
    assert!(res.is_ok());

    // Send 5000 $BLUE
    let send_msg = Cw20ExecuteMsg::Send {
        contract: SWAP.into(),
        amount: Uint128::new(5000),
        msg: Binary::default(),
    };

    let res = router.execute_contract(user.clone(), Addr::unchecked(BLUE), &send_msg, &[]);
    assert!(res.is_ok());

    // Claim tokens
    let claim_msg = ExecuteMsg::Claim {};

    let res = router.execute_contract(user.clone(), Addr::unchecked(SWAP), &claim_msg, &[]);
    assert!(res.is_ok());

    let res: DepositInfoResponse = router
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(SWAP),
            &QueryMsg::DepositInfo {
                address: user.clone(),
            },
        )
        .unwrap();

    assert!(res.red.is_zero());
    assert!(res.blue.is_zero());

    let res: cw20::BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(BLACK),
            &cw20::Cw20QueryMsg::Balance {
                address: user.into(),
            },
        )
        .unwrap();

    assert_eq!(res.balance, Uint128::new(5000));
}

#[test]
fn try_claim_error() {
    let mut router = setup_contract();

    let user = Addr::unchecked(USER);

    // Send 5000 $RED
    let send_msg = Cw20ExecuteMsg::Send {
        contract: SWAP.into(),
        amount: Uint128::new(5000),
        msg: Binary::default(),
    };

    let res = router.execute_contract(user.clone(), Addr::unchecked(RED), &send_msg, &[]);
    assert!(res.is_ok());

    // Claim tokens
    let claim_msg = ExecuteMsg::Claim {};

    let res = router.execute_contract(user.clone(), Addr::unchecked(SWAP), &claim_msg, &[]);
    assert!(res.is_err());

    let res: DepositInfoResponse = router
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(SWAP),
            &QueryMsg::DepositInfo {
                address: user.clone(),
            },
        )
        .unwrap();

    assert_eq!(res.red, Uint128::new(5000));
}
