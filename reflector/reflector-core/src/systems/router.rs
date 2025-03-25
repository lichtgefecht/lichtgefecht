use crate::systems::{system_bc_reply::BroadcastSystem, *};

pub(crate) fn get_systems<'a, T>(
    systems: &'a mut Vec<Box<dyn System<T>>>,
    _msg: &T,
) -> Vec<&'a mut Box<dyn System<T>>> {
    return systems.iter_mut().collect();
}

pub(crate) fn get_registrations() -> Vec<Box<dyn System<crate::CoreMessage>>> {
    vec![
        Box::new(BroadcastSystem {}),
        Box::new(IgnoredMessageHandler {}),
        Box::new(InfraSystem {}),
        Box::new(HitMessageHandler {}),
    ]
}
