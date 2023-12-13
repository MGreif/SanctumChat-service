use std::{sync::Arc, time::SystemTime};

use async_trait::async_trait;
use uuid::Uuid;
use diesel::prelude::*;
use crate::{config::AppState, utils::jwt::Token, models, schema::messages, handler::ws_handler::SocketMessage};

use super::ws_receive_handler::{Receivable, SocketMessageError};


#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SocketMessageDirect {
    pub recipient: Option<String>,
    pub sender: Option<String>,
    pub message: String,
    pub message_signature: String,
    pub message_self_encrypted: String,
    pub message_self_encrypted_signature: String,
    pub TYPE: Option<String>
}

impl SocketMessageDirect {
    pub fn new(sender: Option<String>, recipient: Option<String>, message: String, message_signature: String, message_self_encrypted: String, message_self_encrypted_signature: String) -> SocketMessageDirect {
        SocketMessageDirect { 
            message,
            message_signature,
            message_self_encrypted,
            message_self_encrypted_signature,
            recipient,
            sender,
            TYPE: Some(String::from("SOCKET_MESSAGE_DIRECT"))
         }
    }
}

#[async_trait]
impl Receivable for SocketMessageDirect {
    async fn handle_receive (&self, app_state: Arc<AppState>, token: Token) -> Result<(), super::ws_receive_handler::SocketMessageError> {
        let recipient = match &self.recipient {
            None => return Err(SocketMessageError::new(String::from("No recipient specified"))),
            Some(r) => r
        };
        
        // Get fresh connection to get latest state
        let client_session = app_state.p2p_connections.lock().await.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await.clone();
    
        let message = SocketMessageDirect::new(
            Some(token.sub.clone()),
            Some(recipient.clone()),
            self.message.clone(),
            self.message_signature.clone(),
            self.message_self_encrypted.clone(),
            self.message_self_encrypted_signature.clone()
        );
    
        let message_clone = message.clone();
        // Save message in db
        let message_db = models::Message {
            content: message_clone.message,
            content_signature: message_clone.message_signature,
            content_self_encrypted: message_clone.message_self_encrypted,
            content_self_encrypted_signature: message_clone.message_self_encrypted_signature,
            id: Uuid::new_v4(),
            recipient: recipient.clone(),
            sender: token.sub.clone(),
            sent_at: SystemTime::now()
        };
    
    
        let mut pool = app_state.db_pool.get().expect("Could not get db connection to db to save sent message");
        diesel::insert_into(messages::table).values(&message_db).execute(&mut pool).expect(format!("Could not save message {:?}", &message_db).as_str());
    
        client_session.send_direct_message(SocketMessage::SocketMessageDirect(message.clone())).await;
    
        let p2p = app_state.p2p_connections.lock().await.clone();
        let recipient_session_manager = p2p.get(recipient).clone();
        match recipient_session_manager {
            None => {},
            Some(sm) => {
                sm.lock().await.send_direct_message(SocketMessage::SocketMessageDirect(message.clone())).await;
                
            }
        }
        return Ok(())
    }
}
