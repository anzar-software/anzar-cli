use std::env::var;

use shared::{
    config::AnzarConfiguration,
    intern::{db::DB, key::KeyService},
    utils::crypto::Crypto,
};

pub async fn start() -> KeyService {
    dotenvy::dotenv().ok();

    let env_overrides =
        config::Environment::default().source(Some(std::collections::HashMap::from([
            ("SECURITY.SECRET_KEY".into(), var("SECRET_KEY").unwrap()),
            (
                "DATABASE.CONNECTION_STRING".into(),
                var("DATABASE_URL").unwrap(),
            ),
            ("DATABASE.CACHE.URL".into(), var("CACHE_URL").unwrap()),
        ])));

    let configuration = config::Config::builder()
        .add_source(config::File::with_name("anzar.yml"))
        .add_source(env_overrides)
        .build()
        .unwrap()
        .try_deserialize::<AnzarConfiguration>()
        .unwrap();
    configuration.validate().unwrap();

    let crypto = Crypto::from_configuration(&configuration);
    let database = DB::connect(&configuration.database).await.unwrap();

    KeyService::new(&database, &crypto, &configuration)
}
