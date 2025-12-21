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

pub fn group_conversation_id(
    owner_id: &str,
    group_name: &str,
    participants: &[String],
    random_id: &str,
) -> String {
    let random_id = Uuid::parse_str(random_id)
        .map(|id| id.to_string())
        .unwrap_or_else(|_| random_id.to_string());
    let mut group_id = unique_conversation_id(owner_id, group_name);
    group_id = unique_conversation_id(&group_id, &random_id);

    let mut sorted = participants.to_vec();
    sorted.sort();
    for participant in sorted {
        group_id = unique_conversation_id(&group_id, &participant);
    }
    group_id
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

        assert_eq!(id, "05a671c6-6aef-3a12-8cc0-8b76ea6d30bb".to_string());
    }

    #[test]
    fn test_unique_conversation_id() {
        let id = unique_conversation_id("test1", "test2");
        println!("id: {}", id);
        assert_eq!(id, "beff3fcb-a56f-3967-bc5d-52b843df365e");
    }

    #[test]
    fn test_group_conversation_id() {
        let participants = vec!["user-b".to_string(), "user-a".to_string()];
        let id = group_conversation_id(
            "owner",
            "group",
            &participants,
            "00000000-0000-0000-0000-000000000000",
        );
        let participants_rev = vec!["user-a".to_string(), "user-b".to_string()];
        let id2 = group_conversation_id(
            "owner",
            "group",
            &participants_rev,
            "00000000-0000-0000-0000-000000000000",
        );
        assert_eq!(id, id2);
    }
}
