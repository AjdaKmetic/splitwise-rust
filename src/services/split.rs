use crate::models::user::UserId;

#[derive(Debug, Clone, PartialEq)]
pub enum Split {
    Equal(Vec<UserId>),
    Exact(Vec<(UserId, f64)>),
}

impl Split {
    pub fn compute_shares(&self, total_amount: f64) -> Vec<(UserId, f64)> {
        match self {
            Split::Equal(user_ids) => {
                let share = total_amount / user_ids.len() as f64;
                user_ids.iter().map(|&user_id| (user_id, share)).collect()
            }
            Split::Exact(shares) => shares.clone(),
        }
    }
    pub fn participants(&self) -> Vec<UserId> {
        match self {
            Split::Equal(user_ids) => user_ids.clone(),
            Split::Exact(shares) => shares.iter().map(|(user_id, _)| *user_id).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_equal() {
        let split = Split::Equal(vec![1, 2, 3]);
        let shares = split.compute_shares(90.0);
        assert_eq!(shares, vec![(1, 30.0), (2, 30.0), (3, 30.0)]);
    }

    #[test]
    fn test_split_exact() {
        let split = Split::Exact(vec![(1, 30.0), (2, 40.0), (3, 20.0)]);
        let shares = split.compute_shares(90.0);
        assert_eq!(shares, vec![(1, 30.0), (2, 40.0), (3, 20.0)]);
    }
}