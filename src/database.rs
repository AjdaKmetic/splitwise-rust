use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn connect() -> Result<DatabaseConnection, sea_orm::DbErr> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL mora biti nastavljen v .env datoteki");

    Database::connect(database_url).await
}