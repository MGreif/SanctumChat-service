use crate::{
    appstate::{AppState, IAppState},
    persistence::connection_manager::IConnectionManager,
};
use core::time;
use std::sync::Arc;

use crate::helper::session::ISessionManager;

pub fn initialize_session_cleanup_schedule<S: ISessionManager, C: IConnectionManager>(
    app_state: Arc<AppState<S, C>>,
) {
    let app_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            app_state
                .remove_expired_current_user_connections_sessions()
                .await
        }
        let x = 10;
    });
}
