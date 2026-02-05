use crate::models::{Email, EmailAccount, NewEmail, ConnectionTestResult};
use crate::email::EmailClient;
use crate::AppState;
use tauri::State;

type CommandResult<T> = Result<T, String>;

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn fetch_emails(
    state: State<'_, AppState>,
    folder: String,
) -> CommandResult<Vec<Email>> {
    let db = state.db.lock().await;
    
    // First try to get from database
    let emails = db.get_emails_by_folder(&folder).await.map_err(map_err)?;
    
    if !emails.is_empty() {
        return Ok(emails);
    }
    
    // If no emails in database, return empty (sync will populate)
    Ok(vec![])
}

#[tauri::command]
pub async fn sync_emails(
    state: State<'_, AppState>,
    account_id: String,
    folder: String,
) -> CommandResult<Vec<Email>> {
    let db = state.db.lock().await;
    
    // Get account
    let account = db.get_account(&account_id).await.map_err(map_err)?
        .ok_or_else(|| "Account not found".to_string())?;
    
    // Create email client
    let client = EmailClient::new(account);
    
    // Fetch emails from server
    let emails = client.fetch_emails(&folder, 50).await.map_err(map_err)?;
    
    // Store in database
    for email in &emails {
        // Check if email already exists
        if db.get_email(&email.id).await.map_err(map_err)?.is_none() {
            db.create_email(email).await.map_err(map_err)?;
        }
    }
    
    Ok(emails)
}

#[tauri::command]
pub async fn send_email(
    state: State<'_, AppState>,
    email: NewEmail,
) -> CommandResult<Email> {
    let db = state.db.lock().await;
    
    // Get first account (or could be specified)
    let accounts = db.get_all_accounts().await.map_err(map_err)?;
    let account = accounts.first()
        .ok_or_else(|| "No email account configured".to_string())?;
    
    // Create email client
    let client = EmailClient::new(account.clone());
    
    // Send email
    client.send_email(&email).await.map_err(map_err)?;
    
    // Create sent email record
    let sent_email = Email {
        id: uuid::Uuid::new_v4().to_string(),
        from: email.from,
        to: email.to,
        cc: email.cc,
        bcc: email.bcc,
        subject: email.subject,
        body: email.body,
        html_body: None,
        date: chrono::Utc::now().to_rfc3339(),
        read: true,
        starred: false,
        folder: "sent".to_string(),
        attachments: email.attachments,
        account_id: Some(account.id.clone()),
        message_id: None,
    };
    
    // Store in database
    db.create_email(&sent_email).await.map_err(map_err)?;
    
    Ok(sent_email)
}

#[tauri::command]
pub async fn mark_as_read(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let db = state.db.lock().await;
    db.mark_as_read(&id, true).await.map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub async fn mark_as_unread(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let db = state.db.lock().await;
    db.mark_as_read(&id, false).await.map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_star(
    state: State<'_, AppState>,
    id: String,
    starred: bool,
) -> CommandResult<()> {
    let db = state.db.lock().await;
    db.toggle_star(&id, starred).await.map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_email(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let db = state.db.lock().await;
    db.delete_email(&id).await.map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub async fn move_to_trash(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let db = state.db.lock().await;
    db.move_to_folder(&id, "trash").await.map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub async fn move_to_folder(
    state: State<'_, AppState>,
    id: String,
    folder: String,
) -> CommandResult<()> {
    let db = state.db.lock().await;
    db.move_to_folder(&id, &folder).await.map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub async fn save_account(
    state: State<'_, AppState>,
    account: EmailAccount,
) -> CommandResult<EmailAccount> {
    let db = state.db.lock().await;
    
    // Check if account exists
    if let Some(_) = db.get_account(&account.id).await.map_err(map_err)? {
        // Update existing
        db.update_account(&account).await.map_err(map_err)
    } else {
        // Create new
        db.create_account(&account).await.map_err(map_err)
    }
}

#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let db = state.db.lock().await;
    db.delete_account(&id).await.map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub async fn get_accounts(
    state: State<'_, AppState>,
) -> CommandResult<Vec<EmailAccount>> {
    let db = state.db.lock().await;
    db.get_all_accounts().await.map_err(map_err)
}

#[tauri::command]
pub async fn test_connection(
    account: EmailAccount,
) -> CommandResult<ConnectionTestResult> {
    let client = EmailClient::new(account);
    
    match client.test_connection().await {
        Ok(_) => Ok(ConnectionTestResult {
            success: true,
            message: "Connexion réussie !".to_string(),
        }),
        Err(e) => Ok(ConnectionTestResult {
            success: false,
            message: format!("Échec de la connexion: {}", e),
        }),
    }
}

#[tauri::command]
pub async fn get_db_path(
    state: State<'_, AppState>,
) -> CommandResult<String> {
    Ok(state.db_path.clone())
}
