use crate::expense::Expense;
use crate::user::User;
use crate::user::UserId;

pub type GroupId = u32;

#[derive(Debug, Clone)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<User>,
    pub expenses: Vec<Expense>,
}

impl Group {
    pub fn new(id: GroupId, name: String) -> Self {
        Self {
            id,
            name,
            members: Vec::new(),
            expenses: Vec::new(),
        }
    }

    pub fn add_member(&mut self, user: User) {
        self.members.push(user);
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
    
    pub fn add_expense(&mut self, expense: Expense) {
        self.expenses.push(expense);
    }

}

