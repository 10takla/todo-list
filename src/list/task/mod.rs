pub mod date;

use super::*;
pub use date::Date;
use macros::Table;
use std_reset::prelude::Default;

#[derive(Clone, Deserialize, Serialize, Debug, Default, PartialEq, Table)]
pub struct Task {
    pub title: String,
    pub descr: String,
    pub date: Date,
    pub category: String,
    pub is_done: bool,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = prettytable::Table::new();
        table.add_row(Task::get_keys().iter().collect());

        table.add_row(
            self.get_entries()
                .iter()
                .map(|(key, value)| Task::format_by_key(&key, value.to_string()))
                .collect(),
        );
        write!(f, "{}", table)
    }
}

impl Task {
    pub fn change_by_key(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "title" => {
                self.title = value.to_string();
                Ok(())
            }
            "descr" => {
                self.descr = value.to_string();
                Ok(())
            }
            "date" => value
                .parse()
                .map(|value| {
                    self.date = value;
                })
                .map_err(|_| format!("Ожидается true или false")),
            "category" => {
                self.category = value.to_string();
                Ok(())
            }
            "is_done" => value
                .parse()
                .map(|value| {
                    self.is_done = value;
                })
                .map_err(|_| format!("Ожидается true или false")),
            _ => unreachable!(),
        }
    }

    pub fn format_by_key(key: &str, value: String) -> String {
        if key == "date" {
            NaiveDateTime::parse_from_str(&value.trim_matches('"'), "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| value)
        } else {
            value
        }
    }
}
