#[cfg(test)]
mod tests {
    use crate::{DoraMetricConfig, DoraMetric, DORA_METRIC_CONFIGS};

    #[test]
    fn test_deployment_frequency_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "deployment_frequency")
            .map(|(_, config)| config)
            .unwrap();

        // Test boundary values
        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 0.025);
        assert_eq!(result_0.unit, "deployments/day");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 10.0);
        assert_eq!(result_1.unit, "deployments/day");

        // Test specific points
        let result_025 = config.translate(0.25);
        let expected_025 = ((0.025 + (10.0 - 0.025) * 0.25) * 1000.0).round() / 1000.0; // 2.519
        assert_eq!(result_025.value, expected_025);
        assert_eq!(result_025.unit, "deployments/day");

        let result_05 = config.translate(0.5);
        let expected_05 = ((0.025 + (10.0 - 0.025) * 0.5) * 1000.0).round() / 1000.0; // 5.013
        assert_eq!(result_05.value, expected_05);
        assert_eq!(result_05.unit, "deployments/day");

        let result_067 = config.translate(0.67);
        let expected_067 = ((0.025 + (10.0 - 0.025) * 0.67) * 1000.0).round() / 1000.0; // 6.708
        assert_eq!(result_067.value, expected_067);
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
        assert_eq!(result_1.value, 0.1);
        assert_eq!(result_1.unit, "days");

        // Test specific points
        let result_025 = config.translate(0.25);
        let expected_025 = ((60.0 - (60.0 - 0.1) * 0.25) * 1000.0).round() / 1000.0; // 45.025
        assert_eq!(result_025.value, expected_025);
        assert_eq!(result_025.unit, "days");

        let result_05 = config.translate(0.5);
        let expected_05 = ((60.0 - (60.0 - 0.1) * 0.5) * 1000.0).round() / 1000.0; // 30.05
        assert_eq!(result_05.value, expected_05);
        assert_eq!(result_05.unit, "days");

        let result_067 = config.translate(0.67);
        let expected_067 = ((60.0 - (60.0 - 0.1) * 0.67) * 1000.0).round() / 1000.0; // 19.933
        assert_eq!(result_067.value, expected_067);
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
        let expected_025 = 100.0 - (100.0 - 0.0) * 0.25; // 75.0
        assert_eq!(result_025.value, 75.0);
        assert_eq!(result_025.unit, "%");

        let result_05 = config.translate(0.5);
        let expected_05 = 100.0 - (100.0 - 0.0) * 0.5; // 50.0
        assert_eq!(result_05.value, 50.0);
        assert_eq!(result_05.unit, "%");

        let result_067 = config.translate(0.67);
        let expected_067 = 100.0 - (100.0 - 0.0) * 0.67; // 33.0
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
        assert_eq!(result_1.value, 0.05);
        assert_eq!(result_1.unit, "days");

        // Test specific points
        let result_025 = config.translate(0.25);
        let expected_025 = ((14.0 - (14.0 - 0.05) * 0.25) * 1000.0).round() / 1000.0; // 10.513
        assert_eq!(result_025.value, expected_025);
        assert_eq!(result_025.unit, "days");

        let result_05 = config.translate(0.5);
        let expected_05 = ((14.0 - (14.0 - 0.05) * 0.5) * 1000.0).round() / 1000.0; // 7.025
        assert_eq!(result_05.value, expected_05);
        assert_eq!(result_05.unit, "days");

        let result_067 = config.translate(0.67);
        let expected_067 = ((14.0 - (14.0 - 0.05) * 0.67) * 1000.0).round() / 1000.0; // 4.669
        assert_eq!(result_067.value, expected_067);
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
        assert_eq!(result_0.value, 0.25);
        assert_eq!(result_0.unit, "commits/day/developer");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 10.0);
        assert_eq!(result_1.unit, "commits/day/developer");

        // Test specific points
        let result_025 = config.translate(0.25);
        let expected_025 = ((0.25 + (10.0 - 0.25) * 0.25) * 1000.0).round() / 1000.0; // 2.688
        assert_eq!(result_025.value, expected_025);
        assert_eq!(result_025.unit, "commits/day/developer");

        let result_05 = config.translate(0.5);
        let expected_05 = ((0.25 + (10.0 - 0.25) * 0.5) * 1000.0).round() / 1000.0; // 5.125
        assert_eq!(result_05.value, expected_05);
        assert_eq!(result_05.unit, "commits/day/developer");

        let result_067 = config.translate(0.67);
        let expected_067 = ((0.25 + (10.0 - 0.25) * 0.67) * 1000.0).round() / 1000.0; // 6.783
        assert_eq!(result_067.value, expected_067);
        assert_eq!(result_067.unit, "commits/day/developer");
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
        assert_eq!(result_1.value, 0.05);
        assert_eq!(result_1.unit, "days");

        // Test specific points
        let result_025 = config.translate(0.25);
        let expected_025 = ((30.0 - (30.0 - 0.05) * 0.25) * 1000.0).round() / 1000.0; // 22.513
        assert_eq!(result_025.value, expected_025);
        assert_eq!(result_025.unit, "days");

        let result_05 = config.translate(0.5);
        let expected_05 = ((30.0 - (30.0 - 0.05) * 0.5) * 1000.0).round() / 1000.0; // 15.025
        assert_eq!(result_05.value, expected_05);
        assert_eq!(result_05.unit, "days");

        let result_067 = config.translate(0.67);
        let expected_067 = ((30.0 - (30.0 - 0.05) * 0.67) * 1000.0).round() / 1000.0; // 9.934
        assert_eq!(result_067.value, expected_067);
        assert_eq!(result_067.unit, "days");
    }

    #[test]
    fn test_translation_consistency() {
        // Test that all metrics have consistent behavior
        for (metric_name, config) in DORA_METRIC_CONFIGS {
            // Test that 0.0 gives the expected boundary
            let result_0 = config.translate(0.0);
            if config.inverted {
                assert_eq!(result_0.value, config.max_value);
            } else {
                assert_eq!(result_0.value, config.min_value);
            }

            // Test that 1.0 gives the expected boundary
            let result_1 = config.translate(1.0);
            if config.inverted {
                assert_eq!(result_1.value, config.min_value);
            } else {
                assert_eq!(result_1.value, config.max_value);
            }

            // Test that 0.5 gives the middle value
            let result_05 = config.translate(0.5);
            let expected_middle = (config.min_value + config.max_value) / 2.0;
            assert!((result_05.value - expected_middle).abs() < 0.01, 
                "Metric {}: expected middle value {} but got {}", 
                metric_name, expected_middle, result_05.value);
        }
    }
}
