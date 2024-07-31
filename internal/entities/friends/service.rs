use crate::entities::friends::repository::{FriendDTO, IFriendRepository};

#[derive(Debug)]
pub struct FriendDomain<I: IFriendRepository> {
    friend_repository: I,
}

impl<I: IFriendRepository> FriendDomain<I> {
    pub fn new(friend_repository: I) -> Self {
        return Self { friend_repository };
    }

    pub fn check_if_user_has_friend(
        &self,
        username: &String,
        friend_name: &String,
    ) -> Result<bool, String> {
        let has_friend = match self.friend_repository.get_friend(username, friend_name) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match has_friend {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn get_friends(&self, username: &String) -> Result<Vec<FriendDTO>, String> {
        match self.friend_repository.get_friends(username) {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }
}
