use std::collections::HashMap;
use crate::models::{
    expense::Expense,
    payment::Payment,
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

pub fn apply_payments(balances: &mut HashMap<UserId, f64>, payments: &[Payment]) {
    for payment in payments {
        let from = payment.from_id();
        let to = payment.to_id();
        let amount = payment.amount();

        balances.entry(from)
            .and_modify(|v| *v -= amount)
            .or_insert(-amount);

        balances.entry(to)
            .and_modify(|v| *v += amount)
            .or_insert(amount);
    }
}

pub fn balances_with_payments(expenses: &[Expense], payments: &[Payment]) -> HashMap<UserId, f64> {
    let mut balances = calculate_balances(expenses);
    apply_payments(&mut balances, payments);
    balances
}

pub fn pairwise_balances(balances: &HashMap<UserId, f64>) -> Vec<(UserId, UserId, f64)> {
    let mut pairs = Vec::new();
    let users: Vec<_> = balances.keys().cloned().collect();

    for i in 0..users.len() {
        for j in (i + 1)..users.len() {
            let user_i = users[i];
            let user_j = users[j];
            let balance_i = balances.get(&user_i).copied().unwrap_or(0.0);
            let balance_j = balances.get(&user_j).copied().unwrap_or(0.0);
            let net_balance = balance_i - balance_j;

            if net_balance.abs() > 0.005 {
                pairs.push((user_i, user_j, net_balance));
            }
        }
    }

    pairs
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

    #[test]
    fn test_pairwise_balances() {
        let balances = HashMap::from([
            (1, 60.0),
            (2, -30.0),
            (3, -30.0)
        ]);

        let pairs = pairwise_balances(&balances);

        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&(1, 2, 90.0)));
        assert!(pairs.contains(&(1, 3, 90.0)));
    }
    #[test]
    fn test_apply_payments_basic() {
        let mut balances: HashMap<UserId, f64> = HashMap::new();
        balances.insert(1, -30.0); 
        balances.insert(2, 30.0);  
        let payment = Payment::new(1, 1, 2, 30.0, None).unwrap();
        apply_payments(&mut balances, &[payment]);
        assert_eq!(balances.get(&1), Some(&0.0));
        assert_eq!(balances.get(&2), Some(&0.0));
    }
    #[test]
    fn test_balances_with_payments_clears_debt() {
        let expense = Expense::new(1, "Večerja", 60.0, 2, None,
            Split::new_equal(vec![1, 2]).unwrap());
        let payment = Payment::new(1, 1, 2, 30.0, None).unwrap();
        let balances = balances_with_payments(&[expense], &[payment]);
        assert_eq!(balances.get(&1), Some(&0.0));
        assert_eq!(balances.get(&2), Some(&0.0));
    }

    #[test]
    fn test_balances_with_payments_partial() {
        let expense = Expense::new(1, "Večerja", 60.0, 2, None,
            Split::new_equal(vec![1, 2]).unwrap());
        let payment = Payment::new(1, 1, 2, 10.0, None).unwrap();
        let balances = balances_with_payments(&[expense], &[payment]);
        assert_eq!(balances.get(&1), Some(&-20.0));
        assert_eq!(balances.get(&2), Some(&20.0));
    }

    #[test]
    fn test_pairwise_balances_with_payment_settles() {
        let expense = Expense::new(1, "Večerja", 60.0, 2, None,
            Split::new_equal(vec![1, 2]).unwrap());
        let payment = Payment::new(1, 1, 2, 30.0, None).unwrap();
        let pairwise = pairwise_balances(&balances_with_payments(&[expense], &[payment]));
        assert!(pairwise.is_empty());
    }

    #[test]
    fn test_pairwise_balances_respects_exact_split() {
        let expense = Expense::new(1, "Nakup", 100.0, 1, None,
            Split::new_exact(vec![(1, 10.0), (2, 40.0), (3, 50.0)]).unwrap());
        let pairwise = pairwise_balances(&calculate_balances(&[expense]));
        assert!(pairwise.contains(&(1, 2, 50.0)));
        assert!(pairwise.contains(&(1, 3, 60.0)));
        assert!(pairwise.contains(&(2, 3, 10.0)));
    }

}