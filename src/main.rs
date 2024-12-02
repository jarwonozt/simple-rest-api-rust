use actix_web::{web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::{MySqlPoolOptions, MySqlRow}, MySqlPool};
use sqlx::Row;
use std::env;

// Struct untuk JSON request
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id:u32,
    name: String,
    email: String,
}

// Struct untuk JSON response
#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

async fn get_users(pool: web::Data<MySqlPool>) -> impl Responder {
    let query = "SELECT id, name, email FROM users";
    let result = sqlx::query(query)
        .map(|row: MySqlRow| User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
        })
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(users) => web::Json(users),
        Err(err) => {
            eprintln!("Error fetching users: {}", err);
            web::Json(Vec::<User>::new()) // Mengembalikan array kosong jika ada error
        }
    }
}

// Handler untuk menyimpan data user
async fn create_user(
    pool: web::Data<MySqlPool>,
    user: web::Json<CreateUser>,
) -> impl Responder {
    let query = "INSERT INTO users (name, email) VALUES (?, ?)";
    let result = sqlx::query(query)
        .bind(&user.name)
        .bind(&user.email)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => web::Json(ApiResponse {
            success: true,
            message: "User created successfully".to_string(),
        }),
        Err(err) => web::Json(ApiResponse {
            success: false,
            message: format!("Failed to create user: {}", err),
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create database connection pool
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8090")?
    .run()
    .await
}
