use sea_orm::{
    ActiveModelTrait, 
    ColumnTrait, 
    DatabaseConnection, 
    EntityTrait, 
    QueryFilter, 
    Set
};
use users::Entity as Users;
use crate::{
    entities::users, 
    models::user::UserId
};
use super::auth_service::{
    hash_password,
    verify_password
};


// sign up
pub async fn create_user(db: &DatabaseConnection, name: &str, email: &str, password: &str) -> Result<users::Model, sea_orm::DbErr> {
    let password_hash = hash_password(password);
    let new_user = users::ActiveModel {
        name: Set(name.to_string()),
        email: Set(email.to_string()),
        password_hash: Set(password_hash),
        ..Default::default()
    };

    new_user.insert(db).await
}

pub async fn get_all_users(db: &DatabaseConnection) -> Result<Vec<users::Model>, sea_orm::DbErr> {

    Users::find()
        .all(db)
        .await
}

pub async fn find_user_by_id(db: &DatabaseConnection, id: UserId) -> Result<Option<users::Model>, sea_orm::DbErr> {

    Users::find_by_id(id)
        .one(db)
        .await
}

pub async fn find_user_by_email(db: &DatabaseConnection, email: &str) -> Result<Option<users::Model>, sea_orm::DbErr> {

    Users::find()
        .filter(users::Column::Email.eq(email))
        .one(db)
        .await
}

// log in
pub async fn login_user(db: &DatabaseConnection, email: &str, password: &str) -> Result<Option<users::Model>, sea_orm::DbErr> {
    // poišči userja po emailu
    let user = find_user_by_email(db, email).await;

    // če user obstaja, preveri password (vrni user ali None)
    match user {
        Ok(Some(user)) => {
            if verify_password(password, &user.password_hash) {
               return Ok(Some(user));
            } else {
                return Ok(None);
            }
        }

        Ok(None) => {
            return Ok(None);
        }

        Err(error) => {
            return Err(error);
        }
    }
}

