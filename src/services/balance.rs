use std::collections::HashMap;
use crate::models::{
    expense::Expense,
    payment::Payment,
    user::UserId,
    group::GroupId,
};
use crate::services::split::Split;

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
            .and_modify(|v| *v += amount)
            .or_insert(-amount);

        balances.entry(to)
            .and_modify(|v| *v -= amount)
            .or_insert(amount);
    }
}

pub fn balances_with_payments(expenses: &[Expense], payments: &[Payment]) -> HashMap<UserId, f64> {
    let mut balances = calculate_balances(expenses);
    apply_payments(&mut balances, payments);
    balances
}

pub fn pairwise_balances(expenses: &[Expense], payments: &[Payment], my_id: UserId) -> HashMap<UserId, f64> {
    let mut result: HashMap<UserId, f64> = HashMap::new();

    for expense in expenses {
        let shares = expense.shares();
        let paid_by = expense.paid_by();

        if paid_by == my_id {
            for (p_id, share) in &shares {
                if *p_id == my_id {continue;}
                *result.entry(*p_id).or_insert(0.0) += share;
            }

        } else if let Some((_, my_share)) = shares.iter().find(|(p_id, _)| *p_id == my_id) {
            *result.entry(paid_by).or_insert(0.0) -= my_share;
        }
    }
    for payment in payments {
        if payment.from_id() == my_id {
            *result.entry(payment.to_id()).or_insert(0.0) += payment.amount();
        } else if payment.to_id() == my_id {
            *result.entry(payment.from_id()).or_insert(0.0) -= payment.amount();
        }
    }
    result
}

pub fn pair_debt_in_context(expenses: &[Expense], payments: &[Payment], from_id: UserId, to_id: UserId, group_filter: Option<GroupId>) -> f64 {
    let mut debt = 0.0;
    for expense in expenses {
        if expense.group_id() != group_filter {
            continue;
        }
        let amount = expense.amount();
        let paid_by = expense.paid_by();
        let shares: Vec<(UserId, f64)> = match expense.splits() {
            Split::Equal(user_ids) => {
                if user_ids.is_empty() {
                    continue;
                }
                let share = amount / user_ids.len() as f64;
                user_ids.iter().map(|&user_id| (user_id, share)).collect()
            }
            Split::Exact(pairs) => pairs.iter().cloned().collect(),
        };

        let from_share = shares.iter().find(|(p_id, _)| *p_id == from_id).map(|(_, share)| *share);
        let to_share = shares.iter().find(|(p_id, _)| *p_id == to_id).map(|(_, share)| *share);

        if paid_by == from_id {
            if let Some(to_s) = to_share {
                debt -= to_s;
            }
        } else if paid_by == to_id {
            if let Some(from_s) = from_share {
                debt += from_s;
            }
        }

    }
for payment in payments {
        if payment.group_id() != group_filter {
            continue;
        }
        if payment.from_id() == from_id && payment.to_id() == to_id {
            debt += payment.amount();
        } else if payment.from_id() == to_id && payment.to_id() == from_id {
            debt -= payment.amount();
        }
    }
    debt
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
        let pairwise = pairwise_balances(&[expense], &[payment], 1);
        let owed = pairwise.get(&2).cloned().unwrap_or(0.0);
        assert_eq!(owed, 0.0);
    }

    #[test]
    fn test_pairwise_balances_respects_exact_split() {
        let expense = Expense::new(1, "Nakup", 100.0, 1, None,
            Split::new_exact(vec![(1, 10.0), (2, 40.0), (3, 50.0)]).unwrap());
        let pairwise = pairwise_balances(&[expense], &[], 1);
        let owed_by_2 = pairwise.get(&2).cloned().unwrap_or(0.0);
        let owed_by_3 = pairwise.get(&3).cloned().unwrap_or(0.0);
        assert_eq!(owed_by_2, 40.0);
        assert_eq!(owed_by_3, 50.0);
    }

    #[test]
    fn test_pairwise_balances_from_payer() {
        let expense = Expense::new(1, "Večerja", 60.0, 2, None,
            Split::new_equal(vec![1, 2]).unwrap());
        let pairwise = pairwise_balances(&[expense], &[], 2);
        let owed_by_1 = pairwise.get(&1).cloned().unwrap_or(0.0);
        assert_eq!(owed_by_1, 30.0);
    }

    #[test]
    fn test_pairwise_balances_from_participant() {
        let expense = Expense::new(1, "Večerja", 60.0, 2, None,
            Split::new_equal(vec![1, 2]).unwrap());
        let pairwise = pairwise_balances(&[expense], &[], 1);
        let owed_to_2 = pairwise.get(&2).cloned().unwrap_or(0.0);
        assert_eq!(owed_to_2, -30.0);
    }

    #[test]
    fn group_filter_isolates_contexts() {
        let expense1 = Expense::new(1, "Večerja", 20.0, 2, Some(5),
            Split::new_equal(vec![1, 2]).unwrap());
        let expense2 = Expense::new(2, "Kosilo", 30.0, 2, None,
            Split::new_equal(vec![1, 2]).unwrap());
        let expenses = vec![expense1, expense2];

        let in_group_5 = pair_debt_in_context(&expenses, &[], 1, 2, Some(5));
        assert!((in_group_5 - 10.0).abs() < 1e-9);

        let untagged = pair_debt_in_context(&expenses, &[], 1, 2, None);
        assert!((untagged - 15.0).abs() < 1e-9);

        let other_group = pair_debt_in_context(&expenses, &[], 1, 2, Some(99));
        assert_eq!(other_group, 0.0);
    }


}
