use crate::{split::Split, user::UserId, group::GroupId};

#[derive(Debug, Clone)]
pub struct Expense {
    pub id: u32,
    pub description: String,
    pub paid_by: UserId,
    pub total_amount: f64,
    pub splits: Vec<Split>,
    pub group_id: GroupId,
}

impl Expense {
    pub fn new(
        id: u32,
        description: String,
        paid_by: UserId,
        total_amount: f64,
        splits: Vec<Split>,
        group_id: GroupId,
    ) -> Self {
        Self {
            id,
            description,
            paid_by,
            total_amount,
            splits,
            group_id,
        }
    }

}


