#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,StdError};
use cw2::set_contract_version;
use cw_storage_plus::Item;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg,MigrateMsg};
use crate::state::{State,STATE, Oldstate};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = "0.2";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: 0,
        owner: info.sender.clone(),
        incrementors: vec![],
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => execute::increment(deps,info),
        ExecuteMsg::Reset {} => execute::reset(deps, info),
    }
}

pub mod execute {
    use super::*;

    pub fn increment(deps: DepsMut,info: MessageInfo) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            state.incrementors.push(info.sender);
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn reset(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.count = 0;
            state.incrementors.clear();

            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count,incrementors: state.incrementors })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = cw2::get_contract_version(deps.storage)?;

    if ver.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same contract type").into());
    }

    if ver.version >= CONTRACT_VERSION.to_string(){
        return Err(StdError::generic_err("Cannot upgrade from a newer contract version").into());
    }

    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Perform state migration
    migrate_state(deps)?;

    Ok(Response::default())
}

// Function to perform the state migration
 fn migrate_state(deps: DepsMut) -> Result<(), ContractError> {
    // Read the old state from storage
    // let old_state: Oldstate = singleton_read(deps.storage, b"state").load()?;

    const COUNTER: Item<Oldstate> = Item::new("state");
    let old_state = COUNTER.load(deps.storage)?;

    // Convert old state to new format
    let new_state = State {
        count: old_state.count,
        owner: old_state.owner,
        incrementors: vec![],
    };

    // Save the new state to storage
    STATE.save(deps.storage, &new_state)?;

    Ok(())
}

