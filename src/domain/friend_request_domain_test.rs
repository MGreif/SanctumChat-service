use std::str::FromStr;

use crate::{repositories::friend_request_repository::FriendRequestRepositoryInterface, handler::friend_handler::FriendRequestGETResponseDTO};

struct FriendRequestRepositoryMock {}

 
impl FriendRequestRepositoryInterface for FriendRequestRepositoryMock {
    fn check_if_friend_request_is_present(&mut self, sender: &String, _: &String, _: Option<bool>) -> Result<bool, String> {
        
        if sender == "exists" {
            return Ok(true)
        } else if sender == "error" {
            return Err(String::from("Error checking"))
        } else {
            return Ok(false)
        }
    }

    fn get_friend_requests_sent_to_user(&mut self, usern: &String) -> Result<Vec<crate::handler::friend_handler::FriendRequestGETResponseDTO>, String> {
        if usern == "error2" {
            return Err(String::from("Some Error 2"))
        } else {
            return Ok(vec![FriendRequestGETResponseDTO {
                accepted: Some(true),
                id: uuid::Uuid::from_str("18cb8735-b226-49d5-a726-e6937bd6e841").expect("error"),
                recipient: usern.to_owned(),
                sender_id: String::from("awd")
            }])
        }
    }

    fn save_friend_request(&mut self, friend_request: crate::models::FriendRequest) -> Result<(), String> {
        if friend_request.recipient == "error3" {
            return Err(String::from("Error saving"))
        }

        return  Ok(());
    }

    fn update_friend_request_accepted(&mut self, friend_request_id: &uuid::Uuid, recipient: &String, accepted: bool) -> Result<(), String> {
        if friend_request_id.to_string() == uuid::Uuid::from_str("18cb8735-b226-49d5-a726-e6937ed6e841").expect("error").to_string() {
            return Err(String::from("Error updating"))
        }

        return  Ok(());
    }
}



pub mod friend_request_integration_tests {
    use std::str::FromStr;

    use axum::http::StatusCode;

    use crate::{domain::friend_request_domain::FriendRequestDomain, helper::errors::HTTPResponse};

    use super::FriendRequestRepositoryMock;

    #[test]
    fn test_create_friend_request() {
        let repo = FriendRequestRepositoryMock {};
        let mut domain = FriendRequestDomain::new(repo);
        let sender = String::from("Sender");
        let recipient = String::from("Sender");
        let result = domain.create_friend_request(&sender, &recipient).err().unwrap();
        let expected = HTTPResponse::<()> {
            data: None,
            message: Some(format!("You cannot send yourself a friend request")),
            status: StatusCode::BAD_REQUEST
        };
        assert_eq!(expected, result);


        let sender = String::from("exists");
        let recipient = String::from("SomeUser");
        let result = domain.create_friend_request(&sender, &recipient).err().unwrap();
        let expected = HTTPResponse::<()> {
            data: None,
            message: Some(format!("You already sent a friend request to SomeUser")),
            status: StatusCode::BAD_REQUEST
        };
        assert_eq!(expected, result);


        let sender = String::from("error");
        let recipient = String::from("SomeUser");
        let result = domain.create_friend_request(&sender, &recipient).err().unwrap();
        let expected = HTTPResponse::<()> {
            data: None,
            message: Some(format!("Error checking")),
            status: StatusCode::INTERNAL_SERVER_ERROR
        };
        assert_eq!(expected, result);

        let sender = String::from("SomeSender");
        let recipient = String::from("error3");
        let result = domain.create_friend_request(&sender, &recipient).err().unwrap();
        let expected = HTTPResponse::<()> {
            data: None,
            message: Some(format!("Error saving")),
            status: StatusCode::INTERNAL_SERVER_ERROR
        };
        assert_eq!(expected, result);
    }
}

