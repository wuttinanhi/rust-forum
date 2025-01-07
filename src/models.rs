use crate::schema::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Debug,
    Eq,
    PartialEq,
    AsChangeset,
    Serialize,
    Deserialize,
    Clone,
)]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub user_id: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub published: bool,
    pub user_id: i32,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Associations,
    Debug,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    Clone,
)]
#[diesel(belongs_to(Post))]
#[diesel(table_name = comments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub post_id: i32,
    pub user_id: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = comments)]
pub struct NewComment<'a> {
    pub content: &'a str,
    pub post_id: i32,
    pub user_id: i32,
}

#[derive(
    Queryable, Selectable, Identifiable, Debug, Eq, PartialEq, AsChangeset, Serialize, Deserialize,
)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub user_profile_picture_url: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUserNameAndProfilePicture<'a> {
    pub name: Option<&'a str>,
    pub user_profile_picture_url: Option<&'a str>,
}
