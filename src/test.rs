use crate::serializers::load;

#[test]
fn test_load() {
    let result = load();
    println!("{result:#?}");
    assert!(result.is_ok());
}
