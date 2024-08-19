use super::*;
use ListError::*;

#[test]
fn not_exist_task() {
    let mut list = List(vec![]);
    let title = "title".to_string();

    assert_eq!(list.done(title.clone()), Err(TaskNotExist(title.clone())));

    assert_eq!(
        list.update(title.clone(), &Task::default()),
        Err(TaskNotExist(title.clone()))
    );

    assert_eq!(
        list.delete(String::from("do")),
        Err(TaskNotExist("do".into()))
    );
}

#[test]
fn done() {
    // is_done: false
    let mut list = List(vec![Task::default()]);

    // Завршаем задачу
    assert_eq!(
        list.done(String::default()),
        Ok(&Task {
            is_done: true,
            ..Task::default()
        })
    );

    // Пытаемся завершить уже завершенную задачу -> Ошибка
    assert_eq!(list.done(String::default()), Err(TaskAlreadyCompleted));
}
#[test]
fn update() {
    let mut task = Task::default();
    let mut list = List(vec![Task::default()]);

    // Обновляем задачу, не изменив поля -> Ошибка
    assert_eq!(
        list.update(String::default(), &task.clone()),
        Err(TaskNotChanged)
    );

    // Обновляем задачу изменив одно поле
    task.is_done = true;
    assert_eq!(
        list.update(String::default(), &task.clone()),
        Ok(&Task {
            is_done: true,
            ..task
        })
    );
}

#[test]
fn delete() {
    let mut list = List(vec![Task::default()]);
    assert_eq!(list.delete(String::default()), Ok(Task::default()));
}

#[cfg(test)]
mod select {
    use super::*;

    #[test]
    fn not_valid_query() {
        let list = List(vec![Task::default(), Task::default()]);

        assert_eq!(list.select("select *"), Ok(List(list.clone())));

        // NotValidQuery
        assert_eq!(list.select("select * where"), Err(Sql(NotValidQuery)));
        assert_eq!(list.select("* where"), Err(Sql(NotValidQuery)));

        assert_eq!(
            list.select("select * where name = ''"),
            Err(Sql(NonExistentField("name".into())))
        );
    }

    #[test]
    fn select_by_one_field() {
        let list = List(vec![Task::default(), Task::default()]);

        // String

        // both quotes: "", ''
        assert_eq!(
            list.select("select * where title = ''"),
            Ok(List(list.clone()))
        );
        assert_eq!(
            list.select("select * where title = \"\""),
            Ok(List(list.clone()))
        );

        // Проверка оператора
        assert_eq!(
            list.select("select * where title || ''"),
            Err(Sql(UnhandledOperator(StringConcat)))
        );

        assert_eq!(
            list.select("select * where 22 = ''"),
            Err(Sql(Format("WHERE [Identifier ..]".into())))
        );

        assert_eq!(
            list.select("select * where title > \"\""),
            Err(Sql(Format("title = ..".into())))
        );
        assert_eq!(
            list.select("select * where title = 22"),
            Err(Sql(Format("[.. StringValue]".into())))
        );
        assert_eq!(
            list.select("select * where title = title"),
            Err(Sql(Format("[.. StringValue]".into())))
        );

        // Like

        // both quotes: "", ''
        assert_eq!(
            list.select("select * where title like 'tit'"),
            Ok(List(vec![]))
        );
        assert_eq!(
            list.select("select * where title like \"tit\""),
            Ok(List(vec![]))
        );

        assert_eq!(
            list.select("select * where date like 22"),
            Err(Sql(Format("[StringIdnetifier like ..]".into())))
        );

        assert_eq!(
            list.select("select * where title like 22"),
            Err(Sql(Format("[.. like StringValue]".into())))
        );

        assert_eq!(
            list.select("select * where 22 like 22"),
            Err(Sql(Format("[StringIdnetifier like ..]".into())))
        );

        // Bool
        assert_eq!(
            list.select("select * where is_done = false"),
            Ok(List(list.clone()))
        );
        assert_eq!(
            list.select("select * where is_done = 'false'"),
            Err(Sql(Format("[.. true | false]".into())))
        );
        assert_eq!(
            list.select("select * where is_done > false"),
            Err(Sql(Format("is_done = ..".into())))
        );

        // Date -> operateor Eq
        assert_eq!(
            list.select("select * where date = \"1970-01-01 00:00\""),
            Ok(List(list.clone()))
        );
        assert_eq!(
            list.select("select * where date = \"1970-01-01T00:00:00\""),
            Ok(List(list.clone()))
        );

        assert_eq!(
            list.select("select * where date = false"),
            Err(Sql(Format("[.. \"Date\"]".into())))
        );
        assert_eq!(
            list.select("select * where date = 'false'"),
            Err(Sql(Format("[.. \"Date\"]".into())))
        );
        assert_eq!(
            list.select("select * where date > false"),
            Err(Sql(Format("[.. \"Date\"]".into())))
        );

        // Date -> compares operateors
        assert_eq!(
            list.select("select * where date < \"1970-01-01 00:01\""),
            Ok(List(list.clone()))
        );
        assert_eq!(
            list.select("select * where date > \"1970-01-01 00:01\""),
            Ok(List(vec![]))
        );
        assert_eq!(
            list.select("select * where date <= \"1970-01-01 00:00\""),
            Ok(List(list.clone()))
        );
        assert_eq!(
            list.select("select * where date < \"1970-01-01 00:00\""),
            Ok(List(vec![]))
        );
    }

    #[test]
    fn select_by_combinations() {
        let list = List(vec![Task::default(), Task::default()]);

        // Условие Or оператора -> Ошибка
        assert_eq!(
            list.select("select * where title = '' or title like ''"),
            Err(Sql(UnhandledOperator(Or)))
        );

        // Оператор And

        assert_eq!(
            list.select("select * where title = '' and title like '' and date = \"1970-01-01 00:00\" and date <= \"1970-01-01 00:00\""),
            Ok(List(list.clone()))
        );

        assert_eq!(
            list.select(
                "select * where title = 'name' and title like '' and date = \"1970-01-01 00:00\""
            ),
            Ok(List(vec![]))
        );
        assert_eq!(
            list.select(
                "select * where title = '' and title like 'na' and date = \"1970-01-01 00:00\""
            ),
            Ok(List(vec![]))
        );
        assert_eq!(
            list.select(
                "select * where title = '' and title like 'na' and date > \"1970-01-01 00:00\""
            ),
            Ok(List(vec![]))
        );
    }
}
