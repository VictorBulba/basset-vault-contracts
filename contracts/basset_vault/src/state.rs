use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub governance_contract: Addr,
    pub anchor_token: Addr,
    pub anchor_overseer_contract: Addr,
    pub anchor_market_contract: Addr,
    pub anchor_custody_basset_contract: Addr,
    pub anc_stable_swap_contract: Addr,
    pub psi_stable_swap_contract: Addr,
    pub basset_token: Addr,
    pub aterra_token: Addr,
    pub psi_token: Addr,
    pub basset_vault_strategy_contract: Addr,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
    pub over_loan_balance_value: Decimal256,
    pub nasset_token: Addr,
    pub psi_distributor: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct RepayingLoanState {
    pub iteration_index: u8,
    pub repayed_something: bool,
    pub to_repay_amount: Uint256,
    pub repaying_amount: Uint256,
    pub aim_buffer_size: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ChildContractsInfo {
    pub nasset_token_code_id: u64,
    pub nasset_token_rewards_code_id: u64,
    pub psi_distributor_code_id: u64,
    pub collateral_token_symbol: String,
    pub community_pool_contract_addr: String,
    pub manual_ltv: Decimal256,
    pub fee_rate: Decimal256,
    pub tax_rate: Decimal256,
    pub nasset_psi_swap_contract_addr: String,
    pub terraswap_factory_contract_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GovernanceUpdateState {
    pub new_governance_contract_addr: Addr,
    pub wait_approve_until: u64,
}

static KEY_CONFIG: Item<Config> = Item::new("config");
static KEY_REPAYING_LOAN: Item<RepayingLoanState> = Item::new("repaying");
static KEY_AIM_BUFFER_SIZE: Item<Uint256> = Item::new("aim_buf_size");

static KEY_STABLE_BALANCE_BEFORE_SELL_ANC: Item<Uint128> = Item::new("balance_before_sell_anc");
static KEY_LAST_REWARDS_CLAIMING_HEIGHT: Item<u64> = Item::new("last_rewards_claiming_height");
//need that only for instantiating
static KEY_CHILD_CONTRACTS_INFO: Item<ChildContractsInfo> = Item::new("child_contracts_code_id");
static KEY_NASSET_TOKEN_CONFIG_HOLDER: Item<Addr> = Item::new("nasset_token_config_holder");

static KEY_GOVERNANCE_UPDATE: Item<GovernanceUpdateState> = Item::new("gov_update");

pub fn load_nasset_token_config_holder(storage: &dyn Storage) -> StdResult<Addr> {
    KEY_NASSET_TOKEN_CONFIG_HOLDER.load(storage)
}

pub fn store_nasset_token_config_holder(storage: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
    KEY_NASSET_TOKEN_CONFIG_HOLDER.save(storage, addr)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    KEY_CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    KEY_CONFIG.save(storage, config)
}

pub fn config_set_nasset_token(storage: &mut dyn Storage, nasset_token: Addr) -> StdResult<Config> {
    KEY_CONFIG.update(storage, |mut config: Config| -> StdResult<_> {
        config.nasset_token = nasset_token;
        Ok(config)
    })
}

pub fn config_set_psi_distributor(
    storage: &mut dyn Storage,
    psi_distributor: Addr,
) -> StdResult<Config> {
    KEY_CONFIG.update(storage, |mut config: Config| -> StdResult<_> {
        config.psi_distributor = psi_distributor;
        Ok(config)
    })
}

pub fn load_repaying_loan_state(storage: &dyn Storage) -> StdResult<RepayingLoanState> {
    KEY_REPAYING_LOAN
        .may_load(storage)
        .map(|res| res.unwrap_or_default())
}

pub fn store_repaying_loan_state(
    storage: &mut dyn Storage,
    repaying_loan_state: &RepayingLoanState,
) -> StdResult<()> {
    KEY_REPAYING_LOAN.save(storage, repaying_loan_state)
}

pub fn update_loan_state_part_of_loan_repaid(
    storage: &mut dyn Storage,
) -> StdResult<RepayingLoanState> {
    KEY_REPAYING_LOAN.update(storage, |mut rep_loan: RepayingLoanState| -> StdResult<_> {
        rep_loan.to_repay_amount = rep_loan.to_repay_amount - rep_loan.repaying_amount;
        rep_loan.repayed_something = true;
        Ok(rep_loan)
    })
}

pub fn load_aim_buffer_size(storage: &dyn Storage) -> StdResult<Uint256> {
    KEY_AIM_BUFFER_SIZE.load(storage)
}

pub fn store_aim_buffer_size(storage: &mut dyn Storage, aim_buf_size: &Uint256) -> StdResult<()> {
    KEY_AIM_BUFFER_SIZE.save(storage, aim_buf_size)
}

pub fn load_stable_balance_before_selling_anc(storage: &dyn Storage) -> StdResult<Uint128> {
    KEY_STABLE_BALANCE_BEFORE_SELL_ANC.load(storage)
}

pub fn store_stable_balance_before_selling_anc(
    storage: &mut dyn Storage,
    balance: &Uint128,
) -> StdResult<()> {
    KEY_STABLE_BALANCE_BEFORE_SELL_ANC.save(storage, balance)
}

pub fn load_child_contracts_info(storage: &dyn Storage) -> StdResult<ChildContractsInfo> {
    KEY_CHILD_CONTRACTS_INFO.load(storage)
}

pub fn store_child_contracts_info(
    storage: &mut dyn Storage,
    child_contracts_info: &ChildContractsInfo,
) -> StdResult<()> {
    KEY_CHILD_CONTRACTS_INFO.save(storage, child_contracts_info)
}

pub fn load_last_rewards_claiming_height(storage: &dyn Storage) -> StdResult<u64> {
    KEY_LAST_REWARDS_CLAIMING_HEIGHT
        .may_load(storage)
        .map(|may_value| may_value.unwrap_or_default())
}

pub fn store_last_rewards_claiming_height(
    storage: &mut dyn Storage,
    height: &u64,
) -> StdResult<()> {
    KEY_LAST_REWARDS_CLAIMING_HEIGHT.save(storage, height)
}

pub fn load_gov_update(storage: &dyn Storage) -> StdResult<GovernanceUpdateState> {
    KEY_GOVERNANCE_UPDATE.load(storage)
}

pub fn store_gov_update(
    storage: &mut dyn Storage,
    gov_update: &GovernanceUpdateState,
) -> StdResult<()> {
    KEY_GOVERNANCE_UPDATE.save(storage, gov_update)
}

pub fn remove_gov_update(storage: &mut dyn Storage) -> () {
    KEY_GOVERNANCE_UPDATE.remove(storage)
}
