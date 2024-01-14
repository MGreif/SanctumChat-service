use axum::http::StatusCode;
use uuid::Uuid;

use crate::{repositories::friend_request_repository::FriendRequestRepositoryInterface, helper::errors::HTTPResponse, handler::friend_handler::FriendRequestGETResponseDTO, models::FriendRequest};

pub struct FriendRequestDomain<I: FriendRequestRepositoryInterface> {
    pub friend_request_repository: I
}

impl<I: FriendRequestRepositoryInterface> FriendRequestDomain<I> {

    pub fn new(friend_request_repository: I) -> Self {
        return FriendRequestDomain {
            friend_request_repository
        }
    }

    pub fn get_friend_requests_for_user(&mut self, username: &String) -> Result<Vec<FriendRequestGETResponseDTO>, HTTPResponse<()>> {
        let result = self.friend_request_repository.get_friend_requests_sent_to_user(username);
        match result {
            Err(err) => Err(HTTPResponse::new_internal_error(err)),
            Ok(res) => Ok(res)
        }
    }


    pub fn create_friend_request(&mut self, sender: &String, recipient: &String) -> Result<FriendRequest, HTTPResponse<()>> {
        if recipient == sender {
            return Err(HTTPResponse::<()> {
                status: StatusCode::BAD_REQUEST,
                data: None,
                message: Some(format!("You cannot send yourself a friend request"))
            })
        }
        
        let is_present = match self.friend_request_repository.check_if_friend_request_is_present(&sender, &recipient, None) {
            Ok(res) => res,
            Err(err) => return Err(HTTPResponse::new_internal_error(err)) 
        };

        if is_present {
            return Err(HTTPResponse::<()> {
                data: None,
                message: Some(format!("You already sent a friend request to {}", sender)),
                status: StatusCode::BAD_REQUEST
            })
        };


        let new_request = FriendRequest {
            id: uuid::Uuid::new_v4(),
            accepted: None,
            recipient: recipient.clone(),
            sender: sender.clone()
        };

        match self.friend_request_repository.save_friend_request(new_request.clone()) {
            Err(err) => Err(HTTPResponse::new_internal_error(err)),
            Ok(_) => Ok(new_request) 
        }
    }


    pub fn accept_or_deny_friend_request(&mut self, friend_request_id: &Uuid, recipient: &String, accepted: bool) -> Result<(), HTTPResponse<()>> {
        match self.friend_request_repository.update_friend_request_accepted(friend_request_id, &recipient, accepted) {
            Ok(_) => Ok(()),
            Err(err) => Err(HTTPResponse::new_internal_error(err)) 
        }
    }

    
}