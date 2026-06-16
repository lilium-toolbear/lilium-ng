use crate::pal_work_constants::*;
use tracing::instrument;

#[instrument(level = "debug" fields(total_eff, scale, cap))]
fn capped_bonus(total_eff: f64, scale: f64, cap: f64) -> f64 {
    if total_eff <= 0.0 || scale <= 0.0 {
        return 0.0;
    }
    cap * total_eff / (total_eff + scale)
}

#[instrument(level = "debug" fields(level, cap, scale, power))]
fn s_curve_multiplier(level: i32, cap: f64, scale: f64, power: f64) -> f64 {
    if level <= 0 || cap <= 0.0 || scale <= 0.0 || power <= 0.0 {
        return 1.0;
    }
    let l = level as f64;
    1.0 + cap * l.powf(power) / (l.powf(power) + scale.powf(power))
}

#[instrument(level = "debug" fields(total_watering_eff))]
pub fn calculate_farm_time_bonus(total_watering_eff: f64) -> f64 {
    capped_bonus(total_watering_eff, 500.0, 1.0)
}

#[instrument(level = "debug" fields(total_planting_eff))]
pub fn calculate_farm_capacity_bonus(total_planting_eff: f64) -> f64 {
    capped_bonus(total_planting_eff, 1000.0, 2.5)
}

#[instrument(level = "debug" fields(total_gathering_eff))]
pub fn calculate_farm_harvest_bonus(total_gathering_eff: f64) -> f64 {
    capped_bonus(total_gathering_eff, 1200.0, 1.5)
}

#[instrument(level = "debug" fields(total_hauling_eff))]
pub fn calculate_warehouse_capacity_bonus(total_hauling_eff: f64) -> f64 {
    if total_hauling_eff <= 0.0 {
        return 0.0;
    }
    total_hauling_eff / 333.0
}

#[instrument(level = "debug" fields(total_eff))]
pub fn calculate_credit_income_per_hour(total_eff: f64) -> f64 {
    total_eff * CREDIT_RATE
}

#[instrument(level = "debug" fields(level))]
pub fn calculate_resource_level_multiplier(level: i32) -> f64 {
    s_curve_multiplier(level, 4.0, 6.0, 2.0)
}

#[instrument(level = "debug" fields(level))]
pub fn calculate_resource_cache_hours(level: i32) -> f64 {
    24.0 * s_curve_multiplier(level, 6.0, 10.0, 2.0)
}

#[instrument(level = "debug" fields(total_eff))]
pub fn calculate_resource_worker_multiplier(total_eff: f64) -> f64 {
    capped_bonus(total_eff, 1200.0, 3.0)
}

#[instrument(level = "debug" fields(total_eff, level))]
pub fn calculate_resource_income_per_hour(total_eff: f64, level: i32) -> f64 {
    let level_mult = calculate_resource_level_multiplier(level);
    let worker_mult = calculate_resource_worker_multiplier(total_eff);
    200.0 * 1200.0 * level_mult * worker_mult
}

#[instrument(level = "debug" fields(total_eff, level))]
pub fn calculate_resource_cache_cap(total_eff: f64, level: i32) -> f64 {
    let income = calculate_resource_income_per_hour(total_eff, level);
    let hours = calculate_resource_cache_hours(level);
    income * hours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capped_bonus_positive() {
        let b = capped_bonus(500.0, 500.0, 1.0);
        assert!(b > 0.0 && b < 1.0);
    }

    #[test]
    fn test_capped_bonus_zero() {
        assert_eq!(capped_bonus(0.0, 500.0, 1.0), 0.0);
    }

    #[test]
    fn test_capped_bonus_negative() {
        assert_eq!(capped_bonus(-100.0, 500.0, 1.0), 0.0);
    }

    #[test]
    fn test_capped_bonus_exact() {
        assert_eq!(capped_bonus(500.0, 500.0, 1.0), 0.5);
        assert_eq!(capped_bonus(2000.0, 500.0, 1.0), 2000.0 / 2500.0);
    }

    #[test]
    fn test_s_curve_zero_level() {
        assert_eq!(s_curve_multiplier(0, 4.0, 6.0, 2.0), 1.0);
    }

    #[test]
    fn test_farm_bonuses() {
        assert!(calculate_farm_time_bonus(500.0) > 0.0);
        assert!(calculate_farm_capacity_bonus(1000.0) > 0.0);
        assert!(calculate_farm_harvest_bonus(1200.0) > 0.0);
    }

    #[test]
    fn test_farm_time_bonus_exact() {
        assert!((calculate_farm_time_bonus(500.0) - 0.5).abs() < 0.001);
        assert!((calculate_farm_time_bonus(5000.0) - 5000.0 / 5500.0).abs() < 0.001);
        assert!(calculate_farm_time_bonus(5000.0) < 1.0);
    }

    #[test]
    fn test_farm_capacity_bonus_exact() {
        assert!((calculate_farm_capacity_bonus(500.0) - 2.5 * 500.0 / 1500.0).abs() < 0.001);
        assert!(calculate_farm_capacity_bonus(100_000.0) < 2.5);
    }

    #[test]
    fn test_warehouse_capacity_bonus_exact() {
        assert!((calculate_warehouse_capacity_bonus(333.0) - 1.0).abs() < 0.001);
        assert_eq!(calculate_warehouse_capacity_bonus(0.0), 0.0);
        assert_eq!(calculate_warehouse_capacity_bonus(-100.0), 0.0);
    }

    #[test]
    fn test_credit_income_per_hour_exact() {
        assert_eq!(calculate_credit_income_per_hour(100.0), 10000.0);
        assert_eq!(calculate_credit_income_per_hour(500.0), 50000.0);
    }

    #[test]
    fn test_resource_level_multiplier_exact() {
        assert_eq!(calculate_resource_level_multiplier(0), 1.0);
        assert!((calculate_resource_level_multiplier(1) - (1.0 + 4.0 / 37.0)).abs() < 0.001);
        assert_eq!(calculate_resource_level_multiplier(6), 3.0);
        assert!(calculate_resource_level_multiplier(18) < 5.0);
        assert!(calculate_resource_level_multiplier(18) > calculate_resource_level_multiplier(10));
    }

    #[test]
    fn test_resource_cache_hours_exact() {
        assert_eq!(calculate_resource_cache_hours(0), 24.0);
        assert!((calculate_resource_cache_hours(10) - 96.0).abs() < 0.001);
        assert!(calculate_resource_cache_hours(18) < 168.0);
        assert!(calculate_resource_cache_hours(18) > calculate_resource_cache_hours(10));
    }

    #[test]
    fn test_resource_worker_multiplier_exact() {
        assert_eq!(calculate_resource_worker_multiplier(0.0), 0.0);
        assert!((calculate_resource_worker_multiplier(1200.0) - 1.5).abs() < 0.001);
        assert!(calculate_resource_worker_multiplier(1_000_000.0) < 3.0);
    }

    #[test]
    fn test_resource_income_per_hour_exact() {
        let base = 200.0 * 1200.0;
        assert_eq!(calculate_resource_income_per_hour(0.0, 10), 0.0);
        assert!((calculate_resource_income_per_hour(1200.0, 0) - base * 1.5).abs() < 0.001);
        assert!((calculate_resource_income_per_hour(1200.0, 6) - base * 1.5 * 3.0).abs() < 0.001);
    }
}
