pub type UserId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: UserId, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}

