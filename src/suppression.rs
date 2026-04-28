use std::collections::{HashMap, HashSet};

const DISABLE_NEXT_LINE: &str = "vitest-linter-disable-next-line";
const DISABLE: &str = "vitest-linter-disable";
const ENABLE: &str = "vitest-linter-enable";

/// Sentinel value meaning "suppress all rules".
const ALL_RULES: &str = "";

#[derive(Debug, Clone, Default)]
pub struct SuppressionMap {
    /// Lines where a specific rule (or all rules via `ALL_RULES` sentinel) is
    /// suppressed via `disable-next-line`. Key = target line (1-indexed).
    next_line: HashMap<usize, HashSet<String>>,
    /// Per-line range suppressions accumulated from `disable`/`enable` pairs.
    /// Key = line (1-indexed), value = set of rule IDs (`ALL_RULES` means all).
    range: HashMap<usize, HashSet<String>>,
    /// Per-line exceptions: rules explicitly enabled while an all-range was
    /// active. These rules should NOT be suppressed even if `ALL_RULES` is in
    /// the range set.
    range_exceptions: HashMap<usize, HashSet<String>>,
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
        // Rules explicitly enabled (exceptions) while an all-range is active.
        let mut enable_exceptions: HashSet<String> = HashSet::new();

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
                    &enable_exceptions,
                    &mut map,
                );
                continue;
            };

            if let Some(rest) = comment_text.strip_prefix(DISABLE_NEXT_LINE) {
                let rules = parse_rule_ids(rest.trim());
                let target_line = line_num + 1;
                // Merge with any existing rules for the same target line
                let entry = map
                    .next_line
                    .entry(target_line)
                    .or_insert_with(HashSet::new);
                if rules.is_empty() {
                    // No specific rules = suppress all
                    entry.insert(ALL_RULES.to_string());
                } else {
                    entry.extend(rules);
                }
                Self::propagate_range(
                    line_num,
                    &active_ranges,
                    active_all_range,
                    &enable_exceptions,
                    &mut map,
                );
            } else if let Some(rest) = comment_text.strip_prefix(ENABLE) {
                let rules = parse_rule_ids(rest.trim());
                if rules.is_empty() {
                    active_all_range = None;
                    active_ranges.clear();
                    enable_exceptions.clear();
                } else if active_all_range.is_some() {
                    // Record as exceptions to the active all-range
                    for rule_id in &rules {
                        enable_exceptions.insert(rule_id.clone());
                    }
                } else {
                    for rule_id in &rules {
                        active_ranges.remove(rule_id);
                    }
                }
                Self::propagate_range(
                    line_num,
                    &active_ranges,
                    active_all_range,
                    &enable_exceptions,
                    &mut map,
                );
            } else if let Some(rest) = comment_text.strip_prefix(DISABLE) {
                let rules = parse_rule_ids(rest.trim());
                if rules.is_empty() {
                    active_all_range = Some(line_num);
                    enable_exceptions.clear();
                } else {
                    for rule_id in rules {
                        active_ranges.entry(rule_id).or_insert(line_num);
                    }
                }
            }
            Self::propagate_range(
                line_num,
                &active_ranges,
                active_all_range,
                &enable_exceptions,
                &mut map,
            );
        }

        map
    }

    fn propagate_range(
        line_num: usize,
        active_ranges: &HashMap<String, usize>,
        active_all_range: Option<usize>,
        enable_exceptions: &HashSet<String>,
        map: &mut Self,
    ) {
        if active_all_range.is_some() || !active_ranges.is_empty() {
            let mut suppressed = HashSet::new();
            if active_all_range.is_some() {
                suppressed.insert(ALL_RULES.to_string());
            }
            for rule_id in active_ranges.keys() {
                suppressed.insert(rule_id.clone());
            }
            map.range.insert(line_num, suppressed);
            // Track exceptions separately so is_suppressed can check them
            if !enable_exceptions.is_empty() {
                map.range_exceptions
                    .insert(line_num, enable_exceptions.clone());
            }
        }
    }

    /// Check if a violation at `line` for `rule_id` is suppressed.
    #[must_use]
    pub fn is_suppressed(&self, line: usize, rule_id: &str) -> bool {
        // Check next-line suppressions
        if let Some(rules) = self.next_line.get(&line) {
            if rules.contains(ALL_RULES) || rules.contains(rule_id) {
                return true;
            }
        }
        // Check range suppressions
        if let Some(rules) = self.range.get(&line) {
            // Check if this specific rule is explicitly excepted
            if let Some(exceptions) = self.range_exceptions.get(&line) {
                if exceptions.contains(rule_id) {
                    return false;
                }
            }
            if rules.contains(ALL_RULES) || rules.contains(rule_id) {
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
    fn disable_next_line_consecutive_comments() {
        // Two consecutive disable-next-line comments each suppress their own next line
        let source = r#"// vitest-linter-disable-next-line VITEST-FLK-001
// vitest-linter-disable-next-line VITEST-FLK-002
setTimeout(() => {}, 100);
"#;
        let map = SuppressionMap::parse(source);
        // Line 2 (second comment) is suppressed by line 1's directive
        assert!(map.is_suppressed(2, "VITEST-FLK-001"));
        assert!(!map.is_suppressed(2, "VITEST-FLK-002"));
        // Line 3 (setTimeout) is suppressed by line 2's directive
        assert!(map.is_suppressed(3, "VITEST-FLK-002"));
        assert!(!map.is_suppressed(3, "VITEST-FLK-001"));
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
    fn enable_with_rules_creates_exceptions() {
        let source = r#"// vitest-linter-disable
const x = 1;
// vitest-linter-enable VITEST-FLK-001
const y = 2;
"#;
        let map = SuppressionMap::parse(source);
        // VITEST-FLK-001 is enabled (exception), should not be suppressed
        assert!(!map.is_suppressed(4, "VITEST-FLK-001"));
        // Other rules are still suppressed by the all-range
        assert!(map.is_suppressed(4, "VITEST-MNT-001"));
        assert!(map.is_suppressed(4, "VITEST-FLK-002"));
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
