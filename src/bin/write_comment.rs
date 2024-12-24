use comments::crud::create_comment;
use rust_forum::*;
use std::io::{stdin, Read};

fn main() {
    let connection = &mut establish_connection();

    let mut post_id_input = String::new();
    let mut content_input = String::new();

    println!("post_id?");
    stdin().read_line(&mut post_id_input).unwrap();
    let post_id: i32 = post_id_input.trim().parse().unwrap();

    println!("\ncontent?\n",);
    stdin().read_to_string(&mut content_input).unwrap();

    let comment = create_comment(connection, &1, &post_id, &content_input);
    println!("\nSaved id #{}", comment.id);
}

#[cfg(not(windows))]
#[allow(dead_code)]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
#[allow(dead_code)]
const EOF: &str = "CTRL+Z";
