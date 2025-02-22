use log::{debug, warn};
use reflector_api::lg::BroadcastReply;

use crate::{Core, Device, MessageHandler};

pub struct BroadcastReplyHandler {
    msg: BroadcastReply,
    hid: String,
}
impl BroadcastReplyHandler {
    pub fn new(hid: String, msg: BroadcastReply) -> Box<Self> {
        Box::new(BroadcastReplyHandler { msg, hid })
    }
}
impl MessageHandler for BroadcastReplyHandler {
    fn handle(&self, core: &mut Core) {
        debug!("Handling broadcast reply");
        if !core.state.devices.contains_key(&self.hid) {
            self.try_add_device(core).unwrap_or_else(|e| warn!("{}", e));
        } else {
            //nothing?
        }
    }
}

impl BroadcastReplyHandler {
    fn try_add_device(&self, core: &mut Core) -> Result<(), &'static str> {
        warn!("New device discovered: {:?}", self.hid);

        let client_addr = self
            .msg
            .client_addr
            .as_ref()
            .ok_or("No client addr specified")?
            .clone();

        let device = Device {
            hid: self.hid.clone(),
            device_type: self.msg.device_type,
            client_addr: client_addr.clone(),
        };
        core.state.devices.insert(self.hid.clone(), device);
        core.handle.add_address_entry(self.hid.clone(), client_addr);
        Ok(())
    }
}
