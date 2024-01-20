use crate::{repositories::{friend_repository::{FriendRepositoryInterface, FriendDTO}, message_repository::MessageRepositoryInterface}, models::UserDTOSanitized, helper::errors::HTTPResponse};

pub struct FriendDomain<I: FriendRepositoryInterface> {
    friend_repository: I
}


impl<I: FriendRepositoryInterface> FriendDomain<I> {
    pub fn new(friend_repository: I) -> Self {
        return Self {
            friend_repository
        }
    }

    pub fn check_if_user_has_friend(&mut self, username: &String, friend_name: &String) -> Result<bool, String> {
        let has_friend = match self.friend_repository.get_friend(username, friend_name) {
            Ok(res) => res,
            Err(err) => return Err(err)
        };
        
        match has_friend {
            Some(_) => Ok(true),
            None => Ok(false)
        }
    }

    pub fn get_friends(&mut self, username: &String) -> Result<Vec<FriendDTO>, String> {
        match self.friend_repository.get_friends(username) {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }
}