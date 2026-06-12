use std::collections::HashMap;
use crate::pal_work_constants::*;

pub fn calculate_efficiency(level: i32, suitability: i32, matched: bool, rarity: i32) -> f64 {
    let role_match = if matched { ROLE_MATCH_BONUS } else { ROLE_MISMATCH_PENALTY };
    EFF_BASE
        * (1.0 + EFF_LEVEL_FACTOR * level as f64)
        * (1.0 + EFF_SUIT_RARITY_STEP * rarity as f64).powi(suitability)
        * role_match
}

pub fn calculate_exp_needed(level: i32) -> i64 {
    (PAL_EXP_BASE * (level as f64).powf(PAL_EXP_POWER)) as i64
}

pub fn calculate_exp_gain_per_hour(efficiency: f64) -> f64 {
    efficiency * PAL_EXP_RATE
}

pub fn calculate_turnip_consumption_per_hour(food: i32, level: i32, role: &str) -> f64 {
    food as f64 * (1.0 + CONSUMPTION_LEVEL_FACTOR * level as f64) * role_cost(role)
}

pub fn get_work_score(suitabilities: &HashMap<String, i32>, role: &str) -> i32 {
    let matching = role_suitabilities(role);
    matching
        .iter()
        .filter_map(|t| suitabilities.get(*t))
        .sum()
}

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
    fn test_exp_needed_grows_with_level() {
        let e1 = calculate_exp_needed(1);
        let e10 = calculate_exp_needed(10);
        assert!(e10 > e1);
    }

    #[test]
    fn test_work_score() {
        let mut s = HashMap::new();
        s.insert("采矿".to_string(), 3);
        assert_eq!(get_work_score(&s, "mine"), 3);
        assert_eq!(get_work_score(&s, "farm"), 0);
    }

    #[test]
    fn test_role_matched() {
        let mut s = HashMap::new();
        s.insert("采矿".to_string(), 3);
        assert!(is_role_matched(&s, "mine"));
        assert!(!is_role_matched(&s, "farm"));
    }
}
