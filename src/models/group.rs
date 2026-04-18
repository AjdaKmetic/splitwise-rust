use crate::models::expense::Expense;
use crate::models::user::User;

#[derive(Debug, Clone)]
pub struct Group {
    pub id: u32,
    pub name: String,
    pub members: Vec<User>,
    pub expenses: Vec<Expense>,
}

impl Group {
    pub fn new(id: u32, name: String) -> Self {
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

    pub fn add_expense(&mut self, expense: Expense) {
        self.expenses.push(expense);
    }

}

