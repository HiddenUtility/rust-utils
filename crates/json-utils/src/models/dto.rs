use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub name: Option<String>,
    pub age: Option<u32>,
    pub email: Option<String>,
}
