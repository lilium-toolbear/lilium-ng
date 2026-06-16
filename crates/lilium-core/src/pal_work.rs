use crate::pal_work_constants::*;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(level = "debug" fields(level, suitability, matched, rarity))]
pub fn calculate_efficiency(level: i32, suitability: i32, matched: bool, rarity: i32) -> f64 {
    let role_match = if matched {
        ROLE_MATCH_BONUS
    } else {
        ROLE_MISMATCH_PENALTY
    };
    EFF_BASE
        * (1.0 + EFF_LEVEL_FACTOR * level as f64)
        * (1.0 + EFF_SUIT_RARITY_STEP * rarity as f64).powi(suitability)
        * role_match
}

#[instrument(level = "debug" fields(level))]
pub fn calculate_exp_needed(level: i32) -> i64 {
    (PAL_EXP_BASE * (level as f64).powf(PAL_EXP_POWER)) as i64
}

#[instrument(level = "debug" fields(efficiency))]
pub fn calculate_exp_gain_per_hour(efficiency: f64) -> f64 {
    efficiency * PAL_EXP_RATE
}

#[instrument(level = "debug" fields(food, level, role = %role))]
pub fn calculate_turnip_consumption_per_hour(food: i32, level: i32, role: &str) -> f64 {
    food as f64 * (1.0 + CONSUMPTION_LEVEL_FACTOR * level as f64) * role_cost(role)
}

#[instrument(level = "debug" fields(role = %role, suitabilities_len = suitabilities.len()))]
pub fn get_work_score(suitabilities: &HashMap<String, i32>, role: &str) -> i32 {
    let matching = role_suitabilities(role);
    matching.iter().filter_map(|t| suitabilities.get(*t)).sum()
}

#[instrument(level = "debug" fields(role = %role, suitabilities_len = suitabilities.len()))]
pub fn is_role_matched(suitabilities: &HashMap<String, i32>, role: &str) -> bool {
    let matching = role_suitabilities(role);
    matching
        .iter()
        .any(|t| suitabilities.get(*t).copied().unwrap_or(0) > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_efficiency_increases_with_level() {
        let e1 = calculate_efficiency(1, 3, true, 5);
        let e10 = calculate_efficiency(10, 3, true, 5);
        assert!(e10 > e1);
    }

    #[test]
    fn test_matched_beats_unmatched() {
        let matched = calculate_efficiency(10, 3, true, 5);
        let unmatched = calculate_efficiency(10, 3, false, 5);
        assert!(matched > unmatched);
    }

    #[test]
    fn test_efficiency_level_1_suit_0() {
        let eff = calculate_efficiency(1, 0, true, 1);
        assert!((eff - 41.2).abs() < 0.01);
    }

    #[test]
    fn test_efficiency_level_30_suit_4() {
        let eff = calculate_efficiency(30, 4, true, 1);
        let expected = 40.0 * (1.0 + 0.03 * 30.0) * 1.05f64.powi(4);
        assert!((eff - expected).abs() < 0.01);
    }

    #[test]
    fn test_efficiency_level_30_unmatched() {
        let eff = calculate_efficiency(30, 4, false, 1);
        let expected = 40.0 * (1.0 + 0.03 * 30.0) * 1.05f64.powi(4) * 0.6;
        assert!((eff - expected).abs() < 0.01);
    }

    #[test]
    fn test_efficiency_rarity_5_suit_4() {
        let eff = calculate_efficiency(30, 4, true, 5);
        let expected = 40.0 * (1.0 + 0.03 * 30.0) * (1.0 + 0.05 * 5.0f64).powi(4);
        assert!((eff - expected).abs() < 0.01);
    }

    #[test]
    fn test_efficiency_rarity_no_effect_at_zero_suit() {
        let eff_r1 = calculate_efficiency(10, 0, true, 1);
        let eff_r10 = calculate_efficiency(10, 0, true, 10);
        assert!((eff_r1 - eff_r10).abs() < 0.01);
    }

    #[test]
    fn test_exp_needed_grows_with_level() {
        let e1 = calculate_exp_needed(1);
        let e10 = calculate_exp_needed(10);
        assert!(e10 > e1);
    }

    #[test]
    fn test_exp_needed_level_1() {
        assert_eq!(calculate_exp_needed(1), 12);
    }

    #[test]
    fn test_exp_needed_level_10() {
        assert_eq!(calculate_exp_needed(10), 301);
    }

    #[test]
    fn test_exp_needed_level_60() {
        assert_eq!(calculate_exp_needed(60), 3703);
    }

    #[test]
    fn test_exp_gain_per_hour() {
        assert_eq!(calculate_exp_gain_per_hour(100.0), 50.0);
    }

    #[test]
    fn test_turnip_consumption_mine_level_30() {
        let c = calculate_turnip_consumption_per_hour(5, 30, "mine");
        assert!((c - 8.45).abs() < 0.0001);
    }

    #[test]
    fn test_work_score() {
        let mut s = HashMap::new();
        s.insert("采矿".to_string(), 3);
        assert_eq!(get_work_score(&s, "mine"), 3);
        assert_eq!(get_work_score(&s, "farm"), 0);
    }

    #[test]
    fn test_work_score_upgrade_sum() {
        let mut s = HashMap::new();
        s.insert("手工作业".to_string(), 3);
        s.insert("采矿".to_string(), 2);
        s.insert("伐木".to_string(), 1);
        assert_eq!(get_work_score(&s, "upgrade"), 6);
    }

    #[test]
    fn test_role_matched() {
        let mut s = HashMap::new();
        s.insert("采矿".to_string(), 3);
        assert!(is_role_matched(&s, "mine"));
        assert!(!is_role_matched(&s, "farm"));
    }

    #[test]
    fn test_is_role_matched_multi() {
        let mut s = HashMap::new();
        s.insert("手工作业".to_string(), 3);
        s.insert("采矿".to_string(), 2);
        assert!(is_role_matched(&s, "upgrade"));
        assert!(is_role_matched(&s, "mine"));
        assert!(!is_role_matched(&s, "farm"));
    }
}
