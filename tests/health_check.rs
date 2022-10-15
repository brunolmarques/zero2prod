use std::net::TcpListener;

#[actix_web::test]
async fn helath_check_works() {
    let address = spawn_app();
    
    // 'reqwest' is used to perform HTTP requests agains the application
    let client = reqwest::Client::new();
    
    //Act
    let response = client
                    .get(&format!("{}/health_check", &address))
                    .send()
                    .await
                    .expect("Failed to execute request.");

    //Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

//Launch application in the background
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed o bind random port.");
    // Retrieve port assigned by OS
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);
    // Return application address to caller
    format!("http://127.0.0.1:{}", port)
}