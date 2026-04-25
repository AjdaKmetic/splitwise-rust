use crate::models::user::UserId;

#[derive(Debug, Clone, PartialEq)]
pub enum Split {
    Equal(Vec<UserId>),
    Exact(Vec<(UserId, f64)>),
}

impl Split {
    pub fn new_equal(user_ids: Vec<UserId>) -> Result<Self, String> {
        if user_ids.is_empty() {
            return Err("Equal split must have at least one participant".to_string());
        }

        Ok(Split::Equal(user_ids))
    }

    pub fn new_exact(shares: Vec<(UserId, f64)>) -> Result<Self, String> {
        if shares.is_empty() {
            return Err("Exact split must have at least one participant".to_string());
        }

        for (_, amount) in &shares {
            if *amount < 0.0 {
                return Err("Share amounts cannot be negative".to_string());
            }
        }

        Ok(Split::Exact(shares))
    }

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
        let split = Split::new_equal(vec![1, 2, 3]).unwrap();

        let shares = split.compute_shares(90.0);

        assert_eq!(shares, vec![(1, 30.0), (2, 30.0), (3, 30.0)]);
    }

    #[test]
    fn test_split_exact() {
        let split = Split::new_exact(vec![(1, 30.0), (2, 40.0), (3, 20.0)]).unwrap();

        let shares = split.compute_shares(90.0);

        assert_eq!(shares, vec![(1, 30.0), (2, 40.0), (3, 20.0)]);
    }

    #[test]
    fn test_split_participants_equal() {
        let split = Split::new_equal(vec![1, 2, 3]).unwrap();

        let participants = split.participants();

        assert_eq!(participants, vec![1, 2, 3]);
    }

    #[test]
    fn test_split_participants_exact() {
        let split = Split::new_exact(vec![(1, 10.0), (2, 20.0), (3, 30.0)]).unwrap();

        let participants = split.participants();

        assert_eq!(participants, vec![1, 2, 3]);
    }

    #[test]
    fn test_split_equal_two_users() {
        let split = Split::new_equal(vec![1, 2]).unwrap();

        let shares = split.compute_shares(50.0);

        assert_eq!(shares, vec![(1, 25.0), (2, 25.0)]);
    }

    #[test]
    fn test_split_equal_one_user() {
        let split = Split::new_equal(vec![1]).unwrap();

        let shares = split.compute_shares(50.0);

        assert_eq!(shares, vec![(1, 50.0)]);
    }

    #[test]
    fn test_split_exact_not_dependent_on_total_amount() {
        let split = Split::new_exact(vec![(1, 10.0), (2, 20.0)]).unwrap();

        let shares = split.compute_shares(999.0);

        assert_eq!(shares, vec![(1, 10.0), (2, 20.0)]);
    }

    #[test]
    fn test_new_equal_empty_returns_error() {
        let split = Split::new_equal(vec![]);

        assert!(split.is_err());
        assert_eq!(
            split.unwrap_err(),
            "Equal split must have at least one participant"
        );
    }

    #[test]
    fn test_new_exact_empty_returns_error() {
        let split = Split::new_exact(vec![]);

        assert!(split.is_err());
        assert_eq!(
            split.unwrap_err(),
            "Exact split must have at least one participant"
        );
    }

    #[test]
    fn test_new_exact_negative_amount_returns_error() {
        let split = Split::new_exact(vec![(1, -10.0), (2, 20.0)]);

        assert!(split.is_err());
        assert_eq!(
            split.unwrap_err(),
            "Share amounts cannot be negative"
        );
    }
}