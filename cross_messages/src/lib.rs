pub mod tcp;
pub use tcp::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub header: Header,
    pub body: String,
    pub tail: Tail,
}

impl Message {
    pub fn register() -> Self {
        Message {
            header: Header {
                kind: MessageKind::Register,
                target: ID::Master,
            },
            body: String::new(),
            tail: Tail {
                from: ID::Unregistered,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Header {
    pub kind: MessageKind,
    pub target: ID,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ID {
    Master,
    Unregistered,
    Slave(Uuid),
}

impl ID {
    pub fn new_slave() -> Self {
        ID::Slave(Uuid::new_v4())
    }

    pub fn from_register_reply(msg: Message) -> serde_json::Result<Self> {
        Ok(serde_json::from_str(&msg.body)?)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MessageKind {
    // Handled by Master
    Register,
    Close,
    Reply,
    GetRegDevices,
    // ----------------------
    // Bounced to the target
    // ----------------------
    Clipboard,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Tail {
    pub from: ID,
}
