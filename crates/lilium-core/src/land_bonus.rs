use crate::pal_work_constants::*;

fn capped_bonus(total_eff: f64, scale: f64, cap: f64) -> f64 {
    if total_eff <= 0.0 || scale <= 0.0 {
        return 0.0;
    }
    cap * total_eff / (total_eff + scale)
}

fn s_curve_multiplier(level: i32, cap: f64, scale: f64, power: f64) -> f64 {
    if level <= 0 || cap <= 0.0 || scale <= 0.0 || power <= 0.0 {
        return 1.0;
    }
    let l = level as f64;
    1.0 + cap * l.powf(power) / (l.powf(power) + scale.powf(power))
}

pub fn calculate_farm_time_bonus(total_watering_eff: f64) -> f64 {
    capped_bonus(total_watering_eff, 500.0, 1.0)
}

pub fn calculate_farm_capacity_bonus(total_planting_eff: f64) -> f64 {
    capped_bonus(total_planting_eff, 1000.0, 2.5)
}

pub fn calculate_farm_harvest_bonus(total_gathering_eff: f64) -> f64 {
    capped_bonus(total_gathering_eff, 1200.0, 1.5)
}

pub fn calculate_warehouse_capacity_bonus(total_hauling_eff: f64) -> f64 {
    if total_hauling_eff <= 0.0 {
        return 0.0;
    }
    total_hauling_eff / 333.0
}

pub fn calculate_credit_income_per_hour(total_eff: f64) -> f64 {
    total_eff * CREDIT_RATE
}

pub fn calculate_resource_level_multiplier(level: i32) -> f64 {
    s_curve_multiplier(level, 4.0, 6.0, 2.0)
}

pub fn calculate_resource_cache_hours(level: i32) -> f64 {
    24.0 * s_curve_multiplier(level, 6.0, 10.0, 2.0)
}

pub fn calculate_resource_worker_multiplier(total_eff: f64) -> f64 {
    capped_bonus(total_eff, 1200.0, 3.0)
}

pub fn calculate_resource_income_per_hour(total_eff: f64, level: i32) -> f64 {
    let level_mult = calculate_resource_level_multiplier(level);
    let worker_mult = calculate_resource_worker_multiplier(total_eff);
    200.0 * 1200.0 * level_mult * worker_mult
}

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
    fn test_s_curve_zero_level() {
        assert_eq!(s_curve_multiplier(0, 4.0, 6.0, 2.0), 1.0);
    }

    #[test]
    fn test_farm_bonuses() {
        assert!(calculate_farm_time_bonus(500.0) > 0.0);
        assert!(calculate_farm_capacity_bonus(1000.0) > 0.0);
        assert!(calculate_farm_harvest_bonus(1200.0) > 0.0);
    }
}
