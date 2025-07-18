use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::Message;
use axum::extract::{ws::WebSocket, State};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::stream::SplitStream;
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task::JoinSet;

use crate::pubsub::message::{send_error, send_message};

use super::message::PubSubRequest;
use super::process_subscription_message;
use super::subscriber::subscribe;


#[derive(Deserialize)]
pub struct PubSubState{
    publisher_key: Option<String>,
    
    pub topics: Mutex<HashMap<String, broadcast::Sender<String>>>,

    pub subsciptions: Mutex<HashMap<String, HashMap<SocketAddr, tokio::task::JoinHandle<()>>>>,
}


impl Default for PubSubState{
    fn default() -> Self{
        Self{
            publisher_key: None,
            topics: Mutex::new(HashMap::new()),
            subsciption: Mutex::new(HashMap::new()),
        }
    }
}


impl PubSubState{
    pub fn with_publisher_key(self, publisher_key: Option<String>) -> Self{
        Self{
            publisher_key,
            ..Default::default()
        }
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<PubSubState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>) -> 
    impl IntoResponse{
        ws.on_upgrade(move |socket| webscoket(socket,state, socket_address))
    }

    async fn websocket(stream: WebSocket, state: Arc<PubSubState>, socket_address: Socketaddr){
        let (mut sink, stream) = stream.split();
        let (sender, mut receiver) = mpsc::channel::<String>(1000);

        let mut socket_join_set = JoinSet::new();


        socket_join_set.spawn(async move{
            while let Some(message) = reciever.recv().await{
                if let Err(e) = sink.send(Message:: Text(message)).await{
                    tracing::trace!("Error sending message to client: {}", e);
                    break;
                }
            }
        });


        let recv_task_sender = sender.clone();
        let state_clone = state.clone();
        socket_join_set.spawn(listen_for_messages(
            stream,
            state_clone,
            sender,
            socket_address,
            recv_task_sender,
        ));


        socket_join_set.join_next().await;
        socket_join_set.abort_all();

    tracing::trace!("websocket closed for {}", socket_address);
    }


    async fn listen_for_messages(
        mut stream: SplitStream<WebSocket>,
        state_clone: Arc<PubSubState>,
        sender: mpsc::Sender<String>,
        socket_address: SocketAddr,
        recv_task_sender: mpsc::Sender<String>,
    ) {
        while let Some(Ok(Message::Text(text))) = stream.next().await {
            match process_subscription_message(text) {
                Ok(result) => {
                    tracing::info!("received message from {}: {:?}", socket_address, result);
    
                    handle_message(result, state_clone.clone(), sender.clone(), socket_address).await;
                }
                Err(e) => {
                    tracing::trace!("error parsing message from {}: {}", socket_address, e);
                    if send_error(e.to_string(), recv_task_sender.clone())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    }

    
    async fn handle_message(
        result: PubSubResponse,
        state: Arc<PubSubState>,
        sender: mpsc::Sender<String>,
        socket_address: SocketAddr,
    ) {
        let sender = sender.clone();

        match result{
            PubSubRequest::Subscribe{topics} => {
                for topic in topics{
                    tracing::info!("Subscribing {} to: {}", socket_address, topic);

                    let receiver = get_create_topic(&topic, &state).await;
                    let routine = tokio::spawn(subscribe(receiver, sender.clone()));
                    let mut current_subscriptions = state.subscriptions.lock().await;
                    match current_subscriptions.get_mut(&topic){
                        Some(subscription) => {
                            subscription.inser(socket_address, routine);
                        }
                        None => {
                            let mut new_subscriptions = HashMap::new();
                            new_subscriptions.insert(socket_address, routine);
                            current_subscriptions.insert(topic, new_subscriptions);
                        }
                    }
                }
            }

            PubSubRequest::Unsubscribe(topics) => {
                for topic in topics{
                    let mut current_subscriptions = state.subscription.lock().await;
                    match current_subscription.get_mut(&topic){
                        Some(topics) => {
                            if let Some(routine) = topics.remove(&socket_address){
                                tracing::info!("Unsubscribing {} from: {}", socket_address, topic);
                                routine.abort();
                                topics.remove(&socket_address);
                            }


                            if topics.is_empty(){
                                tracing::trace!("deleting topic: {} since there are no more subscribers", &topic);
                                current_subscriptions.remove(&topic);
                            }
                        }


                        None => {
                            tracing::trace!("{} tried to unsubscribe from non-existent topic: {}", socket_address, topic);
                            continue;
                        }
                    }
                }
            }

            PubSubRequest::Publish{topic, message,key} => {
                if key != state.publisher_key{
                    tracing::trace!("invalid publisher key: {}", key);
                    if send_error("invalid publisher key", sender).await.is_err(){
                        tracing::trace!("error sending error message to {}", socket_address);
                    }
                    return;
                }


                for topic in topics{
                    match state.topics.lock().await.get(&topic){
                        Some(transceiver) => {
                            tracing::info!("publishing message to topic: {}", topic);
                        
                            if send_message(&topic, &message, &sender).await.is_err(){
                                tracing::trace("error sending message to topic: {}", topic)
                            }
                        }
                        None => {
                            tracing::trace!("topic {} does not have any subscribers", topic);
                        }
                    };
                }
            }

        }
    } 



    async fn get_or_create_topic_channel(
        state: &mut Arc<PubSubState>,
        topic: String,
    ) -> broadcast::Receiver<String> {
        let mut topics = state.topics.lock().await;
    
        match topics.get(&topic) {
            Some(tx) => tx.subscribe(),
            None => {
                tracing::trace!("creating new topic {}", topic);
    
                let (tx, _rx) = broadcast::channel(1000);
                topics.insert(topic, tx.clone());
                tx.subscribe()
            }
        }
    }