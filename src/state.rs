use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const ADMIN: Item<Addr> = Item::new("admin");

// Store token addresses from instantiation
pub const RED_TOKEN_ADDRESS: Item<Addr> = Item::new("red_token_address");
pub const BLUE_TOKEN_ADDRESS: Item<Addr> = Item::new("blue_token_address");
pub const TOKEN_ADDRESS: Item<Addr> = Item::new("token_address");

// Total token supply at instantiation
pub const TOTAL_SUPPLY: Item<Uint128> = Item::new("total_supply");

#[cw_serde]
pub struct Deposit {
    pub red_tokens: Uint128,
    pub blue_tokens: Uint128,
}

// Map to store deposit amounts for issuance of BLACK tokens
pub const DEPOSITS: Map<&Addr, Deposit> = Map::new("deposits");
