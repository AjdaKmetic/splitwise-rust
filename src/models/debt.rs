use crate::models::user::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct Debt {
    from: UserId,
    to: UserId,
    amount: f64,
}

impl Debt {
    pub fn new(from: UserId, to: UserId, amount: f64) -> Result<Self, String> {
        if from == to {
            return Err("User cannot owe money to themselves".to_string())
        }

        if amount < 0.0 {
            return Err("Amount cannot be negative".to_string())
        }

        Ok(Debt {
            from,
            to, 
            amount,
        })
    }

    pub fn from(&self) -> UserId {
        self.from
    }

    pub fn to(&self) -> UserId {
        self.to
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn update_amount(&mut self, new_amount: f64) -> Result<(), String>{
        if new_amount < 0.0 {
            return Err("Amount cannot be negative".to_string())
        }

        self.amount = new_amount;
        Ok(())
    }

    pub fn is_settled(&self) -> bool {
        self.amount.abs() < 0.01
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_amount() {
        let mut debt = Debt::new(1, 2, 30.0).unwrap();

        let result = debt.update_amount(50.0);

        assert!(result.is_ok());
        assert_eq!(debt.amount(), 50.0);
    }

    #[test]
    fn test_update_amount_negative_fails() {
        let mut debt = Debt::new(1, 2, 30.0).unwrap();

        let result = debt.update_amount(-10.0);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount cannot be negative");

        assert_eq!(debt.amount(), 30.0);
    }
}