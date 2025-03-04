
// #[macro_use]
// extern crate reflector_macros;
// use reflector_macros::MyTrait;

pub struct Core{
}

pub trait Handler<M>
where
    M: MessageTrait,
{
    fn handle(&self, core: &Core, message: &M);
}

pub trait MessageTrait { }


// TODO Create proc macro
// #[derive(MyTrait)]
// #[my_trait(MsgFoo, MsgBar)]
pub struct LoggingSystem;

pub struct AnotherSystem;


pub struct MsgFoo;
pub struct MsgBar;

impl MessageTrait for CombinedMessage{}
pub enum CombinedMessage{
    Foo(MsgFoo),
    Bar(MsgBar)
}

pub struct SomeState;

impl MessageTrait for MsgFoo{}
impl MessageTrait for MsgBar{}


macro_rules! handle {
    [{$s:ident,$c:ident,$m:ident} $($x:path)* ] => { 
       match $m {
            $(
                $x (msg) => $s.handle($c, msg),
            )*
            _ => {}
        }
    };
}

impl Handler<CombinedMessage> for LoggingSystem {
    fn handle(&self, core: & Core, message: & CombinedMessage) {
        handle![{self, core, message}
            CombinedMessage::Foo
            CombinedMessage::Bar
            ];
    }    
}

impl Handler<CombinedMessage> for AnotherSystem {
    fn handle(&self, core: & Core, message: & CombinedMessage) {
        handle![{self, core, message}]
    }    
}

impl Handler<MsgFoo> for LoggingSystem {
    fn handle(&self, _core: & Core, message: & MsgFoo) {
        println!("3")
    }    
}

impl Handler<MsgBar> for LoggingSystem {
    fn handle(&self, _core: & Core, message: & MsgBar) {
        println!("4")
    }
}




fn main() {
    let core = Core{};
    
    let systems : Vec<Box<dyn Handler<CombinedMessage>>> = vec![
        Box::new(LoggingSystem{}),
        Box::new(AnotherSystem{})
            ];
    

    let msg = CombinedMessage::Foo(MsgFoo);

    // either via router
    let handler = match msg {
        CombinedMessage::Foo(_) => &systems[0],
        CombinedMessage::Bar(_) => &systems[1],
    };
    handler.handle(&core, &msg);


    // or all systems get all messages
    for s in &systems{
        s.handle(&core, &msg);
    }


}