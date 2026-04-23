use crate::{split::Split, user::UserId, group::GroupId};

#[derive(Debug, Clone)]
pub struct Expense {
    pub id: ExpenseId,
    pub description: String,
    pub amount: f64,
    pub paid_by: UserId,
    pub group_id: GroupId,
    pub splits: Split,
}

pub type ExpenseId = u64;

impl Expense {
    pub fn new(id: ExpenseId, description: String, amount: f64, paid_by: UserId, group_id: GroupId, splits: Split) -> Self {
        Self {
            id,
            description,
            amount,
            paid_by,
            group_id,
            splits,
        }
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

    #[test]
    fn test_expense_participants() {
        let expense = Expense::new(
            1,
            "Test Expense".into(),
            90.0,
            1,
            1,
            Split::Equal(vec![1, 2, 3])
        );
        assert_eq!(expense.participants(), vec![1, 2, 3]);
    }

    #[test]
    fn test_expense_shares() {
        let expense = Expense::new(
            1,
            "Test Expense".into(),
            90.0,
            1,
            1,
            Split::Equal(vec![1, 2, 3])
        );
        assert_eq!(expense.shares(), vec![(1, 30.0), (2, 30.0), (3, 30.0)]);
    }
}
