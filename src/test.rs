use crate::serializers::load;

#[test]
fn test_load() {
    let result = load();
    assert!(result.is_ok());
}
