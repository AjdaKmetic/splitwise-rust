use std::cmp::Ordering;
use crate::services::{
    balance::Balance,
};

use crate::models::debt::{Debt};
use crate::models::user::UserId;

const EPSILON: f64 = 0.01;

pub fn simplify_debts(balances: &Balance) -> Vec<Debt> {
    let mut transactions = Vec::new();
    let mut debtors_and_creditors: Vec<(UserId, f64)> = balances.iter()
        .filter(|&(_, &amount)| amount != 0.0)
        .map(|(&user, &amount)| (user, amount))
        .collect();
    while debtors_and_creditors.len() > 1 {
        debtors_and_creditors.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then(a.0.cmp(&b.0))
        });

        let creditor_index = 0;
        let debtor_index = debtors_and_creditors.len() - 1;

        let creditor_amount = debtors_and_creditors[creditor_index].1;
        let debtor_amount = debtors_and_creditors[debtor_index].1;

        if creditor_amount <= EPSILON && creditor_amount >= -EPSILON {
            break;
        }

        let payment = creditor_amount.min(-debtor_amount);
        let from = debtors_and_creditors[debtor_index].0;
        let to = debtors_and_creditors[creditor_index].0;

        if let Ok(debt) = Debt::new(from, to, payment) {
            transactions.push(debt);
        }

        debtors_and_creditors[creditor_index].1 -= payment;
        debtors_and_creditors[debtor_index].1 += payment;

        debtors_and_creditors.retain(|&(_, amount)| amount.abs() > EPSILON);
    }

    transactions

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_empty_balances() {
        let balances = Balance::new();
        let debts = simplify_debts(&balances);
        assert!(debts.is_empty());
    }

    #[test]
    fn test_simplify_all_zero_balances() {
        let mut balances = Balance::new();
        balances.insert(1, 0.0);
        balances.insert(2, 0.0);
        let debts = simplify_debts(&balances);
        assert!(debts.is_empty());
    }

    #[test]
    fn test_simplify_two_users() {
        let mut balances = Balance::new();
        balances.insert(1, 50.0);
        balances.insert(2, -50.0);
        let debts = simplify_debts(&balances);
        assert_eq!(debts.len(), 1);
        assert_eq!(debts[0].from(), 2);
        assert_eq!(debts[0].to(), 1);
        assert_eq!(debts[0].amount(), 50.0);
    }

    #[test]
    fn test_simplify_one_creditor_two_debtors() {
        let mut balances = Balance::new();
        balances.insert(1, 100.0);
        balances.insert(2, -30.0);
        balances.insert(3, -70.0);
        let debts = simplify_debts(&balances);
        assert_eq!(debts.len(), 2);
        assert_eq!(debts[0].from(), 3);
        assert_eq!(debts[0].to(), 1);
        assert_eq!(debts[0].amount(), 70.0);
        assert_eq!(debts[1].from(), 2);
        assert_eq!(debts[1].to(), 1);
        assert_eq!(debts[1].amount(), 30.0);
    }

    #[test]
    fn test_simplify_eliminates_chains() {
        let mut balances = Balance::new();
        balances.insert(1, 50.0);
        balances.insert(2, 0.0);
        balances.insert(3, -50.0);
        let debts = simplify_debts(&balances);
        assert_eq!(debts.len(), 1);
        assert_eq!(debts[0].from(), 3);
        assert_eq!(debts[0].to(), 1);
        assert_eq!(debts[0].amount(), 50.0);
    }

    #[test]
    fn test_simplify_at_most_n_minus_one_debts() {
        let mut balances = Balance::new();
        balances.insert(1, 100.0);
        balances.insert(2, -30.0);
        balances.insert(3, -20.0);
        balances.insert(4, -50.0);
        let debts = simplify_debts(&balances);
        assert!(debts.len() <= 3);
    }

    #[test]
    fn test_simplify_matched_amounts() {
        let mut balances = Balance::new();
        balances.insert(1, 50.0);
        balances.insert(2, -50.0);
        balances.insert(3, 50.0);
        balances.insert(4, -50.0);
        let debts = simplify_debts(&balances);
        assert_eq!(debts.len(), 2);
        let total_amount: f64 = debts.iter().map(|d| d.amount()).sum();
        assert_eq!(total_amount, 100.0);
    }

    #[test]
    fn test_simplify_preserves_total_amount() {
        let mut balances = Balance::new();
        balances.insert(1, 100.0);
        balances.insert(2, -30.0);
        balances.insert(3, -20.0);
        balances.insert(4, -50.0);
        let debts = simplify_debts(&balances);
        let total_amount: f64 = debts.iter().map(|d| d.amount()).sum();
        assert_eq!(total_amount, 100.0);
    }

    #[test]
    fn test_simplify_handles_small_amounts() {
        let mut balances = Balance::new();
        balances.insert(1, 0.005);
        balances.insert(2, -0.005);
        let debts = simplify_debts(&balances);
        assert!(debts.is_empty());
    }
}

