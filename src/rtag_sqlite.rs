use rusqlite::{NO_PARAMS, Statement};
use rusqlite::{Connection, Error, Result};
use prettytable::{Table, Row, Cell};

static dim_fct_rows: &'static [&'static str] = &["ID", "TAG", "PATH", "TIME_CREATED"];

pub fn create_db_and_initialize_tables() -> Result<Connection, Error> {
    let conn = Connection::open("rtag.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS dim_tag (
                id              INTEGER PRIMARY KEY,
                tag_name VARCHAR UNIQUE,
                time_created    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS fct_tag (
                id  INTEGER,
                path VARCHAR
                )",
        NO_PARAMS,
    )
    .unwrap();
    Ok(conn)
}
fn insert_path_tag_to_fct_tag(conn: &Connection, tag_id: i32, path: &str, tag: &str) -> Result<()> {
    let query = format!("insert into fct_tag (id, path) values ({}, '{}')", tag_id, path);
    conn.execute(query.as_str(), NO_PARAMS)?;
    println!("Added path {} to tag {}", path, tag);
    Ok(())
}

fn get_id_of_tag(conn: &Connection, tag_name: &str) -> Result<(i32)> {
    let select_query = format!("select id from dim_tag where tag_name = '{}'", tag_name);
    conn.query_row(select_query.as_str(), NO_PARAMS, |row| row.get(0))
}

fn check_if_path_tag_exists(conn: &Connection, path: &str, tag: &str) -> Result<i32> {
    let select_query = format!("select count(*) from dim_tag a join fct_tag b using (id) where path = '{}' and tag_name = '{}'", path, tag);
    conn.query_row(select_query.as_str(), NO_PARAMS, |row| row.get(0))
}

pub fn insert_path(conn: &Connection, path: &str, tag: &str) -> Result<()> {
    let select_result: Result<i32> =
        get_id_of_tag(conn, tag);
    
    println!("this is insert_path: {:?}", check_if_path_tag_exists(conn, path, tag));
    // todo: check that no double occurances can happen!!!
    if let Ok(0) = check_if_path_tag_exists(conn, path, tag) {
        match select_result {
            Ok(id) => {
                insert_path_tag_to_fct_tag(conn, id, path, tag);
            }
            Err(e) => {
                println!("Couldn't find tag {}. Create new tag", tag);
                create_new_tag(conn, tag);
                let tag_id = get_id_of_tag(conn, tag).unwrap();
                insert_path_tag_to_fct_tag(conn, tag_id, path, tag);
            }
        }
    }
    else {
        println!("The combination of tag {} and path {} already exists", tag, path);
    }
    
    Ok(())
}

pub fn create_new_tag(conn: &Connection, tag: &str) -> Result<()> {
    let query = format!("insert into dim_tag (tag_name) values ('{}')", tag);
    let query_str = query.as_str();
    println!("this is the query str: {}", query_str);
    conn.execute(query_str, NO_PARAMS).unwrap();
    Ok(())
}

#[derive(Debug)]
struct dim_fct_tag {
    id: i32,
    tag: String,
    path: String,
    time_created: String,
}

#[derive(Debug)]
struct dim_tag {
    id: i32,
    tag_name: String,
    time_created: String,
}

#[derive(Debug)]
struct id {
    id: i32,
}

pub fn show_all(conn: &Connection) -> Result<()> {
    let sql = "SELECT id, tag_name, path, time_created FROM dim_tag join fct_tag using (id)";
    show_sql(&conn, sql, dim_fct_rows)
}

pub fn show_sql(conn: &Connection, sql_statement: &str, row_headers: &[&str]) -> Result<()> {
    let mut table = Table::new();
    let stmt = conn.prepare(sql_statement);
    let mut stmt_un = stmt.unwrap();
    let table_iter = stmt_un.query_map(NO_PARAMS, |row| {
        Ok(dim_fct_tag {
            id: row.get(0)?,
            tag: row.get(1)?,
            path: row.get(2)?,
            time_created: row.get(3)?,
        })
    });
    table.add_row(Row::from(row_headers));
    for row in table_iter.unwrap().into_iter() {
        let row_un = row.unwrap();
        table.add_row(row![row_un.id, row_un.tag, row_un.path, row_un.time_created]);
    }
    table.printstd();

    Ok(())
}

pub fn show_tags(conn: &Connection, tags: String) -> Result<()> {
    println!("Before creating prepare");
    let sql = format!("SELECT id, tag_name, path, time_created FROM dim_tag join fct_tag using (id) where tag_name in ({})", tags);
    show_sql(&conn, sql.as_str(), dim_fct_rows)
}

pub fn show_paths(conn: &Connection, paths: Vec<String>) -> Result<()> {
    println!("Before creating prepare in show_paths");
    let end_str = paths.join("%' or path like '%");
    let mut paths_query = String::from("path like '%");
    paths_query.push_str(end_str.as_str());
    paths_query.push_str("%'");
    println!("this is the second paths_query: {}", paths_query);
    let sql = format!("SELECT id, tag_name, path, time_created FROM dim_tag join fct_tag using (id) where {}", paths_query);
    show_sql(&conn, sql.as_str(), dim_fct_rows)
}

pub fn delete_by_id(conn: &Connection, ids: String) -> Result<()> {
    println!("Delete the following ids: {:?}", ids);
    let query_fct_tag = format!("delete from fct_tag where id in ({})", ids);
    let query_dim_tag = format!("delete from dim_tag where id in ({})", ids);
    conn.execute(query_fct_tag.as_str(), NO_PARAMS)?;
    conn.execute(query_dim_tag.as_str(), NO_PARAMS)?;
    Ok(())
}

pub fn delete_by_tag(conn: &Connection, tags: Vec<String>) -> Result<()> {
    let tags_str = tags.join(",");
    let query_get_ids = format!("select id from dim_tag where tag_name in ({})", tags_str);
    let stmt = conn.prepare(query_get_ids.as_str());
    let mut stmt_un = stmt.unwrap();
    let table_iter = stmt_un.query_map(NO_PARAMS, |row| {
        Ok(id {
            id: row.get(0)?
        })
    });
    let ids = table_iter.unwrap().into_iter().map(|row| row.unwrap().id.to_string()).collect::<Vec<String>>().join(",");
    delete_by_id(&conn, ids);
    Ok(())

}