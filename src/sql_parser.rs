use anyhow::{anyhow, Result};
use chumsky::{error::Rich, prelude::*};

#[derive(Debug, PartialEq)]
enum DataType {
    VarChar255,
}

#[derive(Debug, PartialEq)]
struct Column {
    name: String,
    data_type: DataType,
}

// NULL, True, "foo", 21 etc.
#[derive(Debug, PartialEq)]

enum ColVal {
    Boolean(bool),
    String(String),
    Int(u64),
}

#[derive(Debug, PartialEq)]
struct NewColumnVal {
    column_name: String,
    value: ColVal,
}

#[derive(Debug, PartialEq)]
enum Expr {
    Select {
        columns: Vec<String>,
        from_table: String,
    },
    Insert {
        into_table: String,
        columns: Vec<NewColumnVal>,
    },
}
//CreateTable {
//    table_name:  <String>,
//    columns: <Vec<(&'a Column>, // (name, value) pairs
//    column_names: <Vec<String>,
//    column_values: <Vec<String>,
//}

// TRUE, "foo", 21 etc.
fn column_value<'a>() -> impl Parser<'a, &'a str, ColVal, extra::Err<Rich<'a, char>>> {
    let bool_val = just("TRUE").or(just("FALSE")).map(|b| {
        if b == "TRUE" {
            ColVal::Boolean(true)
        } else {
            ColVal::Boolean(false)
        }
    });

    let str_val = just("\"")
        .ignored()
        .then(text::ident())
        .then_ignore(just("\""))
        .map(|(_, s): (_, &str)| ColVal::String(s.to_string()));

    let int_val = text::digits(10)
        .to_slice()
        .try_map(|n: &str, span| match n.parse::<u64>() {
            Ok(num) => Ok(ColVal::Int(num)),
            Err(e) => {
                return Err(Rich::custom(
                    span,
                    format!("Error parsing int as column val: {}", e),
                ))
            }
        });

    return bool_val.or(int_val).or(str_val);
}

// parse column values separated by commas for exmaple:  NULL, True, "foo", 21 etc.
fn column_vals<'a>(//  p: impl Parser<'a, &'a str, ColVal<'a>, extra::Err<Rich<'a, char>>>,
) -> impl Parser<'a, &'a str, Vec<ColVal>, extra::Err<Rich<'a, char>>> {
    return column_value::<'a>()
        .padded()
        .separated_by(just(',').padded().repeated().at_least(1))
        .collect::<Vec<_>>();
}

fn csv<'a>() -> impl Parser<'a, &'a str, Vec<&'a str>, extra::Err<Rich<'a, char>>> {
    let ident = text::ascii::ident().padded();

    // comma separated values, for example foo, bar, goo
    let csv = ident
        .padded()
        .separated_by(just(',').padded().repeated().at_least(1))
        .collect::<Vec<_>>();
    csv
}

/// this is an insert whereby a subset of the columns can be inserted - some columns may be left unspecified
/// INSERT INTO table_name (column1, column2, column3, ...)
/// VALUES (value1, value2, value3, ...);
fn insert_patch<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> {
    return text::keyword("INSERT")
        .padded()
        .then_ignore(text::keyword("INTO").padded())
        .then(text::ident().padded())
        .then_ignore(just("("))
        .padded()
        .then(csv())
        .then_ignore(just(")"))
        .padded()
        .then_ignore(text::keyword("VALUES").padded())
        .then_ignore(just("("))
        .padded()
        .then(column_vals())
        .then_ignore(just(")"))
        .then_ignore(just(';'))
        .try_map(|(((_, table_name), col_names), col_values), span| {
            if col_names.len() != col_values.len() {
                return Err(Rich::custom(
                    span,
                    "Column names has different arity to values".to_string(),
                ));
            }
            let columns: Vec<NewColumnVal> = col_names
                .into_iter()
                .zip(col_values)
                .map(|(column_name, value): (&str, ColVal)| NewColumnVal {
                    column_name: column_name.to_string(),
                    value: value,
                })
                .collect();

            Ok(Expr::Insert {
                into_table: table_name.to_string(),
                columns,
            })
        });
}

/// SELECT name, age FROM users WHERE age > 21;
fn select<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> {
    return text::keyword("SELECT")
        .ignored()
        .padded()
        .then(csv())
        .then_ignore(text::keyword("FROM").padded())
        .then(text::ident().padded())
        .then_ignore(just(';'))
        .map(
            |((_, columns), table_name): ((_, Vec<&str>), &str)| Expr::Select {
                columns: columns.into_iter().map(|c: &str| c.to_string()).collect(),
                from_table: table_name.to_string(),
            },
        );
}

fn parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> {
    //  recursive(|value| {
    select().or(insert_patch()).padded()
}

pub fn parse_and_print(src: &str) {
    match parser().parse(src).into_result() {
        Ok(ast) => println!("{:?}", ast),
        Err(parse_errs) =>
        // println!("uh oh"), //parse_errs
        {
            parse_errs
                .into_iter()
                .for_each(|e| println!("Parse error: {:?}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /*Query
    SELECT name, age FROM users WHERE age > 21;

    AST
    ├── Select
    │   ├── Columns
    │   │   ├── Column "name"
    │   │   └── Column "age"
    │   ├── From
    │   │   └── Table "users"
    │   └── Where
    │       └── GreaterThan
    │           ├── Column "age"
    │           └── Value 21

     */

    /*
    Query
    INSERT INTO users (name, age) VALUES ('Alice', 30);

    AST
    ├── Insert
    │   ├── Table "users"
    │   ├── Columns
    │   │   ├── Column "name"
    │   │   └── Column "age"
    │   └── Values
    │       ├── Value "Alice"
    │       └── Value 30 */

    //INSERT INTO Product(Name, Age) VALUES (Tom', 21);

    #[test]
    fn parse_insert_patch() {
        let query = r#"INSERT INTO Person(Name, Age, Admin) VALUES ("Bob", 21, FALSE);"#;

        assert_eq!(
            parser().parse(query).unwrap(),
            Expr::Insert {
                columns: vec![
                    NewColumnVal {
                        column_name: "Name".to_string(),
                        value: ColVal::String("Bob".to_string())
                    },
                    NewColumnVal {
                        column_name: "Age".to_string(),
                        value: ColVal::Int(21)
                    },
                    NewColumnVal {
                        column_name: "Admin".to_string(),
                        value: ColVal::Boolean(false)
                    }
                ],
                into_table: "Person".to_string()
            }
        );
    }

    #[test]
    fn parse_basic_select() {
        // let result = parse_and_print("SELECT name,age FROM user;");

        assert_eq!(
            parser().parse("SELECT name, age FROM user;").unwrap(),
            Expr::Select {
                columns: vec!["name".to_string(), "age".to_string()],
                from_table: "user".to_string()
            }
        );
    }

    #[test]
    fn parse_basic_select_with_extra_whitespace() {
        assert_eq!(
            parser()
                .parse("  SELECT  name , age FROM   user  ;  ")
                .unwrap(),
            Expr::Select {
                columns: vec!["name".to_string(), "age".to_string()],
                from_table: "user".to_string()
            }
        );
    }

    /*
    Query
    UPDATE users SET age = age + 1 WHERE name = 'Bob';

    AST
     ├── Update
     │   ├── Table "users"
     │   ├── Set
     │   │   └── Assignment
     │   │       ├── Column "age"
     │   │       └── Expression
     │   │           ├── Column "age"
     │   │           └── Plus
     │   │               └── Value 1
     │   └── Where
     │       └── Equals
     │           ├── Column "name"
     │           └── Value "Bob"
      */

    /*
      CreateTableStatement

      CREATE TABLE Persons (
         PersonID int,
         LastName varchar(255),
         FirstName varchar(255),
         Address varchar(255),
         City varchar(255)
      );
    |
    +-- TableName: "users"
    +-- Columns
    |    |
    |    +-- ColumnDefinition
    |    |    |
    |    |    +-- ColumnName: "id"
    |    |    +-- DataType: "INTEGER"
    |    |    +-- Constraints
    |    |        |
    |    |        +-- PrimaryKeyConstraint
    |    +-- ColumnDefinition
    |         |
    |         +-- ColumnName: "name"
    |         +-- DataType: "TEXT"
    |         +-- Constraints
    +-- Constraints
         |
         +-- PrimaryKeyConstraint */
}
