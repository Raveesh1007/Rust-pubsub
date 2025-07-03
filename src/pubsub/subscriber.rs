use tokio::sync::{broadcast::Reciever, mpsc::Sender};

pub async fn subscribe(mut receiver: Receiver<String>, sender: Sender<String>){
    while let Ok(message) = receiver.recv().await{
        if sender.send(message).await.is_err(){
            break;
        }
    }
}