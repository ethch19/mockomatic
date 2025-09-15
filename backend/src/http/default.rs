pub fn default_uuid_v7() -> uuid::Uuid {
    uuid::Uuid::now_v7()
}

pub fn default_uuid_v4() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}