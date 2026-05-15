use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

use crate::entities::users;

pub async fn create_user(db: &DatabaseConnection, name: &str, email: &str) -> Result<users::Model, sea_orm::DbErr> {
    let new_user = users::ActiveModel {
        name: Set(name.to_string()),
        email: Set(email.to_string()),
        ..Default::default()
    };

    new_user.insert(db).await
}