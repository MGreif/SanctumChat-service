use core::time;
use std::sync::Arc;

use crate::config::AppState;

pub fn initialize_session_cleanup_schedule(app_state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            app_state
                .remove_expired_current_user_connections_sessions()
                .await;
        }
    });
}
