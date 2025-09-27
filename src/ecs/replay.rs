//! Replay logging and analysis functionality
//!
//! This module provides comprehensive replay functionality for the ECS framework,
//! enabling debugging through replay and analysis of game sessions.

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ecs::core::WorldOperation;
use crate::ecs::diff::DiffComponentChange;
use crate::ecs::system::{SystemUpdateDiff, WorldUpdateDiff, WorldUpdateHistory};

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
}

impl Default for ReplayLogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_directory: "replay_logs".to_string(),
            file_prefix: "game_replay".to_string(),
            flush_interval: 100,
            include_component_details: true,
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
        }
    }

    /// Generate a unique session ID based on timestamp
    fn generate_session_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}", timestamp)
    }

    /// Initialize logging - create directory and log file
    pub fn initialize(&mut self) -> Result<(), std::io::Error> {
        if !self.config.enabled {
            return Ok(());
        }

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&self.config.log_directory)?;

        // Create log file
        let filename = format!("{}_{}.log", self.config.file_prefix, self.session_id);
        let filepath = Path::new(&self.config.log_directory).join(filename);

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)?;

        let mut writer = BufWriter::new(file);

        // Write header
        writeln!(writer, "# ECS Replay Log")?;
        writeln!(writer, "# Session ID: {}", self.session_id)?;
        writeln!(
            writer,
            "# Timestamp: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;
        writeln!(writer, "# Configuration: {:?}", self.config)?;
        writeln!(writer, "# Format: Each line represents one world update")?;
        writeln!(writer)?;

        self.log_file = Some(writer);

        println!(
            "Replay logging initialized - Session ID: {}",
            self.session_id
        );
        Ok(())
    }

    /// Log a world update diff
    pub fn log_update(&mut self, update: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        if !self.config.enabled || self.log_file.is_none() {
            return Ok(());
        }

        let writer = self.log_file.as_mut().unwrap();
        self.update_count += 1;

        // Write update header
        writeln!(writer, "UPDATE {}", self.update_count)?;
        writeln!(writer, "SYSTEMS: {}", update.system_diffs().len())?;

        // Log each system update
        for (system_idx, system_diff) in update.system_diffs().iter().enumerate() {
            writeln!(writer, "  SYSTEM {}", system_idx)?;

            // Log component changes
            if self.config.include_component_details && !system_diff.diff_changes().is_empty() {
                writeln!(
                    writer,
                    "    COMPONENT_CHANGES: {}",
                    system_diff.diff_changes().len()
                )?;
                for change in system_diff.diff_changes() {
                    match change {
                        DiffComponentChange::Added {
                            entity,
                            type_name,
                            data,
                        } => {
                            writeln!(writer, "      ADD {:?} {} {}", entity, type_name, data)?;
                        }
                        DiffComponentChange::Modified {
                            entity,
                            type_name,
                            diff,
                        } => {
                            writeln!(writer, "      MOD {:?} {} {}", entity, type_name, diff)?;
                        }
                        DiffComponentChange::Removed { entity, type_name } => {
                            writeln!(writer, "      REM {:?} {}", entity, type_name)?;
                        }
                    }
                }
            }

            // Log world operations
            if !system_diff.world_operations().is_empty() {
                writeln!(
                    writer,
                    "    WORLD_OPERATIONS: {}",
                    system_diff.world_operations().len()
                )?;
                for operation in system_diff.world_operations() {
                    match operation {
                        WorldOperation::CreateEntity(entity) => {
                            writeln!(writer, "      CREATE_ENTITY {:?}", entity)?;
                        }
                        WorldOperation::RemoveEntity(entity) => {
                            writeln!(writer, "      REMOVE_ENTITY {:?}", entity)?;
                        }
                        WorldOperation::CreateWorld(world_id) => {
                            writeln!(writer, "      CREATE_WORLD {}", world_id)?;
                        }
                        WorldOperation::RemoveWorld(world_id) => {
                            writeln!(writer, "      REMOVE_WORLD {}", world_id)?;
                        }
                        WorldOperation::AddSystem(system_type) => {
                            writeln!(writer, "      ADD_SYSTEM {}", system_type)?;
                        }
                    }
                }
            }
        }

        writeln!(writer)?; // Empty line between updates

        // Flush periodically
        if self.update_count % self.config.flush_interval == 0 {
            writer.flush()?;
        }

        Ok(())
    }

    /// Finalize logging - flush and close file
    pub fn finalize(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut writer) = self.log_file.take() {
            writeln!(
                writer,
                "# End of replay log - Total updates: {}",
                self.update_count
            )?;
            writer.flush()?;
            println!(
                "Replay logging finalized - {} updates logged",
                self.update_count
            );
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
}

/// Statistics about a replay session
#[derive(Debug)]
pub struct ReplayStats {
    pub total_updates: usize,
    pub total_system_executions: usize,
    pub total_component_changes: usize,
    pub total_world_operations: usize,
    pub entities_created: usize,
    pub entities_removed: usize,
    pub component_types_involved: Vec<String>,
    pub most_active_frame: Option<usize>,
    pub most_changes_in_frame: usize,
}

/// Analyze a world update history and generate statistics
pub fn analyze_replay_history(history: &WorldUpdateHistory) -> ReplayStats {
    let mut stats = ReplayStats {
        total_updates: history.len(),
        total_system_executions: 0,
        total_component_changes: 0,
        total_world_operations: 0,
        entities_created: 0,
        entities_removed: 0,
        component_types_involved: Vec::new(),
        most_active_frame: None,
        most_changes_in_frame: 0,
    };

    let mut component_types = HashSet::new();
    let mut frame_changes: Vec<usize> = Vec::new();

    for update in history.updates() {
        stats.total_system_executions += update.system_diffs().len();

        let mut frame_change_count = 0;

        for system_diff in update.system_diffs() {
            stats.total_component_changes += system_diff.diff_changes().len();
            stats.total_world_operations += system_diff.world_operations().len();
            frame_change_count +=
                system_diff.diff_changes().len() + system_diff.world_operations().len();

            // Collect component types
            for change in system_diff.diff_changes() {
                match change {
                    DiffComponentChange::Added { type_name, .. }
                    | DiffComponentChange::Modified { type_name, .. }
                    | DiffComponentChange::Removed { type_name, .. } => {
                        component_types.insert(type_name.clone());
                    }
                }
            }

            // Count entities created/removed
            for operation in system_diff.world_operations() {
                match operation {
                    WorldOperation::CreateEntity(_) => stats.entities_created += 1,
                    WorldOperation::RemoveEntity(_) => stats.entities_removed += 1,
                    _ => {}
                }
            }
        }

        frame_changes.push(frame_change_count);
    }

    // Find most active frame
    if let Some((frame_idx, max_changes)) = frame_changes
        .iter()
        .enumerate()
        .max_by_key(|(_, &changes)| changes)
    {
        stats.most_active_frame = Some(frame_idx);
        stats.most_changes_in_frame = *max_changes;
    }

    stats.component_types_involved = component_types.into_iter().collect();
    stats.component_types_involved.sort();

    stats
}

/// Print a detailed analysis report of a replay session
pub fn print_replay_analysis(history: &WorldUpdateHistory) {
    let stats = analyze_replay_history(history);

    println!("=== ECS Replay Analysis Report ===");
    println!("Total Updates: {}", stats.total_updates);
    println!("Total System Executions: {}", stats.total_system_executions);
    println!("Total Component Changes: {}", stats.total_component_changes);
    println!("Total World Operations: {}", stats.total_world_operations);
    println!("Entities Created: {}", stats.entities_created);
    println!("Entities Removed: {}", stats.entities_removed);

    if let Some(frame) = stats.most_active_frame {
        println!(
            "Most Active Frame: {} (with {} changes)",
            frame, stats.most_changes_in_frame
        );
    }

    println!("Component Types Involved:");
    for component_type in &stats.component_types_involved {
        println!("  - {}", component_type);
    }

    if stats.total_updates > 0 {
        println!(
            "Average Changes per Frame: {:.2}",
            stats.total_component_changes as f64 / stats.total_updates as f64
        );
    }

    println!("=== End Report ===");
}

/// Find frames with unusual activity (significantly above average)
pub fn find_anomalous_frames(
    history: &WorldUpdateHistory,
    threshold_multiplier: f64,
) -> Vec<usize> {
    let updates = history.updates();
    if updates.is_empty() {
        return Vec::new();
    }

    // Calculate average changes per frame
    let total_changes: usize = updates
        .iter()
        .map(|update| {
            update
                .system_diffs()
                .iter()
                .map(|sys| sys.diff_changes().len() + sys.world_operations().len())
                .sum::<usize>()
        })
        .sum();

    let avg_changes = total_changes as f64 / updates.len() as f64;
    let threshold = avg_changes * threshold_multiplier;

    let mut anomalous_frames = Vec::new();

    for (frame_idx, update) in updates.iter().enumerate() {
        let frame_changes: usize = update
            .system_diffs()
            .iter()
            .map(|sys| sys.diff_changes().len() + sys.world_operations().len())
            .sum();

        if frame_changes as f64 > threshold {
            anomalous_frames.push(frame_idx);
        }
    }

    anomalous_frames
}

/// Read and parse a replay log file
pub fn read_replay_log(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    std::fs::read_to_string(file_path)
        .map(|content| content.lines().map(|line| line.to_string()).collect())
}

/// Parse a replay log file into WorldUpdateHistory
pub fn parse_replay_log(file_path: &str) -> Result<WorldUpdateHistory, Box<dyn std::error::Error>> {
    let lines = read_replay_log(file_path)?;
    let mut history = WorldUpdateHistory::new();
    let mut current_update: Option<WorldUpdateDiff> = None;
    let mut current_system: Option<SystemUpdateDiff> = None;
    for line in lines.into_iter() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if line.starts_with("UPDATE ") {
            // Save previous update if exists
            if let Some(update) = current_update.take() {
                history.record(update);
            }
            current_update = Some(WorldUpdateDiff::new());
        } else if line.starts_with("SYSTEMS: ") {
            // Just metadata, continue
        } else if line.starts_with("  SYSTEM ") {
            // Save previous system if exists
            if let Some(system) = current_system.take() {
                if let Some(ref mut update) = current_update {
                    update.record(system);
                }
            }
            current_system = Some(SystemUpdateDiff::new());
        } else if line.starts_with("    COMPONENT_CHANGES: ") {
            // Component changes section header
        } else if let Some(stripped) = line.strip_prefix("      ADD ") {
            // Parse component addition: "ADD Entity(world_id, entity_id) ComponentType data"
            if let Some(change) = parse_component_add(stripped) {
                if let Some(ref mut system) = current_system {
                    system.record_component_change(change);
                }
            }
        } else if let Some(stripped) = line.strip_prefix("      MOD ") {
            // Parse component modification: "MOD Entity(world_id, entity_id) ComponentType diff"
            if let Some(change) = parse_component_mod(stripped) {
                if let Some(ref mut system) = current_system {
                    system.record_component_change(change);
                }
            }
        } else if let Some(stripped) = line.strip_prefix("      REM ") {
            // Parse component removal: "REM Entity(world_id, entity_id) ComponentType"
            if let Some(change) = parse_component_rem(stripped) {
                if let Some(ref mut system) = current_system {
                    system.record_component_change(change);
                }
            }
        } else if line.starts_with("    WORLD_OPERATIONS: ") {
            // World operations section header
        } else if let Some(stripped) = line.strip_prefix("      CREATE_ENTITY ") {
            // Parse entity creation: "CREATE_ENTITY Entity(world_id, entity_id)"
            if let Some(entity) = parse_entity(stripped) {
                if let Some(ref mut system) = current_system {
                    system.record_world_operation(WorldOperation::CreateEntity(entity));
                }
            }
        } else if let Some(stripped) = line.strip_prefix("      REMOVE_ENTITY ") {
            // Parse entity removal: "REMOVE_ENTITY Entity(world_id, entity_id)"
            if let Some(entity) = parse_entity(stripped) {
                if let Some(ref mut system) = current_system {
                    system.record_world_operation(WorldOperation::RemoveEntity(entity));
                }
            }
        } else if let Some(stripped) = line.strip_prefix("      CREATE_WORLD ") {
            // Parse world creation: "CREATE_WORLD world_id"
            if let Ok(world_id) = stripped.parse::<usize>() {
                if let Some(ref mut system) = current_system {
                    system.record_world_operation(WorldOperation::CreateWorld(world_id));
                }
            }
        } else if let Some(stripped) = line.strip_prefix("      REMOVE_WORLD ") {
            // Parse world removal: "REMOVE_WORLD world_id"
            if let Ok(world_id) = stripped.parse::<usize>() {
                if let Some(ref mut system) = current_system {
                    system.record_world_operation(WorldOperation::RemoveWorld(world_id));
                }
            }
        } else if let Some(stripped) = line.strip_prefix("      ADD_SYSTEM ") {
            // Parse system addition: "ADD_SYSTEM system_type_name"
            let system_type_name = stripped.to_string();
            if let Some(ref mut system) = current_system {
                system.record_world_operation(WorldOperation::AddSystem(system_type_name));
            }
        }
    }

    // Save any remaining data
    if let Some(system) = current_system {
        if let Some(ref mut update) = current_update {
            update.record(system);
        }
    }
    if let Some(update) = current_update {
        history.record(update);
    }

    Ok(history)
}

/// Parse entity from string like "Entity(0, 123)"
fn parse_entity(input: &str) -> Option<crate::ecs::core::Entity> {
    if input.starts_with("Entity(") && input.ends_with(')') {
        let content = &input[7..input.len() - 1];
        let parts: Vec<&str> = content.split(", ").collect();
        if parts.len() == 2 {
            if let (Ok(world_index), Ok(entity_index)) =
                (parts[0].parse::<usize>(), parts[1].parse::<usize>())
            {
                return Some(crate::ecs::core::Entity::new(world_index, entity_index));
            }
        }
    }
    None
}

/// Parse component addition from string like "Entity(0, 123) Position Position { x: 1.0, y: 2.0 }"
fn parse_component_add(input: &str) -> Option<DiffComponentChange> {
    let parts: Vec<&str> = input.splitn(3, ' ').collect();
    if parts.len() >= 3 {
        if let Some(entity) = parse_entity(parts[0]) {
            let type_name = parts[1].to_string();
            let data = if parts.len() > 2 {
                parts[2].to_string()
            } else {
                String::new()
            };
            return Some(DiffComponentChange::Added {
                entity,
                type_name,
                data,
            });
        }
    }
    None
}

/// Parse component modification from string like "Entity(0, 123) Position PositionDiff { x: Some(1.0), y: None }"
fn parse_component_mod(input: &str) -> Option<DiffComponentChange> {
    let parts: Vec<&str> = input.splitn(3, ' ').collect();
    if parts.len() >= 3 {
        if let Some(entity) = parse_entity(parts[0]) {
            let type_name = parts[1].to_string();
            let diff = if parts.len() > 2 {
                parts[2].to_string()
            } else {
                String::new()
            };
            return Some(DiffComponentChange::Modified {
                entity,
                type_name,
                diff,
            });
        }
    }
    None
}

/// Parse component removal from string like "Entity(0, 123) Position"
fn parse_component_rem(input: &str) -> Option<DiffComponentChange> {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    if parts.len() >= 2 {
        if let Some(entity) = parse_entity(parts[0]) {
            let type_name = parts[1].to_string();
            return Some(DiffComponentChange::Removed { entity, type_name });
        }
    }
    None
}
