use miniserde::json;

#[test]
fn test_array() {
    let j = r#"["1","2","3"]"#;
    let array: [String; 3] = json::from_str(j).unwrap();
    let j2 = json::to_string(&array);
    assert_eq!(j, j2);
}

#[test]
fn test_array_too_short() {
    let j = r#"["1","2"]"#;
    json::from_str::<[String; 3]>(j).unwrap_err();
}

#[test]
fn test_array_too_long() {
    let j = r#"["1","2","3","4"]"#;
    json::from_str::<[String; 3]>(j).unwrap_err();
}
