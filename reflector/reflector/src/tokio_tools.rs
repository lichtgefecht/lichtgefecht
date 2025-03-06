use std::sync::mpsc::{RecvError, SendError};

use reflector_core::{api::transport::Duplex, CoreMessage, OutgoingMessage};

pub fn duplex_pair() -> (
    TokioDuplex<OutgoingMessage, CoreMessage>,
    TokioDuplex<CoreMessage, OutgoingMessage>,
) {
    let (txa, rxa) = tokio::sync::mpsc::channel(512);
    let (txb, rxb) = tokio::sync::mpsc::channel(512);
    (
        TokioDuplex { tx: txa, rx: rxb },
        TokioDuplex { tx: txb, rx: rxa },
    )
}

pub struct TokioDuplex<T, R> {
    tx: tokio::sync::mpsc::Sender<T>,
    rx: tokio::sync::mpsc::Receiver<R>,
}
impl<T, R> TokioDuplex<T, R> {
    pub fn crack(self) -> (tokio::sync::mpsc::Sender<T>, tokio::sync::mpsc::Receiver<R>) {
        (self.tx, self.rx)
    }
}

/**
 * Blocking implementation to interface with the tokio types from outside the async runtime.
 * This is located in this crate, so the core crate does not need to be aware of tokio at all.
 */
impl<T, R> Duplex<T, R> for TokioDuplex<T, R> {
    fn send(&self, t: T) -> Result<(), std::sync::mpsc::SendError<T>> {
        self.tx.blocking_send(t).map_err(|e| SendError(e.0))
    }

    fn recv(&mut self) -> Result<R, std::sync::mpsc::RecvError> {
        self.rx.blocking_recv().ok_or(RecvError)
    }
}
