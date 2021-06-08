use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, AllBalanceResponse, Api, BalanceResponse, BankQuery, Binary, Coin, Deps,
    Querier, QuerierWrapper, QueryRequest, StdError, StdResult, Storage, Timestamp, Uint128,
    WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::Cw20QueryMsg;
use cw20::TokenInfoResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// use moneymarket::custody::{BorrowerResponse, QueryMsg as CustodyQueryMsg};

//TODO: if you have `terraswap` as dependency - use it from there!
pub fn query_balance(
    querier: &QuerierWrapper,
    account_addr: &Addr,
    denom: String,
) -> StdResult<Uint128> {
    // load price form the oracle
    let balance: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr.to_string(),
        denom,
    }))?;
    Ok(balance.amount.amount)
}

//TODO: if you have `terraswap` as dependency - use it from there!
pub fn query_all_balances(querier: &QuerierWrapper, account_addr: Addr) -> StdResult<Vec<Coin>> {
    // load price form the oracle
    let all_balances: AllBalanceResponse =
        querier.query(&QueryRequest::Bank(BankQuery::AllBalances {
            address: account_addr.to_string(),
        }))?;
    Ok(all_balances.amount)
}

//TODO: if you have `terraswap` as dependency - use it from there!
pub fn query_token_balance(
    deps: Deps,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    Ok(deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: contract_addr.to_string(),
            key: Binary::from(concat(
                &to_length_prefixed(b"balance").to_vec(),
                (deps.api.addr_canonicalize(account_addr.as_str())?).as_slice(),
            )),
        }))
        .unwrap_or_else(|_| Uint128::zero()))
}

//TODO: if you have `terraswap` as dependency - use it from there!
pub fn query_supply(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<Uint128> {
    let token_info: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.to_string(),
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    Ok(token_info.total_supply)
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

//TODO: use Anchor as dependency
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerResponse {
    pub borrower: String,
    pub balance: Uint256,
    pub spendable: Uint256,
}

pub fn get_basset_in_custody(
    deps: Deps,
    custody_basset_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint256> {
    let borrower_info: BorrowerResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: custody_basset_addr.to_string(),
            key: Binary::from(concat(
                &to_length_prefixed(b"borrower").to_vec(),
                (deps.api.addr_canonicalize(account_addr.as_str())?).as_slice(),
            )),
        }))?;

    Ok(borrower_info.balance)
}

//TODO: use Anchor as dependency
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub rate: Decimal256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OracleQueryMsg {
    Config {},
    Feeder {
        asset: String,
    },
    Price {
        base: String,
        quote: String,
    },
    Prices {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeConstraints {
    pub block_time: Timestamp,
    pub valid_timeframe_millis: u64,
}

pub fn query_price(
    deps: &Deps,
    oracle_addr: &Addr,
    base: String,
    quote: String,
    time_contraints: Option<TimeConstraints>,
) -> StdResult<PriceResponse> {
    let oracle_price: PriceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: oracle_addr.to_string(),
            msg: to_binary(&OracleQueryMsg::Price { base, quote })?,
        }))?;

    if let Some(time_contraints) = time_contraints {
        let valid_update_time = (time_contraints.block_time.nanos() / 1_000_000)
            - time_contraints.valid_timeframe_millis;
        if oracle_price.last_updated_base < valid_update_time
            || oracle_price.last_updated_quote < valid_update_time
        {
            return Err(StdError::generic_err("Price is too old"));
        }
    }

    Ok(oracle_price)
}

//TODO: Use Anchor as dependency
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMarketMsg {
    ClaimRewards { to: Option<String> },
    DepositStable {},
}
