use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::reply_response::MsgInstantiateContractResponse;
use crate::state::Config;
use crate::{
    commands, queries,
    state::{
        config_set_nasset_token, config_set_psi_distributor, load_child_contracts_info,
        load_config, load_nasset_token_config_holder, store_child_contracts_info, store_config,
        store_nasset_token_config_holder, update_loan_state_part_of_loan_repaid,
        ChildContractsInfo,
    },
    SubmsgIds, TOO_HIGH_BORROW_DEMAND_ERR_MSG,
};
use basset_vault::{
    basset_vault::{AnyoneMsg, ExecuteMsg, GovernanceMsg, InstantiateMsg, QueryMsg, YourselfMsg},
    nasset_token::InstantiateMsg as NAssetTokenInstantiateMsg,
    nasset_token_config_holder::{
        AnyoneMsg as NAssetTokenConfigHolderAnyoneMsg,
        ExecuteMsg as NAssetTokenConfigHolderExecuteMsg,
        InstantiateMsg as NAssetTokenConfigHolderInstantiateMsg,
    },
    nasset_token_rewards::InstantiateMsg as NAssetTokenRewardsInstantiateMsg,
    psi_distributor::InstantiateMsg as PsiDistributorInstantiateMsg,
    querier::get_basset_in_custody,
};
use cw20::MinterResponse;
use protobuf::Message;
use std::convert::TryFrom;

//TODO: разделить CONFIG на несколько составляющих чтобы не читать всю эту байду каждый раз

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        governance_contract: deps.api.addr_validate(&msg.gov_addr)?,
        anchor_token: deps.api.addr_validate(&msg.anchor_addr)?,
        anchor_overseer_contract: deps.api.addr_validate(&msg.a_overseer_addr)?,
        anchor_market_contract: deps.api.addr_validate(&msg.a_market_addr)?,
        anchor_custody_basset_contract: deps.api.addr_validate(&msg.a_custody_basset_addr)?,
        anc_stable_swap_contract: deps.api.addr_validate(&msg.anc_stable_swap_addr)?,
        psi_stable_swap_contract: deps.api.addr_validate(&msg.psi_stable_swap_addr)?,
        basset_token: deps.api.addr_validate(&msg.basset_addr)?,
        aterra_token: deps.api.addr_validate(&msg.aterra_addr)?,
        psi_token: deps.api.addr_validate(&msg.psi_addr)?,
        basset_vault_strategy_contract: deps.api.addr_validate(&msg.basset_vs_addr)?,
        stable_denom: msg.stable_denom,
        claiming_rewards_delay: msg.claiming_rewards_delay,
        over_loan_balance_value: msg.over_loan_balance_value,
        nasset_token: Addr::unchecked(""),
        psi_distributor: Addr::unchecked(""),
    };
    store_config(deps.storage, &config)?;

    let child_contracts_info = ChildContractsInfo {
        nasset_token_code_id: msg.nasset_t_ci,
        nasset_token_rewards_code_id: msg.nasset_t_r_ci,
        psi_distributor_code_id: msg.psi_distr_ci,
        collateral_token_symbol: msg.collateral_ts,
        community_pool_contract_addr: msg.community_addr,
        manual_ltv: msg.manual_ltv,
        fee_rate: msg.fee_rate,
        tax_rate: msg.tax_rate,
    };
    store_child_contracts_info(deps.storage, &child_contracts_info)?;

    Ok(Response {
        messages: vec![SubMsg {
            msg: WasmMsg::Instantiate {
                admin: None,
                code_id: msg.nasset_t_ch_ci,
                msg: to_binary(&NAssetTokenConfigHolderInstantiateMsg {
                    governance_contract_addr: msg.gov_addr,
                })?,
                funds: vec![],
                label: "".to_string(),
            }
            .into(),
            gas_limit: None,
            id: SubmsgIds::InitNAssetConfigHolder.id(),
            reply_on: ReplyOn::Success,
        }],
        attributes: vec![],
        data: None,
        events: vec![],
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    let submessage_enum = SubmsgIds::try_from(msg.id)?;
    match submessage_enum {
        SubmsgIds::InitNAssetConfigHolder => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let nasset_token_config_holder = res.get_contract_address();
            store_nasset_token_config_holder(
                deps.storage,
                &Addr::unchecked(nasset_token_config_holder),
            )?;
            let child_contracts_info = load_child_contracts_info(deps.as_ref().storage)?;

            Ok(Response {
                events: vec![],
                messages: vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        admin: None,
                        code_id: child_contracts_info.nasset_token_code_id,
                        msg: to_binary(&NAssetTokenInstantiateMsg {
                            name: "nexus bAsset token share representation".to_string(),
                            symbol: format!("n{}", child_contracts_info.collateral_token_symbol),
                            decimals: 6,
                            initial_balances: vec![],
                            mint: Some(MinterResponse {
                                minter: env.contract.address.to_string(),
                                cap: None,
                            }),
                            config_holder_contract: nasset_token_config_holder.to_string(),
                        })?,
                        funds: vec![],
                        label: "".to_string(),
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::InitNAsset.id(),
                    reply_on: ReplyOn::Success,
                }],
                attributes: vec![
                    attr("action", "nasset_token_config_holder_initialized"),
                    attr(
                        "nasset_token_config_holder_addr",
                        nasset_token_config_holder,
                    ),
                ],
                data: None,
            })
        }

        SubmsgIds::InitNAsset => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let nasset_token = res.get_contract_address();
            let config =
                config_set_nasset_token(deps.storage, deps.api.addr_validate(nasset_token)?)?;
            let child_contracts_info = load_child_contracts_info(deps.as_ref().storage)?;

            Ok(Response {
                events: vec![],
                messages: vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        admin: None,
                        code_id: child_contracts_info.nasset_token_rewards_code_id,
                        msg: to_binary(&NAssetTokenRewardsInstantiateMsg {
                            psi_token_addr: config.psi_token.to_string(),
                            nasset_token_addr: nasset_token.to_string(),
                            governance_contract_addr: config.governance_contract.to_string(),
                        })?,
                        funds: vec![],
                        label: "".to_string(),
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::InitNAssetRewards.id(),
                    reply_on: ReplyOn::Success,
                }],
                attributes: vec![
                    attr("action", "nasset_token_initialized"),
                    attr("nasset_token_addr", nasset_token),
                ],
                data: None,
            })
        }

        SubmsgIds::InitNAssetRewards => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            //we do not need to save nasset_token_rewards addr, cause there is no direct interactions
            let nasset_token_rewards = res.get_contract_address();
            let child_contracts_info = load_child_contracts_info(deps.as_ref().storage)?;
            let nasset_token_config_holder = load_nasset_token_config_holder(deps.storage)?;
            let config = load_config(deps.storage)?;

            Ok(Response {
                events: vec![],
                messages: vec![
                    SubMsg {
                        msg: WasmMsg::Instantiate {
                            admin: None,
                            code_id: child_contracts_info.psi_distributor_code_id,
                            msg: to_binary(&PsiDistributorInstantiateMsg {
                                psi_token_addr: config.psi_token.to_string(),
                                nasset_token_rewards_contract_addr: nasset_token_rewards
                                    .to_string(),
                                governance_contract_addr: config.governance_contract.to_string(),
                                basset_vault_strategy_contract_addr: config
                                    .basset_vault_strategy_contract
                                    .to_string(),
                                community_pool_contract_addr: child_contracts_info
                                    .community_pool_contract_addr,
                                manual_ltv: child_contracts_info.manual_ltv,
                                fee_rate: child_contracts_info.fee_rate,
                                tax_rate: child_contracts_info.tax_rate,
                            })?,
                            funds: vec![],
                            label: "".to_string(),
                        }
                        .into(),
                        gas_limit: None,
                        id: SubmsgIds::InitPsiDistributor.id(),
                        reply_on: ReplyOn::Success,
                    },
                    SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: nasset_token_config_holder.to_string(),
                        funds: vec![],
                        msg: to_binary(&NAssetTokenConfigHolderExecuteMsg::Anyone {
                            anyone_msg: NAssetTokenConfigHolderAnyoneMsg::SetTokenRewardsContract {
                                nasset_token_rewards_contract_addr: nasset_token_rewards
                                    .to_string(),
                            },
                        })
                        .unwrap(),
                    })),
                ],
                attributes: vec![
                    attr("action", "nasset_token_rewards_initialized"),
                    attr("nasset_token_rewards_addr", nasset_token_rewards),
                ],
                data: None,
            })
        }

        SubmsgIds::InitPsiDistributor => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let psi_distributor = res.get_contract_address();
            config_set_psi_distributor(deps.storage, deps.api.addr_validate(psi_distributor)?)?;

            Ok(Response {
                events: vec![],
                messages: vec![],
                attributes: vec![
                    attr("action", "psi_distributor_initialized"),
                    attr("psi_distributor_addr", psi_distributor),
                ],
                data: None,
            })
        }

        SubmsgIds::RedeemStableOnRepayLoan => match msg.result {
            cosmwasm_std::ContractResult::Err(err_msg) => {
                if err_msg
                    .to_lowercase()
                    .contains(TOO_HIGH_BORROW_DEMAND_ERR_MSG)
                {
                    //we need to repay loan a bit, before redeem stables
                    commands::repay_logic_on_reply(deps, env)
                } else {
                    return Err(StdError::generic_err(format!(
                        "fail to redeem stables, reason: {}",
                        err_msg
                    ))
                    .into());
                }
            }
            cosmwasm_std::ContractResult::Ok(_) => commands::repay_logic_on_reply(deps, env),
        },

        SubmsgIds::RepayLoan => {
            let _ = update_loan_state_part_of_loan_repaid(deps.storage)?;
            Ok(Response::default())
        }

        SubmsgIds::Borrowing => commands::borrow_logic_on_reply(deps, env),

        SubmsgIds::RedeemStableOnRemainder => {
            let config = load_config(deps.storage)?;
            //we can't repay loan to unlock aTerra (cause we have 0 loan here),
            //so try to use stable balance in any case (error or not)
            commands::buy_psi_on_remainded_stable_coins(deps.as_ref(), env, config)
        }
    }
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),

        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::Rebalance {} => {
                let config = load_config(deps.storage)?;

                // basset balance in custody contract
                let basset_in_custody = get_basset_in_custody(
                    deps.as_ref(),
                    &config.anchor_custody_basset_contract,
                    &env.contract.address.clone(),
                )?;

                commands::rebalance(deps, env, &config, basset_in_custody, None)
            }

            AnyoneMsg::HonestWork {} => commands::claim_anc_rewards(deps, env),

            AnyoneMsg::ClaimRemainder {} => commands::claim_remainded_stables(deps.as_ref(), env),
        },

        ExecuteMsg::Yourself { yourself_msg } => {
            if info.sender != env.contract.address {
                return Err(StdError::generic_err("unauthorized"));
            }

            match yourself_msg {
                YourselfMsg::SwapAnc {} => commands::swap_anc(deps, env),
                YourselfMsg::DisributeRewards {} => commands::distribute_rewards(deps, env),
            }
        }

        ExecuteMsg::Governance { governance_msg } => {
            let config: Config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(StdError::generic_err("unauthorized"));
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    psi_distributor_addr,
                } => commands::update_config(deps, config, psi_distributor_addr),
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::Rebalance {} => to_binary(&queries::query_rebalance(deps, env)?),
        QueryMsg::ChildContractsCodeId {} => to_binary(&queries::child_contracts_code_id(deps)?),
        QueryMsg::IsRewardsClaimable {} => to_binary(&queries::is_rewards_claimable(deps, env)?),
    }
}
