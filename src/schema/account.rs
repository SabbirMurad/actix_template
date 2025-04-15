use serde::{Deserialize, Serialize};

//role for account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountRole { Administrator, User}
impl std::fmt::Display for AccountRole {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}