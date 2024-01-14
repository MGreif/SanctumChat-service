use crate::{repositories::friend_repository::FriendRepositoryInterface, models::UserDTOSanitized, helper::errors::HTTPResponse};

pub struct FriendDomain<I: FriendRepositoryInterface> {
    friend_repository: I
}

impl<I: FriendRepositoryInterface> FriendDomain<I> {
    pub fn new(friend_repository: I) -> Self {
        return Self {
            friend_repository
        }
    }

    pub fn get_friends(&mut self, username: &String) -> Result<Vec<UserDTOSanitized>, HTTPResponse<()>> {
        match self.friend_repository.get_friends(username) {
            Ok(res) => Ok(res),
            Err(err) => Err(HTTPResponse::new_internal_error(err))
        }
    }
}