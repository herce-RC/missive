use crate::models::{Email, EmailAccount, EmailAddress, NewEmail};
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    transport::smtp::client::{Tls, TlsParametersBuilder},
};
use imap::ClientBuilder;
use mailparse::{addrparse, parse_mail, dateparse, MailAddr, ParsedMail, MailHeaderMap};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("IMAP error: {0}")]
    ImapError(String),
    #[error("SMTP error: {0}")]
    SmtpError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
}

pub type Result<T> = std::result::Result<T, EmailError>;

pub struct EmailClient {
    account: EmailAccount,
}

impl EmailClient {
    pub fn new(account: EmailAccount) -> Self {
        Self { account }
    }
    
    /// Test connection to IMAP and SMTP servers
    pub async fn test_connection(&self) -> Result<()> {
        // Test IMAP + SMTP
        self.test_imap_connection().await?;
        self.test_smtp_connection().await?;
        
        Ok(())
    }
    
    async fn test_smtp_connection(&self) -> Result<()> {
        let mailer = self.build_smtp_transport()?;
        
        mailer
            .test_connection()
            .await
            .map_err(|e| EmailError::SmtpError(e.to_string()))?;
        
        Ok(())
    }

    async fn test_imap_connection(&self) -> Result<()> {
        let account = self.account.clone();
        tokio::task::spawn_blocking(move || {
            let client = ClientBuilder::new(account.imap_server.clone(), account.imap_port)
                .danger_skip_tls_verify(account.allow_invalid_certs)
                .connect()
                .map_err(|e| EmailError::ImapError(e.to_string()))?;

            let mut session = client
                .login(account.username, account.password)
                .map_err(|e| EmailError::AuthError(e.0.to_string()))?;

            session
                .logout()
                .map_err(|e| EmailError::ImapError(e.to_string()))?;

            Ok::<(), EmailError>(())
        })
        .await
        .map_err(|e| EmailError::ConnectionError(e.to_string()))??;

        Ok(())
    }
    
    /// Fetch emails from IMAP server
    /// Note: IMAP functionality is temporarily disabled due to async runtime compatibility issues
    pub async fn fetch_emails(&self, folder: &str, _limit: u32) -> Result<Vec<Email>> {
        let account = self.account.clone();
        let folder = folder.to_string();
        let limit = _limit.max(1) as usize;

        let emails = tokio::task::spawn_blocking(move || {
            let client = ClientBuilder::new(account.imap_server.clone(), account.imap_port)
                .danger_skip_tls_verify(account.allow_invalid_certs)
                .connect()
                .map_err(|e| EmailError::ImapError(e.to_string()))?;

            let mut session = client
                .login(account.username, account.password)
                .map_err(|e| EmailError::AuthError(e.0.to_string()))?;

            session
                .select(&folder)
                .map_err(|e| EmailError::ImapError(e.to_string()))?;

            let uids_set = session
                .uid_search("ALL")
                .map_err(|e| EmailError::ImapError(e.to_string()))?;
            let mut uids: Vec<u32> = uids_set.into_iter().collect();
            uids.sort_unstable();

            if uids.is_empty() {
                let _ = session.logout();
                return Ok::<Vec<Email>, EmailError>(vec![]);
            }

            let start = uids.len().saturating_sub(limit);
            let slice = &uids[start..];
            let sequence = slice
                .iter()
                .map(|u: &u32| u.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let fetches = session
                .uid_fetch(sequence, "(UID FLAGS RFC822)")
                .map_err(|e| EmailError::ImapError(e.to_string()))?;

            let mut result = Vec::new();
            for msg in fetches.iter() {
                let uid = msg.uid.ok_or_else(|| EmailError::ParseError("Missing UID".into()))?;
                let raw = msg.body().ok_or_else(|| EmailError::ParseError("Empty message body".into()))?;
                let parsed = parse_mail(raw).map_err(|e| EmailError::ParseError(e.to_string()))?;

                let subject = header_value(&parsed, "Subject").unwrap_or_else(|| "(Sans objet)".to_string());
                let from_list = parse_addresses(&parsed, "From");
                let to_list = parse_addresses(&parsed, "To");
                let cc_list = parse_addresses(&parsed, "Cc");
                let bcc_list = parse_addresses(&parsed, "Bcc");

                let from = from_list
                    .first()
                    .cloned()
                    .unwrap_or(EmailAddress { name: "".into(), email: "".into() });

                let date = header_value(&parsed, "Date")
                    .and_then(|d| dateparse(&d).ok())
                    .and_then(|ts| {
                        if ts >= 0 {
                            Some(chrono::DateTime::<chrono::Utc>::from(
                                std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts as u64),
                            ).to_rfc3339())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                let body = find_body(&parsed, "text/plain").unwrap_or_default();
                let html_body = find_body(&parsed, "text/html");

                let message_id = header_value(&parsed, "Message-ID");

                let read = msg
                    .flags()
                    .iter()
                    .any(|f| matches!(f, imap::types::Flag::Seen));

                let email = Email {
                    id: format!("{}:{}", account.id, uid),
                    from,
                    to: if to_list.is_empty() { vec![] } else { to_list },
                    cc: if cc_list.is_empty() { None } else { Some(cc_list) },
                    bcc: if bcc_list.is_empty() { None } else { Some(bcc_list) },
                    subject,
                    body,
                    html_body,
                    date,
                    read,
                    starred: false,
                    folder: folder.clone(),
                    attachments: None,
                    account_id: Some(account.id.clone()),
                    message_id,
                };

                result.push(email);
            }

            let _ = session.logout();
            Ok::<Vec<Email>, EmailError>(result)
        })
        .await
        .map_err(|e| EmailError::ConnectionError(e.to_string()))??;

        Ok(emails)
    }
    
    /// Send an email via SMTP
    pub async fn send_email(&self, email: &NewEmail) -> Result<()> {
        let from_mailbox: Mailbox = format!("{} <{}>", email.from.name, email.from.email)
            .parse()
            .map_err(|e: lettre::address::AddressError| EmailError::ParseError(e.to_string()))?;
        
        let mut message_builder = Message::builder()
            .from(from_mailbox)
            .subject(&email.subject);
        
        // Add recipients
        for recipient in &email.to {
            let to_mailbox: Mailbox = if recipient.name.is_empty() {
                recipient.email.parse()
            } else {
                format!("{} <{}>", recipient.name, recipient.email).parse()
            }
            .map_err(|e: lettre::address::AddressError| EmailError::ParseError(e.to_string()))?;
            
            message_builder = message_builder.to(to_mailbox);
        }
        
        // Add CC
        if let Some(cc_list) = &email.cc {
            for cc in cc_list {
                let cc_mailbox: Mailbox = if cc.name.is_empty() {
                    cc.email.parse()
                } else {
                    format!("{} <{}>", cc.name, cc.email).parse()
                }
                .map_err(|e: lettre::address::AddressError| EmailError::ParseError(e.to_string()))?;
                
                message_builder = message_builder.cc(cc_mailbox);
            }
        }
        
        // Add BCC
        if let Some(bcc_list) = &email.bcc {
            for bcc in bcc_list {
                let bcc_mailbox: Mailbox = if bcc.name.is_empty() {
                    bcc.email.parse()
                } else {
                    format!("{} <{}>", bcc.name, bcc.email).parse()
                }
                .map_err(|e: lettre::address::AddressError| EmailError::ParseError(e.to_string()))?;
                
                message_builder = message_builder.bcc(bcc_mailbox);
            }
        }
        
        // Build message body
        let message = if email.attachments.is_some() && !email.attachments.as_ref().unwrap().is_empty() {
            // With attachments - multipart
            let mut multipart = MultiPart::mixed().singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(email.body.clone()),
            );
            
            // Add attachments
            if let Some(attachments) = &email.attachments {
                for attachment in attachments {
                    if let Some(data) = &attachment.data {
                        let content_type = ContentType::parse(&attachment.mime_type)
                            .unwrap_or(ContentType::parse("application/octet-stream").unwrap());
                        
                        multipart = multipart.singlepart(
                            SinglePart::builder()
                                .header(content_type)
                                .header(lettre::message::header::ContentDisposition::attachment(&attachment.filename))
                                .body(data.clone()),
                        );
                    }
                }
            }
            
            message_builder
                .multipart(multipart)
                .map_err(|e| EmailError::SmtpError(e.to_string()))?
        } else {
            // Plain text only
            message_builder
                .header(ContentType::TEXT_PLAIN)
                .body(email.body.clone())
                .map_err(|e| EmailError::SmtpError(e.to_string()))?
        };
        
        // Create SMTP transport
        let mailer = self.build_smtp_transport()?;
        
        // Send the email
        mailer
            .send(message)
            .await
            .map_err(|e| EmailError::SmtpError(e.to_string()))?;
        
        Ok(())
    }

    fn build_smtp_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(
            self.account.username.clone(),
            self.account.password.clone(),
        );

        let tls_parameters = if self.account.allow_invalid_smtp_certs {
            Some(
                TlsParametersBuilder::new(self.account.smtp_server.clone())
                    .dangerous_accept_invalid_hostnames(true)
                    .dangerous_accept_invalid_certs(true)
                    .build()
                    .map_err(|e: lettre::transport::smtp::Error| EmailError::SmtpError(e.to_string()))?,
            )
        } else {
            None
        };

        let builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            self.account.smtp_server.clone(),
        )
        .port(self.account.smtp_port)
        .credentials(creds);

        let builder = if self.account.use_ssl {
            if let Some(tls) = tls_parameters {
                builder.tls(Tls::Required(tls))
            } else {
                builder
            }
        } else if let Some(tls) = tls_parameters {
            builder.tls(Tls::Wrapper(tls))
        } else {
            builder
        };

        Ok(builder.build())
    }
    
    /// Mark email as read/unread on IMAP server
    /// Note: IMAP functionality is temporarily disabled
    pub async fn set_read_flag(&self, _uid: &str, _read: bool) -> Result<()> {
        // TODO: Implement with proper async runtime compatibility
        log::warn!("IMAP set_read_flag not yet implemented");
        Ok(())
    }
    
    /// Delete email (move to trash or permanently delete)
    /// Note: IMAP functionality is temporarily disabled
    pub async fn delete_email(&self, _uid: &str) -> Result<()> {
        // TODO: Implement with proper async runtime compatibility
        log::warn!("IMAP delete_email not yet implemented");
        Ok(())
    }
}

fn header_value(mail: &ParsedMail, name: &str) -> Option<String> {
    mail.get_headers()
        .get_first_value(name)
        .map(|v: String| v.trim().to_string())
}

fn parse_addresses(mail: &ParsedMail, header_name: &str) -> Vec<EmailAddress> {
    let value = match header_value(mail, header_name) {
        Some(v) => v,
        None => return vec![],
    };

    let addrs = match addrparse(&value) {
        Ok(a) => a,
        Err(_) => return vec![],
    };

    addrs
        .iter()
        .flat_map(|addr| match addr {
            MailAddr::Single(info) => vec![EmailAddress {
                name: info.display_name.clone().unwrap_or_default(),
                email: info.addr.clone(),
            }],
            MailAddr::Group(group) => group
                .addrs
                .iter()
                .map(|info| EmailAddress {
                    name: info.display_name.clone().unwrap_or_default(),
                    email: info.addr.clone(),
                })
                .collect::<Vec<_>>(),
        })
        .collect()
}

fn find_body(part: &ParsedMail, mime: &str) -> Option<String> {
    if part.ctype.mimetype.eq_ignore_ascii_case(mime) {
        return part.get_body().ok();
    }

    for sub in &part.subparts {
        if let Some(body) = find_body(sub, mime) {
            return Some(body);
        }
    }

    None
}
