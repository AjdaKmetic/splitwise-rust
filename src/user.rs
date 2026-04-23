pub type UserId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: UserId, name: String, email: String) -> Self {
        Self { id, name, email }
    }

    pub fn update_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new(1, "Janez Novak".into(), "janeznovak@example.com".into());
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Janez Novak");
        assert_eq!(user.email, "janeznovak@example.com");
    }

    #[test]
    fn test_user_update_name() {
        let mut user = User::new(1, "Janez Novak".into(), "janeznovak@example.com".into());
        user.update_name("Janez Novak".into());
        assert_eq!(user.name, "Janez Novak");
    }

    #[test]
    fn test_user_update_email() {
        let mut user = User::new(1, "Janez Novak".into(), "janeznovak@example.com".into());
        user.update_email("janeznovak2@example.com".into());
        assert_eq!(user.email, "janeznovak2@example.com");
    }
}
