use super::System;
use crate::{core::InnerCore, message::CoreMessage};
use log::{debug, info, warn};
use reflector_api::lg::BroadcastReply;

pub struct HitMessageHandler;

impl System<CoreMessage> for HitMessageHandler {
    fn handle(&mut self, _core: &mut InnerCore, msg: &CoreMessage) {
        match msg {
            CoreMessage::TargetHit(hid, from_id) => {
                info!("HitMessageHandler : {hid} got hit by {from_id}");
            },
            _ => {}
        }
    }
}
