pub fn default_uuid() -> uuid::Uuid {
    uuid::Uuid::now_v7()
}

pub fn default_time() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}
