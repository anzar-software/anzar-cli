use std::net::TcpListener;
use std::sync::LazyLock;

use anzar::adapters::cache::{
    CacheAdapter,
    in_memory::InMemoryAdapter,
    memcache::{MemCache, MemCacheAdapter},
    redis::{Redis, RedisAdapter},
};
use anzar::config::{AnzarConfiguration, AppState, database::cache_driver::CacheDriver};

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
    async fn clean_cache(&self, configuration: &AnzarConfiguration) {
        match configuration.database.cache.driver {
            CacheDriver::MemCached => {
                let client = MemCache::start(&configuration.database.cache.url)
                    .await
                    .unwrap();
                let _ = MemCacheAdapter::new(client).flush_all().await;
            }
            CacheDriver::Redis => {
                let connection = Redis::start(&configuration.database.cache.url)
                    .await
                    .unwrap();
                let _ = RedisAdapter::new(connection).flush_all().await;
            }
            CacheDriver::InMemory => {
                let in_memory = InMemoryAdapter::default();
                let _ = in_memory.flush_all().await;
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
        self.clean_cache(&app_state.configuration).await;

        Ok(TestApp {
            address,
            client,
            app_state,
        })
    }
}
