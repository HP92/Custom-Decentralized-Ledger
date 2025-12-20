use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum FeeType {
    Fixed,
    Percent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_type_serialization() {
        let fixed = FeeType::Fixed;
        let serialized = serde_json::to_string(&fixed).unwrap();
        let deserialized: FeeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(fixed, deserialized);

        let percent = FeeType::Percent;
        let serialized = serde_json::to_string(&percent).unwrap();
        let deserialized: FeeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(percent, deserialized);
    }

    #[test]
    fn test_fee_type_clone() {
        let fixed = FeeType::Fixed;
        let cloned = fixed.clone();
        assert_eq!(fixed, cloned);

        let percent = FeeType::Percent;
        let cloned = percent.clone();
        assert_eq!(percent, cloned);
    }

    #[test]
    fn test_fee_type_equality() {
        assert_eq!(FeeType::Fixed, FeeType::Fixed);
        assert_eq!(FeeType::Percent, FeeType::Percent);
        assert_ne!(FeeType::Fixed, FeeType::Percent);
    }
}
