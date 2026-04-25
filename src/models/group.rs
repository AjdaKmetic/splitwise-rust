use crate::models::user::UserId;

pub type GroupId = u64;

#[derive(Debug, Clone)]
pub struct Group {
    pub id: GroupId,
    name: String,
    members: Vec<UserId>,
}

// dodaj še: metodo, ki poišče userja v podatkovni bazi, če poznamo samo njegov id

impl Group {
    pub fn new(id: GroupId, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            members: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn members(&self) -> &[UserId] {
        &self.members
    }

    pub fn update_name(&mut self, new_name: &str) {
        self.name = new_name.to_string()
    }

    pub fn add_member(&mut self, user_id: UserId) {
        if !self.contains_member(user_id) {
            self.members.push(user_id)
        }
    }

    pub fn remove_member(&mut self, user_id: UserId) {
        self.members.retain(|id| *id != user_id)
    }

    pub fn contains_member(&self, user_id: UserId) -> bool {
        self.members.contains(&user_id)
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
    use crate::models::user::User;

    #[test]
    fn test_group_new() {
        let group = Group::new(1, "Amsterdam");
        assert_eq!(group.id, 1);
        assert_eq!(group.name(), "Amsterdam");
        assert!(group.members().is_empty());
    }

    #[test]
    fn test_group_add_member() {
        let mut group = Group::new(1, "Amsterdam");
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");
        group.add_member(user);
        assert_eq!(group.member_count(), 1);
        assert!(group.contains_member(user.id));
    }

    #[test]
    fn test_group_add_member_no_duplicates() {
        let mut group = Group::new(1, "Amsterdam");
        group.add_member(1);
        group.add_member(1);
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_group_remove_member() {
        let mut group = Group::new(1, "Amsterdam");
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");
        let user_id = user.id;
        group.add_member(user_id);
        group.remove_member(user_id);
        assert_eq!(group.member_count(), 0);
        assert!(!group.contains_member(user_id));
    }

    #[test]
    fn test_group_remove_non_existing_member() {
        let mut group = Group::new(1, "Amsterdam");
        group.add_member(1);
        group.remove_member(99);
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_group_contains_member() {
        let mut group = Group::new(1, "Amsterdam");
        let user = User::new(1, "Janez Novak", "janeznovak@example.com");
        let user_id = user.id;
        group.add_member(user_id);
        assert!(group.contains_member(user_id));
        assert!(!group.contains_member(999));
    }

    #[test]
    fn test_group_members() {
        let mut group = Group::new(1, "Amsterdam");
        let user1 = User::new(1, "Janez Novak", "janeznovak@example.com");
        let user2 = User::new(2, "Marija Novak", "marijanovak@example.com");
        group.add_member(user1.id);
        group.add_member(user2.id);
        assert_eq!(group.member_count(), 2);
        assert_eq!(group.members(), &[1, 2]);
    }

    #[test]
    fn test_group_member_count() {
        let mut group = Group::new(1, "Amsterdam");
        let user1 = User::new(1, "Janez Novak", "janeznovak@example.com");
        let user2 = User::new(2, "Marija Novak", "marijanovak@example.com");
        group.add_member(user1);
        group.add_member(user2);
        assert_eq!(group.member_count(), 2);
    }

    #[test]
    fn test_group_is_empty() {
        let group = Group::new(1, "Amsterdam");
        assert!(group.is_empty());
    }

    #[test]
    fn test_group_is_not_empty_after_add() {
        let mut group = Group::new(1, "Amsterdam");

        group.add_member(1);

        assert!(!group.is_empty());
    }

    #[test]
    fn test_group_update_name() {
        let mut group = Group::new(1, "Old");

        group.update_name("New");

        assert_eq!(group.name(), "New");
    }

}
