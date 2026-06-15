use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

pub fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

pub fn i64_to_decimal(i: i64) -> Decimal {
    Decimal::from(i)
}
