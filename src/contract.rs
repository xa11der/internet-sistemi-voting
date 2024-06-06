use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError};
use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
use crate::state::{Poll, Participant, POLLS, PARTICIPANTS, VOTER_STATE, POLL_COUNTER, PARTICIPANT_COUNTER};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    POLL_COUNTER.save(deps.storage, &0)?;
    PARTICIPANT_COUNTER.save(deps.storage, &0)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {    
        ExecuteMsg::CreatePoll  { name, description, duration } => create_poll( deps, info, env, name, description, duration),
        ExecuteMsg::AddParticipant  { poll_id, name } => add_participant( deps, info, poll_id, name),
        ExecuteMsg::Vote  { poll_id, participant_id } => vote( deps, info, poll_id, participant_id),
        ExecuteMsg::ClosePoll  { poll_id } => close_poll( deps, info, poll_id),
    }
}

pub fn create_poll(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    name: String,
    description: String, 
    duration: Option<u64>
) -> StdResult<Response> {
    let mut counter = POLL_COUNTER.load(deps.storage)?;
    counter += 1;

    let start_time = OffsetDateTime::from_unix_timestamp(env.block.time.seconds() as i64)
        .map_err(|_| StdError::generic_err("Invalid start time"))?
        .format(&Rfc3339)
        .map_err(|_| StdError::generic_err("Failed to format start time"))?;
    
    let end_time = duration
        .map(|d| {
            OffsetDateTime::from_unix_timestamp((env.block.time.seconds() as i64) + (d as i64))
                .map_err(|_| StdError::generic_err("Invalid end time"))
                .and_then(|dt| dt.format(&Rfc3339).map_err(|_| StdError::generic_err("Failed to format end time")))
        })
        .transpose()?;
    
    let new_poll = Poll {
        id: counter,
        name,
        description,
        owner: info.sender,
        participants: vec![],
        is_open: true,
        start_time,
        end_time,
    };
    
    POLLS.save(deps.storage, counter, &new_poll)?;
    POLL_COUNTER.save(deps.storage, &counter)?;

    Ok(Response::new()
        .add_attribute("method", "create_poll"))
}

pub fn add_participant(
    deps: DepsMut,
    info: MessageInfo,
    poll_id: u64,
    name: String
) -> StdResult<Response> {
    let mut participant_counter = PARTICIPANT_COUNTER.load(deps.storage)?;
    participant_counter += 1;

    let mut poll = POLLS.load(deps.storage, poll_id)?;
    
    if info.sender != poll.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if !poll.is_open {
        return Err(StdError::generic_err("Poll is closed, cannot add participants"))
    }

    let participant = Participant {
        id: participant_counter,
        poll_id,
        name,
        votes: 0,
    };

    PARTICIPANTS.save(deps.storage, participant_counter, &participant)?;

    poll.participants.push(participant);
    POLLS.save(deps.storage, poll_id, &poll)?;

    PARTICIPANT_COUNTER.save(deps.storage, &participant_counter)?;
    
    Ok(Response::new()
        .add_attribute("method", "add_participant"))
}

pub fn vote(
    deps: DepsMut,
    info: MessageInfo,
    poll_id: u64,
    participant_id: u64
) -> StdResult<Response> {
    let poll = POLLS.load(deps.storage, poll_id)?;
    
    if !poll.is_open {
        return Err(StdError::generic_err("Poll is closed"));
    }

    if info.sender == poll.owner {
        return Err(StdError::generic_err("Owner cannot vote in this poll"));
    }

    let voter_key = (poll_id, info.sender.clone());
    let has_voted = VOTER_STATE.may_load(deps.storage, voter_key.clone())?;
    
    if has_voted.unwrap_or(false) {
        return Err(StdError::generic_err("Voter has already voted in this poll"));
    }

    let mut participant = PARTICIPANTS.load(deps.storage, participant_id)?;
    if participant.poll_id != poll_id {
        return Err(StdError::generic_err("Participant does not belong to this poll"));
    }

    participant.votes += 1;
    PARTICIPANTS.save(deps.storage, participant_id, &participant)?;
    VOTER_STATE.save(deps.storage, voter_key, &true)?;

    Ok(Response::new().add_attribute("method", "vote").add_attribute("voter", info.sender.to_string()))
}

pub fn close_poll(
    deps: DepsMut,
    info: MessageInfo,
    poll_id: u64
) -> StdResult<Response> {
    let mut poll = POLLS.load(deps.storage, poll_id)?;
    if info.sender != poll.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if !poll.is_open {
        return Err(StdError::generic_err("Poll is already closed"));
    }

    poll.is_open = false;
    POLLS.save(deps.storage, poll_id, &poll)?;

    Ok(Response::new().add_attribute("method", "close_poll"))
}

#[entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg
) -> StdResult<Binary> {
    
    match msg {
        QueryMsg::GetPoll { poll_id } => to_json_binary(&query_poll(deps, poll_id)?),
        QueryMsg::GetPollResults { poll_id } => to_json_binary(&query_poll_results(deps, poll_id)?),
    }
}

fn query_poll(deps: Deps, poll_id: u64) -> StdResult<Poll> {
    let poll = POLLS.load(deps.storage, poll_id)?;
    let participants_details: Vec<Participant> = poll.participants
        .iter()
        .map(|participant| PARTICIPANTS.load(deps.storage, participant.id))
        .collect::<StdResult<Vec<_>>>()?;

    let poll_info = Poll {
        id: poll.id,
        name: poll.name,
        description: poll.description,
        owner: poll.owner,
        is_open: poll.is_open,
        start_time: poll.start_time,
        end_time: poll.end_time,
        participants: participants_details,
    };

    Ok(poll_info)
}

pub fn query_poll_results(
    deps: Deps,
    poll_id: u64
) -> StdResult<Poll> {
    let poll = POLLS.load(deps.storage, poll_id)?;
    if poll.is_open {
        return Err(StdError::generic_err("Poll is still open"));
    }

    let participants_details: Vec<Participant> = poll.participants
        .iter()
        .map(|participant| PARTICIPANTS.load(deps.storage, participant.id))
        .collect::<StdResult<Vec<_>>>()?;

    let poll_info = Poll {
        id: poll.id,
        name: poll.name,
        description: poll.description,
        owner: poll.owner,
        is_open: poll.is_open,
        start_time: poll.start_time,
        end_time: poll.end_time,
        participants: participants_details,
    };

    Ok(poll_info)
}

