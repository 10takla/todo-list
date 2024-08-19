pub mod task;
#[cfg(test)]
mod tests;

use crate::traits::Table;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlparser::{
    ast::{
        self,
        BinaryOperator::{self, *},
        Expr, SetExpr, Statement,
    },
    dialect::GenericDialect,
    parser::{Parser, ParserError},
};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    iter::FromIterator,
};
use std_reset::prelude::Deref;
use task::*;

#[derive(Deref, Deserialize, Serialize, Debug, PartialEq)]
pub struct List(pub Vec<Task>);

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = prettytable::Table::new();
        table.add_row(Task::get_keys().iter().collect());
        self.iter().for_each(|task| {
            table.add_row(
                task.get_entries()
                    .iter()
                    .map(|(key, value)| Task::format_by_key(&key, value.to_string()))
                    .collect(),
            );
        });
        write!(f, "{}", table)
    }
}

impl<'a> FromIterator<&'a Task> for List {
    fn from_iter<T: IntoIterator<Item = &'a Task>>(iter: T) -> Self {
        Self(iter.into_iter().cloned().collect::<Vec<_>>())
    }
}

#[derive(Debug, PartialEq)]
pub enum SqlError {
    NotValidQuery,
    NonExistentField(String),
    UnhandledOperator(BinaryOperator),
    Format(String),
}

#[derive(Debug, PartialEq)]
pub enum ListError {
    TaskAlreadyExists,
    TaskAlreadyCompleted,
    TaskNotChanged,
    TaskNotExist(String),
    Sql(SqlError),
}

use ListError::*;
use SqlError::*;

impl Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TaskAlreadyCompleted => "Задача уже выполнена".into(),
                TaskNotExist(title) => format!("Задача \"{title}\" не найдена"),
                TaskAlreadyExists => "Задача уже существует".into(),
                TaskNotChanged => "Задача не изменена".into(),
                Sql(a) => match a {
                    NotValidQuery => {
                        "Ожидаемый формат запроса: SELECT * [WHERE where_condition]".to_string()
                    }
                    NonExistentField(m) => m.to_string(),
                    UnhandledOperator(op) => format!("Оператор {op} не обрабатывается"),
                    Format(format) => format!("Ожидается формат: {format}"),
                },
            }
        )
    }
}

impl List {
    pub fn get_task(&self, title: &str) -> Result<&Task, ListError> {
        self.iter()
            .find(|task| task.title == title)
            .ok_or(TaskNotExist(title.into()))
    }
    pub fn add(&mut self, task: Task) -> Result<&Task, ListError> {
        if !self.iter().any(|t| *t == task) {
            self.push(task);
            Ok(self.last().as_ref().unwrap())
        } else {
            Err(TaskAlreadyExists)
        }
    }
    pub fn done(&mut self, title: String) -> Result<&Task, ListError> {
        let task = self
            .iter_mut()
            .find(|task| task.title == title)
            .ok_or(TaskNotExist(title))?;

        if task.is_done {
            Err(TaskAlreadyCompleted)
        } else {
            task.is_done = true;
            Ok(task)
        }
    }
    pub fn update(&mut self, title: String, task: &Task) -> Result<&Task, ListError> {
        let finded_task = self
            .iter_mut()
            .find(|task| task.title == title)
            .ok_or(TaskNotExist(title))?;

        if *finded_task == *task {
            Err(TaskNotChanged)
        } else {
            *finded_task = task.clone();
            Ok(finded_task)
        }
    }
    pub fn delete(&mut self, title: String) -> Result<Task, ListError> {
        if let Some(index) = self
            .clone()
            .into_iter()
            .position(|task| task.title == title)
        {
            Ok(self.remove(index))
        } else {
            Err(TaskNotExist(title))
        }
    }
    pub fn select(&self, sql: &str) -> Result<List, ListError> {
        // Парсим AST из sql
        let select = Parser::parse_sql(&GenericDialect {}, sql)
            .and_then(|ast| {
                ast.get(0)
                    .and_then(|stmt| {
                        if let Statement::Query(query) = stmt {
                            Some(query.clone())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| ParserError::ParserError("".into()))
            })
            .and_then(|query| {
                if let SetExpr::Select(select) = (*query.body).clone() {
                    Ok(select)
                } else {
                    Err(ParserError::ParserError("".into()))
                }
            })
            .map_err(|_| Sql(NotValidQuery))?;

        // Если WHERE выражения нет, то вернуть весь список
        let Some(where_) = select.selection else {
            return Ok(List((*self).clone()));
        };

        let mut curr_list = (*self).clone();
        let mut query = VecDeque::from([where_]);

        let binary_op_check = |left| {
            if let Expr::Identifier(ident) = left {
                Ok(ident.value)
            } else {
                Err(Sql(Format("WHERE [Identifier ..]".into())))
            }
        };

        let check_string = |format, right| {
            match right {
                Expr::Value(ast::Value::SingleQuotedString(value)) => Some(value),
                Expr::Identifier(ident) => {
                    matches!(ident.quote_style, Some('"')).then(|| ident.value)
                }
                _ => None,
            }
            .ok_or(Sql(Format(format)))
        };

        while let Some(expr) = query.pop_front() {
            match expr {
                Expr::Nested(expr) => {
                    query.push_back((*expr).clone());
                    continue;
                }
                Expr::BinaryOp { left, right, op } => {
                    let (left, right) = ((*left).clone(), (*right).clone());
                    match op {
                        And => {
                            query.push_back(left);
                            query.push_back(right);
                        }
                        Eq | Gt | Lt | GtEq | LtEq => {
                            let left = binary_op_check(left)?;

                            let not_eq_check = || {
                                if let Eq = op {
                                    Ok(())
                                } else {
                                    Err(Sql(Format(format!("{} = ..", left.as_str()))))
                                }
                            };

                            match left.as_str() {
                                "title" => {
                                    not_eq_check()?;
                                    let value = check_string(format!("[.. StringValue]"), right)?;
                                    curr_list = filter(curr_list, |task| &task.title, &op, &value);
                                }
                                "descr" => {
                                    not_eq_check()?;
                                    let value = check_string(format!("[.. StringValue]"), right)?;
                                    curr_list = filter(curr_list, |task| &task.descr, &op, &value);
                                }
                                "date" => {
                                    let value = check_string(format!("[.. \"Date\"]"), right)
                                        .and_then(|value| {
                                            value
                                                .parse::<Date>()
                                                .map_err(|_| Sql(Format(format!("[.. \"Date\"]"))))
                                        })?;
                                    curr_list = filter(curr_list, |task| &task.date, &op, &value);
                                }
                                "category" => {
                                    not_eq_check()?;
                                    let value = check_string(format!("[.. StringValue]"), right)?;
                                    curr_list =
                                        filter(curr_list, |task| &task.category, &op, &value);
                                }
                                "is_done" => {
                                    not_eq_check()?;

                                    if let Expr::Value(ast::Value::Boolean(value)) = right.clone() {
                                        curr_list =
                                            filter(curr_list, |task| &task.is_done, &op, &value);
                                    } else {
                                        return Err(Sql(Format(format!("[.. true | false]"))));
                                    }
                                }
                                _ => return Err(Sql(NonExistentField(left))),
                            };
                        }
                        op => return Err(Sql(UnhandledOperator(op))),
                    }
                }
                Expr::Like { expr, pattern, .. } => {
                    let left = binary_op_check(*expr)
                        .map_err(|_| Sql(Format("[StringIdnetifier like ..]".into())))?;

                    if ["title", "descr", "category"].contains(&left.as_str()) {
                        let value = check_string(format!("[.. like StringValue]"), *pattern)?;
                        curr_list = curr_list
                            .into_iter()
                            .filter(|task| {
                                task.get_value(&left).unwrap().to_string().contains(&value)
                            })
                            .collect();
                    } else {
                        return Err(Sql(Format("[StringIdnetifier like ..]".into())));
                    }
                }
                _ => return Err(Sql(NotValidQuery)),
            }
        }

        Ok(List(curr_list))
    }
}

fn filter<T: PartialEq + PartialOrd>(
    list: Vec<Task>,
    lhs: fn(&Task) -> &T,
    op: &BinaryOperator,
    rhs: &T,
) -> Vec<Task> {
    list.clone()
        .into_iter()
        .filter(|task| {
            let lhs = lhs(task);
            match op {
                Eq => lhs == rhs,
                Gt => lhs > rhs,
                Lt => lhs < rhs,
                GtEq => lhs >= rhs,
                LtEq => lhs <= rhs,
                _ => {
                    unreachable!()
                }
            }
        })
        .collect()
}
