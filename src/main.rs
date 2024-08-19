#[cfg(test)]
mod tests;

use clap::{Arg, ArgMatches, Command};
use crossterm::style::Stylize;
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    fs,
    io::{self, Write},
};
use todo_list::{
    list::{
        task::{Date, Task},
        List, ListError,
    },
    traits::Table,
};

fn main() {
    let mut list = {
        let tasks_json = fs::read_to_string("tasks.json").unwrap();
        serde_json::from_str::<List>(&tasks_json).unwrap()
    };

    let matches = get_app().get_matches();
    let subcommand = matches.subcommand().unwrap();

    if let Err(e) = execute_command(subcommand, &mut list) {
        println!("{e}");
    }

    if ["add", "done", "update", "delete"].contains(&subcommand.0) {
        fs::write("tasks.json", serde_json::to_string_pretty(&list).unwrap()).unwrap();
    }
}

// clap-интерфейс
fn get_app() -> Command {
    let title = &Arg::new("title").help("Название задачи").required(true);

    Command::new("ToDoList")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("add")
                .about("Добавляет новую задачу")
                .arg(title)
                .arg(Arg::new("descr").help("Описание задачи").required(true))
                .arg(
                    Arg::new("date")
                        .help("Срок выполнения задачи")
                        .required(true)
                        .value_parser(|date_str: &str| {
                            date_str.parse::<Date>().map_err(|e| {
                                clap::Error::raw(clap::error::ErrorKind::InvalidValue, e)
                            })
                        }),
                )
                .arg(Arg::new("category").help("Категория задачи").required(true)),
        )
        .subcommand(
            Command::new("done")
                .about("Помечает задачу как выполненную")
                .arg(title),
        )
        .subcommand(
            Command::new("update")
                .about("Обновляет существующую задачу")
                .arg(title.clone().help("Название задачи для обновления")),
        )
        .subcommand(Command::new("delete").about("Удаляет задачу").arg(title))
        .subcommand(
            Command::new("select").about("Отфильтровать список задач по определенному критерию"),
        )
}

#[derive(Debug)]
enum ExecuteError {
    ErrorsList(ListError),
    String(&'static str),
}

impl Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ExecuteError::*;
        write!(f, "{}: ", "Error".red())?;
        match self {
            ErrorsList(e) => Display::fmt(e, f),
            String(str) => Display::fmt(str, f),
        }
    }
}

fn execute_command(
    (command, sub_m): (&str, &ArgMatches),
    list: &mut List,
) -> Result<(), ExecuteError> {
    match command {
        "add" => handle_add(sub_m, list),
        "done" => handle_done(sub_m, list),
        "update" => handle_update(sub_m, list),
        "delete" => handle_delete(sub_m, list),
        "select" => handle_select(list),
        _ => unreachable!(),
    }
}

fn handle_add(sub_m: &ArgMatches, list: &mut List) -> Result<(), ExecuteError> {
    let new_task = Task {
        title: sub_m.get_one::<String>("title").unwrap().clone(),
        descr: sub_m.get_one::<String>("descr").unwrap().clone(),
        date: sub_m.get_one::<Date>("date").unwrap().clone(),
        category: sub_m.get_one::<String>("category").unwrap().clone(),
        is_done: false,
    };
    list.add(new_task)
        .map(|added_task| {
            println!("Добавлена задача:\n{}", added_task);
        })
        .map_err(|e| ExecuteError::ErrorsList(e))
}

fn handle_done(sub_m: &ArgMatches, list: &mut List) -> Result<(), ExecuteError> {
    let title = sub_m.get_one::<String>("title").unwrap().clone();
    list.done(title)
        .map(|done_task| {
            println!("Задача выполнена:\n{}", done_task);
        })
        .map_err(|e| ExecuteError::ErrorsList(e))
}

fn handle_update(sub_m: &ArgMatches, list: &mut List) -> Result<(), ExecuteError> {
    let title = sub_m.get_one::<String>("title").unwrap().clone();

    // Находим задачу для изменения
    let mut new_task = list
        .get_task(&title)
        .map_err(|e| ExecuteError::ErrorsList(e))?
        .clone();

    let mut empty_field_count = 0;
    let mut is_again = false;
    let task_entries = new_task.get_entries();
    let mut queue = VecDeque::from(task_entries.clone());

    // Изменяем поля найденной задач
    while let Some((key, value)) = queue.get(0) {
        // Интерактивный ввод значения полей
        let value = {
            let prompt = if is_again {
                format!("Введите {key}: > ")
            } else {
                format!(
                    "Текущее значение: {}.\nВведите {key}: > ",
                    Task::format_by_key(key, value.to_string())
                )
            };

            interactive_input(&prompt)
        };
        if !value.is_empty() {
            if let Err(err) = new_task.change_by_key(key, &value) {
                println!("{}: {err}", "Error".red());
                is_again = true;
                continue;
            }
        } else {
            empty_field_count += 1;
        }
        is_again = false;
        queue.pop_front();
    }

    if empty_field_count == task_entries.len() {
        Err(ExecuteError::String("Данные не изменились"))
    } else {
        list.update(title, &new_task)
            .map(|updated_task| {
                println!("Задача обновлена:\n{}", updated_task);
            })
            .map_err(|e| ExecuteError::ErrorsList(e))
    }
}

fn handle_delete(sub_m: &ArgMatches, list: &mut List) -> Result<(), ExecuteError> {
    let title = sub_m.get_one::<String>("title").unwrap().clone();
    list.delete(title)
        .map(|_| {
            println!("Задача удалена");
        })
        .map_err(|e| ExecuteError::ErrorsList(e))
}

fn handle_select(list: &mut List) -> Result<(), ExecuteError> {
    let sql = interactive_input("Введите запрос: > ");
    list.select(&sql)
        .map_err(|e| ExecuteError::ErrorsList(e))
        .map(|selected_list| {
            println!("{}", selected_list);
        })
}

fn interactive_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}
