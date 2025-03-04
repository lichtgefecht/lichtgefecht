use reflector_api::lg::msg;


pub struct Core{
}

pub trait Handler
{
    fn handle(&self, core: &Core, message: &CombinedMessage);
}

pub trait MessageTrait {
}
pub struct IgnoredMessageHandler;
pub struct AnotherMessageHandler;

impl IgnoredMessageHandler {
}

pub struct MsgFoo;
pub struct MsgBar;
pub enum CombinedMessage{
    Foo(MsgFoo),
    Bar(MsgBar)
}

impl Handler for Box<dyn Handler>{
    fn handle(&self, core: &Core, message: &CombinedMessage) {
        self.as_ref().handle(core, message);
    }
}

pub struct SomeState;

impl MessageTrait for MsgFoo{}


impl Handler for IgnoredMessageHandler {
    fn handle(&self, _core: &Core, message: &CombinedMessage) {
        println!("aaa")
    }
    
}

impl Handler for AnotherMessageHandler {
    fn handle(&self, _core: &Core, message: &CombinedMessage) {
        println!("bbb")
    }
}


// fn get_handler(handlers: &Vec<Box<dyn Handler>>) -> &Box<dyn Handler>{
fn get_handler<'a>(msg: &CombinedMessage, handlers: &'a Vec<Box<dyn Handler>>) -> &'a dyn Handler{
    return match msg {
        CombinedMessage::Foo(msg_foo) => &handlers[0],
        CombinedMessage::Bar(msg_bar) => &handlers[1],
    }
    // return &handlers[0];
}

fn main() {
    let core = Core{};
    let mut handlers: Vec<Box<dyn Handler>> = Vec::new();
    handlers.push(Box::new(IgnoredMessageHandler{}));
    handlers.push(Box::new(AnotherMessageHandler{}));
    let msg = CombinedMessage::Bar(MsgBar);
    // let handler = &handlers[0];
    let handler = get_handler(&msg, &handlers);
    handler.handle(&core, &msg);
}