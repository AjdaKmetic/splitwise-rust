use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::{user::UserId, group::GroupId};

pub type PaymentId = u64;

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

#[derive(Debug, Clone)]
pub struct Payment {
    pub id: PaymentId,
    from_id: UserId,
    to_id: UserId,
    amount: f64,
    group_id: Option<GroupId>,
    created_at: u64,
}

impl Payment {
    pub fn new(
        id: PaymentId,
        from_id: UserId,
        to_id: UserId,
        amount: f64,
        group_id: Option<GroupId>,
    ) -> Result<Self, String> {
        Self::with_timestamp(id, from_id, to_id, amount, group_id, now_ms())
    }

    pub fn with_timestamp(
        id: PaymentId,
        from_id: UserId,
        to_id: UserId,
        amount: f64,
        group_id: Option<GroupId>,
        created_at: u64,
    ) -> Result<Self, String> {
        if amount <= 0.0 {
            return Err("Payment amount must be positive".to_string());
        }
        if from_id == to_id {
            return Err("Sender and recipient cannot be the same".to_string());
        }
        Ok(Self {
            id,
            from_id,
            to_id,
            amount,
            group_id,
            created_at,
        })
    }

    pub fn from_id(&self) -> UserId { self.from_id }
    pub fn to_id(&self) -> UserId { self.to_id }
    pub fn amount(&self) -> f64 { self.amount }
    pub fn group_id(&self) -> Option<GroupId> { self.group_id }
    pub fn created_at(&self) -> u64 { self.created_at }

    pub fn update_amount(&mut self, new_amount: f64) -> Result<(), String> {
        if new_amount <= 0.0 {
            return Err("Payment amount must be positive".to_string());
        }
        self.amount = new_amount;
        Ok(())
    }

    pub fn assign_to_group(&mut self, group_id: GroupId) {
        self.group_id = Some(group_id);
    }

    pub fn remove_from_group(&mut self) {
        self.group_id = None;
    }

    pub fn is_group_payment(&self) -> bool {
        self.group_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_new() {
        let p = Payment::new(1, 10, 20, 30.0, Some(5)).unwrap();
        assert_eq!(p.id, 1);
        assert_eq!(p.from_id(), 10);
        assert_eq!(p.to_id(), 20);
        assert_eq!(p.amount(), 30.0);
        assert_eq!(p.group_id(), Some(5));
        assert!(p.is_group_payment());
        assert!(p.created_at() > 0);
    }

    #[test]
    fn test_payment_new_without_group() {
        let p = Payment::new(1, 10, 20, 30.0, None).unwrap();
        assert_eq!(p.group_id(), None);
        assert!(!p.is_group_payment());
    }

    #[test]
    fn test_payment_new_zero_amount_returns_error() {
        let result = Payment::new(1, 10, 20, 0.0, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_new_negative_amount_returns_error() {
        let result = Payment::new(1, 10, 20, -5.0, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_new_same_sender_recipient_returns_error() {
        let result = Payment::new(1, 10, 10, 30.0, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_update_amount() {
        let mut p = Payment::new(1, 10, 20, 30.0, None).unwrap();
        p.update_amount(50.0).unwrap();
        assert_eq!(p.amount(), 50.0);
    }

    #[test]
    fn test_payment_update_amount_invalid_returns_error() {
        let mut p = Payment::new(1, 10, 20, 30.0, None).unwrap();
        let result = p.update_amount(-1.0);
        assert!(result.is_err());
        assert_eq!(p.amount(), 30.0); 
    }

    #[test]
    fn test_payment_assign_to_group() {
        let mut p = Payment::new(1, 10, 20, 30.0, None).unwrap();
        p.assign_to_group(7);
        assert_eq!(p.group_id(), Some(7));
    }

    #[test]
    fn test_payment_remove_from_group() {
        let mut p = Payment::new(1, 10, 20, 30.0, Some(7)).unwrap();
        p.remove_from_group();
        assert_eq!(p.group_id(), None);
    }
}