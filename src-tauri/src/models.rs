use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EmailAddress {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub size: u64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<EmailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<EmailAddress>>,
    pub subject: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "htmlBody")]
    pub html_body: Option<String>,
    pub date: String,
    pub read: bool,
    pub starred: bool,
    pub folder: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fromUserId")]
    pub from_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "toUserIds")]
    pub to_user_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ccUserIds")]
    pub cc_user_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bccUserIds")]
    pub bcc_user_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub id: String,
    pub email: String,
    pub name: String,
    #[serde(rename = "imapServer")]
    pub imap_server: String,
    #[serde(rename = "imapPort")]
    pub imap_port: u16,
    #[serde(rename = "smtpServer")]
    pub smtp_server: String,
    #[serde(rename = "smtpPort")]
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    #[serde(rename = "useSsl")]
    pub use_ssl: bool,
    #[serde(rename = "allowInvalidCerts")]
    pub allow_invalid_certs: bool,
    #[serde(rename = "allowInvalidSmtpCerts")]
    pub allow_invalid_smtp_certs: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEmail {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<EmailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<EmailAddress>>,
    pub subject: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
}

impl Email {
    pub fn new(
        from: EmailAddress,
        to: Vec<EmailAddress>,
        subject: String,
        body: String,
        folder: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            cc: None,
            bcc: None,
            subject,
            body,
            html_body: None,
            date: Utc::now().to_rfc3339(),
            read: false,
            starred: false,
            folder,
            attachments: None,
            account_id: None,
            message_id: None,
            from_user_id: None,
            to_user_ids: None,
            cc_user_ids: None,
            bcc_user_ids: None,
        }
    }
}

impl EmailAccount {
    pub fn new(
        email: String,
        name: String,
        imap_server: String,
        imap_port: u16,
        smtp_server: String,
        smtp_port: u16,
        username: String,
        password: String,
        use_ssl: bool,
        allow_invalid_certs: bool,
        allow_invalid_smtp_certs: bool,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email,
            name,
            imap_server,
            imap_port,
            smtp_server,
            smtp_port,
            username,
            password,
            use_ssl,
            allow_invalid_certs,
            allow_invalid_smtp_certs,
            user_id: None,
        }
    }
}
