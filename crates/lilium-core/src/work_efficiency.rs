use crate::pal_work_constants::*;
use tracing::instrument;

#[instrument(fields(numerator, denominator))]
fn clamped_ratio(numerator: i32, denominator: i32) -> f64 {
    if denominator <= 0 {
        return 0.0;
    }
    (numerator.max(0) as f64 / denominator as f64).min(1.0)
}

#[instrument(fields(male_count, female_count))]
pub fn calculate_gender_balance(male_count: i32, female_count: i32) -> f64 {
    let m = male_count.max(0);
    let f = female_count.max(0);
    let total = m + f;
    if total == 0 || m == 0 || f == 0 {
        return 0.0;
    }
    1.0 - ((m - f).abs() as f64 / total as f64)
}

#[instrument(fields(
    discovered_species,
    total_species,
    active_unique_species,
    active_elements,
    paired_species,
    active_male_count,
    active_female_count
))]
pub fn calculate_work_efficiency_multiplier(
    discovered_species: i32,
    total_species: i32,
    active_unique_species: i32,
    active_elements: i32,
    paired_species: i32,
    active_male_count: i32,
    active_female_count: i32,
) -> f64 {
    let collection_progress = clamped_ratio(discovered_species, total_species);
    let m_collect = 1.0 + WORK_COLLECTION_BONUS_MAX * collection_progress.sqrt();

    let active_target = WORK_ACTIVE_SPECIES_TARGET.max(1);
    let active_species_progress =
        (active_unique_species.max(0) as f64).ln_1p() / (active_target as f64).ln_1p();
    let m_rich = 1.0 + WORK_ACTIVE_SPECIES_BONUS_MAX * active_species_progress.min(1.0);

    let element_progress = clamped_ratio(active_elements, WORK_TOTAL_ELEMENTS);
    let m_element = 1.0 + WORK_ELEMENT_BONUS_MAX * element_progress;

    let pair_progress = clamped_ratio(paired_species, discovered_species);
    let m_gender_collect = 1.0 + WORK_GENDER_COLLECTION_BONUS_MAX * pair_progress.sqrt();

    let active_balance = calculate_gender_balance(active_male_count, active_female_count);
    let m_gender_active = 1.0 + WORK_GENDER_ACTIVE_BONUS_MAX * active_balance;

    let total = m_collect * m_rich * m_element * m_gender_collect * m_gender_active;
    total.clamp(1.0, WORK_MULTIPLIER_CAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplier_range() {
        let m = calculate_work_efficiency_multiplier(10, 100, 5, 3, 2, 3, 2);
        assert!(m >= 1.0);
        assert!(m <= WORK_MULTIPLIER_CAP);
    }

    #[test]
    fn test_zero_species() {
        let m = calculate_work_efficiency_multiplier(0, 100, 0, 0, 0, 0, 0);
        assert!(m >= 1.0);
    }

    #[test]
    fn test_gender_balance_edge_cases() {
        assert_eq!(calculate_gender_balance(0, 5), 0.0);
        assert_eq!(calculate_gender_balance(7, 0), 0.0);
    }

    #[test]
    fn test_gender_balance_perfect_split() {
        assert_eq!(calculate_gender_balance(5, 5), 1.0);
    }

    #[test]
    fn test_multiplier_full_diversity_capped() {
        let m = calculate_work_efficiency_multiplier(226, 226, 12, 9, 226, 6, 6);
        assert!((m - 1.55).abs() < 1e-9);
    }

    #[test]
    fn test_multiplier_floor() {
        let m = calculate_work_efficiency_multiplier(0, 226, 0, 0, 0, 0, 0);
        assert_eq!(m, 1.0);
    }

    #[test]
    fn test_multiplier_increases_with_more_active_species() {
        let m_low = calculate_work_efficiency_multiplier(50, 226, 2, 2, 10, 6, 4);
        let m_high = calculate_work_efficiency_multiplier(50, 226, 10, 6, 10, 6, 4);
        assert!(m_high > m_low);
    }
}
