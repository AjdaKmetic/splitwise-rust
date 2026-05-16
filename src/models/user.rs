pub type UserId = i32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

impl User {
    pub fn new(id: UserId, name: &str, email: &str, password_hash: &str) -> Self {
        Self { 
            id, 
            name: name.to_string(), 
            email: email.to_string(),
            password_hash: password_hash.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn update_name(&mut self, new_name: &str) {
        self.name = new_name.to_string()
    }

    pub fn update_email(&mut self, new_email: &str) {
        self.email = new_email.to_string()
    }

    pub fn update_password_hash(&mut self, new_password_hash: &str) {
        self.password_hash = new_password_hash.to_string();
    }
}

impl From<crate::entities::users::Model> for User {
    fn from(model: crate::entities::users::Model) -> Self {
        User {
            id: model.id,
            name: model.name,
            email: model.email,
            password_hash: model.password_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new(1, "Janez Novak", "janeznovak@example.com", "hashed_password");

        assert_eq!(user.id, 1);
        assert_eq!(user.name(), "Janez Novak");
        assert_eq!(user.email(), "janeznovak@example.com");
    }

    #[test]
    fn test_user_update_name() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com", "hashed_password");

        user.update_name("Marko Novak");

        assert_eq!(user.name(), "Marko Novak");
    }

    #[test]
    fn test_user_update_email() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com", "hashed_password");

        user.update_email("janeznovak2@example.com");

        assert_eq!(user.email(), "janeznovak2@example.com");
    }

    #[test]
    fn test_user_id_is_unchanged() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com", "hashed_password");

        user.update_name("Marko");
        user.update_email("marko@example.com");

        assert_eq!(user.id, 1);
    }

    #[test]
    fn test_multiple_updates() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com", "hashed_password");

        user.update_name("Marko");
        user.update_email("marko@example.com");

        assert_eq!(user.name(), "Marko");
        assert_eq!(user.email(), "marko@example.com");
    }

    #[test]
    fn test_password_hash_getter() {
        let user = User::new(
            1,
            "Janez Novak",
            "janeznovak@example.com",
            "my_hash",
        );

        assert_eq!(user.password_hash(), "my_hash");
    }

    #[test]
    fn test_empty_name_and_email() {
        let user = User::new(
            1,
            "",
            "",
            "hashed_password",
        );

        assert_eq!(user.name(), "");
        assert_eq!(user.email(), "");
    }

    #[test]
    fn test_multiple_password_hash_updates() {
        let mut user = User::new(
            1,
            "Janez Novak",
            "janeznovak@example.com",
            "hash1",
        );

        user.update_password_hash("hash2");
        user.update_password_hash("hash3");

        assert_eq!(user.password_hash(), "hash3");
    }
}
