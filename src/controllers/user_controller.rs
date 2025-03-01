use crate::services::user_service::UserService;

pub struct UserController {
    user_service: dyn UserService,
}
