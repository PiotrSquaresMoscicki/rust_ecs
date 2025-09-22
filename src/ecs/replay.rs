//! Replay functionality for debugging and analysis
//!
//! This module provides the replay logging and playback capabilities
//! that are core to the ECS framework's debugging features.

use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use crate::ecs::system::{WorldUpdateDiff};
use crate::ecs::diff::BinaryDiffComponentChange;

/// Magic number for binary log files to identify format
const BINARY_LOG_MAGIC: u32 = 0x52454353; // "RECS" in ASCII
/// Binary log format version
const BINARY_LOG_VERSION: u32 = 1;

/// Binary log file header
#[derive(Serialize, Deserialize, Debug)]
struct BinaryLogHeader {
    magic: u32,
    version: u32,
    session_id: String,
    timestamp: u64,
}

/// Configuration for automatic replay logging
#[derive(Debug, Clone)]
pub struct ReplayLogConfig {
    /// Whether logging is enabled
    pub enabled: bool,
    /// Directory to save replay files
    pub log_directory: String,
    /// Base name for log files (timestamp will be appended)
    pub file_prefix: String,
    /// Maximum number of updates to keep in memory before flushing to disk
    pub flush_interval: usize,
    /// Whether to include detailed component changes in logs
    pub include_component_details: bool,
    /// Use minimal logging to reduce overhead
    pub minimal_mode: bool,
    /// Maximum size of in-memory buffer before forcing flush (in bytes)
    pub max_buffer_size: usize,
    /// Use binary format for diff recording (more efficient)
    pub binary_format: bool,
}

impl ReplayLogConfig {
    /// Create a configuration optimized for performance with minimal overhead
    pub fn optimized_performance() -> Self {
        Self {
            enabled: true,
            log_directory: "replay_logs".to_string(),
            file_prefix: "game_replay".to_string(),
            flush_interval: 1000,  // Flush less frequently
            include_component_details: false,  // Reduce data size
            minimal_mode: true,  // Use minimal logging
            max_buffer_size: 2 * 1024 * 1024,  // 2MB buffer for better batching
            binary_format: true,  // Use binary format for maximum efficiency
        }
    }

    /// Create a configuration for debugging with full details
    pub fn debug_full() -> Self {
        Self {
            enabled: true,
            log_directory: "debug_logs".to_string(),
            file_prefix: "debug_replay".to_string(),
            flush_interval: 1,  // Flush immediately for debugging
            include_component_details: true,  // Full details
            minimal_mode: false,  // Full logging
            max_buffer_size: 1024 * 1024,
            binary_format: false,  // Use text format for human readability
        }
    }

    /// Create a configuration optimized for binary storage with maximum compression
    pub fn binary_optimized() -> Self {
        Self {
            enabled: true,
            log_directory: "binary_logs".to_string(),
            file_prefix: "binary_replay".to_string(),
            flush_interval: 2000,  // Large batches for binary efficiency
            include_component_details: true,  // Full details in binary
            minimal_mode: false,  // Binary format handles compression
            max_buffer_size: 4 * 1024 * 1024,  // 4MB buffer for large binary batches
            binary_format: true,  // Binary format enabled
        }
    }
}

impl Default for ReplayLogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_directory: "replay_logs".to_string(),
            file_prefix: "game_replay".to_string(),
            flush_interval: 100,
            include_component_details: true,
            minimal_mode: false,
            max_buffer_size: 1024 * 1024,  // 1MB buffer
            binary_format: false,  // Default to text format for compatibility
        }
    }
}

/// Automatic replay logger that saves game history to files for analysis
#[derive(Debug)]
pub struct AutoReplayLogger {
    config: ReplayLogConfig,
    log_file: Option<BufWriter<File>>,
    session_id: String,
    update_count: usize,
    in_memory_buffer: Vec<LogEntry>,
    buffer_size_bytes: usize,
}

/// Log entry that can be either text or binary format
#[derive(Debug)]
enum LogEntry {
    Text(String),
    Binary(Vec<u8>),
}

impl AutoReplayLogger {
    /// Create a new auto replay logger with the given configuration
    pub fn new(config: ReplayLogConfig) -> Self {
        let session_id = Self::generate_session_id();
        
        Self {
            config,
            log_file: None,
            session_id,
            update_count: 0,
            in_memory_buffer: Vec::new(),
            buffer_size_bytes: 0,
        }
    }

    /// Initialize the logger by creating the log file
    pub fn initialize(&mut self) -> Result<(), std::io::Error> {
        if !self.config.enabled {
            return Ok(());
        }

        // Create the log directory if it doesn't exist
        std::fs::create_dir_all(&self.config.log_directory)?;

        // Create the log file path with appropriate extension
        let log_file_path = if self.config.binary_format {
            format!(
                "{}/{}_{}.binlog",
                self.config.log_directory,
                self.config.file_prefix,
                self.session_id
            )
        } else {
            format!(
                "{}/{}_{}.log",
                self.config.log_directory,
                self.config.file_prefix,
                self.session_id
            )
        };

        // Create or truncate the log file
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)?;

        let mut writer = BufWriter::new(log_file);

        // Write initial headers based on format
        if self.config.binary_format {
            // Write binary format header
            let header = BinaryLogHeader {
                magic: BINARY_LOG_MAGIC,
                version: BINARY_LOG_VERSION,
                session_id: self.session_id.clone(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            match bincode::serialize(&header) {
                Ok(header_data) => {
                    let header_len = header_data.len() as u32;
                    writer.write_all(&header_len.to_le_bytes())?;
                    writer.write_all(&header_data)?;
                }
                Err(e) => {
                    eprintln!("Failed to write binary header: {}", e);
                    // Fallback to text format marker
                    writeln!(writer, "# Binary format failed, using text fallback")?;
                }
            }
        } else {
            // Write text format headers
            writeln!(writer, "# Replay Log Generated by Rust ECS Framework")?;
            writeln!(writer, "# Session ID: {}", self.session_id)?;
            writeln!(writer, "# Generated: {}", Self::current_timestamp())?;
            writeln!(writer, "# Format: Component changes and world operations per update")?;
            writeln!(writer)?;
        }

        self.log_file = Some(writer);

        println!("Replay logging initialized");
        println!("Session ID: {}", self.session_id);
        println!("Log file: {}", log_file_path);

        Ok(())
    }

    /// Log a world update to the replay file
    pub fn log_update(&mut self, update_diff: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        if !self.config.enabled {
            return Ok(());
        }

        self.update_count += 1;

        if self.config.minimal_mode {
            self.log_update_minimal(update_diff)
        } else {
            self.log_update_full(update_diff)
        }
    }

    /// Log update with minimal information to reduce overhead
    fn log_update_minimal(&mut self, update_diff: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        if self.config.binary_format {
            self.log_update_minimal_binary(update_diff)
        } else {
            // Use efficient string building
            let entry = format!("U{} S{}\n", 
                self.update_count, 
                update_diff.system_diffs().len()
            );
            
            let size = entry.len();
            self.in_memory_buffer.push(LogEntry::Text(entry));
            self.buffer_size_bytes += size;
            
            // Check if we should flush
            self.check_and_flush()?;
            
            Ok(())
        }
    }

    /// Log update with minimal information in binary format
    fn log_update_minimal_binary(&mut self, update_diff: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        #[derive(Serialize)]
        struct MinimalUpdate {
            update_count: usize,
            system_count: usize,
        }

        let minimal_update = MinimalUpdate {
            update_count: self.update_count,
            system_count: update_diff.system_diffs().len(),
        };

        match bincode::serialize(&minimal_update) {
            Ok(binary_data) => {
                let size = binary_data.len();
                self.in_memory_buffer.push(LogEntry::Binary(binary_data));
                self.buffer_size_bytes += size;
                
                // Check if we should flush
                self.check_and_flush()?;
                
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to serialize minimal update: {}", e);
                // Fallback to text format
                let entry = format!("U{} S{}\n", 
                    self.update_count, 
                    update_diff.system_diffs().len()
                );
                
                let size = entry.len();
                self.in_memory_buffer.push(LogEntry::Text(entry));
                self.buffer_size_bytes += size;
                
                self.check_and_flush()?;
                Ok(())
            }
        }
    }

    /// Log update with full details (original behavior)
    fn log_update_full(&mut self, update_diff: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        if self.config.binary_format {
            self.log_update_full_binary(update_diff)
        } else {
            // Build the full log entry in memory first to reduce I/O
            let mut entry = format!("UPDATE {}\nSYSTEMS: {}\n", 
                self.update_count, 
                update_diff.system_diffs().len()
            );

            for (system_index, system_diff) in update_diff.system_diffs().iter().enumerate() {
                entry.push_str(&format!("  SYSTEM {}\n", system_index));
                
                if self.config.include_component_details {
                    entry.push_str(&format!("    COMPONENT_CHANGES: {}\n", system_diff.diff_changes().len()));
                    for change in system_diff.diff_changes() {
                        entry.push_str(&format!("      {:?}\n", change));
                    }
                    
                    entry.push_str(&format!("    WORLD_OPERATIONS: {}\n", system_diff.world_operations().len()));
                    for operation in system_diff.world_operations() {
                        entry.push_str(&format!("      {:?}\n", operation));
                    }
                } else {
                    entry.push_str(&format!("    COMPONENT_CHANGES: {}\n", system_diff.component_changes().len()));
                    entry.push_str(&format!("    WORLD_OPERATIONS: {}\n", system_diff.world_operations().len()));
                }
            }
            
            entry.push('\n'); // Empty line for readability
            
            let size = entry.len();
            self.in_memory_buffer.push(LogEntry::Text(entry));
            self.buffer_size_bytes += size;
            
            // Check if we should flush
            self.check_and_flush()?;
            
            Ok(())
        }
    }

    /// Log update with full details in binary format
    fn log_update_full_binary(&mut self, update_diff: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        #[derive(Serialize)]
        struct FullUpdate {
            update_count: usize,
            system_diffs: Vec<SerializableSystemDiff>,
        }

        #[derive(Serialize)]
        struct SerializableSystemDiff {
            diff_changes: Vec<BinaryDiffComponentChange>,
            component_changes_count: usize,
            world_operations_count: usize,
        }

        let mut system_diffs = Vec::new();
        for system_diff in update_diff.system_diffs() {
            // Convert text-based diff changes to binary format where possible
            let binary_diff_changes: Vec<BinaryDiffComponentChange> = system_diff
                .diff_changes()
                .iter()
                .map(|change| {
                    // Convert text-based changes to binary representation
                    // For now, we'll store the original text format as binary
                    match change {
                        crate::ecs::diff::DiffComponentChange::Added { entity, type_name, diff_string } => {
                            BinaryDiffComponentChange::Added {
                                entity: *entity,
                                type_name: type_name.clone(),
                                diff_data: diff_string.as_bytes().to_vec(),
                            }
                        }
                        crate::ecs::diff::DiffComponentChange::Modified { entity, type_name, diff_string } => {
                            BinaryDiffComponentChange::Modified {
                                entity: *entity,
                                type_name: type_name.clone(),
                                diff_data: diff_string.as_bytes().to_vec(),
                            }
                        }
                        crate::ecs::diff::DiffComponentChange::Removed { entity, type_name } => {
                            BinaryDiffComponentChange::Removed {
                                entity: *entity,
                                type_name: type_name.clone(),
                            }
                        }
                    }
                })
                .collect();

            system_diffs.push(SerializableSystemDiff {
                diff_changes: binary_diff_changes,
                component_changes_count: system_diff.component_changes().len(),
                world_operations_count: system_diff.world_operations().len(),
            });
        }

        let full_update = FullUpdate {
            update_count: self.update_count,
            system_diffs,
        };

        match bincode::serialize(&full_update) {
            Ok(binary_data) => {
                let size = binary_data.len();
                self.in_memory_buffer.push(LogEntry::Binary(binary_data));
                self.buffer_size_bytes += size;
                
                // Check if we should flush
                self.check_and_flush()?;
                
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to serialize full update: {}", e);
                // Fallback to text format
                self.log_update_full_text(update_diff)
            }
        }
    }

    /// Fallback text logging method
    fn log_update_full_text(&mut self, update_diff: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        let mut entry = format!("UPDATE {}\nSYSTEMS: {}\n", 
            self.update_count, 
            update_diff.system_diffs().len()
        );

        for (system_index, system_diff) in update_diff.system_diffs().iter().enumerate() {
            entry.push_str(&format!("  SYSTEM {}\n", system_index));
            
            if self.config.include_component_details {
                entry.push_str(&format!("    COMPONENT_CHANGES: {}\n", system_diff.diff_changes().len()));
                for change in system_diff.diff_changes() {
                    entry.push_str(&format!("      {:?}\n", change));
                }
                
                entry.push_str(&format!("    WORLD_OPERATIONS: {}\n", system_diff.world_operations().len()));
                for operation in system_diff.world_operations() {
                    entry.push_str(&format!("      {:?}\n", operation));
                }
            } else {
                entry.push_str(&format!("    COMPONENT_CHANGES: {}\n", system_diff.component_changes().len()));
                entry.push_str(&format!("    WORLD_OPERATIONS: {}\n", system_diff.world_operations().len()));
            }
        }
        
        entry.push('\n'); // Empty line for readability
        
        let size = entry.len();
        self.in_memory_buffer.push(LogEntry::Text(entry));
        self.buffer_size_bytes += size;
        
        // Check if we should flush
        self.check_and_flush()?;
        
        Ok(())
    }

    /// Check if buffer should be flushed and flush if needed
    fn check_and_flush(&mut self) -> Result<(), std::io::Error> {
        let should_flush = self.update_count % self.config.flush_interval == 0 ||
                          self.buffer_size_bytes >= self.config.max_buffer_size;
        
        if should_flush {
            self.flush_buffer()?;
        }
        
        Ok(())
    }

    /// Flush the in-memory buffer to disk
    fn flush_buffer(&mut self) -> Result<(), std::io::Error> {
        if self.in_memory_buffer.is_empty() {
            return Ok(());
        }

        if let Some(ref mut file) = self.log_file {
            // Write all buffered entries at once
            for entry in &self.in_memory_buffer {
                match entry {
                    LogEntry::Text(text) => {
                        file.write_all(text.as_bytes())?;
                    }
                    LogEntry::Binary(data) => {
                        // For binary format, we can optionally add a length prefix
                        // to help with parsing later
                        let length = data.len() as u32;
                        file.write_all(&length.to_le_bytes())?;
                        file.write_all(data)?;
                    }
                }
            }
            file.flush()?;
        }

        // Clear the buffer
        self.in_memory_buffer.clear();
        self.buffer_size_bytes = 0;

        Ok(())
    }

    /// Finalize the log file and close it
    pub fn finalize(&mut self) -> Result<(), std::io::Error> {
        // Flush any remaining buffer
        self.flush_buffer()?;
        
        if let Some(mut file) = self.log_file.take() {
            writeln!(file, "# End of replay log - Total updates: {}", self.update_count)?;
            file.flush()?;
        }
        Ok(())
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the current update count
    pub fn update_count(&self) -> usize {
        self.update_count
    }

    /// Generate a unique session ID based on timestamp
    fn generate_session_id() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string()
    }

    /// Get current timestamp as a formatted string
    fn current_timestamp() -> String {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                format!("Unix timestamp: {}", secs)
            }
            Err(_) => "Unknown time".to_string(),
        }
    }
}

/// Replay data analysis utilities for developers
pub mod analysis {
    use crate::ecs::core::Entity;
    use crate::ecs::system::WorldUpdateHistory;

    /// Statistics about a replay log
    #[derive(Debug)]
    pub struct ReplayStats {
        pub total_updates: usize,
        pub total_system_executions: usize,
        pub total_component_changes: usize,
        pub total_world_operations: usize,
        pub unique_entities: std::collections::HashSet<Entity>,
        pub component_types_involved: std::collections::HashSet<String>,
    }

    /// Analyze a WorldUpdateHistory and return statistics
    pub fn analyze_replay_history(history: &WorldUpdateHistory) -> ReplayStats {
        let mut stats = ReplayStats {
            total_updates: history.len(),
            total_system_executions: 0,
            total_component_changes: 0,
            total_world_operations: 0,
            unique_entities: std::collections::HashSet::new(),
            component_types_involved: std::collections::HashSet::new(),
        };

        // Analyze each update in the history
        for update_diff in history.updates() {
            for system_diff in update_diff.system_diffs() {
                stats.total_system_executions += 1;
                stats.total_component_changes += system_diff.component_changes().len();
                stats.total_world_operations += system_diff.world_operations().len();

                // Collect unique entities and component types from diff changes
                for diff_change in system_diff.diff_changes() {
                    match diff_change {
                        crate::ecs::diff::DiffComponentChange::Added { entity, type_name, .. } => {
                            stats.unique_entities.insert(*entity);
                            stats.component_types_involved.insert(type_name.clone());
                        }
                        crate::ecs::diff::DiffComponentChange::Modified { entity, type_name, .. } => {
                            stats.unique_entities.insert(*entity);
                            stats.component_types_involved.insert(type_name.clone());
                        }
                        crate::ecs::diff::DiffComponentChange::Removed { entity, type_name } => {
                            stats.unique_entities.insert(*entity);
                            stats.component_types_involved.insert(type_name.clone());
                        }
                    }
                }

                // Also collect from regular component changes
                for component_change in system_diff.component_changes() {
                    stats.unique_entities.insert(component_change.entity);
                    // Note: We can't get the type name from ComponentChange since it only has TypeId
                    // The type names are better extracted from diff_changes above
                }
            }
        }

        stats
    }

    /// Parse a replay log file and return statistics
    pub fn analyze_replay_log(file_path: &str) -> Result<ReplayStats, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        
        let mut stats = ReplayStats {
            total_updates: 0,
            total_system_executions: 0,
            total_component_changes: 0,
            total_world_operations: 0,
            unique_entities: std::collections::HashSet::new(),
            component_types_involved: std::collections::HashSet::new(),
        };

        for line in content.lines() {
            if line.starts_with("UPDATE ") {
                stats.total_updates += 1;
            } else if line.trim().starts_with("SYSTEM ") {
                stats.total_system_executions += 1;
            } else if line.trim().starts_with("COMPONENT_CHANGES: ") {
                if let Some(count_str) = line.split(": ").nth(1) {
                    if let Ok(count) = count_str.parse::<usize>() {
                        stats.total_component_changes += count;
                    }
                }
            } else if line.trim().starts_with("WORLD_OPERATIONS: ") {
                if let Some(count_str) = line.split(": ").nth(1) {
                    if let Ok(count) = count_str.parse::<usize>() {
                        stats.total_world_operations += count;
                    }
                }
            }
            // Parse entity information from component changes
            else if line.trim().starts_with("MOD Entity(") || line.trim().starts_with("ADD Entity(") || line.trim().starts_with("REM Entity(") {
                if let Some(entity) = parse_entity_from_line(line) {
                    stats.unique_entities.insert(entity);
                }
            }
        }

        Ok(stats)
    }

    /// Parse an entity from a replay log line
    fn parse_entity_from_line(line: &str) -> Option<Entity> {
        // Look for pattern like "Entity(0, 123)"
        if let Some(start) = line.find("Entity(") {
            if let Some(end) = line[start..].find(")") {
                let entity_str = &line[start + 7..start + end];
                let parts: Vec<&str> = entity_str.split(", ").collect();
                if parts.len() == 2 {
                    if let (Ok(gen), Ok(id)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        return Some(Entity::new(id, gen));
                    }
                }
            }
        }
        None
    }

    /// Parse a replay log file and return the parsed history
    pub fn parse_replay_log(file_path: &str) -> Result<WorldUpdateHistory, Box<dyn std::error::Error>> {
        let _content = std::fs::read_to_string(file_path)?;
        
        // For now, return an empty history
        // A full implementation would parse the file and reconstruct the history
        let history = WorldUpdateHistory::new();
        
        Ok(history)
    }
}