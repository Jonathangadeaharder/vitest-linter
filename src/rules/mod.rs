use crate::config::Config;
use crate::models::{Category, ParsedModule, Severity, Violation};

pub struct LintContext<'a> {
    pub config: &'a Config,
    pub all_modules: &'a [ParsedModule],
}

pub trait Rule {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn category(&self) -> Category;
    fn check(&self, module: &ParsedModule, ctx: &LintContext<'_>) -> Vec<Violation>;
}

pub mod dependencies;
pub mod flakiness;
pub mod maintenance;

#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(flakiness::TimeoutRule),
        Box::new(flakiness::DateMockRule),
        Box::new(flakiness::NetworkImportRule),
        Box::new(flakiness::FakeTimersCleanupRule),
        Box::new(flakiness::NonDeterministicRule),
        Box::new(maintenance::NoAssertionRule),
        Box::new(maintenance::MultipleExpectRule),
        Box::new(maintenance::ConditionalLogicRule),
        Box::new(maintenance::TryCatchRule),
        Box::new(maintenance::EmptyTestRule),
        Box::new(maintenance::NestedDescribeRule),
        Box::new(maintenance::ReturnInTestRule),
        Box::new(maintenance::MissingAwaitAssertionRule),
        Box::new(maintenance::FocusedTestRule),
        Box::new(maintenance::MissingMockCleanupRule),
        Box::new(dependencies::BannedModuleMockRule),
        Box::new(dependencies::ProductionSingletonImportRule),
        Box::new(dependencies::ResetEscapeHatchRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 18);
    }

    #[test]
    fn all_rule_ids_present() {
        let rules = all_rules();
        let expected = [
            "VITEST-FLK-001",
            "VITEST-FLK-002",
            "VITEST-FLK-003",
            "VITEST-FLK-004",
            "VITEST-FLK-005",
            "VITEST-MNT-001",
            "VITEST-MNT-002",
            "VITEST-MNT-003",
            "VITEST-MNT-004",
            "VITEST-MNT-005",
            "VITEST-STR-001",
            "VITEST-STR-002",
            "VITEST-MNT-006",
            "VITEST-MNT-007",
            "VITEST-MNT-008",
            "VITEST-DEP-001",
            "VITEST-DEP-002",
            "VITEST-DEP-003",
        ];
        let ids: Vec<&str> = rules.iter().map(|r| r.id()).collect();
        for id in &expected {
            assert!(ids.contains(id), "Missing rule: {}", id);
        }
    }

    #[test]
    fn all_rules_unique_ids() {
        let rules = all_rules();
        let ids: Vec<&str> = rules.iter().map(|r| r.id()).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(ids.len(), unique.len(), "Duplicate rule IDs found");
    }

    #[test]
    fn rule_categories() {
        let rules = all_rules();
        let flk: Vec<_> = rules
            .iter()
            .filter(|r| r.category() == Category::Flakiness)
            .collect();
        let mnt: Vec<_> = rules
            .iter()
            .filter(|r| r.category() == Category::Maintenance)
            .collect();
        let str_: Vec<_> = rules
            .iter()
            .filter(|r| r.category() == Category::Structure)
            .collect();
        let dep: Vec<_> = rules
            .iter()
            .filter(|r| r.category() == Category::Dependencies)
            .collect();
        assert_eq!(flk.len(), 5);
        assert_eq!(mnt.len(), 8);
        assert_eq!(str_.len(), 2);
        assert_eq!(dep.len(), 3);
    }

    #[test]
    fn all_rule_names_correct() {
        let rules = all_rules();
        let expected = [
            ("VITEST-FLK-001", "TimeoutRule"),
            ("VITEST-FLK-002", "DateMockRule"),
            ("VITEST-FLK-003", "NetworkImportRule"),
            ("VITEST-FLK-004", "FakeTimersCleanupRule"),
            ("VITEST-FLK-005", "NonDeterministicRule"),
            ("VITEST-MNT-001", "NoAssertionRule"),
            ("VITEST-MNT-002", "MultipleExpectRule"),
            ("VITEST-MNT-003", "ConditionalLogicRule"),
            ("VITEST-MNT-004", "TryCatchRule"),
            ("VITEST-MNT-005", "EmptyTestRule"),
            ("VITEST-STR-001", "NestedDescribeRule"),
            ("VITEST-STR-002", "ReturnInTestRule"),
            ("VITEST-MNT-006", "MissingAwaitAssertionRule"),
            ("VITEST-MNT-007", "FocusedTestRule"),
            ("VITEST-MNT-008", "MissingMockCleanupRule"),
            ("VITEST-DEP-001", "BannedModuleMockRule"),
            ("VITEST-DEP-002", "ProductionSingletonImportRule"),
            ("VITEST-DEP-003", "ResetEscapeHatchRule"),
        ];
        for (id, name) in &expected {
            let rule = rules.iter().find(|r| r.id() == *id).unwrap();
            assert_eq!(
                rule.name(),
                *name,
                "Rule {} should have name '{}'",
                id,
                name
            );
        }
    }
}
