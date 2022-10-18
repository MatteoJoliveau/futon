mod common;
use common::setup;

#[tokio::test]
async fn it_checks_status_up() {
    let futon = setup();
    let up = futon.meta().is_up().await.unwrap();
    assert!(up);
}

#[tokio::test]
async fn it_fetches_server_info() {
    let futon = setup();
    let info = futon.meta().server_info().await.unwrap();

    assert_eq!(&info.couchdb, "Welcome");
    assert!(!info.uuid.is_empty());
    assert!(!info.git_sha.is_empty());
    assert!(info.version.starts_with("3.2."));
    assert_eq!(&info.vendor.name, "The Apache Software Foundation");
    assert!(info.features.contains(&"partitioned".to_string()));
}
