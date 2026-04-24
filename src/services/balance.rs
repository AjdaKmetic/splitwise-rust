use std::collections::HashMap;
use crate::models::{
    expense::Expense, 
    user::UserId
};

pub fn calculate_balances(expenses: &[Expense]) -> HashMap<UserId, f64> {
    let mut balances: HashMap<UserId, f64> = HashMap::new();

    for expense in expenses {
        let payer = expense.paid_by();
        let amount = expense.amount();
        balances.entry(payer)
        .and_modify(|v| *v += amount)
        .or_insert(amount);

        for (participant, share) in expense.shares() {
            balances.entry(participant)
            .and_modify(|v| *v -= share)
            .or_insert(-share);
        }
    }

    balances
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::expense::Expense;
    use crate::services::split::Split;

    #[test]
    fn test_single_expense_equal_split() {
        let expense = Expense::new(
            1,
            "Večerja",
            90.0,
            1,
            None,
            Split::new_equal(vec![1, 2, 3]).unwrap(),
        );

        let balances = calculate_balances(&[expense]);

        assert_eq!(balances.get(&1), Some(&60.0));
        assert_eq!(balances.get(&2), Some(&-30.0));
        assert_eq!(balances.get(&3), Some(&-30.0));
    }

    #[test]
    fn test_single_expense_exact_split() {
        let expense = Expense::new(
            1,
            "Nakup",
            90.0,
            1,
            None,
            Split::new_exact(vec![(1, 10.0), (2, 30.0), (3, 50.0)]).unwrap(),
        );

        let balances = calculate_balances(&[expense]);

        assert_eq!(balances.get(&1), Some(&80.0));
        assert_eq!(balances.get(&2), Some(&-30.0));
        assert_eq!(balances.get(&3), Some(&-50.0));
    }

    #[test]
    fn test_multiple_expenses() {
        let expense1 = Expense::new(
            1,
            "Večerja",
            90.0,
            1,
            None,
            Split::new_equal(vec![1, 2, 3]).unwrap(),
        );

        let expense2 = Expense::new(
            2,
            "Kosilo",
            60.0,
            2,
            None,
            Split::new_equal(vec![1, 2]).unwrap(),
        );

        let balances = calculate_balances(&[expense1, expense2]);

        assert_eq!(balances.get(&1), Some(&30.0));
        assert_eq!(balances.get(&2), Some(&0.0));
        assert_eq!(balances.get(&3), Some(&-30.0));
    }
}