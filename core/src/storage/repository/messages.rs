//! Messages repository - database operations for encrypted messages

use crate::{
    messaging::{Message, MessageStatus, MessageEnvelope},
    Result, PersonId, CoarseTimestamp,
    storage::Database,
};
use rusqlite::{params, Row};
use uuid::Uuid;

/// Direction of message (sent or received)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    Sent = 0,
    Received = 1,
}

/// Repository for message operations
pub struct MessageRepository<'db> {
    db: &'db Database,
}

impl<'db> MessageRepository<'db> {
    /// Create a new message repository
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Save a message to the database
    pub fn save(&self, message: &Message, direction: MessageDirection) -> Result<()> {
        let conn = self.db.conn();

        // Serialize message to binary (using bincode for compactness)
        let content_bytes = bincode::serialize(message)?;

        let contact_id = match direction {
            MessageDirection::Sent => message.recipient,
            MessageDirection::Received => message.sender,
        };

        conn.execute(
            "INSERT OR REPLACE INTO messages (
                id, contact_id, direction, content, status,
                created_at, read_at, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                message.id.as_bytes(),
                contact_id.0.as_bytes(),
                direction as i32,
                content_bytes,
                message.status as i32,
                message.created_at.as_secs(),
                None::<i64>, // read_at
                message.expires_at.map(|t| t.as_secs()),
            ],
        )?;

        Ok(())
    }

    /// Get a message by ID
    pub fn get(&self, id: Uuid) -> Result<Option<(Message, MessageDirection)>> {
        let conn = self.db.conn();

        let result = conn.query_row(
            "SELECT content, direction FROM messages WHERE id = ?1",
            params![id.as_bytes()],
            |row| self.row_to_message(row),
        );

        match result {
            Ok(msg) => Ok(Some(msg)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all messages for a contact
    pub fn get_conversation(&self, contact_id: PersonId) -> Result<Vec<(Message, MessageDirection)>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT content, direction FROM messages
             WHERE contact_id = ?1
             ORDER BY created_at DESC"
        )?;

        let messages = stmt.query_map([contact_id.0.as_bytes()], |row| self.row_to_message(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    /// Get recent messages (last N)
    pub fn get_recent(&self, limit: usize) -> Result<Vec<(Message, MessageDirection)>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT content, direction FROM messages
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let messages = stmt.query_map([limit as i64], |row| self.row_to_message(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    /// Get unread messages count
    pub fn count_unread(&self) -> Result<usize> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages
             WHERE direction = ?1 AND read_at IS NULL",
            [MessageDirection::Received as i32],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Mark a message as read
    pub fn mark_read(&self, id: Uuid) -> Result<()> {
        let conn = self.db.conn();

        let now = CoarseTimestamp::now();
        conn.execute(
            "UPDATE messages SET read_at = ?1 WHERE id = ?2",
            params![now.as_secs(), id.as_bytes()],
        )?;

        Ok(())
    }

    /// Update message status
    pub fn update_status(&self, id: Uuid, status: MessageStatus) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "UPDATE messages SET status = ?1 WHERE id = ?2",
            params![status as i32, id.as_bytes()],
        )?;

        Ok(())
    }

    /// Get messages waiting to be sent (queued)
    pub fn get_queued(&self) -> Result<Vec<Message>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT content FROM messages
             WHERE status = ?1 OR status = ?2
             ORDER BY created_at"
        )?;

        let messages = stmt.query_map(
            [MessageStatus::Queued as i32, MessageStatus::Sending as i32],
            |row| {
                let content_bytes: Vec<u8> = row.get(0)?;
                let msg: Message = bincode::deserialize(&content_bytes)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok(msg)
            }
        )?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    /// Delete a message
    pub fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "DELETE FROM messages WHERE id = ?1",
            params![id.as_bytes()],
        )?;

        Ok(())
    }

    /// Delete expired messages
    pub fn delete_expired(&self) -> Result<usize> {
        let conn = self.db.conn();

        let now = CoarseTimestamp::now();
        let deleted = conn.execute(
            "DELETE FROM messages WHERE expires_at IS NOT NULL AND expires_at < ?1",
            params![now.as_secs()],
        )?;

        Ok(deleted)
    }

    /// Get all contacts with messages (for conversation list)
    pub fn get_conversation_list(&self) -> Result<Vec<(PersonId, usize, Option<Message>)>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT contact_id,
                    COUNT(*) as message_count,
                    MAX(created_at) as last_message_time
             FROM messages
             GROUP BY contact_id
             ORDER BY last_message_time DESC"
        )?;

        let conversations: Vec<_> = stmt.query_map([], |row| {
            let contact_id_bytes: Vec<u8> = row.get(0)?;
            let contact_id = PersonId(Uuid::from_slice(&contact_id_bytes).unwrap());
            let message_count: i64 = row.get(1)?;
            Ok((contact_id, message_count as usize))
        })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        // Get last message for each conversation
        let mut results = Vec::new();
        for (contact_id, count) in conversations {
            let last_msg = self.get_conversation(contact_id)?
                .first()
                .map(|(msg, _)| msg.clone());
            results.push((contact_id, count, last_msg));
        }

        Ok(results)
    }

    /// Helper: Convert database row to Message
    fn row_to_message(&self, row: &Row) -> rusqlite::Result<(Message, MessageDirection)> {
        let content_bytes: Vec<u8> = row.get(0)?;
        let direction_int: i32 = row.get(1)?;

        let message: Message = bincode::deserialize(&content_bytes)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let direction = match direction_int {
            0 => MessageDirection::Sent,
            1 => MessageDirection::Received,
            _ => MessageDirection::Received,
        };

        Ok((message, direction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Database, DatabaseConfig};
    use crate::messaging::MessageType;
    use tempfile::TempDir;
    use zeroize::Zeroizing;

    fn test_db() -> (TempDir, Database) {
        let tmp = TempDir::new().unwrap();
        let config = DatabaseConfig::new(tmp.path().join("test.db"));
        let key = Zeroizing::new([42u8; 32]);
        let db = Database::open(config, &key).unwrap();
        (tmp, db)
    }

    fn test_message() -> Message {
        let sender = PersonId::new();
        let recipient = PersonId::new();
        Message::new_text(sender, recipient, "Test message")
    }

    #[test]
    fn test_save_and_get_message() {
        let (_tmp, db) = test_db();
        let repo = MessageRepository::new(&db);

        let message = test_message();
        let id = message.id;

        // Save
        repo.save(&message, MessageDirection::Sent).unwrap();

        // Get
        let retrieved = repo.get(id).unwrap();
        assert!(retrieved.is_some());

        let (retrieved_msg, direction) = retrieved.unwrap();
        assert_eq!(retrieved_msg.id, message.id);
        assert_eq!(direction, MessageDirection::Sent);
    }

    #[test]
    fn test_get_conversation() {
        let (_tmp, db) = test_db();
        let repo = MessageRepository::new(&db);

        let contact_id = PersonId::new();

        // Create multiple messages
        for i in 0..5 {
            let mut msg = test_message();
            msg.recipient = contact_id;
            msg.message_type = MessageType::Text {
                content: format!("Message {}", i),
            };
            repo.save(&msg, MessageDirection::Sent).unwrap();
        }

        let conversation = repo.get_conversation(contact_id).unwrap();
        assert_eq!(conversation.len(), 5);
    }

    #[test]
    fn test_mark_read() {
        let (_tmp, db) = test_db();
        let repo = MessageRepository::new(&db);

        let message = test_message();
        let id = message.id;

        repo.save(&message, MessageDirection::Received).unwrap();
        repo.mark_read(id).unwrap();

        // Verify read status
        let count = repo.count_unread().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_update_status() {
        let (_tmp, db) = test_db();
        let repo = MessageRepository::new(&db);

        let mut message = test_message();
        message.status = MessageStatus::Queued;
        let id = message.id;

        repo.save(&message, MessageDirection::Sent).unwrap();
        repo.update_status(id, MessageStatus::Sent).unwrap();

        let (retrieved, _) = repo.get(id).unwrap().unwrap();
        assert_eq!(retrieved.status, MessageStatus::Sent);
    }

    #[test]
    fn test_get_queued() {
        let (_tmp, db) = test_db();
        let repo = MessageRepository::new(&db);

        let mut msg1 = test_message();
        msg1.status = MessageStatus::Queued;
        repo.save(&msg1, MessageDirection::Sent).unwrap();

        let mut msg2 = test_message();
        msg2.status = MessageStatus::Sent;
        repo.save(&msg2, MessageDirection::Sent).unwrap();

        let queued = repo.get_queued().unwrap();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].id, msg1.id);
    }
}
