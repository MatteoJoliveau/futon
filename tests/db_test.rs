mod common;
use tracing_test::traced_test;

use common::setup;

const DB_NAME: &str = "futon-test";

#[tokio::test]
#[traced_test]
async fn it_fetches_db_info() {
    let name = format!("{DB_NAME}-{}", rand::random::<u8>());
    let futon = setup();
    let db = futon.db(&name).unwrap();

    let err = db.info().await.unwrap_err();
    assert!(err.is_not_found());

    db.create().await.unwrap();

    let info = db.info().await.unwrap();

    assert_eq!(&info.db_name, &name);

    db.delete().await.unwrap();

    let err = db.info().await.unwrap_err();
    assert!(err.is_not_found());
}
