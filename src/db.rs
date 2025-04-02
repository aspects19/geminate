use rusqlite::{params, Connection, Result};

/// Initializes a database file and add necessary tables and field if unavailable
/// 
/// # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// 
/// # Returns 
/// 
/// Null
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chats(
            id INTEGER PRIMARY KEY,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages(
            id INTEGER PRIMARY KEY,
            chat_id INTEGER NOT NULL,
            role TEXT CHECK(role IN('user', 'gemini')),
            content TEXT NOT NULL,
            timestamp DATATIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(chat_id) REFERENCES chats(id) ON DELETE CASCADE
        );", 
        [],
    )?;
    Ok(())
}

/// Adds a new chat to the database
/// 
/// # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// 
/// # Returns
/// 
/// Integer of the rowId of the most recentry INSERTED row
pub fn add_chat(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.execute("INSERT INTO chats DEFAULT VALUES", [])?;
    Ok(conn.last_insert_rowid()) 
}

/// Adds an new message object to the database
/// 
/// # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// * `chat_id` - An id for which this message will be stored
/// * `role` - A role String either `user` or `gemini` 
/// * `content` - User input or Gemini response text
/// 
/// # Returns
/// 
/// Null
pub fn add_message(conn: &Connection, chat_id: i64, role: &str, content: &str) -> Result<()> {
    
    conn.execute(
        "INSERT INTO messages(chat_id, role, content) VALUES (?1, ?2, ?3)",
        params![chat_id, role, content],
    )?;
    Ok(())
}

/// Retrives chat Vector list from a database
/// 
/// # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// 
/// # Returns
/// 
/// Vector list of Chats where the last chat appear first
pub fn get_chats(conn: &Connection) -> Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare("SELECT id, created_at FROM chats ORDER BY created_at DESC")?;
    
    let chats = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))? 
        .collect::<Result<Vec<_>, _>>()?; 

    Ok(chats)
}

/// Retrives message Vector list from a database
/// 
/// # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// * `chat_id` - chat ID whose messages are to be returned
/// 
/// # Returns
/// 
/// Vector list of Messages where the first chat appear first
pub fn get_messages(conn: &Connection, chat_id: i64) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT role, content, timestamp FROM messages WHERE chat_id = ? ORDER BY timestamp",
    )?;
    
    let messages = stmt
        .query_map(params![chat_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

/// Deletes a chat and its messages whose ID has been provided from a database
/// 
/// # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// * `chat_id` - chat ID whose chat and messages is to be deleted
/// 
/// # Returns
/// 
/// Null
 
pub fn delete_chat(conn: &Connection, chat_id: i64) -> Result<()> {
    conn.execute("DELETE FROM chats WHERE id = ?", params![chat_id])?;
    Ok(())
}

/// Deletes a message whose messageID has been provided from a database
/// 
/// /// # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// * `message_id` - message ID for the message to be deleted
/// 
/// # Returns
/// 
/// Null

pub fn delete_message(conn: &Connection, message_id: i64) -> Result<()> {
    conn.execute("DELETE FROM messages WHERE id = ?", params![message_id])?;
    Ok(())
}

/// Clears the whole database making it empty
/// 
///  # Arguments
/// 
/// * `conn` - A connection to the sqlite database
/// 
/// # Returns
/// 
/// Null
 
pub fn clear_db(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM chats", [])?;
    conn.execute("DELETE FROM messages", [])?;
    Ok(())
}

