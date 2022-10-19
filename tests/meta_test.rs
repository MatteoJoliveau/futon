mod common;

#[tokio::test]
async fn it_checks_status_up() {
    tracing_subscriber::fmt::init();

    common::with_couchdb(|_name, futon| async move {
        let up = futon.meta().is_up().await?;
        assert!(up);

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn it_fetches_server_info() {
    tracing_subscriber::fmt::init();

    common::with_couchdb(|_name, futon| async move {
        let info = futon.meta().server_info().await.unwrap();

        assert_eq!(&info.couchdb, "Welcome");
        assert!(!info.uuid.is_empty());
        assert!(!info.git_sha.is_empty());
        assert!(info.version.starts_with("3.2."));
        assert_eq!(&info.vendor.name, "The Apache Software Foundation");
        assert!(info.features.contains(&"partitioned".to_string()));
        Ok(())
    })
    .await
    .unwrap();
}
