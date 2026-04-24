use crate::models::{
    user::UserId,
    group::GroupId,
};
use crate::services::split::Split;

pub type ExpenseId = u64;

#[derive(Debug, Clone)]
pub struct Expense {
    pub id: ExpenseId,
    description: String,
    amount: f64,
    paid_by: UserId,
    group_id: Option<GroupId>, 
    splits: Split,
}

impl Expense {
    pub fn new(id: ExpenseId, description: &str, amount: f64, paid_by: UserId, group_id: Option<GroupId>, splits: Split) -> Self {
        Self {
            id,
            description: description.to_string(),
            amount,
            paid_by,
            group_id,
            splits,
        }
    }

    pub fn paid_by(&self) -> UserId {
        self.paid_by
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn group_id(&self) -> Option<GroupId> {
        self.group_id
    }

    pub fn splits(&self) -> &Split {
        &self.splits
    }

    pub fn update_description(&mut self, new_description: &str) {
        self.description = new_description.to_string()
    }

    pub fn update_amount(&mut self, new_amount: f64) {
        self.amount = new_amount
    }

    pub fn update_paid_by(&mut self, new_paid_by: UserId) {
        self.paid_by = new_paid_by
    }

    pub fn update_splits(&mut self, new_splits: Split) {
        self.splits = new_splits
    }

    pub fn assign_to_group(&mut self, group_id: GroupId) {
        self.group_id = Some(group_id)
    }

    pub fn remove_from_group(&mut self) {
        self.group_id = None
    }

    pub fn is_group_expense(&self) -> bool {
        self.group_id.is_some()
    }

    pub fn participants(&self) -> Vec<UserId> {
        self.splits.participants()
    }

    pub fn shares(&self) -> Vec<(UserId, f64)> {
        self.splits.compute_shares(self.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::User;

    fn test_user() -> User {
        User::new(1, "Nika Novak", "nika.novak@example.com")
    }

    #[test]
    fn test_expense_new() {
        let user = test_user();

        let expense = Expense::new(
            1,
            "Testni strošek",
            90.0,
            user.id,
            Some(1),
            Split::Equal(vec![1, 2, 3]),
        );

        assert_eq!(expense.id, 1);
        assert_eq!(expense.description(), "Testni strošek");
        assert_eq!(expense.amount(), 90.0);
        assert_eq!(expense.paid_by(), 1);
        assert_eq!(expense.group_id(), Some(1));
        assert!(expense.is_group_expense());
    }

    #[test]
    fn test_expense_participants() {
        let user = test_user();

        let expense = Expense::new(
            1,
            "Testni strošek",
            90.0,
            user.id,
            Some(1),
            Split::Equal(vec![1, 2, 3]),
        );

        assert_eq!(expense.participants(), vec![1, 2, 3]);
    }

    #[test]
    fn test_expense_shares_equal_split() {
        let user = test_user();

        let expense = Expense::new(
            1,
            "Testni strošek",
            90.0,
            user.id,
            Some(1),
            Split::Equal(vec![1, 2, 3]),
        );

        assert_eq!(expense.shares(), vec![(1, 30.0), (2, 30.0), (3, 30.0)]);
    }

    #[test]
    fn test_expense_shares_exact_split() {
        let user = test_user();

        let expense = Expense::new(
            1,
            "Testni strošek",
            90.0,
            user.id,
            Some(1),
            Split::Exact(vec![(1, 20.0), (2, 30.0), (3, 40.0)]),
        );

        assert_eq!(expense.shares(), vec![(1, 20.0), (2, 30.0), (3, 40.0)]);
    }

    #[test]
    fn test_expense_without_group() {
        let user = test_user();

        let expense = Expense::new(
            1,
            "Kava",
            12.0,
            user.id,
            None,
            Split::Equal(vec![1, 2]),
        );

        assert_eq!(expense.group_id(), None);
        assert!(!expense.is_group_expense());
    }

    #[test]
    fn test_update_description() {
        let user = test_user();

        let mut expense = Expense::new(
            1,
            "Stari opis",
            90.0,
            user.id,
            Some(1),
            Split::Equal(vec![1, 2, 3]),
        );

        expense.update_description("Nov opis");

        assert_eq!(expense.description(), "Nov opis");
    }

    #[test]
    fn test_update_amount() {
        let user = test_user();

        let mut expense = Expense::new(
            1,
            "Testni strošek",
            90.0,
            user.id,
            Some(1),
            Split::Equal(vec![1, 2, 3]),
        );

        expense.update_amount(120.0);

        assert_eq!(expense.amount(), 120.0);
        assert_eq!(expense.shares(), vec![(1, 40.0), (2, 40.0), (3, 40.0)]);
    }

    #[test]
    fn test_update_paid_by() {
        let user = test_user();

        let mut expense = Expense::new(
            1,
            "Testni strošek",
            90.0,
            user.id,
            Some(1),
            Split::Equal(vec![1, 2, 3]),
        );

        expense.update_paid_by(2);

        assert_eq!(expense.paid_by(), 2);
    }

    #[test]
    fn test_update_splits() {
        let user = test_user();

        let mut expense = Expense::new(
            1,
            "Testni strošek",
            90.0,
            user.id,
            Some(1),
            Split::Equal(vec![1, 2, 3]),
        );

        expense.update_splits(Split::Equal(vec![1, 2]));

        assert_eq!(expense.participants(), vec![1, 2]);
        assert_eq!(expense.shares(), vec![(1, 45.0), (2, 45.0)]);
    }

    #[test]
    fn test_assign_to_group() {
        let user = test_user();

        let mut expense = Expense::new(
            1,
            "Kava",
            12.0,
            user.id,
            None,
            Split::Equal(vec![1, 2]),
        );

        expense.assign_to_group(5);

        assert_eq!(expense.group_id(), Some(5));
        assert!(expense.is_group_expense());
    }

    #[test]
    fn test_remove_from_group() {
        let user = test_user();

        let mut expense = Expense::new(
            1,
            "Kava",
            12.0,
            user.id,
            Some(5),
            Split::Equal(vec![1, 2]),
        );

        expense.remove_from_group();

        assert_eq!(expense.group_id(), None);
        assert!(!expense.is_group_expense());
    }
}
