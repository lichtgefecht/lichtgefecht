use reflector_api::lg::{broadcast_reply::ClientAddr, BroadcastReply, Msg};

use crate::systems::MessageTrait;

#[derive(Debug)]
pub enum CoreMessage {
    BroadcastReply(String, BroadcastReply),
    Shutdown,
    UnknownMessage,
    TargetHit(String, i32)
}

impl MessageTrait for CoreMessage {}

impl From<Msg> for CoreMessage {
    fn from(value: Msg) -> Self {
        let msg = value.inner.unwrap(); // todo
        match msg {
            reflector_api::lg::msg::Inner::Broadcast(_) => CoreMessage::UnknownMessage,
            reflector_api::lg::msg::Inner::BroadcastReply(broadcast_reply) => {
                CoreMessage::BroadcastReply(value.hid, broadcast_reply)
            }
            reflector_api::lg::msg::Inner::TargetHit(hit_msg) => CoreMessage::TargetHit(value.hid, hit_msg.from_id),
        }
    }
}

#[derive(Debug)]
pub enum OutgoingMessage {
    CreateNewSession(CreateNewSessionMsg),
    MsgWithTarget(MsgWithTarget),
}

#[derive(Debug)]
pub struct CreateNewSessionMsg {
    pub hid: String,
    pub addr: ClientAddr,
    pub device_type: i32,
}

#[derive(Debug)]
pub struct MsgWithTarget {
    pub target_hid: String,
    pub msg: Msg,
}

impl MessageTrait for OutgoingMessage {}
