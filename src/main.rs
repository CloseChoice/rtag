#[macro_use] extern crate prettytable;
extern crate clap;
pub mod rtag_sqlite;

use rusqlite::Connection;

use crate::rtag_sqlite::{create_db_and_initialize_tables, create_new_tag, insert_path, show_all, show_tags, show_paths, delete_by_id, delete_by_tag};
use clap::{App, AppSettings, Arg, SubCommand};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let conn = create_db_and_initialize_tables().unwrap();

    let matches = App::new("rtag")
        .about("Revolutional tagging")
        .version("1.0")
        .author("Me")
        .subcommand(
            // todo: must take two arguments!!!
            SubCommand::with_name("tag")
                .about("tag files")
                .arg(
                    Arg::with_name("tag")
                        .help("Tag to use for the path")
                        .required(true),
                )
                .arg(
                    Arg::with_name("path")
                        .help("The path to tag")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("search").about("search in tags").arg(
                Arg::with_name("pattern")
                    .help("The remote repo to push things to")
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("create").about("create new tag").arg(
                Arg::with_name("tag")
                    .help("New tag to create")
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("show").about("show tags")
                .arg(
                    Arg::with_name("all")
                        .long("all")
                        .short("a"))
                .arg(
                    Arg::with_name("tags")
                        .long("tags")
                        .short("t")
                        .takes_value(true)
                        .multiple(true)
                        .conflicts_with("all")
                        .conflicts_with("paths")
                )
                .arg(
                    Arg::with_name("paths")
                        .long("paths")
                        .short("p")
                        .takes_value(true)
                        .multiple(true)
                        .conflicts_with("all")
                        .conflicts_with("tags"))
            )  
        .subcommand(
            SubCommand::with_name("delete").about("delete existing tags")
            .arg(
                Arg::with_name("ids")
                .long("ids")
                .short("i")
                .takes_value(true)
                .multiple(true))
            .arg(
                Arg::with_name("tags")
                .long("tags")
                .short("t")
                .takes_value(true)
                .multiple(true)
            )
        )
        .get_matches();

    // At this point, the matches we have point to git. Keep this in mind...

    // You can check if one of git's subcommands was used
    if matches.is_present("tag") {
        println!("'tag' was run.");
    }

    // You can see which subcommand was used
    if let Some(subcommand) = matches.subcommand_name() {
        println!("'rtag {}' was used", subcommand);

        // It's important to note, this *only* check's git's DIRECT children, **NOT** it's
        // grandchildren, great grandchildren, etc.
        //
        // i.e. if the command `git push remove --stuff foo` was run, the above will only print out,
        // `git push` was used. We'd need to get push's matches to see further into the tree
    }

    // An alternative to checking the name is matching on known names. Again notice that only the
    // direct children are matched here.
    match matches.subcommand_name() {
        Some("tag") => println!("'rtag tag' was used"),
        Some("search") => println!("'rtag search' was used"),
        Some("create") => println!("'rtag create' was used"),
        Some("show") => println!("'rtag show' was used"),
        Some("delete") => println!("'rtag delete' was used"),
        None => println!("No subcommand was used"),
        _ => unreachable!(), // Assuming you've listed all direct children above, this is unreachable
    }

    // You could get the independent subcommand matches, although this is less common
    if let Some(clone_matches) = matches.subcommand_matches("tag") {
        // Now we have a reference to clone's matches
        println!("Tagging path: {}", clone_matches.value_of("path").unwrap());
    }

    // The most common way to handle subcommands is via a combined approach using
    // `ArgMatches::subcommand` which returns a tuple of both the name and matches
    match matches.subcommand() {
        ("tag", Some(clone_matches)) => {
            // Now we have a reference to clone's matches
            tag_path(
                conn,
                clone_matches.value_of("path"),
                clone_matches.value_of("tag"),
            );
            println!("Tagging {}", clone_matches.value_of("path").unwrap());
        }
        ("search", Some(push_matches)) => {
            // Now we have a reference to push's matches
            println!(
                "Searching for {}",
                push_matches.value_of("pattern").unwrap()
            );
        }
        ("create", Some(create_tag_matches)) => {
            create_new_tag(&conn, create_tag_matches.value_of("tag").unwrap()).unwrap();
            println!("Create tag {}", create_tag_matches.value_of("tag").unwrap());
        }
        ("show", Some(show_matches)) => {
            if show_matches.is_present("all") {
            show_all(&conn);
            }
            else if show_matches.is_present("tags") {
                let tag_vec: Vec<String> = show_matches.value_of("tags").unwrap().split(',').map(|c| "'".to_string() + c + "'").collect();
                let tags = tag_vec.join(",");
                show_tags(&conn, tags);
                println!("tags is present. Value: {:?}", show_matches.value_of("tags").unwrap().split(',').collect::<Vec<&str>>());
            }
            else if show_matches.is_present("paths") {
                let path_vec = show_matches.value_of("paths").unwrap().split(',').map(|w| String::from(w)).collect::<Vec<String>>();
                println!("paths is present");
                show_paths(&conn, path_vec);
            }
            else {
                panic!("Didn't find anything in search which I can work with!!!")
            }
        }
        ("delete", Some(delete_matches)) => {
            println!("inside delete");
            if delete_matches.is_present("tags") {
                println!("tags are present");
                let tag_vec: Vec<String> = delete_matches.value_of("tags").unwrap().split(',').map(|c| "'".to_string() + c + "'").collect();
                println!("these are the tags");
                delete_by_tag(&conn, tag_vec);
            }
            if delete_matches.is_present("ids") {
                let ids: String  = String::from(delete_matches.value_of("ids").unwrap());
                delete_by_id(&conn, ids);
            }
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }

    // create the sqlite db if not already present

    // Continued program logic goes here...
}

fn tag_path(conn: Connection, path_as_str: Option<&str>, tag: Option<&str>) {
    let path = PathBuf::from(path_as_str.unwrap());
    match fs::canonicalize(&path) {
        Ok(path) => {
            println!("This will be saved to the db: {}", path.to_str().unwrap());
            insert_path(&conn, path.to_str().unwrap(), tag.unwrap());
        }
        Err(error) => panic!(
            "Couldn't find the path {}. Received error: {:?}",
            path_as_str.unwrap(),
            error
        ),
    }
}

fn save_to_sqlite() {}
