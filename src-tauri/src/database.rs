use surrealdb::engine::local::{Db, SurrealKV};
use surrealdb::Surreal;
use crate::models::{Email, EmailAccount};
use thiserror::Error;
use serde_json;
use std::path::Path;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SurrealError(#[from] surrealdb::Error),
    #[error("Serialization error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;


const EMAIL_SELECT_FIELDS: &str = "emailId AS id, from, to, cc, bcc, subject, body, htmlBody, date, read, starred, folder, attachments, account_id, message_id, fromUserId, toUserIds, ccUserIds, bccUserIds";

pub struct Database {
    db: Surreal<Db>,
}

impl Database {

    fn user_key(email: &str) -> Option<String> {
        let trimmed = email.trim().to_lowercase();
        if trimmed.is_empty() {
            return None;
        }
        let key: String = trimmed
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        Some(key)
    }

    pub async fn get_or_create_user(&self, email: &str, name: Option<&str>) -> Result<Option<String>> {
        let key = match Self::user_key(email) {
            Some(k) => k,
            None => return Ok(None),
        };

        let content = serde_json::json!({
            "email": email,
            "name": name,
        });

        let _: Option<serde_json::Value> = self.db
            .update(("user", &key))
            .content(content)
            .await?;

        Ok(Some(format!("user:{}", key)))
    }
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = Surreal::new::<SurrealKV>(path.as_ref()).await?;
        
        db.use_ns("missive").use_db("main").await?;
        
        db.query(r#"
            DEFINE TABLE email SCHEMAFULL;
            DEFINE FIELD emailId ON email TYPE string;
            DEFINE FIELD from ON email TYPE object;
            DEFINE FIELD from.name ON email TYPE string;
            DEFINE FIELD from.email ON email TYPE string;
            DEFINE FIELD fromUserId ON email TYPE option<string>;
            DEFINE FIELD to ON email TYPE array;
            DEFINE FIELD to.*.name ON email TYPE string;
            DEFINE FIELD to.*.email ON email TYPE string;
            DEFINE FIELD toUserIds ON email TYPE option<array<string>>;
            DEFINE FIELD cc ON email TYPE option<array>;
            DEFINE FIELD cc.*.name ON email TYPE string;
            DEFINE FIELD cc.*.email ON email TYPE string;
            DEFINE FIELD ccUserIds ON email TYPE option<array<string>>;
            DEFINE FIELD bcc ON email TYPE option<array>;
            DEFINE FIELD bcc.*.name ON email TYPE string;
            DEFINE FIELD bcc.*.email ON email TYPE string;
            DEFINE FIELD bccUserIds ON email TYPE option<array<string>>;
            DEFINE FIELD subject ON email TYPE string;
            DEFINE FIELD body ON email TYPE string;
            DEFINE FIELD htmlBody ON email TYPE option<string>;
            DEFINE FIELD date ON email TYPE string;
            DEFINE FIELD read ON email TYPE bool DEFAULT false;
            DEFINE FIELD starred ON email TYPE bool DEFAULT false;
            DEFINE FIELD folder ON email TYPE string DEFAULT 'inbox';
            DEFINE FIELD attachments ON email TYPE option<array>;
            DEFINE FIELD account_id ON email TYPE option<string>;
            DEFINE FIELD message_id ON email TYPE option<string>;
            DEFINE INDEX email_id ON email FIELDS emailId UNIQUE;
            DEFINE INDEX email_folder ON email FIELDS folder;
            DEFINE INDEX email_date ON email FIELDS date;
        "#).await?;

        db.query(r#"
            DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE option<string>;
            DEFINE FIELD email ON user TYPE string;
            DEFINE INDEX user_email ON user FIELDS email UNIQUE;
        "#).await?;
        
        db.query(r#"
            DEFINE TABLE account SCHEMAFULL;
            DEFINE FIELD accountId ON account TYPE string;
            DEFINE FIELD email ON account TYPE string;
            DEFINE FIELD name ON account TYPE option<string>;
            DEFINE FIELD imapServer ON account TYPE option<string>;
            DEFINE FIELD imapPort ON account TYPE option<int>;
            DEFINE FIELD smtpServer ON account TYPE option<string>;
            DEFINE FIELD smtpPort ON account TYPE option<int>;
            DEFINE FIELD username ON account TYPE option<string>;
            DEFINE FIELD password ON account TYPE option<string>;
            DEFINE FIELD useSsl ON account TYPE bool DEFAULT true;
            DEFINE FIELD allowInvalidCerts ON account TYPE bool DEFAULT false;
            DEFINE FIELD allowInvalidSmtpCerts ON account TYPE bool DEFAULT false;
            DEFINE FIELD userId ON account TYPE option<string>;
            DEFINE INDEX account_email ON account FIELDS email UNIQUE;
            DEFINE INDEX account_id ON account FIELDS accountId UNIQUE;
        "#).await?;
        
        Ok(Self { db })
    }
    
    pub async fn create_email(&self, email: &Email) -> Result<Email> {
        let mut content = serde_json::to_value(email)?;
        if let Some(obj) = content.as_object_mut() {
            if let Some(id) = obj.remove("id") {
                obj.insert("emailId".to_string(), id);
            }
        }

        let _: Vec<serde_json::Value> = self.db
            .create("email")
            .content(content)
            .await?;

        self.get_email(&email.id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound("Failed to create email".to_string()))
    }
    
    pub async fn get_email(&self, id: &str) -> Result<Option<Email>> {
        let mut result = self.db
            .query(&format!("SELECT {} FROM email WHERE emailId = $id LIMIT 1", EMAIL_SELECT_FIELDS))
            .bind(("id", id))
            .await?;

        let emails: Vec<Email> = result.take(0)?;
        Ok(emails.into_iter().next())
    }
    
    pub async fn get_emails_by_folder(&self, folder: &str) -> Result<Vec<Email>> {
        let mut result = self.db
            .query(&format!("SELECT {} FROM email WHERE folder = $folder ORDER BY date DESC", EMAIL_SELECT_FIELDS))
            .bind(("folder", folder))
            .await?;
        
        let emails: Vec<Email> = result.take(0)?;
        Ok(emails)
    }
    
    pub async fn get_all_emails(&self) -> Result<Vec<Email>> {
        let mut result = self.db
            .query(&format!("SELECT {} FROM email", EMAIL_SELECT_FIELDS))
            .await?;

        let emails: Vec<Email> = result.take(0)?;
        Ok(emails)
    }
    
    pub async fn update_email(&self, email: &Email) -> Result<Email> {
        let mut content = serde_json::to_value(email)?;
        if let Some(obj) = content.as_object_mut() {
            if let Some(id) = obj.remove("id") {
                obj.insert("emailId".to_string(), id);
            }
        }

        let mut result = self.db
            .query("UPDATE email CONTENT $data WHERE emailId = $id")
            .bind(("id", &email.id))
            .bind(("data", content))
            .await?;

        let emails: Vec<Email> = result.take(0)?;
        emails
            .into_iter()
            .next()
            .ok_or_else(|| DatabaseError::NotFound(format!("Email {} not found", email.id)))
    }
    
    pub async fn delete_email(&self, id: &str) -> Result<()> {
        self.db
            .query("DELETE email WHERE emailId = $id")
            .bind(("id", id))
            .await?;

        Ok(())
    }
    
    pub async fn mark_as_read(&self, id: &str, read: bool) -> Result<()> {
        self.db
            .query("UPDATE email SET read = $read WHERE emailId = $id")
            .bind(("id", id))
            .bind(("read", read))
            .await?;
        
        Ok(())
    }
    
    pub async fn toggle_star(&self, id: &str, starred: bool) -> Result<()> {
        self.db
            .query("UPDATE email SET starred = $starred WHERE emailId = $id")
            .bind(("id", id))
            .bind(("starred", starred))
            .await?;
        
        Ok(())
    }
    
    pub async fn move_to_folder(&self, id: &str, folder: &str) -> Result<()> {
        self.db
            .query("UPDATE email SET folder = $folder WHERE emailId = $id")
            .bind(("id", id))
            .bind(("folder", folder))
            .await?;
        
        Ok(())
    }
    
    pub async fn search_emails(&self, query: &str) -> Result<Vec<Email>> {
        let mut result = self.db
            .query(&format!("SELECT {} FROM email WHERE string::lowercase(subject) CONTAINS $query OR string::lowercase(body) CONTAINS $query OR string::lowercase(from.email) CONTAINS $query OR string::lowercase(from.name) CONTAINS $query ORDER BY date DESC", EMAIL_SELECT_FIELDS))
            .bind(("query", query.to_lowercase()))
            .await?;
        
        let emails: Vec<Email> = result.take(0)?;
        Ok(emails)
    }
    
    pub async fn create_account(&self, account: &EmailAccount) -> Result<EmailAccount> {
        let mut content = serde_json::to_value(account)?;
        if let Some(obj) = content.as_object_mut() {
            if let Some(id) = obj.remove("id") {
                obj.insert("accountId".to_string(), id);
            }
        }

        let _: Vec<serde_json::Value> = self.db
            .create("account")
            .content(content)
            .await?;

        self.get_account(&account.id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound("Failed to create account".to_string()))
    }
    
    pub async fn get_account(&self, id: &str) -> Result<Option<EmailAccount>> {
        let mut result = self.db
            .query("SELECT accountId AS id, email, name, imapServer, imapPort, smtpServer, smtpPort, username, password, useSsl, allowInvalidCerts, allowInvalidSmtpCerts, userId FROM account WHERE accountId = $id LIMIT 1")
            .bind(("id", id))
            .await?;

        let accounts: Vec<EmailAccount> = result.take(0)?;
        Ok(accounts.into_iter().next())
    }

    pub async fn get_account_by_email(&self, email: &str) -> Result<Option<EmailAccount>> {
        let mut result = self.db
            .query("SELECT accountId AS id, email, name, imapServer, imapPort, smtpServer, smtpPort, username, password, useSsl, allowInvalidCerts, allowInvalidSmtpCerts, userId FROM account WHERE email = $email LIMIT 1")
            .bind(("email", email))
            .await?;

        let accounts: Vec<EmailAccount> = result.take(0)?;
        Ok(accounts.into_iter().next())
    }
    
    pub async fn get_all_accounts(&self) -> Result<Vec<EmailAccount>> {
        let mut result = self.db
            .query("SELECT accountId AS id, email, name, imapServer, imapPort, smtpServer, smtpPort, username, password, useSsl, allowInvalidCerts, allowInvalidSmtpCerts, userId FROM account")
            .await?;

        let accounts: Vec<EmailAccount> = result.take(0)?;
        Ok(accounts)
    }
    
    pub async fn update_account(&self, account: &EmailAccount) -> Result<EmailAccount> {
        let mut content = serde_json::to_value(account)?;
        if let Some(obj) = content.as_object_mut() {
            if let Some(id) = obj.remove("id") {
                obj.insert("accountId".to_string(), id);
            }
        }

        let _: Vec<serde_json::Value> = self.db
            .query("UPDATE account CONTENT $data WHERE accountId = $id")
            .bind(("id", &account.id))
            .bind(("data", content))
            .await?
            .take(0)?;

        self.get_account(&account.id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("Account {} not found", account.id)))
    }
    
    pub async fn delete_account(&self, id: &str) -> Result<()> {
        self.db
            .query("DELETE account WHERE accountId = $id")
            .bind(("id", id))
            .await?;

        self.db
            .query("DELETE FROM email WHERE account_id = $id")
            .bind(("id", id))
            .await?;

        Ok(())
    }
    
    pub async fn get_unread_count(&self, folder: &str) -> Result<i64> {
        let mut result = self.db
            .query("SELECT count() FROM email WHERE folder = $folder AND read = false GROUP ALL")
            .bind(("folder", folder))
            .await?;
        
        #[derive(serde::Deserialize)]
        struct CountResult {
            count: i64,
        }
        
        let count: Option<CountResult> = result.take(0)?;
        Ok(count.map(|c| c.count).unwrap_or(0))
    }
    
    pub async fn get_folder_count(&self, folder: &str) -> Result<i64> {
        let mut result = self.db
            .query("SELECT count() FROM email WHERE folder = $folder GROUP ALL")
            .bind(("folder", folder))
            .await?;
        
        #[derive(serde::Deserialize)]
        struct CountResult {
            count: i64,
        }
        
        let count: Option<CountResult> = result.take(0)?;
        Ok(count.map(|c| c.count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EmailAddress;
    
    #[tokio::test]
    async fn test_database_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).await.unwrap();
        
        let email = Email::new(
            EmailAddress { name: "Test".to_string(), email: "test@example.com".to_string() },
            vec![EmailAddress { name: "Recipient".to_string(), email: "recipient@example.com".to_string() }],
            "Test Subject".to_string(),
            "Test Body".to_string(),
            "inbox".to_string(),
        );
        
        let created = db.create_email(&email).await.unwrap();
        assert_eq!(created.subject, "Test Subject");
        
        let fetched = db.get_email(&email.id).await.unwrap();
        assert!(fetched.is_some());
        
        let mut updated_email = email.clone();
        updated_email.read = true;
        let updated = db.update_email(&updated_email).await.unwrap();
        assert!(updated.read);
        
        db.delete_email(&email.id).await.unwrap();
        let deleted = db.get_email(&email.id).await.unwrap();
        assert!(deleted.is_none());
    }
}
