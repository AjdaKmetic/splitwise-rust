use crate::models::split::Split;

#[derive(Debug, Clone)]
pub struct Expense {
    pub id: u32,
    pub description: String,
    pub paid_by: u32,
    pub total_amount: f64,
    pub splits: Vec<Split>,
}

impl Expense {
    pub fn new(
        id: u32,
        description: String,
        paid_by: u32,
        total_amount: f64,
        splits: Vec<Split>,
    ) -> Self {
        Self {
            id,
            description,
            paid_by,
            total_amount,
            splits,
        }
    }

}


