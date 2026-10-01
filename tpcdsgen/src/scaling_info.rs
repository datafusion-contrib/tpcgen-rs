use crate::{check_argument, error::Result, TpcdsError};
use std::collections::HashMap;

/// Scaling models for table row count calculation (ScalingInfo.ScalingModel)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalingModel {
    Static,
    Linear,
    Logarithmic,
}

/// Scaling information for table row count calculation (ScalingInfo)
#[derive(Debug, Clone)]
pub struct ScalingInfo {
    /// Multiplier for calculations
    multiplier: i32,
    /// Scaling model to use
    scaling_model: ScalingModel,
    /// Map from scale factors to row counts
    scales_to_row_counts_map: HashMap<i32, i32>, // Using i32 for scale keys for simpler lookup
    /// Update percentage
    update_percentage: i32,
}

impl ScalingInfo {
    /// Defined scale factors (DEFINED_SCALES)
    pub const DEFINED_SCALES: [f64; 10] = [
        0.0, 1.0, 10.0, 100.0, 300.0, 1000.0, 3000.0, 10000.0, 30000.0, 100000.0,
    ];
    /// Largest scale factor defined by TPC-DS (100_000).
    const MAX_DEFINED_SCALE: f64 = Self::DEFINED_SCALES[Self::DEFINED_SCALES.len() - 1];

    /// Create new ScalingInfo
    pub fn new(
        multiplier: i32,
        scaling_model: ScalingModel,
        row_counts_per_scale: &[i32],
        update_percentage: i32,
    ) -> Result<Self> {
        check_argument!(
            multiplier >= 0,
            "multiplier is not greater than or equal to 0"
        );
        check_argument!(
            update_percentage >= 0,
            "updatePercentage is not greater than or equal to zero"
        );
        check_argument!(
            row_counts_per_scale.len() == Self::DEFINED_SCALES.len(),
            "row_counts_per_scale length must match DEFINED_SCALES length"
        );

        let mut scales_to_row_counts_map = HashMap::new();
        for (i, &row_count) in row_counts_per_scale.iter().enumerate() {
            check_argument!(row_count >= 0, "row counts cannot be negative");
            // Convert float scale to int key for HashMap (multiply by 1000 to preserve precision)
            let scale_key = (Self::DEFINED_SCALES[i] * 1000.0) as i32;
            scales_to_row_counts_map.insert(scale_key, row_count);
        }

        Ok(ScalingInfo {
            multiplier,
            scaling_model,
            scales_to_row_counts_map,
            update_percentage,
        })
    }

    /// Get the multiplier
    pub fn get_multiplier(&self) -> i32 {
        self.multiplier
    }

    /// Get the scaling model
    pub fn get_scaling_model(&self) -> ScalingModel {
        self.scaling_model
    }

    /// Get the update percentage
    pub fn get_update_percentage(&self) -> i32 {
        self.update_percentage
    }

    /// Get row count for a given scale (getRowCountForScale)
    pub fn get_row_count_for_scale(&self, scale: f64) -> Result<i64> {
        check_argument!(
            scale.is_finite() && scale >= 0.0,
            "scale must be a non-negative number"
        );

        let scale_key = (scale * 1000.0) as i32;
        if let Some(&row_count) = self.scales_to_row_counts_map.get(&scale_key) {
            return Ok(row_count as i64);
        }

        // Get the scaling model for the table
        match self.scaling_model {
            ScalingModel::Static => self.compute_count_using_static_scale(),
            ScalingModel::Linear => self.compute_count_using_linear_scale(scale),
            ScalingModel::Logarithmic => self.compute_count_using_log_scale(scale),
        }
    }

    /// Compute count using static scale model
    fn compute_count_using_static_scale(&self) -> Result<i64> {
        self.get_row_count_for_scale(1.0)
    }

    /// Compute count using logarithmic scale model (computeCountUsingLogScale)
    fn compute_count_using_log_scale(&self, scale: f64) -> Result<i64> {
        let scale_slot = Self::get_scale_slot(scale);
        let delta = self.get_row_count_for_scale(Self::DEFINED_SCALES[scale_slot])?
            - self.get_row_count_for_scale(Self::DEFINED_SCALES[scale_slot - 1])?;

        let float_offset = (scale - Self::DEFINED_SCALES[scale_slot - 1])
            / (Self::DEFINED_SCALES[scale_slot] - Self::DEFINED_SCALES[scale_slot - 1]);

        let base_row_count = if scale < 1.0 {
            self.get_row_count_for_scale(Self::DEFINED_SCALES[0])?
        } else if scale <= Self::MAX_DEFINED_SCALE {
            self.get_row_count_for_scale(Self::DEFINED_SCALES[1])?
        } else {
            // Scaling above 100_000 is not defined by TPC-DS.
            // Simulate growth above 100_000.
            self.get_row_count_for_scale(Self::DEFINED_SCALES[scale_slot - 1])?
        };

        let count = ((float_offset * delta as f64) as i64) + base_row_count;
        Ok(if count == 0 { 1 } else { count })
    }

    /// Get scale slot for a given scale (getScaleSlot)
    ///
    /// Scales above 100_000 are undefined, so fall back to the slot of 100_000.
    fn get_scale_slot(scale: f64) -> usize {
        Self::DEFINED_SCALES
            .iter()
            .position(|&defined_scale| scale <= defined_scale)
            .unwrap_or(Self::DEFINED_SCALES.len() - 1)
    }

    /// Compute count using linear scale model (computeCountUsingLinearScale)
    fn compute_count_using_linear_scale(&self, scale: f64) -> Result<i64> {
        let mut row_count = 0i64;
        let mut target_gb = scale;

        if scale < 1.0 {
            let base_count = self.get_row_count_for_scale(Self::DEFINED_SCALES[1])?;
            row_count = (scale * base_count as f64).round() as i64;
            return Ok(if row_count == 0 { 1 } else { row_count });
        }

        // Work from large scales down
        for i in (1..Self::DEFINED_SCALES.len()).rev() {
            // Use the defined rowcounts to build up the target GB volume
            while target_gb >= Self::DEFINED_SCALES[i] {
                row_count += self.get_row_count_for_scale(Self::DEFINED_SCALES[i])?;
                target_gb -= Self::DEFINED_SCALES[i];
            }
        }

        Ok(row_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaling_info_creation() {
        let row_counts = [0, 100, 500, 2000, 5000, 12000, 30000, 65000, 80000, 100000];
        let scaling_info = ScalingInfo::new(3, ScalingModel::Logarithmic, &row_counts, 0).unwrap();

        assert_eq!(scaling_info.get_multiplier(), 3);
        assert_eq!(scaling_info.get_scaling_model(), ScalingModel::Logarithmic);
        assert_eq!(scaling_info.get_update_percentage(), 0);
    }

    #[test]
    fn test_scaling_info_validation() {
        let row_counts = [0, 100, 500, 2000, 5000, 12000, 30000, 65000, 80000, 100000];

        // Test negative multiplier
        assert!(ScalingInfo::new(-1, ScalingModel::Static, &row_counts, 0).is_err());

        // Test negative update percentage
        assert!(ScalingInfo::new(0, ScalingModel::Static, &row_counts, -1).is_err());

        // Test wrong array length
        let wrong_counts = [0, 100, 500];
        assert!(ScalingInfo::new(0, ScalingModel::Static, &wrong_counts, 0).is_err());

        // Test negative row count
        let negative_counts = [0, -100, 500, 2000, 5000, 12000, 30000, 65000, 80000, 100000];
        assert!(ScalingInfo::new(0, ScalingModel::Static, &negative_counts, 0).is_err());
    }

    #[test]
    fn test_static_scaling() {
        let row_counts = [
            0, 73049, 73049, 73049, 73049, 73049, 73049, 73049, 73049, 73049,
        ];
        let scaling_info = ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0).unwrap();

        // Static scaling should always return the scale=1 value
        assert_eq!(scaling_info.get_row_count_for_scale(1.0).unwrap(), 73049);
        assert_eq!(scaling_info.get_row_count_for_scale(10.0).unwrap(), 73049);
        assert_eq!(scaling_info.get_row_count_for_scale(1000.0).unwrap(), 73049);
    }

    #[test]
    fn test_exact_scale_matches() {
        let row_counts = [0, 100, 500, 2000, 5000, 12000, 30000, 65000, 80000, 100000];
        let scaling_info = ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0).unwrap();

        // Test exact matches from the defined scales
        assert_eq!(scaling_info.get_row_count_for_scale(0.0).unwrap(), 0);
        assert_eq!(scaling_info.get_row_count_for_scale(1.0).unwrap(), 100);
        assert_eq!(scaling_info.get_row_count_for_scale(10.0).unwrap(), 500);
        assert_eq!(scaling_info.get_row_count_for_scale(100.0).unwrap(), 2000);
        assert_eq!(scaling_info.get_row_count_for_scale(1000.0).unwrap(), 12000);
    }

    #[test]
    fn test_logarithmic_scaling_interpolation() {
        let row_counts = [0, 100, 500, 2000, 5000, 12000, 30000, 65000, 80000, 100000];
        let scaling_info = ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0).unwrap();

        // Test interpolation - should be between defined points
        let result_5 = scaling_info.get_row_count_for_scale(5.0).unwrap();
        assert!(result_5 > 100); // Greater than scale=1 result
        assert!(result_5 < 500); // Less than scale=10 result
    }

    #[test]
    fn test_linear_scaling_fractional() {
        let row_counts = [
            0, 24, 240, 2400, 7200, 24000, 72000, 240000, 720000, 2400000,
        ];
        let scaling_info = ScalingInfo::new(4, ScalingModel::Linear, &row_counts, 0).unwrap();

        // Test fractional scaling (< 1.0)
        let result = scaling_info.get_row_count_for_scale(0.5).unwrap();
        assert_eq!(result, 12); // 0.5 * 24 = 12

        // Test that zero result becomes 1
        let result_tiny = scaling_info.get_row_count_for_scale(0.001).unwrap();
        assert_eq!(result_tiny, 1);
    }

    #[test]
    fn test_linear_scaling_large() {
        let row_counts = [
            0, 24, 240, 2400, 7200, 24000, 72000, 240000, 720000, 2400000,
        ];
        let scaling_info = ScalingInfo::new(4, ScalingModel::Linear, &row_counts, 0).unwrap();

        // Test larger scale that requires multiple additions
        let result = scaling_info.get_row_count_for_scale(1100.0).unwrap();
        // Linear scaling works from largest to smallest:
        // 1100.0 - 1000.0 = 100.0 remaining, use scale[1000] = 24000
        // 100.0 - 100.0 = 0.0 remaining, use scale[100] = 2400
        // Total: 24000 + 2400 = 26400
        assert_eq!(result, 26400);
    }

    #[test]
    fn test_scale_validation() {
        let row_counts = [0, 100, 500, 2000, 5000, 12000, 30000, 65000, 80000, 100000];
        for model in [
            ScalingModel::Static,
            ScalingModel::Linear,
            ScalingModel::Logarithmic,
        ] {
            let scaling_info = ScalingInfo::new(0, model, &row_counts, 0).unwrap();

            for scale in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(scaling_info.get_row_count_for_scale(scale).is_err());
            }
            // Above 100000 is accepted
            assert!(scaling_info.get_row_count_for_scale(100001.0).is_ok());
        }
    }

    #[test]
    fn test_get_scale_slot() {
        assert_eq!(ScalingInfo::get_scale_slot(0.0), 0);
        assert_eq!(ScalingInfo::get_scale_slot(0.5), 1);
        assert_eq!(ScalingInfo::get_scale_slot(1.0), 1);
        assert_eq!(ScalingInfo::get_scale_slot(5.0), 2);
        assert_eq!(ScalingInfo::get_scale_slot(10.0), 2);
        assert_eq!(ScalingInfo::get_scale_slot(50.0), 3);
        assert_eq!(ScalingInfo::get_scale_slot(100000.0), 9);

        // Undefined above 100_000, falls back to the slot of 100_000
        assert_eq!(ScalingInfo::get_scale_slot(100_001.0), 9);
        assert_eq!(ScalingInfo::get_scale_slot(500_000.0), 9);
    }

    #[test]
    fn test_defined_scales_constant() {
        let expected = [
            0.0, 1.0, 10.0, 100.0, 300.0, 1000.0, 3000.0, 10000.0, 30000.0, 100000.0,
        ];
        assert_eq!(ScalingInfo::DEFINED_SCALES, expected);
    }

    #[test]
    fn test_scaling_models() {
        assert_eq!(format!("{:?}", ScalingModel::Static), "Static");
        assert_eq!(format!("{:?}", ScalingModel::Linear), "Linear");
        assert_eq!(format!("{:?}", ScalingModel::Logarithmic), "Logarithmic");
    }

    #[test]
    fn test_java_call_center_example() {
        // Test with actual CallCenter values from Java Table.java
        let row_counts = [0, 3, 12, 15, 18, 21, 24, 27, 30, 30];
        let scaling_info = ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0).unwrap();

        // Test some specific scale calculations
        assert_eq!(scaling_info.get_row_count_for_scale(1.0).unwrap(), 3);
        assert_eq!(scaling_info.get_row_count_for_scale(100000.0).unwrap(), 30);

        // Test interpolation
        let result_5 = scaling_info.get_row_count_for_scale(5.0).unwrap();
        assert!((3..=12).contains(&result_5));
    }
}
