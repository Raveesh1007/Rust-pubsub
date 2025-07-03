use serde:: {Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};

#[derive(PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]

pub enum Message{
    Subscribe{
        topic: Vec<String>,
    },
    Unsubscribe{
        topic: Vec<String>,        
    },
    Publish{
        topic: Vec<String>,
        data: String,
        key: Option<String>,
    },
}


#[derive(PartialEq, Eq, Hash, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PubSubResponse{
    Message{topic: String, message: String},
    Error{ error: String},
}

pub fn process_subscription_message(message: impl ToString) -> serde_json::Result<PubSubResponse>{
    serde_json::from_str(&message.to_string())
}

pub fn send_message(topic: &impl ToString, message: impl ToString, sender: &broadcast::Sender<String>) 
-> Result<(), Box<dyn std:: error:: Error>>{
    let message = PubSubResponse::Message{
        topic: topic.to_string(),
        message: message.to_string(),
    };

    let message = serde_json::to_string(&message)?;
    sender.send(message)?;
    Ok(())
}

pub async fn send_error(message: impl ToString, sender: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>>{
    let message = PubSubResponse::Error{
        message: message.to_string(),
    };
    let message = serde_json::to_string(&message)?;
    sender.send(message)?;
    Ok(())
}
