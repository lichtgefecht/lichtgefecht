use log::{debug, info, warn};
use reflector_api::lg::BroadcastReply;

use crate::{core::InnerCore, game::state::Device, Core, CoreMessage, CreateNewSessionMsg, OutgoingMessage};

use super::System;

pub struct BroadcastSystem {
    
}

impl System<CoreMessage> for BroadcastSystem {
    fn handle(&mut self, core: &mut InnerCore, msg: &CoreMessage) {
        match msg {
            CoreMessage::BroadcastReply(hid, reply) => {
                debug!("Handling broadcast reply");
                if !core.state.devices.contains_key(hid) {
                    self.try_add_device(core, hid, reply).unwrap_or_else(|e| warn!("{}", e));
                } else {
                    //nothing?
                    info!("Ignoring duplicate broadcast reply")
                }
            },
            _=>()
        }
    }
}

impl BroadcastSystem {
    fn try_add_device(&self, core: &mut InnerCore, hid: &String, reply: &BroadcastReply) -> Result<(), &'static str> {
        info!("New device discovered: {:?}", hid);

        let client_addr: reflector_api::lg::broadcast_reply::ClientAddr = reply
            .client_addr
            .as_ref()
            .ok_or("No client addr specified")?
            .clone();

        let device = Device {
            hid: hid.clone(),
            device_type: reply.device_type,
        };
        core.state.devices.insert(hid.clone(), device);

        let msg = CreateNewSessionMsg{
            hid: hid.clone(),
            addr: client_addr.clone(),
            device_type: reply.device_type,
        };
        let out_msg = OutgoingMessage::CreateNewSession(msg);
        if let Err(e) = core.duplex.send(out_msg){
            warn!("Error sending outgoing message: {e}")
        }
        Ok(())
    }
}
