use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ExecuteMsg {
    CreatePoll {
        name: String,
        description: String,
        duration: Option<u64>,
    },
    AddParticipant {
        poll_id: u64,
        name: String,
    },
    Vote {
        poll_id: u64,
        participant_id: u64,
    },
    ClosePoll {
        poll_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum QueryMsg {
    GetPoll {
        poll_id: u64,
    },
    GetPollResults {
        poll_id: u64,
    },
}
