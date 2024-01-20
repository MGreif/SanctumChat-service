use std::time::SystemTime;

use uuid::Uuid;

use crate::{repositories::message_repository::MessageRepositoryInterface, models::Message, helper::{errors::HTTPResponse, pagination::Pagination}, handler::socket_handler::{ws_handle_direct::SocketMessageDirect, ws_receive_handler::SocketMessageError}};



pub struct MessageDomain<I: MessageRepositoryInterface> {
    message_repository: I
}

impl<I: MessageRepositoryInterface> MessageDomain<I> {
    pub fn new(message_repository: I) -> Self {
        return Self {
            message_repository
        }
    }

    pub fn get_messages(&mut self, username: &String, origin: &String, pagination: Pagination) -> Result<Vec<Message>, HTTPResponse<()>> {
        match self.message_repository.get_messages(username, origin, pagination) {
            Ok(res) => Ok(res),
            Err(err) => Err(HTTPResponse::new_internal_error(err))
        }
    }

    pub fn direct_message_to_message_entity(&self, direct_message: &SocketMessageDirect) -> Result<Message, String> {
        let direct_message = direct_message.clone();
        
        let sender = match direct_message.sender {
            None => return Err(String::from("A sender has to be specified")),
            Some(sender) => sender
        };

        let recipient = match direct_message.recipient {
            None => return Err(String::from("A recipient has to be specified")),
            Some(recipient) => recipient
        };
    
        
        let message_db = Message {
            content: direct_message.message,
            content_signature: direct_message.message_signature,
            content_self_encrypted: direct_message.message_self_encrypted,
            content_self_encrypted_signature: direct_message.message_self_encrypted_signature,
            id: direct_message.id.unwrap_or(Uuid::new_v4()),
            recipient: recipient,
            is_read: false,
            sender: sender,
            sent_at: SystemTime::now()
        };

        return Ok(message_db)
    }

    pub fn save_message(&mut self, message: &Message) -> Result<(), SocketMessageError> {
        let result = self.message_repository.save_message(message);
        match result {
            Err(err) => return Err(SocketMessageError::new(err)),
            Ok(res) => Ok(res)
        }
    }

    pub fn set_message_read(&mut self, message_ids: &Vec<Uuid>, is_read: &bool, issuer: &String) -> Result<(), String> {
        let result = self.message_repository.set_message_read(message_ids, is_read, issuer);
        match result {
            Err(err) => return Err(err),
            Ok(res) => Ok(res)
        }
    }
}