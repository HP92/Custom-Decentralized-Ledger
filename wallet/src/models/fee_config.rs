use super::FeeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FeeConfig {
    fee_type: FeeType,
    value: f64,
}

impl FeeConfig {
    pub fn new(fee_type: FeeType, value: f64) -> Self {
        Self { fee_type, value }
    }

    pub fn fee_type(&self) -> &FeeType {
        &self.fee_type
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_config_creation_fixed() {
        let fee_config = FeeConfig::new(FeeType::Fixed, 100.0);
        assert_eq!(fee_config.fee_type(), &FeeType::Fixed);
        assert_eq!(fee_config.value(), 100.0);
    }

    #[test]
    fn test_fee_config_creation_percent() {
        let fee_config = FeeConfig::new(FeeType::Percent, 2.5);
        assert_eq!(fee_config.fee_type(), &FeeType::Percent);
        assert_eq!(fee_config.value(), 2.5);
    }

    #[test]
    fn test_fee_config_serialization() {
        let fee_config = FeeConfig::new(FeeType::Fixed, 50.0);
        let serialized = serde_json::to_string(&fee_config).unwrap();
        let deserialized: FeeConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(fee_config.fee_type(), deserialized.fee_type());
        assert_eq!(fee_config.value(), deserialized.value());
    }

    #[test]
    fn test_fee_config_clone() {
        let fee_config = FeeConfig::new(FeeType::Percent, 1.5);
        let cloned = fee_config.clone();
        
        assert_eq!(fee_config.fee_type(), cloned.fee_type());
        assert_eq!(fee_config.value(), cloned.value());
    }

    #[test]
    fn test_fee_config_zero_value() {
        let fee_config = FeeConfig::new(FeeType::Fixed, 0.0);
        assert_eq!(fee_config.value(), 0.0);
    }

    #[test]
    fn test_fee_config_large_percent() {
        let fee_config = FeeConfig::new(FeeType::Percent, 99.99);
        assert_eq!(fee_config.value(), 99.99);
    }
}
