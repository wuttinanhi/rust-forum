use actix_web::dev::ServiceResponse;

mod users_test;

pub async fn debug_response_data(resp: ServiceResponse<crate::servers::server_actix::NestedBody>) {
    dbg!((&resp).response().status());

    dbg!(&resp.headers());

    // get response body
    let resp_read_body_bytes = actix_web::test::read_body(resp).await;
    let response_body = String::from_utf8(resp_read_body_bytes.to_vec()).unwrap();
    dbg!(&response_body);
}
