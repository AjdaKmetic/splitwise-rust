use std::collections::HashMap;
use std::cmp::Ordering;

use crate::models::debt::{Debt};
use crate::models::user::UserId;

const EPSILON: f64 = 0.01;

pub fn simplify_debts(balances: &HashMap<UserId, f64>) -> Vec<Debt> {
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