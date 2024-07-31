use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

use axum::async_trait;
use diesel::{
    r2d2::{self},
    PgConnection,
};
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    config::ConfigManager,
    entities::friends::{repository::IFriendRepository, service::FriendDomain},
    handler::ws_handler::SocketMessageNotification,
    helper::{
        jwt::check_token_expiration,
        session::{ISession, ISessionManager, SessionManager},
        sql::get_friends_for_user_from_db,
    },
    persistence::connection_manager::IConnectionManager,
};

#[async_trait]
pub trait IAppState<F: IFriendRepository, SM: ISessionManager<S, F>, S: ISession<F>>:
    Debug + Send + Sync
{
    fn get_db_pool(&self) -> r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>;
    fn get_session_manager(&self) -> &SM;
    fn get_config(&self) -> ConfigManager;
}

#[derive(Debug)]
pub struct AppState<SM, S, C, F>
where
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    C: IConnectionManager,
    F: IFriendRepository,
{
    pub connection_manager: C,
    pub broadcast: broadcast::Sender<String>,
    // Hashmap of currently logged in users
    pub current_user_connections: SM,
    pub config: ConfigManager,
    pub phantom1: PhantomData<S>,
    pub phantom2: PhantomData<F>,
}

unsafe impl<SM: ISessionManager<S, F>, F: IFriendRepository, S: ISession<F>, C: IConnectionManager>
    Send for AppState<SM, S, C, F>
{
}
unsafe impl<SM: ISessionManager<S, F>, F: IFriendRepository, S: ISession<F>, C: IConnectionManager>
    Sync for AppState<SM, S, C, F>
{
}

impl<S: ISession<F>, SM: ISessionManager<S, F>, F: IFriendRepository, C: IConnectionManager>
    AppState<SM, S, C, F>
{
    pub fn new(cm: C, config: ConfigManager, session_manager: SM) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        AppState {
            connection_manager: cm,
            broadcast: tx,
            config: config,
            current_user_connections: session_manager,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
}

#[async_trait]
impl<SM: ISessionManager<S, F>, F: IFriendRepository, S: ISession<F>, C: IConnectionManager>
    IAppState<F, SM, S> for AppState<SM, S, C, F>
{
    fn get_config(&self) -> ConfigManager {
        self.config.clone()
    }
    fn get_session_manager(&self) -> &SM {
        &self.current_user_connections
    }
    fn get_db_pool(&self) -> r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>> {
        self.connection_manager
            .get()
            .expect("Could not get db pool")
    }
}
