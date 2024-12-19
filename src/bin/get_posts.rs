use diesel::prelude::*;
use rust_forum::{establish_connection, models::Post};

fn main() {
    use rust_forum::schema::posts::dsl::*;

    let connection = &mut establish_connection();
    let results = posts
        .filter(published.eq(true))
        .limit(5)
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");

    println!("Displaying {} posts\n\n\n", results.len());

    for post in results {
        println!("post title: {}", post.title);
        println!("post body: {}", post.body);
        println!("-----------\n");
    }
}
