pub type UserId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    name: String,
    email: String,
}

impl User {
    pub fn new(id: UserId, name: &str, email: &str) -> Self {
        Self { 
            id, 
            name: name.to_string(), 
            email: email.to_string() 
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn update_name(&mut self, new_name: &str) {
        self.name = new_name.to_string()
    }

    pub fn update_email(&mut self, new_email: &str) {
        self.email = new_email.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");

        assert_eq!(user.id, 1);
        assert_eq!(user.name(), "Janez Novak");
        assert_eq!(user.email(), "janeznovak@example.com");
    }

    #[test]
    fn test_user_update_name() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com");

        user.update_name("Marko Novak");

        assert_eq!(user.name(), "Marko Novak");
    }

    #[test]
    fn test_user_update_email() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com");

        user.update_email("janeznovak2@example.com");

        assert_eq!(user.email(), "janeznovak2@example.com");
    }

    #[test]
    fn test_user_id_is_unchanged() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com");

        user.update_name("Marko");
        user.update_email("marko@example.com");

        assert_eq!(user.id, 1);
    }

    #[test]
    fn test_multiple_updates() {
        let mut user = User::new(1, "Janez Novak", "janeznovak@example.com");

        user.update_name("Marko");
        user.update_email("marko@example.com");

        assert_eq!(user.name(), "Marko");
        assert_eq!(user.email(), "marko@example.com");
    }
}
