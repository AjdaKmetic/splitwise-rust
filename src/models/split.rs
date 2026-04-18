#[derive(Debug, Clone, PartialEq)]
pub struct Split {
    pub user_id: u32,
    pub amount: f64,
}

impl Split {
    pub fn new(user_id: u32, amount: f64) -> Self {
        Self { user_id, amount }
    }
}

