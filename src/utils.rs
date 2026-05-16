use uuid::Uuid;

use crate::mix_address::hash256;

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

pub fn hash_members<T, I>(ids: I) -> String
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut ids: Vec<String> = ids.into_iter().map(|id| id.as_ref().to_string()).collect();
    ids.sort();
    hex::encode(hash256(ids.join("").as_bytes()))
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
    fn test_hash_members() {
        let hash = hash_members(["965e5c6e-434c-3fa9-b780-c50f43cd955c"]);
        assert_eq!(
            hash,
            "b9f49cf777dc4d03bc54cd1367eebca319f8603ea1ce18910d09e2c540c630d8"
        );

        let ids = [
            "965e5c6e-434c-3fa9-b780-c50f43cd955c",
            "d1e9ec7e-199d-4578-91a0-a69d9a7ba048",
        ];
        let reverse_ids = [
            "d1e9ec7e-199d-4578-91a0-a69d9a7ba048",
            "965e5c6e-434c-3fa9-b780-c50f43cd955c",
        ];
        assert_eq!(
            hash_members(ids),
            "6064ec68a229a7d2fe2be652d11477f21705a742e08b75564fd085650f1deaeb"
        );
        assert_eq!(hash_members(ids), hash_members(reverse_ids));
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
