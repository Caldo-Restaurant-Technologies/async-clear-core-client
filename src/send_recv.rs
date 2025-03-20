use crate::controller::Message;
use log::{debug, error};
use std::future::Future;
use tokio::sync::{mpsc, oneshot};

pub(crate) trait SendRecv {
    fn get_sender(&self) -> &mpsc::Sender<Message>;
    fn write(&self, buffer: &[u8]) -> impl Future<Output = Vec<u8>>
    where
        Self: Sync,
    {
        async {
            let (resp_tx, resp_rx) = oneshot::channel();
            let msg = Message {
                buffer: buffer.to_vec(),
                response: resp_tx,
            };
            debug!("Sending msg: {:?}", msg);
            if let Err(e) = self.get_sender().send(msg).await {
                error!("Send error: {:?}", e);
            }
            resp_rx.await.expect("No MSG from client")
        }
    }
}
