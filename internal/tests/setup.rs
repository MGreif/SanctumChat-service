use std::sync::{self, Once};

static UNIT_TEST_TRACK: Once = Once::new();

pub fn initialize_testing_environment() {
    UNIT_TEST_TRACK.call_once(|| tracing_subscriber::fmt().json().init())
}
