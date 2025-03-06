use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bcrypt::{hash, DEFAULT_COST};

use crate::{
    db::WebError,
    entities::user::{user_to_user_public, validate_user_password, UserPublic},
    models::{UpdateUserNameAndProfilePicture, User},
};

use super::user_repository::UserRepository;

pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<i32, User>>>,
    next_id: Arc<Mutex<i32>>,
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    type Error = WebError;

    fn create_user(&self, name: &str, email: &str, password: &str) -> Result<User, WebError> {
        let hashed = hash(password, DEFAULT_COST).unwrap();

        let mut id_guard = self.next_id.lock().unwrap();
        let user_id = *id_guard;
        *id_guard += 1;

        let new_user = User {
            id: user_id,
            email: email.to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now().naive_utc(),
            role: "user".to_string(),
            updated_at: chrono::Utc::now().naive_utc(),
            user_profile_picture_url: Some("".to_string()),
            password: hashed,
        };

        self.users.lock().unwrap().insert(user_id, new_user.clone());
        Ok(new_user)
    }

    fn login_user(&self, user_email: &str, user_password: &str) -> Result<User, WebError> {
        let users = self.users.lock().unwrap();
        let user = users.values().find(|u| u.email == user_email);

        if let Some(user) = user {
            if validate_user_password(user, user_password) {
                return Ok(user.clone());
            }
        }

        Err(Box::new(diesel::result::Error::NotFound))
    }

    fn get_user_by_id(&self, user_id: i32) -> Result<User, WebError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .get(&user_id)
            .cloned()
            .ok_or_else(|| Box::new(diesel::result::Error::NotFound))?)
    }

    fn get_user_by_email(&self, user_email: &str) -> Result<User, WebError> {
        let users = self.users.lock().unwrap();

        let user = users
            .values()
            .find(|u| u.email == user_email)
            .cloned()
            .ok_or_else(|| Box::new(diesel::result::Error::NotFound))?;

        Ok(user)
    }

    fn get_user_sanitized_by_id(&self, target_user_id: i32) -> Result<UserPublic, WebError> {
        let user = self.get_user_by_id(target_user_id)?;
        Ok(user_to_user_public(&user))
    }

    fn update_user_password(&self, user: &User, new_password: &str) -> Result<(), WebError> {
        let hashed = hash(new_password, DEFAULT_COST).unwrap();
        let mut users = self.users.lock().unwrap();
        if let Some(u) = users.get_mut(&user.id) {
            u.password = hashed;
            return Ok(());
        }
        Err(Box::new(diesel::result::Error::NotFound))
    }

    fn update_user_email(&self, user: &User, new_email: &str) -> Result<(), WebError> {
        let mut users = self.users.lock().unwrap();
        if let Some(u) = users.get_mut(&user.id) {
            u.email = new_email.to_string();
            return Ok(());
        }
        Err(Box::new(diesel::result::Error::NotFound))
    }

    fn update_user_data(
        &self,
        user: &User,
        new_data: &UpdateUserNameAndProfilePicture,
    ) -> Result<(), WebError> {
        let mut users = self.users.lock().unwrap();
        if let Some(u) = users.get_mut(&user.id) {
            u.name = new_data.name.unwrap().to_string();
            return Ok(());
        }
        Err(Box::new(diesel::result::Error::NotFound))
    }

    fn delete_user(&self, user: &User) -> Result<(), WebError> {
        let mut users = self.users.lock().unwrap();
        if users.remove(&user.id).is_some() {
            return Ok(());
        }
        Err(Box::new(diesel::result::Error::NotFound))
    }
}
