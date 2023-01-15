#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, Uint128,
    WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Balance, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{DepositInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SupplyInfoResponse};
use crate::state::{
    Deposit, ADMIN, BLUE_TOKEN_ADDRESS, DEPOSITS, RED_TOKEN_ADDRESS, TOKEN_ADDRESS, TOTAL_SUPPLY,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:black";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    RED_TOKEN_ADDRESS.save(deps.storage, &msg.red_token_address)?;
    BLUE_TOKEN_ADDRESS.save(deps.storage, &msg.blue_token_address)?;
    TOKEN_ADDRESS.save(deps.storage, &msg.token_address)?;
    TOTAL_SUPPLY.save(deps.storage, &msg.total_supply)?;
    ADMIN.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("red_token_address", msg.red_token_address)
        .add_attribute("blue_token_address", msg.blue_token_address)
        .add_attribute("total_supply", msg.total_supply))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(receive_msg) => execute_receive(deps, env, info, receive_msg),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
    }
}

pub fn execute_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // Wallet that executed the "Send" on the cw20 contract
    let sender = deps.api.addr_validate(&wrapper.sender)?;

    let admin = ADMIN.load(deps.storage)?;
    if sender == admin {
        return Ok(Response::new().add_attribute("method", "initial_token_deposit"));
    };

    // Check that token is either $RED or $BLUE
    let red_token_address = RED_TOKEN_ADDRESS.load(deps.storage)?;
    let blue_token_address = BLUE_TOKEN_ADDRESS.load(deps.storage)?;
    if info.sender != red_token_address && info.sender != blue_token_address {
        return Err(ContractError::Unauthorized {});
    };

    // Attempt to load balance of user
    let deposit = DEPOSITS.load(deps.storage, &sender);

    match deposit {
        Ok(deposit) => {
            let data = if info.sender == red_token_address {
                Deposit {
                    red_tokens: deposit.red_tokens + wrapper.amount,
                    blue_tokens: deposit.blue_tokens,
                }
            } else {
                Deposit {
                    red_tokens: deposit.red_tokens,
                    blue_tokens: deposit.blue_tokens + wrapper.amount,
                }
            };
            DEPOSITS.update(deps.storage, &sender, |deposit| match deposit {
                Some(_) => Ok(data),
                None => Err(ContractError::NotFound {}),
            })?;
        }
        Err(_) => {
            let data = if info.sender == red_token_address {
                Deposit {
                    red_tokens: wrapper.amount,
                    blue_tokens: Uint128::zero(),
                }
            } else {
                Deposit {
                    red_tokens: Uint128::zero(),
                    blue_tokens: wrapper.amount,
                }
            };
            DEPOSITS.save(deps.storage, &sender, &data)?;
        }
    };

    Ok(Response::new().add_attribute("method", "deposit"))
}

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let deposit = DEPOSITS.load(deps.storage, &info.sender)?;

    if deposit.blue_tokens.is_zero() || deposit.red_tokens.is_zero() {
        return Err(ContractError::Payment(cw_utils::PaymentError::NoFunds {}));
    }

    let claim_amount = if deposit.blue_tokens < deposit.red_tokens {
        deposit.blue_tokens
    } else {
        deposit.red_tokens
    };

    let token_address = TOKEN_ADDRESS.load(deps.storage)?;
    let send_msg = send_cw20_tokens(
        &info.sender,
        &Cw20CoinVerified {
            address: token_address,
            amount: claim_amount,
        },
    )?;

    DEPOSITS.update(deps.storage, &info.sender, |deposit| match deposit {
        Some(deposit) => Ok(Deposit {
            red_tokens: deposit.red_tokens - claim_amount,
            blue_tokens: deposit.blue_tokens - claim_amount,
        }),
        None => Err(ContractError::NotFound {}),
    })?;

    Ok(Response::new()
        .add_attribute("method", "claim")
        .add_attribute("sender", info.sender)
        .add_attribute("amount", claim_amount)
        .add_submessage(send_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SupplyInfo {} => to_binary(&query_supply_info(deps, env)?),
        QueryMsg::DepositInfo { address } => to_binary(&query_deposit_info(deps, address)?),
    }
}

fn query_supply_info(deps: Deps, env: Env) -> StdResult<SupplyInfoResponse> {
    let total_supply = TOTAL_SUPPLY.load(deps.storage)?;
    let token_address = TOKEN_ADDRESS.load(deps.storage)?;

    let balances = deps.querier.query_all_balances(env.contract.address)?;
    let balance = Balance::from(balances);

    let remaining_supply: Uint128 = match balance {
        Balance::Cw20(token) => {
            if token.address == token_address {
                token.amount
            } else {
                Uint128::zero()
            }
        }
        Balance::Native(_) => Uint128::zero(),
    };

    let claimed_supply = total_supply - remaining_supply;

    Ok(SupplyInfoResponse {
        total_supply,
        claimed_supply,
        remaining_supply,
    })
}

fn query_deposit_info(deps: Deps, address: Addr) -> StdResult<DepositInfoResponse> {
    let deposit = DEPOSITS.load(deps.storage, &address)?;

    Ok(DepositInfoResponse {
        red: deposit.red_tokens,
        blue: deposit.blue_tokens,
    })
}

// Send Cw20 tokens to another address
pub fn send_cw20_tokens(to: &Addr, balance: &Cw20CoinVerified) -> StdResult<SubMsg> {
    let msg = Cw20ExecuteMsg::Transfer {
        recipient: to.into(),
        amount: balance.amount,
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: balance.address.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    });

    Ok(exec)
}
