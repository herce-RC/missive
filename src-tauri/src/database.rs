use surrealdb::engine::local::{Db, SurrealKV};
use surrealdb::Surreal;
use crate::models::{Email, EmailAccount};
use thiserror::Error;
use std::path::Path;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SurrealError(#[from] surrealdb::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub struct Database {
    db: Surreal<Db>,
}

impl Database {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = Surreal::new::<SurrealKV>(path.as_ref()).await?;
        
        // Select namespace and database
        db.use_ns("email_client").use_db("main").await?;
        
        // Create tables schema
        db.query(r#"
            DEFINE TABLE email SCHEMAFULL;
            DEFINE FIELD id ON email TYPE string;
            DEFINE FIELD from ON email TYPE object;
            DEFINE FIELD from.name ON email TYPE string;
            DEFINE FIELD from.email ON email TYPE string;
            DEFINE FIELD to ON email TYPE array;
            DEFINE FIELD cc ON email TYPE option<array>;
            DEFINE FIELD bcc ON email TYPE option<array>;
            DEFINE FIELD subject ON email TYPE string;
            DEFINE FIELD body ON email TYPE string;
            DEFINE FIELD html_body ON email TYPE option<string>;
            DEFINE FIELD date ON email TYPE string;
            DEFINE FIELD read ON email TYPE bool DEFAULT false;
            DEFINE FIELD starred ON email TYPE bool DEFAULT false;
            DEFINE FIELD folder ON email TYPE string DEFAULT 'inbox';
            DEFINE FIELD attachments ON email TYPE option<array>;
            DEFINE FIELD account_id ON email TYPE option<string>;
            DEFINE FIELD message_id ON email TYPE option<string>;
            DEFINE INDEX email_id ON email FIELDS id UNIQUE;
            DEFINE INDEX email_folder ON email FIELDS folder;
            DEFINE INDEX email_date ON email FIELDS date;
        "#).await?;
        
        db.query(r#"
            DEFINE TABLE account SCHEMAFULL;
            DEFINE FIELD id ON account TYPE string;
            DEFINE FIELD email ON account TYPE string;
            DEFINE FIELD name ON account TYPE string;
            DEFINE FIELD imap_server ON account TYPE string;
            DEFINE FIELD imap_port ON account TYPE int;
            DEFINE FIELD smtp_server ON account TYPE string;
            DEFINE FIELD smtp_port ON account TYPE int;
            DEFINE FIELD username ON account TYPE string;
            DEFINE FIELD password ON account TYPE string;
            DEFINE FIELD use_ssl ON account TYPE bool DEFAULT true;
            DEFINE FIELD allow_invalid_certs ON account TYPE bool DEFAULT false;
            DEFINE FIELD allow_invalid_smtp_certs ON account TYPE bool DEFAULT false;
            DEFINE INDEX account_id ON account FIELDS id UNIQUE;
            DEFINE INDEX account_email ON account FIELDS email UNIQUE;
        "#).await?;
        
        Ok(Self { db })
    }
    
    // Email operations
    pub async fn create_email(&self, email: &Email) -> Result<Email> {
        let result: Option<Email> = self.db
            .create(("email", &email.id))
            .content(email)
            .await?;
        
        result.ok_or_else(|| DatabaseError::NotFound("Failed to create email".to_string()))
    }
    
    pub async fn get_email(&self, id: &str) -> Result<Option<Email>> {
        let result: Option<Email> = self.db
            .select(("email", id))
            .await?;
        
        Ok(result)
    }
    
    pub async fn get_emails_by_folder(&self, folder: &str) -> Result<Vec<Email>> {
        let mut result = self.db
            .query("SELECT * FROM email WHERE folder = $folder ORDER BY date DESC")
            .bind(("folder", folder))
            .await?;
        
        let emails: Vec<Email> = result.take(0)?;
        Ok(emails)
    }
    
    pub async fn get_all_emails(&self) -> Result<Vec<Email>> {
        let result: Vec<Email> = self.db
            .select("email")
            .await?;
        
        Ok(result)
    }
    
    pub async fn update_email(&self, email: &Email) -> Result<Email> {
        let result: Option<Email> = self.db
            .update(("email", &email.id))
            .content(email)
            .await?;
        
        result.ok_or_else(|| DatabaseError::NotFound(format!("Email {} not found", email.id)))
    }
    
    pub async fn delete_email(&self, id: &str) -> Result<()> {
        let _: Option<Email> = self.db
            .delete(("email", id))
            .await?;
        
        Ok(())
    }
    
    pub async fn mark_as_read(&self, id: &str, read: bool) -> Result<()> {
        self.db
            .query("UPDATE email SET read = $read WHERE id = $id")
            .bind(("id", id))
            .bind(("read", read))
            .await?;
        
        Ok(())
    }
    
    pub async fn toggle_star(&self, id: &str, starred: bool) -> Result<()> {
        self.db
            .query("UPDATE email SET starred = $starred WHERE id = $id")
            .bind(("id", id))
            .bind(("starred", starred))
            .await?;
        
        Ok(())
    }
    
    pub async fn move_to_folder(&self, id: &str, folder: &str) -> Result<()> {
        self.db
            .query("UPDATE email SET folder = $folder WHERE id = $id")
            .bind(("id", id))
            .bind(("folder", folder))
            .await?;
        
        Ok(())
    }
    
    pub async fn search_emails(&self, query: &str) -> Result<Vec<Email>> {
        let mut result = self.db
            .query(r#"
                SELECT * FROM email 
                WHERE string::lowercase(subject) CONTAINS $query 
                   OR string::lowercase(body) CONTAINS $query
                   OR string::lowercase(from.email) CONTAINS $query
                   OR string::lowercase(from.name) CONTAINS $query
                ORDER BY date DESC
            "#)
            .bind(("query", query.to_lowercase()))
            .await?;
        
        let emails: Vec<Email> = result.take(0)?;
        Ok(emails)
    }
    
    // Account operations
    pub async fn create_account(&self, account: &EmailAccount) -> Result<EmailAccount> {
        let result: Option<EmailAccount> = self.db
            .create(("account", &account.id))
            .content(account)
            .await?;
        
        result.ok_or_else(|| DatabaseError::NotFound("Failed to create account".to_string()))
    }
    
    pub async fn get_account(&self, id: &str) -> Result<Option<EmailAccount>> {
        let result: Option<EmailAccount> = self.db
            .select(("account", id))
            .await?;
        
        Ok(result)
    }
    
    pub async fn get_all_accounts(&self) -> Result<Vec<EmailAccount>> {
        let result: Vec<EmailAccount> = self.db
            .select("account")
            .await?;
        
        Ok(result)
    }
    
    pub async fn update_account(&self, account: &EmailAccount) -> Result<EmailAccount> {
        let result: Option<EmailAccount> = self.db
            .update(("account", &account.id))
            .content(account)
            .await?;
        
        result.ok_or_else(|| DatabaseError::NotFound(format!("Account {} not found", account.id)))
    }
    
    pub async fn delete_account(&self, id: &str) -> Result<()> {
        let _: Option<EmailAccount> = self.db
            .delete(("account", id))
            .await?;
        
        // Also delete all emails associated with this account
        self.db
            .query("DELETE FROM email WHERE account_id = $id")
            .bind(("id", id))
            .await?;
        
        Ok(())
    }
    
    // Statistics
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
        
        // Create test email
        let email = Email::new(
            EmailAddress { name: "Test".to_string(), email: "test@example.com".to_string() },
            vec![EmailAddress { name: "Recipient".to_string(), email: "recipient@example.com".to_string() }],
            "Test Subject".to_string(),
            "Test Body".to_string(),
            "inbox".to_string(),
        );
        
        // Test create
        let created = db.create_email(&email).await.unwrap();
        assert_eq!(created.subject, "Test Subject");
        
        // Test get
        let fetched = db.get_email(&email.id).await.unwrap();
        assert!(fetched.is_some());
        
        // Test update
        let mut updated_email = email.clone();
        updated_email.read = true;
        let updated = db.update_email(&updated_email).await.unwrap();
        assert!(updated.read);
        
        // Test delete
        db.delete_email(&email.id).await.unwrap();
        let deleted = db.get_email(&email.id).await.unwrap();
        assert!(deleted.is_none());
    }
}
