use std::{sync::Arc, time::Duration};

use crate::{
    appstate::IAppState, handler::ws_handler::SocketMessage,
    persistence::connection_manager::IConnectionManager, AppState,
};
use axum::async_trait;
use tokio::sync::broadcast::Sender;

use crate::helper::session::ISessionManager;

#[derive(Debug, Clone)]
struct MockSessionManager {
    a: u8,
}

#[async_trait]
impl ISessionManager for MockSessionManager {
    fn get_token(&self) -> crate::helper::jwt::Token {
        return crate::helper::jwt::Token {
            exp: Duration::new(1000, 0),
            public_key: String::from("abc"),
            sub: String::from("Sub"),
        };
    }
    fn get_user(&self) -> crate::models::UserDTO {
        return crate::models::UserDTO {
            password: String::from("Pass"),
            username: String::from("User"),
            public_key: vec![{ '\x11' as u8 }],
        };
    }
    fn get_user_socket(
        &self,
    ) -> tokio::sync::broadcast::Sender<crate::handler::ws_handler::SocketMessage> {
        return Sender::new(1);
    }
    fn new(user: crate::models::UserDTO, token: crate::helper::jwt::Token) -> Self {
        return Self { a: '\x11' as u8 };
    }

    async fn notify_online(&self, app_state: &impl IAppState<MockSessionManager>) {}
    async fn notify_offline(&self, app_state: &impl IAppState<MockSessionManager>) {}
    async fn send_direct_message(&self, message: SocketMessage) {}
}

#[derive(Debug)]
struct MockConnectionManager {}

impl IConnectionManager for MockConnectionManager {
    fn get(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        String,
    > {
        Err(String::from("Its mocked, so its okay"))
    }
}

mod tests {
    use std::{collections::HashMap, sync::Arc};

    use tokio::sync::broadcast;

    use super::*;
    use crate::config::ConfigManager;

    #[tokio::test]
    async fn test_app_state_setup() {
        let current_user_connections: HashMap<String, Arc<tokio::sync::Mutex<MockSessionManager>>> =
            HashMap::new();
        let app_state = AppState {
            broadcast: broadcast::Sender::new(1),
            config: ConfigManager::new(),
            current_user_connections: Arc::new(tokio::sync::Mutex::new(current_user_connections)),
            connection_manager: MockConnectionManager {},
        };
    }
}
