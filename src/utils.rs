use uuid::Uuid;

pub fn unique_object_id<T, I>(args: I) -> String
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut hasher = md5::Context::new();
    for s in args {
        hasher.consume(s.as_ref().as_bytes());
    }
    let mut sum: [u8; 16] = hasher.compute().into();
    // Set UUID version to 3 (MD5 hash based)
    sum[6] = (sum[6] & 0x0f) | 0x30;
    // Set UUID variant to RFC 4122
    sum[8] = (sum[8] & 0x3f) | 0x80;
    Uuid::from_bytes(sum).to_string()
}

pub fn unique_conversation_id(user_id: &str, recipient_id: &str) -> String {
    let (min_id, max_id) = if user_id <= recipient_id {
        (user_id, recipient_id)
    } else {
        (recipient_id, user_id)
    };
    let mut hasher = md5::Context::new();
    hasher.consume(min_id.as_bytes());
    hasher.consume(max_id.as_bytes());
    let mut sum: [u8; 16] = hasher.compute().into();

    sum[6] = (sum[6] & 0x0f) | 0x30;
    sum[8] = (sum[8] & 0x3f) | 0x80;
    Uuid::from_bytes(sum).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_object_id() {
        let id = unique_object_id(["test", "test"]);
        println!("{}", id);

        let id2 = unique_object_id(["test".to_string(), "test".to_string()]);
        println!("{}", id2);

        let id3 = unique_object_id(["test".to_string(), "test".to_string()]);
        println!("{}", id3);

        assert_eq!(id, "e7228969313a152303c749a26322b7a912627448".to_string());
    }

    #[test]
    fn test_unique_conversation_id() {
        let id = unique_conversation_id("test1", "test2");
        println!("id: {}", id);
        assert_eq!(id, "beff3fcb-a56f-3967-bc5d-52b843df365e");
    }
}
