use rand::{distributions::Alphanumeric, Rng}; // 0.8

pub fn generate_random_token(length: u8) -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length.into())
        .map(char::from)
        .collect();
    s
}
