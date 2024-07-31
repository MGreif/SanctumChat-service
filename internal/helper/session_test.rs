use std::{marker::PhantomData, sync::Arc, time::Duration};

use crate::{
    appstate::IAppState,
    entities::friends::repository::{FriendDTO, IFriendRepository},
    handler::ws_handler::SocketMessage,
    helper::session::ISessionManager,
    models::UserDTO,
    persistence::connection_manager::IConnectionManager,
    AppState,
};
use axum::async_trait;
use tokio::sync::broadcast::Sender;

use crate::helper::session::ISession;

use super::jwt::Token;
use crate::tests::setup;

#[derive(Debug, Clone)]
struct MockFriendRepository {}

impl IFriendRepository for MockFriendRepository {
    fn get_friend(
        &self,
        username: &String,
        friend_name: &String,
    ) -> Result<Option<crate::models::UserDTOSanitized>, String> {
        Err(String::from("abc"))
    }
    fn get_friends(
        &self,
        username: &String,
    ) -> Result<Vec<crate::entities::friends::repository::FriendDTO>, String> {
        let mut friends = Vec::<crate::entities::friends::repository::FriendDTO>::new();
        let friend1 = FriendDTO {
            username: String::from("Friend1"),
            public_key: String::from("pub"),
            unread_message_count: 1,
        };
        friends.push(friend1);
        let friend2 = FriendDTO {
            username: String::from("Friend2"),
            public_key: String::from("pub"),
            unread_message_count: 1,
        };
        friends.push(friend2);
        return Ok(friends);
    }
}

#[derive(Debug, Clone)]
struct MockSession<F> {
    pub user: UserDTO,
    pub token: Token,
    pub phantom: PhantomData<F>,
}

#[async_trait]
impl<F: IFriendRepository + Clone> ISession<F> for MockSession<F> {
    fn get_token(&self) -> crate::helper::jwt::Token {
        return self.token.clone();
    }
    fn get_user(&self) -> crate::models::UserDTO {
        return self.user.clone();
    }
    fn get_user_socket(
        &self,
    ) -> tokio::sync::broadcast::Sender<crate::handler::ws_handler::SocketMessage> {
        return Sender::new(1);
    }
    fn new(user: crate::models::UserDTO, token: crate::helper::jwt::Token) -> Self {
        Self {
            user,
            token,
            phantom: PhantomData,
        }
    }

    async fn notify_online(&self, session_manager: &impl ISessionManager<Self, F>) {}
    async fn notify_offline(&self, app_state: &impl ISessionManager<Self, F>) {}
    async fn send_direct_message(&self, message: SocketMessage) {}
}

#[derive(Debug)]
pub struct MockConnectionManager {}

impl IConnectionManager for MockConnectionManager {
    fn get(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        String,
    > {
        Err(String::from("Its mocked, so its okay"))
    }
    fn new(env: crate::config::EnvConfig) -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        marker::PhantomData,
        ops::{Add, Sub},
        sync::Arc,
    };

    use setup::initialize_testing_environment;
    use tokio::sync::broadcast;

    use super::*;
    use crate::{
        config::ConfigManager,
        entities::friends::service::FriendDomain,
        helper::{
            jwt::{get_time_since_epoch, Token},
            session::SessionManager,
        },
    };

    #[tokio::test]
    async fn test_that_session_manager_can_add_sessions() {
        let session_manager: SessionManager<
            MockSession<MockFriendRepository>,
            MockFriendRepository,
        > = SessionManager::new(FriendDomain::new(MockFriendRepository {}));
        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .len()
                == 0
        );

        let mock_session1 = MockSession::new(
            UserDTO {
                username: String::from("Test"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: Duration::from_micros(10000),
                public_key: String::from("abc"),
                sub: String::from("Sub"),
            },
        );
        session_manager
            .insert_into_current_user_connections(mock_session1)
            .await;

        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .len()
                == 1
        );
    }

    #[tokio::test]
    async fn test_that_session_manager_can_remove_sessions() {
        let session_manager: SessionManager<
            MockSession<MockFriendRepository>,
            MockFriendRepository,
        > = SessionManager::new(FriendDomain::new(MockFriendRepository {}));

        let mock_session1 = MockSession::new(
            UserDTO {
                username: String::from("Test"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: Duration::from_micros(10000),
                public_key: String::from("abc"),
                sub: String::from("Sub"),
            },
        );
        let mock_session2 = MockSession::new(
            UserDTO {
                username: String::from("Delete-Me"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: Duration::from_micros(10000),
                public_key: String::from("abc"),
                sub: String::from("Sub"),
            },
        );

        session_manager
            .insert_into_current_user_connections(mock_session1)
            .await;

        session_manager
            .insert_into_current_user_connections(mock_session2)
            .await;

        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .len()
                == 2
        );

        session_manager
            .remove_from_current_user_connections(&String::from("Delete-Me"))
            .await
            .expect("");

        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .len()
                == 1
        );

        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .contains_key("Delete-Me")
                == false
        );
    }

    #[tokio::test]
    async fn test_that_session_manager_removes_expired_sessions() {
        initialize_testing_environment();
        let session_manager: SessionManager<
            MockSession<MockFriendRepository>,
            MockFriendRepository,
        > = SessionManager::new(FriendDomain::new(MockFriendRepository {}));

        let mock_session1 = MockSession::new(
            UserDTO {
                username: String::from("Should-Be-Removed"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: get_time_since_epoch().sub(Duration::from_secs(300)), // Invalid token, expired 5 min ago
                public_key: String::from("abc"),
                sub: String::from("Should expire"),
            },
        );
        let mock_session2 = MockSession::new(
            UserDTO {
                username: String::from("Should-Stay"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: get_time_since_epoch().add(Duration::from_secs(300)), // Valid token, expires in 5 min
                public_key: String::from("abc"),
                sub: String::from("Should stay"),
            },
        );

        session_manager
            .insert_into_current_user_connections(mock_session1)
            .await;

        session_manager
            .insert_into_current_user_connections(mock_session2)
            .await;

        session_manager
            .remove_expired_current_user_connections_sessions()
            .await;

        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .len()
                == 1
        );

        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .contains_key("Should-Be-Removed")
                == false
        );
        assert!(
            session_manager
                .get_current_user_connections()
                .lock()
                .await
                .contains_key("Should-Stay")
                == true
        );
    }

    #[tokio::test]
    async fn test_that_session_manager_gets_current_sessions_friends() {
        initialize_testing_environment();

        let session_manager: SessionManager<
            MockSession<MockFriendRepository>,
            MockFriendRepository,
        > = SessionManager::new(FriendDomain::new(MockFriendRepository {}));

        let mock_session1 = MockSession::new(
            UserDTO {
                username: String::from("User"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: Duration::from_micros(10000),
                public_key: String::from("abc"),
                sub: String::from("Sub"),
            },
        );
        let mock_session2 = MockSession::new(
            UserDTO {
                username: String::from("Friend1"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: Duration::from_micros(10000),
                public_key: String::from("abc"),
                sub: String::from("Sub"),
            },
        );

        let mock_session3 = MockSession::new(
            UserDTO {
                username: String::from("Friend2"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: Duration::from_micros(10000),
                public_key: String::from("abc"),
                sub: String::from("Sub"),
            },
        );

        let mock_session4 = MockSession::new(
            UserDTO {
                username: String::from("Random-User"),
                password: String::from("Pass"),
                public_key: Vec::<u8>::new(),
            },
            Token {
                exp: Duration::from_micros(10000),
                public_key: String::from("abc"),
                sub: String::from("Sub"),
            },
        );

        session_manager
            .insert_into_current_user_connections(mock_session1)
            .await;
        session_manager
            .insert_into_current_user_connections(mock_session2)
            .await;
        session_manager
            .insert_into_current_user_connections(mock_session3)
            .await;
        session_manager
            .insert_into_current_user_connections(mock_session4)
            .await;

        let friends = session_manager
            .get_friends_in_current_user_connections(&String::from("User"))
            .await;

        assert!(friends.len() == 2);
        assert!(friends.contains_key(&String::from("Friend1")));
        assert!(friends.contains_key(&String::from("Friend2")));
    }
}
