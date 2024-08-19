use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug, Default, PartialEq, PartialOrd)]
pub struct Date(NaiveDateTime);

impl FromStr for Date {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let formats = &vec![
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%d",
            "%Y-%m-%dT%H:%M:%S",
            "%Y/%m/%d %H:%M:%S",
        ];
        for format in formats {
            if let Ok(date) = NaiveDateTime::parse_from_str(s, format) {
                return Ok(Date(date));
            }
        }
        Err(format!("Ожидается формат: {}", formats.join(" | ")))
    }
}
