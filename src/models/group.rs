use crate::models::user::User;
use crate::models::user::UserId;

pub type GroupId = u32;

#[derive(Debug, Clone)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<User>,
}

impl Group {
    pub fn new(id: GroupId, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            members: Vec::new(),
        }
    }

// v vektor članov skupine bi shranila vse podatke o uporabniku

    pub fn add_member(&mut self, user_id: UserId) {
        if !self.contains_member(user_id) {
            self.members.push(User { id: user_id, name: String::new(), email: String::new() });
        }
    }

    pub fn remove_member(&mut self, user_id: UserId) {
        self.members.retain(|user| user.id != user_id);
    }

    pub fn contains_member(&self, user_id: UserId) -> bool {
        self.members.iter().any(|user| user.id == user_id)
    }

    pub fn members(&self) -> Vec<User> {
        self.members.clone()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_new() {
        let group = Group::new(1, "Amsterdam");
        assert_eq!(group.id, 1);
        assert_eq!(group.name, "Amsterdam");
        assert!(group.members.is_empty());
    }

    #[test]
    fn test_group_add_member() {
        let mut group = Group::new(1, "Amsterdam");
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");
        group.add_member(user.id);
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_group_remove_member() {
        let mut group = Group::new(1, "Amsterdam");
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");
        group.add_member(user.id);
        group.remove_member(user.id);
        assert_eq!(group.member_count(), 0);
    }

    #[test]
    fn test_group_contains_member() {
        let mut group = Group::new(1, "Amsterdam");
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");
        group.add_member(user.id);
        assert!(group.contains_member(user.id));
    }

    #[test]
    fn test_group_members() {
        let mut group = Group::new(1, "Amsterdam");
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");
        group.add_member(user.id);
        assert_eq!(group.members(), vec![user]);

    }

    #[test]
    fn test_group_member_count() {
        let mut group = Group::new(1, "Amsterdam");
        let user1 = User::new(1, "Janez Novak", "janeznovak@example.com");
        let user2 = User::new(2, "Marija Novak", "marijanovak@example.com");
        group.add_member(user1.id);
        group.add_member(user2.id);
        assert_eq!(group.member_count(), 2);
    }

    #[test]
    fn test_group_is_empty() {
        let group = Group::new(1, "Amsterdam");
        assert!(group.is_empty());
    }

}
