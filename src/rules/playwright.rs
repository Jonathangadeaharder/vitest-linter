use crate::models::{Category, ModuleGraph, ParsedModule, Severity, TestRuntime, Violation};
use crate::rules::Rule;

pub struct PwWaitForTimeoutRule;

impl Rule for PwWaitForTimeoutRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-001"
    }
    fn name(&self) -> &'static str {
        "PwWaitForTimeoutRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        pw.calls
            .iter()
            .filter(|c| c.call_name.contains("waitForTimeout"))
            .map(|c| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "waitForTimeout causes flaky waits — prefer auto-retry assertions or waitForSelector".to_string(),
                file_path: module.file_path.clone(),
                line: c.line,
                col: None,
                suggestion: Some("Replace with await page.waitForSelector(...) or an assertion-based wait".to_string()),
                test_name: None,
            })
            .collect()
    }
}

pub struct PwXPathSelectorRule;

impl Rule for PwXPathSelectorRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-003"
    }
    fn name(&self) -> &'static str {
        "PwXPathSelectorRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        let mut violations = Vec::new();
        let mut flagged_lines = std::collections::HashSet::new();
        for chain in &pw.locator_chains {
            if chain.method == "xpath" {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message:
                        "XPath selectors are brittle — prefer role/text/accessible-name locators"
                            .to_string(),
                    file_path: module.file_path.clone(),
                    line: chain.line,
                    col: None,
                    suggestion: Some(
                        "Use getByRole, getByText, or getByTestId instead".to_string(),
                    ),
                    test_name: None,
                });
                flagged_lines.insert(chain.line);
                continue;
            }
            if let Some(arg) = &chain.raw_arg {
                if arg.starts_with("xpath=") || arg.starts_with("//") {
                    violations.push(Violation {
                        rule_id: self.id().to_string(),
                        rule_name: self.name().to_string(),
                        severity: self.severity(),
                        category: self.category(),
                        message: "XPath selectors are brittle — prefer role/text/accessible-name locators".to_string(),
                        file_path: module.file_path.clone(),
                        line: chain.line,
                        col: None,
                        suggestion: Some("Use getByRole, getByText, or getByTestId instead".to_string()),
                        test_name: None,
                    });
                    flagged_lines.insert(chain.line);
                }
            }
        }
        for call in &pw.calls {
            if call.call_name.contains(".xpath") && !flagged_lines.contains(&call.line) {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message:
                        "XPath selectors are brittle — prefer role/text/accessible-name locators"
                            .to_string(),
                    file_path: module.file_path.clone(),
                    line: call.line,
                    col: None,
                    suggestion: Some(
                        "Use getByRole, getByText, or getByTestId instead".to_string(),
                    ),
                    test_name: None,
                });
            }
        }
        violations
    }
}

pub struct PwLocatorNthRule;

impl Rule for PwLocatorNthRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-004"
    }
    fn name(&self) -> &'static str {
        "PwLocatorNthRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        let mut violations = Vec::new();
        for chain in &pw.locator_chains {
            if chain.method == "nth" {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: ".nth() positional locator is fragile — DOM order may change"
                        .to_string(),
                    file_path: module.file_path.clone(),
                    line: chain.line,
                    col: None,
                    suggestion: Some(
                        "Use a more specific locator (getByRole, getByTestId) or .filter() instead"
                            .to_string(),
                    ),
                    test_name: None,
                });
            }
        }
        for call in &pw.calls {
            if call.call_name.contains(".nth") && !violations.iter().any(|v| v.line == call.line) {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: ".nth() positional locator is fragile — DOM order may change"
                        .to_string(),
                    file_path: module.file_path.clone(),
                    line: call.line,
                    col: None,
                    suggestion: Some(
                        "Use a more specific locator (getByRole, getByTestId) or .filter() instead"
                            .to_string(),
                    ),
                    test_name: None,
                });
            }
        }
        violations
    }
}

pub struct PwPageDollarRule;

impl Rule for PwPageDollarRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-005"
    }
    fn name(&self) -> &'static str {
        "PwPageDollarRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        pw.calls
            .iter()
            .filter(|c| c.call_name.contains("page.$") && !c.call_name.contains("page.$."))
            .map(|c| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: format!(
                    "{} returns raw ElementHandle — prefer locator API",
                    c.call_name
                ),
                file_path: module.file_path.clone(),
                line: c.line,
                col: None,
                suggestion: Some(
                    "Use page.locator(...) instead for auto-retrying and auto-waiting".to_string(),
                ),
                test_name: None,
            })
            .collect()
    }
}

pub struct PwArbitrarySleepRule;

impl Rule for PwArbitrarySleepRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-010"
    }
    fn name(&self) -> &'static str {
        "PwArbitrarySleepRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        pw.calls
            .iter()
            .filter(|c| c.call_name.contains("setTimeout"))
            .map(|c| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Arbitrary sleep (setTimeout in Promise) — prefer Playwright auto-waiting assertions".to_string(),
                file_path: module.file_path.clone(),
                line: c.line,
                col: None,
                suggestion: Some("Use await expect(...).toBeVisible() or page.waitForSelector() instead".to_string()),
                test_name: None,
            })
            .collect()
    }
}

pub struct PwCssIdSelectorRule;

impl Rule for PwCssIdSelectorRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-002"
    }
    fn name(&self) -> &'static str {
        "PwCssIdSelectorRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        let mut violations = Vec::new();
        for chain in &pw.locator_chains {
            if let Some(arg) = &chain.raw_arg {
                if arg.starts_with('#')
                    || (arg.contains('#')
                        && !arg.starts_with("[data-testid")
                        && !arg.contains("data-testid"))
                {
                    violations.push(Violation {
                        rule_id: self.id().to_string(),
                        rule_name: self.name().to_string(),
                        severity: self.severity(),
                        category: self.category(),
                        message: "CSS ID selector — IDs may not be stable across app updates"
                            .to_string(),
                        file_path: module.file_path.clone(),
                        line: chain.line,
                        col: None,
                        suggestion: Some(
                            "Use getByRole, getByText, or data-testid instead".to_string(),
                        ),
                        test_name: None,
                    });
                }
            }
        }
        violations
    }
}

pub struct PwEvaluateInnerTextRule;

impl Rule for PwEvaluateInnerTextRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-006"
    }
    fn name(&self) -> &'static str {
        "PwEvaluateInnerTextRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        pw.evaluate_inner_text
            .iter()
            .map(|line| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "page.evaluate with innerText — prefer ARIA text assertions or getByText"
                    .to_string(),
                file_path: module.file_path.clone(),
                line: *line,
                col: None,
                suggestion: Some(
                    "Use await expect(page.getByText(...)).toBeVisible() instead".to_string(),
                ),
                test_name: None,
            })
            .collect()
    }
}

pub struct PwHardCssClassChainRule;

impl Rule for PwHardCssClassChainRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-011"
    }
    fn name(&self) -> &'static str {
        "PwHardCssClassChainRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        let mut violations = Vec::new();
        for chain in &pw.locator_chains {
            if let Some(arg) = &chain.raw_arg {
                let has_child = arg.contains(" > ");
                let has_descendant =
                    arg.split_whitespace().count() > 1 && !has_child && arg.contains('.');
                if (has_child || has_descendant) && arg.contains('.') {
                    violations.push(Violation {
                        rule_id: self.id().to_string(),
                        rule_name: self.name().to_string(),
                        severity: self.severity(),
                        category: self.category(),
                        message: "Hard CSS class selector chain — fragile to DOM structure changes".to_string(),
                        file_path: module.file_path.clone(),
                        line: chain.line,
                        col: None,
                        suggestion: Some("Use a single semantic locator (getByRole, getByTestId) or simplify the chain".to_string()),
                        test_name: None,
                    });
                }
            }
        }
        violations
    }
}

pub struct PwDuplicateSpecFileRule;

impl Rule for PwDuplicateSpecFileRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-009"
    }
    fn name(&self) -> &'static str {
        "PwDuplicateSpecFileRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let stem = module
            .file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        let stem = match stem {
            Some(s) => s,
            None => return vec![],
        };
        let overlaps: Vec<_> = ctx
            .all_modules
            .iter()
            .filter(|m| {
                m.runtime == TestRuntime::Playwright
                    && m.file_path != module.file_path
                    && m.file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_lowercase())
                        .is_some_and(|o| o.contains(&stem) || stem.contains(&o))
            })
            .collect();
        if overlaps.is_empty() {
            return vec![];
        }
        vec![Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            severity: self.severity(),
            category: self.category(),
            message: format!(
                "Duplicate spec coverage: '{}' overlaps with {} other Playwright file(s)",
                module.file_path.display(),
                overlaps.len()
            ),
            file_path: module.file_path.clone(),
            line: 1,
            col: None,
            suggestion: Some("Consolidate overlapping spec files".to_string()),
            test_name: None,
        }]
    }
}

pub struct PwTextAssertionOverRoleRule;

impl Rule for PwTextAssertionOverRoleRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-007"
    }
    fn name(&self) -> &'static str {
        "PwTextAssertionOverRoleRule"
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        pw.locator_chains.iter().filter(|c| c.root == "getByText" && c.raw_arg.is_some())
            .map(|c| Violation {
                rule_id: self.id().to_string(), rule_name: self.name().to_string(),
                severity: self.severity(), category: self.category(),
                message: "getByText used — prefer getByRole for interactive elements".to_string(),
                file_path: module.file_path.clone(), line: c.line, col: None,
                suggestion: Some("Use getByRole for buttons/links/inputs; reserve getByText for non-role content".to_string()),
                test_name: None,
            }).collect()
    }
}

pub struct PwTestIdOverSemanticRoleRule;

impl Rule for PwTestIdOverSemanticRoleRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-008"
    }
    fn name(&self) -> &'static str {
        "PwTestIdOverSemanticRoleRule"
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        let testid = pw
            .locator_chains
            .iter()
            .filter(|c| c.root == "getByTestId")
            .count();
        let semantic = pw
            .locator_chains
            .iter()
            .filter(|c| matches!(c.root.as_str(), "getByRole" | "getByText" | "getByLabel"))
            .count();
        if testid > 0 && semantic == 0 {
            pw.locator_chains.iter().filter(|c| c.root == "getByTestId").take(1)
                .map(|c| Violation {
                    rule_id: self.id().to_string(), rule_name: self.name().to_string(),
                    severity: self.severity(), category: self.category(),
                    message: "Only testId locators — prefer semantic role/text locators".to_string(),
                    file_path: module.file_path.clone(), line: c.line, col: None,
                    suggestion: Some("Use getByRole for buttons/inputs; reserve testId for non-semantic elements".to_string()),
                    test_name: None,
                }).collect()
        } else {
            vec![]
        }
    }
}

pub struct PwMissingWebFirstAssertionRule;

impl Rule for PwMissingWebFirstAssertionRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-012"
    }
    fn name(&self) -> &'static str {
        "PwMissingWebFirstAssertionRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        if module.test_blocks.is_empty() {
            return vec![];
        }
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        if pw.locator_chains.is_empty() && !pw.calls.is_empty() {
            let first = pw.calls.first().unwrap();
            vec![Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "No accessor locators — use web-first assertions".to_string(),
                file_path: module.file_path.clone(),
                line: first.line,
                col: None,
                suggestion: Some(
                    "Use getByRole/getByText/getByTestId with expect().toBeVisible()".to_string(),
                ),
                test_name: None,
            }]
        } else {
            vec![]
        }
    }
}

pub struct PwMissingAxeScanRule;

impl Rule for PwMissingAxeScanRule {
    fn id(&self) -> &'static str {
        "VITEST-PW-100"
    }
    fn name(&self) -> &'static str {
        "PwMissingAxeScanRule"
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
    fn category(&self) -> Category {
        Category::Playwright
    }
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        runtime == TestRuntime::Playwright
    }
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let pw = match &module.playwright {
            Some(pw) => pw,
            None => return vec![],
        };
        if pw.uses_axe || module.test_blocks.len() < 2 {
            return vec![];
        }
        vec![Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            severity: self.severity(),
            category: self.category(),
            message: "No axe accessibility scan — consider @axe-core/playwright".to_string(),
            file_path: module.file_path.clone(),
            line: 1,
            col: None,
            suggestion: Some(
                "Add `import { injectAxe, checkA11y } from '@axe-core/playwright'`".to_string(),
            ),
            test_name: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LocatorChain, PlaywrightCall, PlaywrightModule};
    use crate::rules::LintContext;
    use std::path::PathBuf;

    fn make_pw_module(calls: Vec<PlaywrightCall>, chains: Vec<LocatorChain>) -> ParsedModule {
        ParsedModule {
            file_path: PathBuf::from("test.spec.ts"),
            runtime: TestRuntime::Playwright,
            playwright: Some(PlaywrightModule {
                calls,
                locator_chains: chains,
                evaluate_inner_text: vec![],
                uses_axe: false,
            }),
            ..ParsedModule::default()
        }
    }

    #[test]
    fn pw001_flags_wait_for_timeout() {
        let module = make_pw_module(
            vec![PlaywrightCall {
                call_name: "waitForTimeout".to_string(),
                line: 5,
                raw_arg: Some("1000".to_string()),
            }],
            vec![],
        );
        let ctx = LintContext::default();
        let graph = ModuleGraph::default();
        let violations = PwWaitForTimeoutRule.check(&module, &ctx, &graph);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "VITEST-PW-001");
    }

    #[test]
    fn pw003_flags_xpath_method() {
        let module = make_pw_module(
            vec![],
            vec![LocatorChain {
                root: "page".to_string(),
                raw_arg: Some("//div".to_string()),
                method: "xpath".to_string(),
                line: 3,
            }],
        );
        let ctx = LintContext::default();
        let graph = ModuleGraph::default();
        let violations = PwXPathSelectorRule.check(&module, &ctx, &graph);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "VITEST-PW-003");
    }

    #[test]
    fn pw003_flags_xpath_prefix_in_arg() {
        let module = make_pw_module(
            vec![],
            vec![LocatorChain {
                root: "page".to_string(),
                raw_arg: Some("xpath=//div".to_string()),
                method: "locator".to_string(),
                line: 3,
            }],
        );
        let ctx = LintContext::default();
        let graph = ModuleGraph::default();
        let violations = PwXPathSelectorRule.check(&module, &ctx, &graph);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn pw004_flags_nth_locator() {
        let module = make_pw_module(
            vec![],
            vec![LocatorChain {
                root: "page".to_string(),
                raw_arg: None,
                method: "nth".to_string(),
                line: 7,
            }],
        );
        let ctx = LintContext::default();
        let graph = ModuleGraph::default();
        let violations = PwLocatorNthRule.check(&module, &ctx, &graph);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "VITEST-PW-004");
    }

    #[test]
    fn pw005_flags_page_dollar() {
        let module = make_pw_module(
            vec![
                PlaywrightCall {
                    call_name: "page.$".to_string(),
                    line: 4,
                    raw_arg: Some(".btn".to_string()),
                },
                PlaywrightCall {
                    call_name: "page.$$".to_string(),
                    line: 5,
                    raw_arg: Some(".item".to_string()),
                },
            ],
            vec![],
        );
        let ctx = LintContext::default();
        let graph = ModuleGraph::default();
        let violations = PwPageDollarRule.check(&module, &ctx, &graph);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn pw010_flags_settimeout_in_promise() {
        let module = make_pw_module(
            vec![PlaywrightCall {
                call_name: "setTimeout_in_promise".to_string(),
                line: 8,
                raw_arg: None,
            }],
            vec![],
        );
        let ctx = LintContext::default();
        let graph = ModuleGraph::default();
        let violations = PwArbitrarySleepRule.check(&module, &ctx, &graph);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "VITEST-PW-010");
    }

    #[test]
    fn pw_rules_skip_vitest_files() {
        let module = ParsedModule {
            file_path: PathBuf::from("test.test.ts"),
            runtime: TestRuntime::Vitest,
            ..ParsedModule::default()
        };
        let ctx = LintContext::default();
        let graph = ModuleGraph::default();
        assert!(PwWaitForTimeoutRule.check(&module, &ctx, &graph).is_empty());
        assert!(PwXPathSelectorRule.check(&module, &ctx, &graph).is_empty());
        assert!(PwLocatorNthRule.check(&module, &ctx, &graph).is_empty());
        assert!(PwPageDollarRule.check(&module, &ctx, &graph).is_empty());
        assert!(PwArbitrarySleepRule.check(&module, &ctx, &graph).is_empty());
    }
}
