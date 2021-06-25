use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance_contract: Addr,
    //TODO: test that this value is SET after initialization
    pub psi_distributor_addr: Addr,
    pub anchor_token: Addr,
    pub anchor_overseer_contract: Addr,
    pub anchor_market_contract: Addr,
    pub anchor_custody_basset_contract: Addr,
    pub anc_stable_swap_contract: Addr,
    pub psi_stable_swap_contract: Addr,
    pub casset_token: Addr,
    pub basset_token: Addr,
    pub aterra_token: Addr,
    //what part of profit from selling ANC spend to buy PSI
    pub psi_token: Addr,
    pub basset_farmer_config_contract: Addr,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
    //UST value in balance should be more than loan
    //on what portion.
    //for example: 1.01 means 1% more than loan
    pub over_loan_balance_value: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct RepayingLoanState {
    pub iteration_index: u8,
    pub to_repay_amount: Uint256,
    pub repaying_amount: Uint256,
    pub aim_buffer_size: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct ChildContractsCodeId {
    pub nasset_token: u64,
    pub nasset_staker: u64,
    pub psi_distributor: u64,
}

const CONFIG: Item<Config> = Item::new("config");
const REPAYING_LOAN: Item<RepayingLoanState> = Item::new("repaying");
const AIM_BUFFER_SIZE: Item<Uint256> = Item::new("aim_buf_size");
const STABLE_BALANCE_BEFORE_SELL_ANC: Item<Uint128> = Item::new("balance_before_sell_anc");
const LAST_REWARDS_CLAIMING_HEIGHT: Item<u64> = Item::new("last_rewards_claiming_height");
//need that only for instantiating
const CHILD_CONTRACTS_CODE_ID: Item<ChildContractsCodeId> = Item::new("child_contracts_code_id");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn config_set_casset_token(storage: &mut dyn Storage, casset_token: Addr) -> StdResult<Config> {
    CONFIG.update(storage, |mut config| -> StdResult<_> {
        config.casset_token = casset_token;
        Ok(config)
    })
}

pub fn config_set_psi_distributor(
    storage: &mut dyn Storage,
    psi_distributor: Addr,
) -> StdResult<Config> {
    CONFIG.update(storage, |mut config| -> StdResult<_> {
        config.psi_distributor_addr = psi_distributor;
        Ok(config)
    })
}

pub fn load_repaying_loan_state(storage: &dyn Storage) -> StdResult<RepayingLoanState> {
    REPAYING_LOAN
        .may_load(storage)
        .map(|res| res.unwrap_or_default())
}

pub fn store_repaying_loan_state(
    storage: &mut dyn Storage,
    repaying_loan_state: &RepayingLoanState,
) -> StdResult<()> {
    REPAYING_LOAN.save(storage, repaying_loan_state)
}

pub fn update_loan_state_part_of_loan_repaid(
    storage: &mut dyn Storage,
) -> StdResult<RepayingLoanState> {
    REPAYING_LOAN.update(storage, |mut rep_loan| -> StdResult<_> {
        rep_loan.to_repay_amount = rep_loan.to_repay_amount - rep_loan.repaying_amount;
        Ok(rep_loan)
    })
}

pub fn load_aim_buffer_size(storage: &dyn Storage) -> StdResult<Uint256> {
    AIM_BUFFER_SIZE.load(storage)
}

pub fn store_aim_buffer_size(storage: &mut dyn Storage, aim_buf_size: &Uint256) -> StdResult<()> {
    AIM_BUFFER_SIZE.save(storage, aim_buf_size)
}

pub fn load_stable_balance_before_selling_anc(storage: &dyn Storage) -> StdResult<Uint128> {
    STABLE_BALANCE_BEFORE_SELL_ANC.load(storage)
}

pub fn store_stable_balance_before_selling_anc(
    storage: &mut dyn Storage,
    balance: &Uint128,
) -> StdResult<()> {
    STABLE_BALANCE_BEFORE_SELL_ANC.save(storage, balance)
}

pub fn load_child_contracts_code_id(storage: &dyn Storage) -> StdResult<ChildContractsCodeId> {
    CHILD_CONTRACTS_CODE_ID.load(storage)
}

pub fn store_child_contracts_code_id(
    storage: &mut dyn Storage,
    child_contracts_code_id: &ChildContractsCodeId,
) -> StdResult<()> {
    CHILD_CONTRACTS_CODE_ID.save(storage, child_contracts_code_id)
}

pub fn load_last_rewards_claiming_height(storage: &dyn Storage) -> StdResult<u64> {
    LAST_REWARDS_CLAIMING_HEIGHT
        .may_load(storage)
        .map(|may_value| may_value.unwrap_or_default())
}

pub fn store_last_rewards_claiming_height(
    storage: &mut dyn Storage,
    height: &u64,
) -> StdResult<()> {
    LAST_REWARDS_CLAIMING_HEIGHT.save(storage, height)
}
