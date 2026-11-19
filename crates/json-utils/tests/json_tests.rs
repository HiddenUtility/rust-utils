use json_utils::{from_json, to_json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestStruct {
    name: String,
    age: u32,
}

#[test]
fn test_to_json_should_work() {
    let test_struct = TestStruct {
        name: "Alice".to_string(),
        age: 30,
    };
    let json_str = to_json(&test_struct).unwrap();
    assert_eq!(json_str, r#"{"name":"Alice","age":30}"#);
}

#[test]
fn test_from_json_should_work() {
    let json_str = r#"{"name":"Bob","age":25}"#;
    let decoded: TestStruct = from_json(json_str).unwrap();
    assert_eq!(
        decoded,
        TestStruct {
            name: "Bob".to_string(),
            age: 25,
        }
    );
}
