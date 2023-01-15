use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not Found")]
    NotFound {},

    #[error("Custom Error")]
    CustomError {},

    #[error("Custom Error val: {val:?}")]
    CustomErrorParam { val: String },
}
