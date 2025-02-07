use axum::{
    routing::{get, post},
    Router,
    Json
};
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/users/login", post(authentication));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Serialize)]
struct User {
    email: String,
    token: String,
    username: String,
    bio: String,
    image: Option<String>
}

#[derive(Debug, Serialize)]
struct ResponseUser {
    user: User
}

#[derive(Debug, Deserialize)]
struct Authentication {
    user: AuthenticationUser
}

#[derive(Debug, Deserialize)]
struct AuthenticationUser {
    email: String,
    password: String
}

async fn authentication(Json(authenticate): Json<Authentication>) -> Json<ResponseUser> {
    println!("{:#?}", authenticate);

    Json(ResponseUser {
        user: User {
            email: String::from("jake@jake.jake"),
            token: String::from("jwt.token.here"),
            username: String::from("jake"),
            bio: String::from("I work at statefarm"),
            image: None
        }
    })
}
