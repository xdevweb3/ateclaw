//! Agent Discovery — auto-generate AGENTS.md for multi-agent context injection.
//!
//! When ≤15 agents: generates full AGENTS.md with capabilities, tools, schedule.
//! When >15 agents: generates compact version + enables FTS search.
//! Edge-friendly: no vector DB, just string formatting.

/// Generate AGENTS.md content for injection into agent context.
///
/// This gives each agent awareness of what other agents can do,
/// enabling delegation and handoff decisions.
pub fn generate_agents_md(agents: &[AgentInfo]) -> String {
    if agents.is_empty() {
        return String::new();
    }

    let mut md = String::from("# Available Agents\n\n");

    if agents.len() <= 15 {
        // Full details for small teams
        md.push_str(&format!(
            "You are part of a team of {} agents. You can delegate tasks or handoff conversations.\n\n",
            agents.len()
        ));

        for agent in agents {
            md.push_str(&format!("## {}\n", agent.name));
            md.push_str(&format!("- **Role**: {}\n", agent.role));
            if !agent.description.is_empty() {
                md.push_str(&format!("- **Description**: {}\n", agent.description));
            }
            if !agent.model.is_empty() {
                md.push_str(&format!("- **Model**: {}\n", agent.model));
            }
            if !agent.tools.is_empty() {
                let tools_str = agent.tools.join(", ");
                md.push_str(&format!("- **Tools**: {}\n", tools_str));
            }
            md.push('\n');
        }

        md.push_str("## How to Delegate\n");
        md.push_str("Use the `delegate` tool: `{\"to\": \"agent-name\", \"task\": \"description\"}`\n\n");
        md.push_str("## How to Handoff\n");
        md.push_str("Use the `handoff` tool: `{\"to\": \"agent-name\", \"reason\": \"why\"}`\n");
    } else {
        // Compact version for large teams
        md.push_str(&format!(
            "You are part of a team of {} agents. Use `delegate_search` to find the right agent.\n\n",
            agents.len()
        ));

        md.push_str("| Agent | Role | Model |\n");
        md.push_str("|-------|------|-------|\n");
        for agent in agents {
            let model = if agent.model.is_empty() {
                "default"
            } else {
                &agent.model
            };
            md.push_str(&format!("| {} | {} | {} |\n", agent.name, agent.role, model));
        }
    }

    md
}

/// Agent info for discovery document generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub role: String,
    pub description: String,
    pub model: String,
    pub tools: Vec<String>,
}

/// Simple FTS search over agent metadata (for >15 agent teams).
pub fn search_agents<'a>(agents: &'a [AgentInfo], query: &str) -> Vec<&'a AgentInfo> {
    let query_lower = query.to_lowercase();
    let keywords: Vec<&str> = query_lower.split_whitespace().collect();

    let mut scored: Vec<(&AgentInfo, u32)> = agents
        .iter()
        .filter_map(|a| {
            let searchable = format!(
                "{} {} {} {}",
                a.name,
                a.role,
                a.description,
                a.tools.join(" ")
            )
            .to_lowercase();

            let score: u32 = keywords
                .iter()
                .filter(|kw| searchable.contains(**kw))
                .count() as u32;

            if score > 0 {
                Some((a, score))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(a, _)| a).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_agents() -> Vec<AgentInfo> {
        vec![
            AgentInfo {
                name: "ceo".into(),
                role: "Chief Executive".into(),
                description: "Strategic decisions".into(),
                model: "claude-4-sonnet".into(),
                tools: vec!["delegate".into()],
            },
            AgentInfo {
                name: "dev".into(),
                role: "Developer".into(),
                description: "Code implementation".into(),
                model: "deepseek-chat".into(),
                tools: vec!["shell".into(), "file".into()],
            },
        ]
    }

    #[test]
    fn test_generate_agents_md() {
        let agents = sample_agents();
        let md = generate_agents_md(&agents);
        assert!(md.contains("# Available Agents"));
        assert!(md.contains("## ceo"));
        assert!(md.contains("## dev"));
        assert!(md.contains("delegate"));
    }

    #[test]
    fn test_search_agents() {
        let agents = sample_agents();
        let results = search_agents(&agents, "code developer");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "dev");
    }

    #[test]
    fn test_empty_agents() {
        let md = generate_agents_md(&[]);
        assert!(md.is_empty());
    }
}
