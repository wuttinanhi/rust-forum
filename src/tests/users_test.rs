#[cfg(test)]
mod tests {
    use crate::tests::debug_response_data;
    use crate::{
        entities::user::{UserLoginFormData, UserRegisterFormData},
        servers::server_actix::create_actix_app,
        tests, AppKit,
    };
    use actix_web::http::StatusCode;
    use dotenv::dotenv;

    #[actix_web::test]
    async fn test_should_get_users_register_route() {
        dotenv().ok();

        let app_kit = AppKit::new_for_testing();

        let actix_app = create_actix_app(app_kit);

        let app = actix_web::test::init_service(actix_app).await;

        let req = actix_web::test::TestRequest::get().uri("/").to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        // let body: serde_json::Value = actix_web::test::read_body_json(resp).await;

        // let body = actix_web::test::read_body(resp).await;
        // dbg!(&body);
    }

    #[actix_web::test]
    async fn test_should_able_to_register_user() {
        dotenv().ok();

        let app_kit = AppKit::new_for_testing();

        let actix_app = create_actix_app(app_kit.clone());

        let app = actix_web::test::init_service(actix_app).await;

        let user_register_form_data = UserRegisterFormData {
            email: "adam@example.com".to_string(),
            name: "adam example".to_string(),
            password: "adampassword".to_string(),
            cf_turnstile_response: None,
        };

        let req = actix_web::test::TestRequest::post()
            .uri("/users/register")
            .set_form(&user_register_form_data)
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;

        let get_user_by_email = app_kit.user_service.get_user_by_email("adam@example.com");

        dbg!(&get_user_by_email);

        let user = get_user_by_email.unwrap();
        assert_eq!(user.email, "adam@example.com");
        assert_eq!(user.name, "adam example");
        assert_eq!(user.role, "user");

        assert_eq!(resp.status(), StatusCode::FOUND);

        tests::debug_response_data(resp).await;
    }

    #[actix_web::test]
    async fn test_should_able_to_login_user() {
        dotenv().ok();

        env_logger::init();

        let app_kit = AppKit::new_for_testing();

        let actix_app = create_actix_app(app_kit.clone());

        let app = actix_web::test::init_service(actix_app).await;

        let user_register_form_data = UserRegisterFormData {
            email: "adam@example.com".to_string(),
            name: "adam rustforum".to_string(),
            password: "adampassword".to_string(),
            cf_turnstile_response: None,
        };

        let register_req = actix_web::test::TestRequest::post()
            .uri("/users/register")
            .set_form(&user_register_form_data)
            .to_request();

        let register_resp = actix_web::test::call_service(&app, register_req).await;

        assert_eq!(register_resp.response().status(), StatusCode::FOUND);

        let user_login_form = UserLoginFormData {
            email: "adam@example.com".to_string(),
            password: "adampassword".to_string(),
            cf_turnstile_response: None,
        };

        let login_req = actix_web::test::TestRequest::post()
            .uri("/users/login")
            .set_form(&user_login_form)
            .to_request();

        let login_resp = actix_web::test::call_service(&app, login_req).await;

        assert_eq!(login_resp.response().status(), StatusCode::FOUND);

        debug_response_data(login_resp).await;
    }
}
