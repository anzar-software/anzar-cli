use redis::TypedCommands;
use std::net::TcpListener;
use std::sync::LazyLock;

use anzar::config::AppState;
use anzar::config::database::cache_driver::CacheDriver;

use anzar::config::AnzarConfiguration;

use crate::shared::TestApp;
use anzar::telemetry::{get_subscriber, init_subscriber};

pub static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber_name = "test";
    let default_filter_level = "debug";

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct Common;
impl Common {
    fn clean_cache(&self, configuration: &AnzarConfiguration) {
        match configuration.database.cache.driver {
            CacheDriver::MemCached => {
                let client =
                    memcache::Client::connect(configuration.database.cache.clone().url).unwrap();
                client.flush().unwrap();
            }
            CacheDriver::Redis => {
                let client = redis::Client::open(configuration.database.cache.clone().url).unwrap();
                let _ = client.get_connection().unwrap().flushall();
            }
        }
    }
    pub async fn spawn_app(&self) -> Result<TestApp, std::io::Error> {
        //FIXME remove hardcoded jwt token from tests
        LazyLock::force(&TRACING);

        let listener = TcpListener::bind("localhost:0").expect("Failed to random port");
        let port = listener.local_addr()?.port();
        let address = format!("http://localhost:{port}");

        let app_state = AppState::testing(&address)
            .await
            .expect("Failed to load AppState");
        let server = anzar::startup::run(listener, app_state.clone())
            .await
            .expect("Failed to bind address");

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to initiate reqwest Client");

        actix_web::rt::spawn(server);
        self.clean_cache(&app_state.configuration);

        Ok(TestApp {
            address,
            client,
            configuration: app_state.configuration,
        })
    }
}
