use std::collections::{HashMap, HashSet};

const DISABLE_NEXT_LINE: &str = "vitest-linter-disable-next-line";
const DISABLE: &str = "vitest-linter-disable";
const ENABLE: &str = "vitest-linter-enable";

#[derive(Debug, Clone, Default)]
pub struct SuppressionMap {
    /// Lines where a specific rule (or all rules if empty) is suppressed via
    /// `disable-next-line`. Key = target line (1-indexed), value = set of rule
    /// IDs (empty means suppress all).
    next_line: HashMap<usize, HashSet<String>>,
    /// Per-line range suppressions accumulated from `disable`/`enable` pairs.
    /// Key = line (1-indexed), value = set of rule IDs (empty means all).
    range: HashMap<usize, HashSet<String>>,
}

impl SuppressionMap {
    /// Parse all comments in `source` and build a suppression map.
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let mut map = Self::default();
        let lines: Vec<&str> = source.lines().collect();

        // Track active range suppressions: rule_id -> start_line
        let mut active_ranges: HashMap<String, usize> = HashMap::new();
        let mut active_all_range: Option<usize> = None;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Only process single-line comments
            let comment_text = if let Some(text) = trimmed.strip_prefix("//") {
                text.trim()
            } else {
                // Not a comment - propagate range suppressions
                Self::propagate_range(
                    line_num,
                    &active_ranges,
                    active_all_range,
                    &mut map,
                );
                continue;
            };

            if comment_text.starts_with(DISABLE_NEXT_LINE) {
                let rest = comment_text[DISABLE_NEXT_LINE.len()..].trim();
                let rules = parse_rule_ids(rest);
                let target_line = line_num + 1;
                map.next_line.insert(target_line, rules);
                Self::propagate_range(
                    line_num,
                    &active_ranges,
                    active_all_range,
                    &mut map,
                );
            } else if comment_text.starts_with(ENABLE) {
                let rest = comment_text[ENABLE.len()..].trim();
                let rules = parse_rule_ids(rest);
                if rules.is_empty() {
                    active_all_range = None;
                    active_ranges.clear();
                } else {
                    for rule_id in &rules {
                        active_ranges.remove(rule_id);
                    }
                }
                Self::propagate_range(
                    line_num,
                    &active_ranges,
                    active_all_range,
                    &mut map,
                );
            } else if let Some(rest) = comment_text.strip_prefix(DISABLE) {
                let rules = parse_rule_ids(rest.trim());
                if rules.is_empty() {
                    active_all_range = Some(line_num);
                } else {
                    for rule_id in rules {
                        active_ranges.entry(rule_id).or_insert(line_num);
                    }
                }
                Self::propagate_range(
                    line_num,
                    &active_ranges,
                    active_all_range,
                    &mut map,
                );
            } else {
                // Regular comment - propagate range suppressions
                Self::propagate_range(
                    line_num,
                    &active_ranges,
                    active_all_range,
                    &mut map,
                );
            }
        }

        map
    }

    fn propagate_range(
        line_num: usize,
        active_ranges: &HashMap<String, usize>,
        active_all_range: Option<usize>,
        map: &mut SuppressionMap,
    ) {
        if active_all_range.is_some() || !active_ranges.is_empty() {
            let mut suppressed = HashSet::new();
            if active_all_range.is_some() {
                suppressed.insert(String::new());
            }
            for rule_id in active_ranges.keys() {
                suppressed.insert(rule_id.clone());
            }
            map.range.insert(line_num, suppressed);
        }
    }

    /// Check if a violation at `line` for `rule_id` is suppressed.
    #[must_use]
    pub fn is_suppressed(&self, line: usize, rule_id: &str) -> bool {
        // Check next-line suppressions
        if let Some(rules) = self.next_line.get(&line) {
            if rules.is_empty() || rules.contains("") || rules.contains(rule_id) {
                return true;
            }
        }
        // Check range suppressions
        if let Some(rules) = self.range.get(&line) {
            // Empty string "" in the set means "suppress all rules"
            if rules.is_empty() || rules.contains("") || rules.contains(rule_id) {
                return true;
            }
        }
        false
    }
}

fn parse_rule_ids(text: &str) -> HashSet<String> {
    text.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable_next_line_single_rule() {
        let source = r#"// vitest-linter-disable-next-line VITEST-FLK-001
setTimeout(() => {}, 100);
"#;
        let map = SuppressionMap::parse(source);
        assert!(map.is_suppressed(2, "VITEST-FLK-001"));
        assert!(!map.is_suppressed(2, "VITEST-FLK-002"));
    }

    #[test]
    fn disable_next_line_all_rules() {
        let source = r#"// vitest-linter-disable-next-line
setTimeout(() => {}, 100);
"#;
        let map = SuppressionMap::parse(source);
        assert!(map.is_suppressed(2, "VITEST-FLK-001"));
        assert!(map.is_suppressed(2, "VITEST-MNT-001"));
    }

    #[test]
    fn disable_next_line_multiple_rules() {
        let source = r#"// vitest-linter-disable-next-line VITEST-FLK-001, VITEST-FLK-002
setTimeout(() => {}, 100);
"#;
        let map = SuppressionMap::parse(source);
        assert!(map.is_suppressed(2, "VITEST-FLK-001"));
        assert!(map.is_suppressed(2, "VITEST-FLK-002"));
        assert!(!map.is_suppressed(2, "VITEST-FLK-003"));
    }

    #[test]
    fn disable_range() {
        let source = r#"// vitest-linter-disable VITEST-FLK-001
const x = 1;
const y = 2;
// vitest-linter-enable VITEST-FLK-001
const z = 3;
"#;
        let map = SuppressionMap::parse(source);
        assert!(map.is_suppressed(2, "VITEST-FLK-001"));
        assert!(map.is_suppressed(3, "VITEST-FLK-001"));
        assert!(!map.is_suppressed(5, "VITEST-FLK-001"));
    }

    #[test]
    fn disable_all_range() {
        let source = r#"// vitest-linter-disable
const x = 1;
// vitest-linter-enable
const y = 2;
"#;
        let map = SuppressionMap::parse(source);
        assert!(map.is_suppressed(2, "VITEST-FLK-001"));
        assert!(map.is_suppressed(2, "VITEST-MNT-001"));
        assert!(!map.is_suppressed(4, "VITEST-FLK-001"));
    }

    #[test]
    fn no_suppression() {
        let source = r#"const x = 1;
setTimeout(() => {}, 100);
"#;
        let map = SuppressionMap::parse(source);
        assert!(!map.is_suppressed(2, "VITEST-FLK-001"));
    }
}
