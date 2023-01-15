use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub red_token_address: Addr,
    pub blue_token_address: Addr,
    pub token_address: Addr,
    pub total_supply: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Handler for receiving cw20 token deposits
    Receive(Cw20ReceiveMsg),
    /// Use deposited tokens to claim BLACK tokens
    Claim {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get supply info
    #[returns(SupplyInfoResponse)]
    SupplyInfo {},
    /// Get deposit info for an address
    #[returns(DepositInfoResponse)]
    DepositInfo { address: Addr },
}

// We define a custom struct for each query response

#[cw_serde]
pub struct SupplyInfoResponse {
    pub total_supply: Uint128,
    pub claimed_supply: Uint128,
    pub remaining_supply: Uint128,
}

#[cw_serde]
pub struct DepositInfoResponse {
    pub red: Uint128,
    pub blue: Uint128,
}
