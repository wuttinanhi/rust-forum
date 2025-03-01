use crate::services::user_service::UserService;

pub struct UserController {
    pub user_service: dyn UserService,
}
