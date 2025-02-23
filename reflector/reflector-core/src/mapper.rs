use reflector_api::lg::{msg::Inner, Msg};

use crate::handlers::*;

pub fn to_message_handler(msg: Msg) -> Box<dyn MessageHandler> {
    match msg {
        Msg {
            inner: Some(Inner::Broadcast(_)),
            ..
        } => IgnoredMessageHandler::new(),
        Msg {
            hid,
            inner: Some(Inner::BroadcastReply(broadcast_reply)),
            ..
        } => BroadcastReplyHandler::new(hid, broadcast_reply),
        i => UnimplementedMessageHandler::new(format!("{i:?}")),
    }
}
