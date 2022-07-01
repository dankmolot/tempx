use rand::Rng;
use rocket::serde::uuid;

fn rng() -> [u8; 16] {
    rand::thread_rng().gen::<[u8; 16]>()
}

pub fn uuid4() -> uuid::Uuid {
    let random_bytes = rng();

    uuid::Builder::from_random_bytes( random_bytes ).into_uuid()
}

pub fn get_file_prefix(filename: &str) -> Option<&str> {
    filename.split_once('.')
        .map(|v| v.0)
}