//! SKILL.md Loader — inject domain expertise into Hand execution context.
//!
//! Reads SKILL.md files and injects them into the agent's system prompt
//! before each Hand phase execution. Lightweight: reads once, caches in-memory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A loaded skill — domain expertise for a Hand.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill name (derived from filename or YAML frontmatter).
    pub name: String,
    /// Full markdown content.
    pub content: String,
    /// Byte size of the content.
    pub size_bytes: usize,
}

/// Skill Registry — caches loaded SKILL.md files.
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    /// Base directory for skill files.
    base_dir: PathBuf,
}

impl SkillRegistry {
    /// Create a new skill registry rooted at `base_dir`.
    pub fn new(base_dir: &Path) -> Self {
        let mut reg = Self {
            skills: HashMap::new(),
            base_dir: base_dir.to_path_buf(),
        };
        reg.scan();
        reg
    }

    /// Default path: ~/.bizclaw/skills/
    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".bizclaw").join("skills")
    }

    /// Scan base_dir for SKILL.md files and load them.
    pub fn scan(&mut self) {
        if !self.base_dir.exists() {
            std::fs::create_dir_all(&self.base_dir).ok();
            return;
        }

        // Look for both flat SKILL.md and subdirectory/SKILL.md patterns
        if let Ok(entries) = std::fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Direct .md files: skills/research.md
                if path.is_file() && path.extension().is_some_and(|e| e == "md") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        self.load_skill(name, &path);
                    }
                }

                // Subdirectory pattern: skills/research/SKILL.md
                if path.is_dir() {
                    let skill_file = path.join("SKILL.md");
                    if skill_file.exists() {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            self.load_skill(name, &skill_file);
                        }
                    }
                }
            }
        }

        if !self.skills.is_empty() {
            tracing::info!("📚 Loaded {} skill(s)", self.skills.len());
        }
    }

    /// Load a single skill file.
    fn load_skill(&mut self, name: &str, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let size = content.len();
                // Strip YAML frontmatter if present
                let content = strip_frontmatter(&content);
                self.skills.insert(
                    name.to_lowercase(),
                    Skill {
                        name: name.to_string(),
                        content,
                        size_bytes: size,
                    },
                );
                tracing::debug!("  📖 Skill '{}' loaded ({} bytes)", name, size);
            }
            Err(e) => {
                tracing::warn!("Failed to load skill '{}': {}", name, e);
            }
        }
    }

    /// Get a skill by name.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(&name.to_lowercase())
    }

    /// Build context injection string for a Hand.
    /// Combines the Hand's own skill + any referenced skills.
    pub fn build_context(&self, hand_name: &str, max_chars: usize) -> Option<String> {
        let skill = self.get(hand_name)?;
        let mut context = format!(
            "[Domain Expertise: {}]\n{}\n[End expertise]",
            skill.name, skill.content
        );
        // Truncate to fit edge device context windows
        if context.len() > max_chars {
            context.truncate(max_chars);
            context.push_str("\n...[truncated]");
        }
        Some(context)
    }

    /// List all loaded skills.
    pub fn list(&self) -> Vec<&Skill> {
        let mut skills: Vec<_> = self.skills.values().collect();
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        skills
    }

    /// Total count.
    pub fn count(&self) -> usize {
        self.skills.len()
    }

    /// Total size in bytes.
    pub fn total_size(&self) -> usize {
        self.skills.values().map(|s| s.size_bytes).sum()
    }
}

/// Strip YAML frontmatter (--- ... ---) from markdown content.
fn strip_frontmatter(content: &str) -> String {
    if let Some(after_prefix) = content.strip_prefix("---") {
        if let Some(end) = after_prefix.find("---") {
            return after_prefix[end + 3..].trim_start().to_string();
        }
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let md = "---\nname: test\ndescription: demo\n---\n\n# Hello World\nContent here.";
        let result = strip_frontmatter(md);
        assert!(result.starts_with("# Hello World"));
    }

    #[test]
    fn test_no_frontmatter() {
        let md = "# Just Markdown\nNo frontmatter.";
        let result = strip_frontmatter(md);
        assert_eq!(result, md);
    }

    #[test]
    fn test_skill_registry_empty() {
        let tmp = std::env::temp_dir().join("bizclaw_test_skills_empty");
        let reg = SkillRegistry::new(&tmp);
        assert_eq!(reg.count(), 0);
        std::fs::remove_dir_all(&tmp).ok();
    }
}
