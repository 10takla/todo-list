use super::*;
use todo_list::list::{task::Date, List};

#[test]
fn execute() {
    use ListError::*;

    let template_task = Task {
        title: "Test Task".into(),
        descr: "This is a test task".into(),
        category: "TestCategory".into(),
        date: "2024-08-20 12:00".parse::<Date>().unwrap(),
        is_done: false,
    };

    let mut list = List(vec![]);

    let execute = |args, list: &mut List| {
        let matches = get_app().get_matches_from(args);
        execute_command(matches.subcommand().unwrap(), list).map_err(|e| {
            let ExecuteError::ErrorsList(e) = e else {
                unreachable!()
            };
            e
        })
    };

    // add
    let add_args = vec![
        "todo_app",
        "add",
        "Test Task",
        "This is a test task",
        "2024-08-20 12:00",
        "TestCategory",
    ];

    assert_eq!(execute(add_args.clone(), &mut list), Ok(()));

    // Добавляем задачу в пустой список
    assert_eq!(list.len(), 1);
    assert_eq!(list[0], template_task);

    // Добавляем задачу с таким же именем -> Ошибка
    assert_eq!(execute(add_args, &mut list), Err(TaskAlreadyExists));
    assert_eq!(list.len(), 1);

    // done
    let done_args = vec!["todo_app", "done", "Test Task"];

    // Завершаем задачу
    assert_eq!(execute(done_args.clone(), &mut list), Ok(()));
    assert_eq!(
        list[0],
        Task {
            is_done: true,
            ..template_task
        }
    );

    // Задача уже была заверешена -> Ошибка
    assert_eq!(execute(done_args, &mut list), Err(TaskAlreadyCompleted));
    // Задача не существует заверешена -> Ошибка
    assert_eq!(
        execute(vec!["todo_app", "done", "Do"], &mut list),
        Err(TaskNotExist("Do".into()))
    );

    // delete
    let delete_args = vec!["todo_app", "delete", "Test Task"];

    assert_eq!(execute(delete_args.clone(), &mut list), Ok(()));
    assert_eq!(list.len(), 0);

    assert_eq!(
        execute(delete_args.clone(), &mut list),
        Err(TaskNotExist("Test Task".into()))
    );
}
