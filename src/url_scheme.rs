use std::collections::HashMap;
use std::fmt;
use url::Url;

const URL_SCHEME: &str = "mixin";

/// scheme of a user
///
/// userId required
///
/// https://developers.mixin.one/docs/schema#popups-user-profile
pub fn scheme_users(user_id: &str) -> String {
    let mut u = Url::parse(&format!("{}://users", URL_SCHEME)).unwrap();
    u.set_path(user_id);
    u.to_string()
}

/// scheme of a transfer
///
/// userId required
///
/// https://developers.mixin.one/docs/schema#invoke-transfer-page
pub fn scheme_transfer(user_id: &str) -> String {
    let mut u = Url::parse(&format!("{}://transfer", URL_SCHEME)).unwrap();
    u.set_path(user_id);
    u.to_string()
}

/// scheme of a pay
///
/// assetId required
/// recipientId required, receiver's user id
/// amount require, transfer amount
/// traceId optional, UUID, prevent duplicate payment
/// memo optional, transaction memo
///
/// https://developers.mixin.one/docs/schema#invoke-payment-page
pub fn scheme_pay(
    asset_id: &str,
    trace_id: &str,
    recipient_id: &str,
    memo: &str,
    amount: &str,
) -> String {
    let mut u = Url::parse(&format!("{}://pay", URL_SCHEME)).unwrap();
    u.query_pairs_mut()
        .append_pair("asset", asset_id)
        .append_pair("trace", trace_id)
        .append_pair("amount", amount)
        .append_pair("recipient", recipient_id)
        .append_pair("memo", memo);
    u.to_string()
}

/// scheme of a code
///
/// code required
///
/// https://developers.mixin.one/docs/schema#popus-code-info
pub fn scheme_codes(code_id: &str) -> String {
    let mut u = Url::parse(&format!("{}://codes", URL_SCHEME)).unwrap();
    u.set_path(code_id);
    u.to_string()
}

/// scheme of a snapshot
///
/// snapshotId required if no traceId
/// traceId required if no snapshotId
///
/// https://developers.mixin.one/docs/schema#transfer-details-interface
pub fn scheme_snapshots(snapshot_id: &str, trace_id: &str) -> String {
    let mut u = Url::parse(&format!("{}://snapshots", URL_SCHEME)).unwrap();
    if !snapshot_id.is_empty() {
        u.set_path(snapshot_id);
    }
    if !trace_id.is_empty() {
        u.query_pairs_mut().append_pair("trace", trace_id);
    }
    u.to_string()
}

/// scheme of a conversation
///
/// userID optional, for user conversation only, if there's not conversation with the user, messenger will create the conversation first
///
/// https://developers.mixin.one/docs/schema#open-an-conversation
pub fn scheme_conversations(conversation_id: &str, user_id: &str) -> String {
    let mut u = Url::parse(&format!("{}://conversations", URL_SCHEME)).unwrap();
    if !conversation_id.is_empty() {
        u.set_path(conversation_id);
    }
    if !user_id.is_empty() {
        u.query_pairs_mut().append_pair("user", user_id);
    }
    u.to_string()
}

/// scheme of an app
///
/// appID required, userID of an app
/// action optional, action about this scheme, default is "open"
/// params optional, parameters of any name or type can be passed when opening the bot homepage to facilitate the development of features like invitation codes, visitor tracking, etc
///
/// https://developers.mixin.one/docs/schema#popups-bot-profile
pub fn scheme_apps(app_id: &str, action: &str, params: &HashMap<String, String>) -> String {
    let mut u = Url::parse(&format!("{}://apps", URL_SCHEME)).unwrap();
    if !app_id.is_empty() {
        u.set_path(app_id);
    }

    let mut query = u.query_pairs_mut();
    if !action.is_empty() {
        query.append_pair("action", action);
    } else {
        query.append_pair("action", "open");
    }

    for (k, v) in params {
        query.append_pair(k, v);
    }
    // query_pairs_mut() returns a mutable borrow which must be dropped
    // before `u` can be used again.
    drop(query);

    u.to_string()
}

#[derive(Debug)]
pub enum SendSchemeCategory {
    Text,
    Image,
    Contact,
    AppCard,
    Live,
    Post,
}

impl fmt::Display for SendSchemeCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SendSchemeCategory::Text => write!(f, "text"),
            SendSchemeCategory::Image => write!(f, "image"),
            SendSchemeCategory::Contact => write!(f, "contact"),
            SendSchemeCategory::AppCard => write!(f, "app_card"),
            SendSchemeCategory::Live => write!(f, "live"),
            SendSchemeCategory::Post => write!(f, "post"),
        }
    }
}

/// scheme of a share
///
/// category required, category of shared content
/// data required, shared content
/// conversationID optional, If you specify conversation and it is the conversation of the user's current session, the confirmation box shown above will appear, the message will be sent after the user clicks the confirmation; if the conversation is not specified or is not the conversation of the current session, an interface where the user chooses which session to share with will show up.
///
/// https://developers.mixin.one/docs/schema#sharing
pub fn scheme_send(category: SendSchemeCategory, data: &[u8], conversation_id: &str) -> String {
    let mut u = Url::parse(&format!("{}://send", URL_SCHEME)).unwrap();
    let mut query = u.query_pairs_mut();
    query.append_pair("category", &category.to_string());
    if !data.is_empty() {
        let encoded_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        query.append_pair("data", &encoded_data);
    }
    if !conversation_id.is_empty() {
        query.append_pair("conversation", conversation_id);
    }
    drop(query);
    u.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_users() {
        let url = scheme_users("123");
        assert_eq!(url, "mixin://users/123");
    }

    #[test]
    fn test_scheme_transfer() {
        let url = scheme_transfer("123");
        assert_eq!(url, "mixin://transfer/123");
    }

    #[test]
    fn test_scheme_pay() {
        let url = scheme_pay("123", "456", "789", "memo", "10.5");
        assert_eq!(
            url,
            "mixin://pay?asset=123&trace=456&amount=10.5&recipient=789&memo=memo"
        );
    }

    #[test]
    fn test_scheme_codes() {
        let url = scheme_codes("123");
        assert_eq!(url, "mixin://codes/123");
    }

    #[test]
    fn test_scheme_conversations() {
        let url = scheme_conversations("123", "456");
        assert_eq!(url, "mixin://conversations/123?user=456");
    }

    #[test]
    fn test_scheme_apps() {
        let url = scheme_apps("123", "456", &HashMap::new());
        assert_eq!(url, "mixin://apps/123?action=456");
    }

    #[test]
    fn test_scheme_send() {
        let url = scheme_send(SendSchemeCategory::Text, &[], "456");
        assert_eq!(url, "mixin://send?category=text&conversation=456");
    }
}
