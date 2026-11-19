use crate::error::JsonError;
use crate::models::domain::User;
use crate::models::dto::UserDto;

pub trait TryFromDto<D> {
    fn try_from_dto(dto: D) -> Result<Self, JsonError>
    where
        Self: Sized;
}

impl TryFromDto<UserDto> for User {
    fn try_from_dto(dto: UserDto) -> Result<Self, JsonError> {
        let name = dto.name.unwrap_or_else(|| "Unknown".to_string());
        if name.trim().is_empty() {
            return Err(JsonError::Validation("Name cannot be empty".to_string()));
        }

        let age = dto.age.unwrap_or(0);

        let email = dto
            .email
            .unwrap_or_else(|| "no-email@example.com".to_string());

        Ok(User { name, age, email })
    }
}
