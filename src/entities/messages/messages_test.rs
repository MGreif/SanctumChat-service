use crate::repositories::message_repository::MessageRepositoryInterface;

struct MessageRepositoryMock {}

impl MessageRepositoryInterface for MessageRepositoryMock {
    fn get_messages(&mut self, _: &String, _: &String, _: crate::helper::pagination::Pagination) -> Result<Vec<crate::models::Message>, String> {
        return Ok(vec![])
    }

    fn save_message(&mut self, _: &crate::models::Message) -> Result<(), String> {
        return Ok(())
    }

    fn set_message_read(&mut self, ids: &Vec<uuid::Uuid>, is_read: &bool, issuer: &String) -> Result<(), String> {
        return Ok(())
    }
}




pub mod message_integration_tests {
    use std::str::FromStr;

    use uuid::Uuid;

    use crate::{domain::message_domain::MessageDomain, handler::socket_handler::ws_handle_direct::SocketMessageDirect};

    use super::MessageRepositoryMock;

    #[test]
    fn test_direct_message_to_message_entity() {
        let domain = MessageDomain::new(MessageRepositoryMock {});
        
        let mut direct_message = SocketMessageDirect {
            TYPE: Some(String::from("SOCKET_MESSAGE_DIRECT")),
            message: String::from("Message"),
            id: Some(Uuid::from_str("18cb8735-b226-49d5-a726-e6937bd6e841").unwrap()),
            message_self_encrypted: String::from("Message_self encrypted"),
            message_self_encrypted_signature: String::from("Message_self encrypted signature"),
            message_signature: String::from("Message signature"),
            recipient: Some(String::from("Recipient")),
            sender: Some(String::from("Sender"))
        };

        
        let result = domain.direct_message_to_message_entity(&direct_message).unwrap();
        assert_eq!(result.content, direct_message.message);
        assert_eq!(result.content_signature, direct_message.message_signature);
        assert_eq!(result.content_self_encrypted, direct_message.message_self_encrypted);
        assert_eq!(result.content_self_encrypted_signature, direct_message.message_self_encrypted_signature);
        assert_eq!(result.recipient, direct_message.clone().recipient.unwrap());
        assert_eq!(result.sender, direct_message.clone().sender.unwrap());


        direct_message.sender = None;
        let result = domain.direct_message_to_message_entity(&direct_message.clone()).unwrap_err();
        assert_eq!(result, String::from("A sender has to be specified"));

        direct_message.sender = Some(String::from("aa"));
        direct_message.recipient = None;
        let result = domain.direct_message_to_message_entity(&direct_message.clone()).unwrap_err();
        assert_eq!(result, String::from("A recipient has to be specified"))


    }
}