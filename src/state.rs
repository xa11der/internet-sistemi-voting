use cosmwasm_std::{Addr};
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Map, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Poll {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub owner: Addr,
    pub participants: Vec<Participant>,
    pub is_open: bool,
    pub start_time: String,
    pub end_time: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Participant {
    pub id: u64,
    pub poll_id: u64,
    pub name: String,
    pub votes: u32,
}

// State
pub const POLLS: Map<u64, Poll> = Map::new("polls");
pub const PARTICIPANTS: Map<u64, Participant> = Map::new("participants");
pub const VOTER_STATE: Map<(u64, Addr), bool> = Map::new("voter_state");
pub const POLL_COUNTER: Item<u64> = Item::new("poll_counter");
pub const PARTICIPANT_COUNTER: Item<u64> = Item::new("participant_counter");

