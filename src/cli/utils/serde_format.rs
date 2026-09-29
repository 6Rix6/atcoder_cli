pub mod duration_ms_f64 {
    use serde::{Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ms = duration.as_secs_f64() * 1000.0;
        serializer.serialize_f64(ms)
    }
}
