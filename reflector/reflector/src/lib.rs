use std::sync::mpsc::{RecvError, SendError};

use reflector_api::lg::Msg;
use reflector_core::{Duplex, MsgWithTarget};

pub mod transport;

pub fn duplex_pair() -> (TokioDuplex<MsgWithTarget, Msg>, TokioDuplex<Msg,MsgWithTarget>) {
    let (txa,rxa) = tokio::sync::mpsc::channel(512);
    let (txb,rxb) = tokio::sync::mpsc::channel(512);
    (
        TokioDuplex{ tx: txa, rx: rxb },
        TokioDuplex{ tx: txb, rx: rxa },

    )
}

pub struct TokioDuplex<T,R>{
    tx: tokio::sync::mpsc::Sender<T>,
    rx: tokio::sync::mpsc::Receiver<R>
}
impl<T,R> TokioDuplex<T,R> {
    fn crack(self) -> (tokio::sync::mpsc::Sender<T>, tokio::sync::mpsc::Receiver<R>) {
        (self.tx, self.rx)
    }
}

impl<T,R> Duplex<T,R> for TokioDuplex<T,R>{
    fn send(&self, t: T) -> Result<(), std::sync::mpsc::SendError<T>>  {
        self.tx.blocking_send(t).map_err(|e| 
            SendError(e.0)
        )
    }

    fn recv(&mut self) -> Result<R, std::sync::mpsc::RecvError> {
        self.rx.blocking_recv().ok_or(RecvError)
    }
}
