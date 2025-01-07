use rust_forum::posts::repository::create_post;
use rust_forum::*;
use std::io::{stdin, Read};

fn main() {
    let connection = &mut establish_connection();

    let mut title = String::new();
    let mut body = String::new();

    println!("What would you like your title to be?");
    stdin().read_line(&mut title).unwrap();
    let title = title.trim_end(); // Remove the trailing newline

    println!("\nOk! Let's write {title} (Press {EOF} when finished)\n",);
    stdin().read_to_string(&mut body).unwrap();

    let post_result = create_post(connection, &1, title, &body);

    match post_result {
        Ok(post) => println!("\nSaved post id #{}", post.id),
        Err(e) => println!("failed to save post: {}", e),
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
#[allow(dead_code)]
const EOF: &str = "CTRL+Z";
