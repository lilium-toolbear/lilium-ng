pub const PAL_MAX_LEVEL: i32 = 120;
pub const PAL_EXP_BASE: f64 = 12.0;
pub const PAL_EXP_POWER: f64 = 1.4;
pub const PAL_EXP_RATE: f64 = 0.5;

pub const EFF_BASE: f64 = 40.0;
pub const EFF_LEVEL_FACTOR: f64 = 0.03;
pub const EFF_SUIT_RARITY_STEP: f64 = 0.05;
pub const ROLE_MATCH_BONUS: f64 = 1.0;
pub const ROLE_MISMATCH_PENALTY: f64 = 0.6;

pub const UPGRADE_WORK_BASE: f64 = 8000.0;
pub const UPGRADE_WORK_MULTIPLIER: f64 = 2.0;

pub const CREDIT_RATE: f64 = 100.0;

pub const CONSUMPTION_LEVEL_FACTOR: f64 = 0.01;

pub const TICK_INTERVAL_MINUTES: i64 = 10;
pub const TICKS_PER_HOUR: i64 = 6;

pub const WORK_COLLECTION_BONUS_MAX: f64 = 0.10;
pub const WORK_ACTIVE_SPECIES_BONUS_MAX: f64 = 0.20;
pub const WORK_ELEMENT_BONUS_MAX: f64 = 0.10;
pub const WORK_GENDER_COLLECTION_BONUS_MAX: f64 = 0.08;
pub const WORK_GENDER_ACTIVE_BONUS_MAX: f64 = 0.07;
pub const WORK_ACTIVE_SPECIES_TARGET: i32 = 12;
pub const WORK_TOTAL_ELEMENTS: i32 = 9;
pub const WORK_MULTIPLIER_CAP: f64 = 1.55;

pub fn role_cost(role: &str) -> f64 {
    match role {
        "farm" => 1.0,
        "warehouse" => 1.1,
        "upgrade" => 1.2,
        "mine" => 1.3,
        "lumber_mill" => 1.3,
        "workshop" => 1.1,
        "dormitory" => 1.1,
        _ => 1.0,
    }
}

pub fn role_suitabilities(role: &str) -> &'static [&'static str] {
    match role {
        "farm" => &["浇水", "播种", "采集", "牧场", "制药"],
        "warehouse" => &["搬运", "手工作业", "采矿", "伐木", "冷却"],
        "upgrade" => &["手工作业", "采矿", "伐木", "发电", "生火"],
        "mine" => &["采矿"],
        "lumber_mill" => &["伐木"],
        "workshop" => &["手工作业"],
        "dormitory" => &["手工作业", "搬运", "制药", "牧场"],
        _ => &[],
    }
}
