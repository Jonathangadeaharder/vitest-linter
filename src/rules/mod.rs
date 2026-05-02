use crate::config::Config;
use crate::models::{Category, ModuleGraph, ParsedModule, Severity, Violation};

/// Context passed to each rule during evaluation, including the active
/// configuration and all modules in the current group.
pub struct LintContext<'a> {
    pub config: &'a Config,
    pub all_modules: &'a [ParsedModule],
}

impl Default for LintContext<'_> {
    fn default() -> Self {
        use std::sync::OnceLock;
        static DEFAULT_CONFIG: OnceLock<Config> = OnceLock::new();
        Self {
            config: DEFAULT_CONFIG.get_or_init(Config::default),
            all_modules: &[],
        }
    }
}

/// Trait implemented by every lint rule.
pub trait Rule {
    /// Unique rule identifier (e.g. `VITEST-FLK-001`).
    fn id(&self) -> &'static str;
    /// Human-readable rule name (e.g. `TimeoutRule`).
    fn name(&self) -> &'static str;
    /// Default severity level for this rule.
    fn severity(&self) -> Severity;
    /// Category this rule belongs to.
    fn category(&self) -> Category;
    /// Evaluate the rule against a parsed module and return any violations.
    fn check(
        &self,
        module: &ParsedModule,
        ctx: &LintContext<'_>,
        graph: &ModuleGraph,
    ) -> Vec<Violation>;
}

pub mod consistency;
pub mod dependencies;
pub mod flakiness;
pub mod maintenance;
pub mod no_rules;
pub mod prefer;
pub mod require;
pub mod validation;

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
        Box::new(maintenance::WeakAssertionRule),
        Box::new(maintenance::ImplementationCoupledRule),
        Box::new(dependencies::BannedModuleMockRule),
        Box::new(dependencies::ProductionSingletonImportRule),
        Box::new(dependencies::ResetEscapeHatchRule),
        Box::new(dependencies::MockExportValidationRule),
        Box::new(validation::ValidExpectRule),
        Box::new(validation::ValidExpectInPromiseRule),
        Box::new(validation::ValidDescribeCallbackRule),
        Box::new(validation::ValidTitleRule),
        Box::new(validation::NoUnneededAsyncExpectFunctionRule),
        // E12: No-rules
        Box::new(no_rules::NoStandaloneExpectRule),
        Box::new(no_rules::NoIdenticalTitleRule),
        Box::new(no_rules::NoCommentedOutTestsRule),
        Box::new(no_rules::NoTestPrefixesRule),
        Box::new(no_rules::NoDuplicateHooksRule),
        Box::new(no_rules::NoImportNodeTestRule),
        Box::new(no_rules::NoInterpolationInSnapshotsRule),
        Box::new(no_rules::NoLargeSnapshotsRule),
        Box::new(no_rules::NoDoneCallbackRule),
        Box::new(no_rules::NoConditionalExpectRule),
        // E13: Prefer-rules
        Box::new(prefer::PreferToBeRule),
        Box::new(prefer::PreferToContainRule),
        Box::new(prefer::PreferToHaveLengthRule),
        Box::new(prefer::PreferSpyOnRule),
        Box::new(prefer::PreferCalledOnceRule),
        Box::new(prefer::PreferHooksOnTopRule),
        Box::new(prefer::PreferHooksInOrderRule),
        Box::new(prefer::PreferTodoRule),
        Box::new(prefer::PreferMockPromiseShorthandRule),
        Box::new(prefer::PreferExpectResolvesRule),
        // E14: Require-rules
        Box::new(require::RequireHookRule),
        Box::new(require::RequireTopLevelDescribeRule),
        Box::new(require::RequireToThrowMessageRule),
        // E15: Consistency-rules
        Box::new(consistency::ConsistentTestItRule),
        Box::new(consistency::ConsistentVitestViRule),
        Box::new(consistency::HoistedApisOnTopRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 52);
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
            "VITEST-MNT-009",
            "VITEST-MNT-010",
            "VITEST-MNT-010",
            "VITEST-DEP-001",
            "VITEST-DEP-002",
            "VITEST-DEP-003",
            "VITEST-DEP-004",
            "VITEST-DEP-004",
            "VITEST-VAL-001",
            "VITEST-VAL-002",
            "VITEST-VAL-003",
            "VITEST-VAL-004",
            "VITEST-VAL-005",
            "VITEST-NO-001",
            "VITEST-NO-002",
            "VITEST-NO-003",
            "VITEST-NO-005",
            "VITEST-NO-006",
            "VITEST-NO-007",
            "VITEST-NO-008",
            "VITEST-NO-009",
            "VITEST-NO-013",
            "VITEST-NO-014",
            "VITEST-PREF-001",
            "VITEST-PREF-002",
            "VITEST-PREF-003",
            "VITEST-PREF-005",
            "VITEST-PREF-007",
            "VITEST-PREF-009",
            "VITEST-PREF-010",
            "VITEST-PREF-012",
            "VITEST-PREF-013",
            "VITEST-PREF-014",
            "VITEST-REQ-001",
            "VITEST-REQ-002",
            "VITEST-REQ-003",
            "VITEST-CON-001",
            "VITEST-CON-003",
            "VITEST-CON-004",
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
        let val: Vec<_> = rules
            .iter()
            .filter(|r| r.category() == Category::Validation)
            .collect();
        assert_eq!(flk.len(), 5);
        assert_eq!(mnt.len(), 17);
        assert_eq!(str_.len(), 9);
        assert_eq!(dep.len(), 7);
        assert_eq!(val.len(), 14);
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
            ("VITEST-MNT-009", "WeakAssertionRule"),
            ("VITEST-MNT-010", "ImplementationCoupledRule"),
            ("VITEST-DEP-001", "BannedModuleMockRule"),
            ("VITEST-DEP-002", "ProductionSingletonImportRule"),
            ("VITEST-DEP-003", "ResetEscapeHatchRule"),
            ("VITEST-DEP-004", "MockExportValidationRule"),
            ("VITEST-VAL-001", "ValidExpectRule"),
            ("VITEST-VAL-002", "ValidExpectInPromiseRule"),
            ("VITEST-VAL-003", "ValidDescribeCallbackRule"),
            ("VITEST-VAL-004", "ValidTitleRule"),
            ("VITEST-VAL-005", "NoUnneededAsyncExpectFunctionRule"),
            ("VITEST-NO-001", "NoStandaloneExpectRule"),
            ("VITEST-NO-002", "NoIdenticalTitleRule"),
            ("VITEST-NO-003", "NoCommentedOutTestsRule"),
            ("VITEST-NO-005", "NoTestPrefixesRule"),
            ("VITEST-NO-006", "NoDuplicateHooksRule"),
            ("VITEST-NO-007", "NoImportNodeTestRule"),
            ("VITEST-NO-008", "NoInterpolationInSnapshotsRule"),
            ("VITEST-NO-009", "NoLargeSnapshotsRule"),
            ("VITEST-NO-013", "NoDoneCallbackRule"),
            ("VITEST-NO-014", "NoConditionalExpectRule"),
            ("VITEST-PREF-001", "PreferToBeRule"),
            ("VITEST-PREF-002", "PreferToContainRule"),
            ("VITEST-PREF-003", "PreferToHaveLengthRule"),
            ("VITEST-PREF-005", "PreferSpyOnRule"),
            ("VITEST-PREF-007", "PreferCalledOnceRule"),
            ("VITEST-PREF-009", "PreferHooksOnTopRule"),
            ("VITEST-PREF-010", "PreferHooksInOrderRule"),
            ("VITEST-PREF-012", "PreferTodoRule"),
            ("VITEST-PREF-013", "PreferMockPromiseShorthandRule"),
            ("VITEST-PREF-014", "PreferExpectResolvesRule"),
            ("VITEST-REQ-001", "RequireHookRule"),
            ("VITEST-REQ-002", "RequireTopLevelDescribeRule"),
            ("VITEST-REQ-003", "RequireToThrowMessageRule"),
            ("VITEST-CON-001", "ConsistentTestItRule"),
            ("VITEST-CON-003", "ConsistentVitestViRule"),
            ("VITEST-CON-004", "HoistedApisOnTopRule"),
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
