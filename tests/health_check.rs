use sqlx::{PgConnection, Connection};
use zero2prod::configuration::get_configuration;

use std::net::TcpListener;
use zero2prod::startup::run;

//Spin up an instance of our application
// and returns its address (i.e. http://localhost:XXXX)
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed o bind random port.");
    // Retrieve port assigned by OS
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);
    // Return application address to caller
    format!("http://127.0.0.1:{}", port)
}

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

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration file");
    let connection_string = configuration.database.connection_string();
    // The `Connection` trait must be in scope in order to be invoked
    // 'PgConnection::connect` - it's not an inherent method of the struct!
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions",&app_address))
        .header("Content-Type","application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200,response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
                    .fetch_one(&mut connection)
                    .await
                    .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
        ];
        
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions",&app_address))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
            
        // Assert
        assert_eq!(400,response.status().as_u16(),
        // Additional customised error message on test failure
        "The API did not fail with 400 Bad Request when the payload was {}.",
        error_message
        );
    }
}