pub mod types;
pub mod dora;
pub mod validation;
pub mod parser;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(test)]
mod tests {
    use crate::dora::*;

    #[test]
    fn test_deployment_frequency_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "deployment_frequency")
            .map(|(_, config)| config)
            .unwrap();

        // Test boundary values
        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 0.001); // 0.001 rounded to 2 decimal places
        assert_eq!(result_0.unit, "deployments/day");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 10.0);
        assert_eq!(result_1.unit, "deployments/day");

        // Test specific points
        let result_025 = config.translate(0.25);
        assert_eq!(result_025.value, 2.501); // 2.501 rounded to 2 decimal places
        assert_eq!(result_025.unit, "deployments/day");

        let result_05 = config.translate(0.5);
        assert_eq!(result_05.value, 5.0);
        assert_eq!(result_05.unit, "deployments/day");

        let result_067 = config.translate(0.67);
        assert_eq!(result_067.value, 6.7);
        assert_eq!(result_067.unit, "deployments/day");
    }

    #[test]
    fn test_lead_time_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "lead_time")
            .map(|(_, config)| config)
            .unwrap();

        // Test boundary values (inverted - lower slider = better)
        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 60.0);
        assert_eq!(result_0.unit, "days");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 0.04);
        assert_eq!(result_1.unit, "days");

        // Test specific points
        let result_025 = config.translate(0.25);
        assert_eq!(result_025.value, 45.01);
        assert_eq!(result_025.unit, "days");

        let result_05 = config.translate(0.5);
        assert_eq!(result_05.value, 30.02);
        assert_eq!(result_05.unit, "days");

        let result_067 = config.translate(0.67);
        assert_eq!(result_067.value, 19.827); // 19.827 rounded to 2 decimal places
        assert_eq!(result_067.unit, "days");
    }

    #[test]
    fn test_change_failure_rate_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "change_failure_rate")
            .map(|(_, config)| config)
            .unwrap();

        // Test boundary values (inverted - lower slider = better)
        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 100.0);
        assert_eq!(result_0.unit, "%");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 0.0);
        assert_eq!(result_1.unit, "%");

        // Test specific points
        let result_025 = config.translate(0.25);
        
        assert_eq!(result_025.value, 75.0);
        assert_eq!(result_025.unit, "%");

        let result_05 = config.translate(0.5);
        
        assert_eq!(result_05.value, 50.0);
        assert_eq!(result_05.unit, "%");

        let result_067 = config.translate(0.67);
        
        assert_eq!(result_067.value, 33.0);
        assert_eq!(result_067.unit, "%");
    }

    #[test]
    fn test_mttr_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "mttr")
            .map(|(_, config)| config)
            .unwrap();

        // Test boundary values (inverted - lower slider = better)
        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 14.0);
        assert_eq!(result_0.unit, "days");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 0.012); // 0.012 rounded to 2 decimal places
        assert_eq!(result_1.unit, "days");

        // Test specific points
        let result_025 = config.translate(0.25);
        assert_eq!(result_025.value, 10.503); // 10.503 rounded to 2 decimal places
        assert_eq!(result_025.unit, "days");

        let result_05 = config.translate(0.5);
        assert_eq!(result_05.value, 7.006); // 7.006 rounded to 2 decimal places
        assert_eq!(result_05.unit, "days");

        let result_067 = config.translate(0.67);
        assert_eq!(result_067.value, 4.628); // 4.628 rounded to 2 decimal places
        assert_eq!(result_067.unit, "days");
    }

    #[test]
    fn test_commit_frequency_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "commit_frequency")
            .map(|(_, config)| config)
            .unwrap();

        // Test boundary values
        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 0.063); // 0.063 rounded to 2 decimal places
        assert_eq!(result_0.unit, "commits/day per developer");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 10.0);
        assert_eq!(result_1.unit, "commits/day per developer");

        // Test specific points
        let result_025 = config.translate(0.25);
        assert_eq!(result_025.value, 2.547); // 2.547 rounded to 2 decimal places
        assert_eq!(result_025.unit, "commits/day per developer");

        let result_05 = config.translate(0.5);
        assert_eq!(result_05.value, 5.031); // 5.031 rounded to 2 decimal places
        assert_eq!(result_05.unit, "commits/day per developer");

        let result_067 = config.translate(0.67);
        assert_eq!(result_067.value, 6.721); // 6.721 rounded to 2 decimal places
        assert_eq!(result_067.unit, "commits/day per developer");
    }

    #[test]
    fn test_branch_lifetime_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "branch_lifetime")
            .map(|(_, config)| config)
            .unwrap();

        // Test boundary values (inverted - lower slider = better)
        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 30.0);
        assert_eq!(result_0.unit, "days");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 0.013); // 0.013 rounded to 2 decimal places
        assert_eq!(result_1.unit, "days");

        // Test specific points
        let result_025 = config.translate(0.25);
        assert_eq!(result_025.value, 22.503); // 22.503 rounded to 2 decimal places
        assert_eq!(result_025.unit, "days");

        let result_05 = config.translate(0.5);
        assert_eq!(result_05.value, 15.006); // 15.006 rounded to 2 decimal places
        assert_eq!(result_05.unit, "days");

        let result_067 = config.translate(0.67);
        assert_eq!(result_067.value, 9.908); // 9.908 rounded to 2 decimal places
        assert_eq!(result_067.unit, "days");
    }

    #[test]
    fn test_translation_consistency() {
        // Test that all metrics have consistent behavior
        for (metric_name, config) in DORA_METRIC_CONFIGS {
            // Test that 0.0 gives the expected boundary (accounting for rounding)
            let result_0 = config.translate(0.0);
            if config.inverted {
                assert_eq!(result_0.value, config.max_value);
            } else {
                // For non-inverted metrics, 0.0 should give min_value (with rounding)
                assert!((result_0.value - config.min_value).abs() < 0.01, 
                    "Metric {}: expected close to min_value {} but got {}", 
                    metric_name, config.min_value, result_0.value);
            }

            // Test that 1.0 gives the expected boundary (accounting for rounding)
            let result_1 = config.translate(1.0);
            if config.inverted {
                // For inverted metrics, 1.0 should give min_value (with rounding)
                assert!((result_1.value - config.min_value).abs() < 0.01, 
                    "Metric {}: expected close to min_value {} but got {}", 
                    metric_name, config.min_value, result_1.value);
            } else {
                assert_eq!(result_1.value, config.max_value);
            }

            // Test that 0.5 gives the middle value
            let result_05 = config.translate(0.5);
            let expected_middle = (config.min_value + config.max_value) / 2.0;
            assert!((result_05.value - expected_middle).abs() < 0.05, 
                "Metric {}: expected middle value {} but got {}", 
                metric_name, expected_middle, result_05.value);
        }
    }
}
