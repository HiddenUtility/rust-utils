use json_utils::{TryFromDto, User, UserDto};

#[test]
fn test_conversion_with_defaults() {
    let dto = UserDto {
        name: None,
        age: None,
        email: None,
    };
    let user = User::try_from_dto(dto).unwrap();
    assert_eq!(user.name, "Unknown");
    assert_eq!(user.age, 0);
    assert_eq!(user.email, "no-email@example.com");
}

#[test]
fn test_conversion_with_values() {
    let dto = UserDto {
        name: Some("Alice".to_string()),
        age: Some(30),
        email: Some("alice@example.com".to_string()),
    };
    let user = User::try_from_dto(dto).unwrap();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
    assert_eq!(user.email, "alice@example.com");
}

#[test]
fn test_conversion_validation_error() {
    let dto = UserDto {
        name: Some("".to_string()),
        age: Some(30),
        email: Some("alice@example.com".to_string()),
    };
    let result = User::try_from_dto(dto);
    assert!(result.is_err());
    if let Err(json_utils::JsonError::Validation(msg)) = result {
        assert_eq!(msg, "Name cannot be empty");
    } else {
        panic!("Expected Validation error");
    }
}
