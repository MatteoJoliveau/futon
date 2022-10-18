use futon::Futon;

pub fn setup() -> Futon {
    dotenv::dotenv().ok();
    Futon::new(
        std::env::var("COUCHDB_URL")
            .expect("envar COUCHDB_URL not found")
            .parse::<url::Url>()
            .unwrap(),
    )
}
