mod containers;
mod maven;

#[tokio::test]
async fn integration_test_xml() { 
    containers::mssql::start_container();
    containers::mssql::stop_container();
}