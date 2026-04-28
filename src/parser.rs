use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::models::{
    DescribeBlock, HookCall, HookKind, ImportEntry, MockScope, ParsedModule, TestBlock, ViMockCall,
};

pub struct TsParser;

#[derive(Default)]
struct Context {
    imports: Vec<String>,
    imports_parsed: Vec<ImportEntry>,
    vi_mocks: Vec<ViMockCall>,
    hook_calls: Vec<HookCall>,
    test_blocks: Vec<TestBlock>,
    describe_blocks: Vec<DescribeBlock>,
}

impl TsParser {
    #[allow(clippy::missing_errors_doc)]
    pub const fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn parse_file(&self, path: &Path) -> anyhow::Result<ParsedModule> {
        let mut parser = Parser::new();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let language = if ext == "tsx" || ext == "jsx" {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        };
        parser.set_language(&language)?;

        let source = std::fs::read_to_string(path)?;
        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse file: {}", path.display()))?;

        let root = tree.root_node();
        let mut ctx = Context::default();

        Self::collect(root, &source, path, 0, MockScope::Module, &mut ctx);

        let has_fake_timers = source.contains("useFakeTimers");

        Ok(ParsedModule {
            file_path: path.to_path_buf(),
            imports: ctx.imports,
            imports_parsed: ctx.imports_parsed,
            vi_mocks: ctx.vi_mocks,
            hook_calls: ctx.hook_calls,
            test_blocks: ctx.test_blocks,
            describe_blocks: ctx.describe_blocks,
            has_fake_timers,
        })
    }

    fn collect(
        node: Node,
        source: &str,
        path: &Path,
        describe_depth: usize,
        scope: MockScope,
        ctx: &mut Context,
    ) {
        for i in 0..node.named_child_count() {
            let Some(child) = node.named_child(i) else {
                continue;
            };
            match child.kind() {
                "import_statement" => {
                    let text = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    ctx.imports.push(text);
                    if let Some(entry) = Self::parse_import(child, source) {
                        ctx.imports_parsed.push(entry);
                    }
                }
                "call_expression" => {
                    Self::handle_call(child, source, path, describe_depth, scope, ctx);
                }
                _ => {
                    Self::collect(child, source, path, describe_depth, scope, ctx);
                }
            }
        }
    }

    fn handle_call(
        node: Node,
        source: &str,
        path: &Path,
        describe_depth: usize,
        scope: MockScope,
        ctx: &mut Context,
    ) {
        let Some(func_node) = node.child_by_field_name("function") else {
            Self::collect(node, source, path, describe_depth, scope, ctx);
            return;
        };

        let (func_name, is_skip, is_only) = Self::parse_callee(func_node, source);
        let full_callee = func_node
            .utf8_text(source.as_bytes())
            .unwrap_or("")
            .to_string();

        // vi.mock(...) — module-scope hoisted mock when at module scope, but we
        // record it with the actual lexical scope so rules can decide.
        if full_callee == "vi.mock" {
            if let Some(entry) = Self::extract_vi_mock(node, source, scope) {
                ctx.vi_mocks.push(entry);
            }
        }

        match func_name.as_str() {
            "test" | "it" => {
                if let Some(tb) = Self::extract_test(node, source, path, describe_depth, is_skip, is_only) {
                    ctx.test_blocks.push(tb);
                }
                // Recurse into body with Test scope so nested vi.* calls
                // (rare, and a smell themselves) get tagged correctly.
                if let Some(body) = Self::callback_body(node) {
                    Self::collect(body, source, path, describe_depth, MockScope::Test, ctx);
                }
            }
            "describe" => {
                // Record describe block for .only detection
                let name_node = node
                    .child_by_field_name("arguments")
                    .and_then(|args| args.named_child(0));
                let name = name_node
                    .and_then(|n| Self::string_value(n, source))
                    .unwrap_or_default();
                ctx.describe_blocks.push(DescribeBlock {
                    name,
                    file_path: path.to_path_buf(),
                    line: node.start_position().row + 1,
                    is_only,
                    depth: describe_depth,
                });

                if let Some(body) = Self::callback_body(node) {
                    Self::collect(body, source, path, describe_depth + 1, scope, ctx);
                } else {
                    Self::collect(node, source, path, describe_depth, scope, ctx);
                }
            }
            "beforeEach" | "afterEach" | "beforeAll" | "afterAll" => {
                let kind = match func_name.as_str() {
                    "beforeEach" => HookKind::BeforeEach,
                    "afterEach" => HookKind::AfterEach,
                    "beforeAll" => HookKind::BeforeAll,
                    "afterAll" => HookKind::AfterAll,
                    _ => unreachable!(),
                };
                let mut vi_calls = Vec::new();
                if let Some(body) = Self::single_callback_body(node) {
                    Self::collect_vi_calls(body, source, &mut vi_calls);
                    Self::collect(body, source, path, describe_depth, MockScope::Hook, ctx);
                }
                ctx.hook_calls.push(HookCall {
                    kind,
                    line: node.start_position().row + 1,
                    vi_calls,
                });
            }
            _ => {
                Self::collect(node, source, path, describe_depth, scope, ctx);
            }
        }
    }

    fn collect_vi_calls(node: Node, source: &str, out: &mut Vec<String>) {
        if node.kind() == "call_expression" {
            if let Some(func) = node.child_by_field_name("function") {
                let text = func.utf8_text(source.as_bytes()).unwrap_or("");
                if text.starts_with("vi.") {
                    out.push(text.to_string());
                }
            }
        }
        for i in 0..node.named_child_count() {
            if let Some(child) = node.named_child(i) {
                Self::collect_vi_calls(child, source, out);
            }
        }
    }

    fn extract_vi_mock(node: Node, source: &str, scope: MockScope) -> Option<ViMockCall> {
        let args = node.child_by_field_name("arguments")?;
        if args.named_child_count() == 0 {
            return None;
        }
        let first = args.named_child(0)?;
        // Handle vi.mock("path"), vi.mock(`path`), and vi.mock(import("path"))
        let src = Self::string_value(first, source).or_else(|| {
            // Check for import("path") call expression.
            if first.kind() == "call_expression" {
                if let Some(func) = first.child_by_field_name("function") {
                    if func.kind() == "import" && first.child_by_field_name("arguments").is_some() {
                        let import_args = first.child_by_field_name("arguments")?;
                        let import_first = import_args.named_child(0)?;
                        return Self::string_value(import_first, source);
                    }
                }
            }
            None
        })?;
        Some(ViMockCall {
            source: src,
            line: node.start_position().row + 1,
            scope,
        })
    }

    fn parse_import(node: Node, source: &str) -> Option<ImportEntry> {
        // tree-sitter-typescript: import_statement has a `source` field
        // (string literal). The clause is one of: identifier, namespace_import,
        // named_imports — we walk the named children to find them.
        let mut entry = ImportEntry {
            source: String::new(),
            named: Vec::new(),
            default: None,
            namespace: None,
            line: node.start_position().row + 1,
        };

        for i in 0..node.named_child_count() {
            let child = node.named_child(i)?;
            match child.kind() {
                "string" => {
                    let raw = child.utf8_text(source.as_bytes()).unwrap_or("");
                    entry.source = raw
                        .trim_matches(|c: char| c == '"' || c == '\'' || c == '`')
                        .to_string();
                }
                "import_clause" => {
                    Self::walk_import_clause(child, source, &mut entry);
                }
                _ => {}
            }
        }

        if entry.source.is_empty() {
            return None;
        }
        Some(entry)
    }

    fn walk_import_clause(node: Node, source: &str, entry: &mut ImportEntry) {
        for i in 0..node.named_child_count() {
            let Some(child) = node.named_child(i) else {
                continue;
            };
            match child.kind() {
                "identifier" => {
                    let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    if !name.is_empty() {
                        entry.default = Some(name);
                    }
                }
                "namespace_import" => {
                    for j in 0..child.named_child_count() {
                        if let Some(inner) = child.named_child(j) {
                            if inner.kind() == "identifier" {
                                entry.namespace = Some(
                                    inner.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                                );
                            }
                        }
                    }
                }
                "named_imports" => {
                    for j in 0..child.named_child_count() {
                        if let Some(spec) = child.named_child(j) {
                            if spec.kind() == "import_specifier" {
                                // import_specifier has `name` and optional `alias`.
                                let name = spec
                                    .child_by_field_name("name")
                                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                    .unwrap_or("");
                                if !name.is_empty() {
                                    entry.named.push(name.to_string());
                                } else {
                                    // Fallback: first identifier child.
                                    for k in 0..spec.named_child_count() {
                                        if let Some(c) = spec.named_child(k) {
                                            if c.kind() == "identifier" {
                                                let n = c
                                                    .utf8_text(source.as_bytes())
                                                    .unwrap_or("")
                                                    .to_string();
                                                if !n.is_empty() {
                                                    entry.named.push(n);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_callee(node: Node, source: &str) -> (String, bool, bool) {
        match node.kind() {
            "identifier" => {
                let name = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                (name, false, false)
            }
            "member_expression" => {
                let full = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let is_skip = full.contains(".skip") || full.contains(".todo");
                let is_only = full.contains(".only");
                let base = full.split('.').next().unwrap_or("").to_string();
                (base, is_skip, is_only)
            }
            _ => (String::new(), false, false),
        }
    }

    fn callback_body(call_node: Node) -> Option<Node> {
        let args = call_node.child_by_field_name("arguments")?;
        if args.named_child_count() < 2 {
            return None;
        }
        let callback = args.named_child(1)?;
        Self::func_body(callback)
    }

    fn single_callback_body(call_node: Node) -> Option<Node> {
        let args = call_node.child_by_field_name("arguments")?;
        if args.named_child_count() == 0 {
            return None;
        }
        let callback = args.named_child(0)?;
        Self::func_body(callback)
    }

    fn func_body(func_node: Node) -> Option<Node> {
        if func_node.kind() != "arrow_function" && func_node.kind() != "function_expression" {
            return None;
        }
        for i in 0..func_node.named_child_count() {
            let child = func_node.named_child(i).unwrap();
            if child.kind() == "statement_block" {
                return Some(child);
            }
        }
        func_node.child_by_field_name("body")
    }

    fn extract_test(
        node: Node,
        source: &str,
        path: &Path,
        describe_depth: usize,
        is_skip: bool,
        is_only: bool,
    ) -> Option<TestBlock> {
        let args = node.child_by_field_name("arguments")?;
        if args.named_child_count() < 1 {
            return None;
        }

        let name_node = args.named_child(0)?;
        let name = Self::string_value(name_node, source)?;

        let body = if args.named_child_count() >= 2 {
            let cb = args.named_child(1)?;
            Self::func_body(cb)
        } else {
            None
        };

        let st = body.map_or_else(Analysis::default, |b| Self::analyze(b, source));

        Some(TestBlock {
            name,
            file_path: path.to_path_buf(),
            line: node.start_position().row + 1,
            has_assertions: st.assertion_count > 0,
            assertion_count: st.assertion_count,
            has_conditional_logic: st.has_conditional,
            has_try_catch: st.has_try_catch,
            uses_settimeout: st.uses_settimeout,
            uses_datemock: st.uses_datemock,
            has_multiple_expects: st.assertion_count > 1,
            is_skipped: is_skip,
            is_only,
            is_nested: describe_depth > 3,
            has_return_statement: st.has_return,
            unawaited_async_assertions: st.unawaited_async_assertions,
            uses_fake_timers: st.uses_fake_timers,
        })
    }

    fn string_value(node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "string" => {
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                Some(
                    text.trim_matches(|c| c == '"' || c == '\'' || c == '`')
                        .to_string(),
                )
            }
            "template_string" => {
                // Reject templates with interpolations (${...}).
                for i in 0..node.named_child_count() {
                    if let Some(child) = node.named_child(i) {
                        if child.kind() == "template_substitution" {
                            return None;
                        }
                    }
                }
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                Some(text.trim_matches('`').to_string())
            }
            _ => None,
        }
    }

    fn is_awaited(node: Node) -> bool {
        let mut curr = node;
        while let Some(parent) = curr.parent() {
            if parent.kind() == "await_expression" {
                return true;
            }
            if parent.kind() == "expression_statement"
                || parent.kind() == "lexical_declaration"
                || parent.kind() == "variable_declaration"
                || parent.kind() == "statement_block"
            {
                break;
            }
            curr = parent;
        }
        false
    }

    fn analyze(node: Node, source: &str) -> Analysis {
        let mut st = Analysis::default();
        Self::walk_body(node, source, &mut st);
        st
    }

    fn walk_body(node: Node, source: &str, st: &mut Analysis) {
        match node.kind() {
            "call_expression" => {
                let func = node.child_by_field_name("function").unwrap();
                let text = func.utf8_text(source.as_bytes()).unwrap_or("");
                if text.starts_with("expect") {
                    st.assertion_count += 1;
                    if (text.contains(".resolves") || text.contains(".rejects"))
                        && !Self::is_awaited(node)
                    {
                        st.unawaited_async_assertions += 1;
                    }
                }
                if text == "setTimeout" {
                    st.uses_settimeout = true;
                }
                if text.starts_with("Date.") {
                    st.uses_datemock = true;
                }
                if text == "vi.useFakeTimers" {
                    st.uses_fake_timers = true;
                }
                let args = node.child_by_field_name("arguments").unwrap();
                for i in 0..args.named_child_count() {
                    let child = args.named_child(i).unwrap();
                    Self::walk_body(child, source, st);
                }
                return;
            }
            "new_expression" => {
                let ctor = node.child_by_field_name("constructor").unwrap();
                if ctor.utf8_text(source.as_bytes()).unwrap_or("") == "Date" {
                    st.uses_datemock = true;
                }
            }
            "if_statement" | "switch_statement" => {
                st.has_conditional = true;
            }
            "try_statement" => {
                st.has_try_catch = true;
            }
            "return_statement" => {
                st.has_return = true;
            }
            _ => {}
        }

        for i in 0..node.named_child_count() {
            let child = node.named_child(i).unwrap();
            Self::walk_body(child, source, st);
        }
    }
}

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
struct Analysis {
    assertion_count: usize,
    has_conditional: bool,
    has_try_catch: bool,
    uses_settimeout: bool,
    uses_datemock: bool,
    has_return: bool,
    unawaited_async_assertions: usize,
    uses_fake_timers: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp(content: &str, name: &str) -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        dir
    }

    #[test]
    fn parse_simple_test_file() {
        let dir = write_temp(
            r#"
import { test, expect } from 'vitest';

test('adds numbers', () => {
    expect(1 + 1).toBe(2);
});
"#,
            "simple.test.ts",
        );
        let path = dir.path().join("simple.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert_eq!(module.test_blocks[0].name, "adds numbers");
        assert!(module.test_blocks[0].has_assertions);
        assert_eq!(module.test_blocks[0].assertion_count, 1);
        assert!(!module.test_blocks[0].is_skipped);
        assert!(!module.test_blocks[0].is_nested);
    }

    #[test]
    fn parse_detects_fake_timers() {
        let dir = write_temp(
            r#"
import { test, expect, vi } from 'vitest';

test('with fake timers', () => {
    vi.useFakeTimers();
    expect(true).toBe(true);
});
"#,
            "fake.test.ts",
        );
        let path = dir.path().join("fake.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert!(module.has_fake_timers);
    }

    #[test]
    fn parse_skipped_test() {
        let dir = write_temp(
            r#"
import { test, expect } from 'vitest';

test.skip('skipped', () => {
    expect(1).toBe(1);
});
"#,
            "skip.test.ts",
        );
        let path = dir.path().join("skip.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert!(module.test_blocks[0].is_skipped);
    }

    #[test]
    fn parse_nested_describe() {
        let dir = write_temp(
            r#"
import { describe, test, expect } from 'vitest';

describe('level1', () => {
    describe('level2', () => {
        describe('level3', () => {
            describe('level4', () => {
                test('deeply nested', () => {
                    expect(1).toBe(1);
                });
            });
        });
    });
});
"#,
            "nested.test.ts",
        );
        let path = dir.path().join("nested.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert!(module.test_blocks[0].is_nested);
    }

    #[test]
    fn parse_imports() {
        let dir = write_temp(
            r#"
import { test, expect } from 'vitest';
import axios from 'axios';
"#,
            "imports.test.ts",
        );
        let path = dir.path().join("imports.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert!(module.imports.iter().any(|i| i.contains("axios")));
        assert!(module.imports.iter().any(|i| i.contains("vitest")));
    }

    #[test]
    fn parse_describe_with_extra_args() {
        let dir = write_temp(
            r#"
import { describe, test, expect } from 'vitest';

describe('with extra', () => {
    test('inside', () => {
        expect(1).toBe(1);
    });
}, extraConfig);
"#,
            "extra.test.ts",
        );
        let path = dir.path().join("extra.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert_eq!(module.test_blocks[0].name, "inside");
        assert!(module.test_blocks[0].has_assertions);
    }

    #[test]
    fn parse_test_name_only_no_callback() {
        let dir = write_temp(
            r#"
import { test } from 'vitest';

test('name only');
"#,
            "nameonly.test.ts",
        );
        let path = dir.path().join("nameonly.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert_eq!(module.test_blocks[0].name, "name only");
        assert!(!module.test_blocks[0].has_assertions);
    }

    #[test]
    fn parse_test_with_function_expression() {
        let dir = write_temp(
            r#"
import { test, expect } from 'vitest';

test('function expr', function() {
    expect(1).toBe(1);
});
"#,
            "funcexpr.test.ts",
        );
        let path = dir.path().join("funcexpr.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert!(module.test_blocks[0].has_assertions);
        assert_eq!(module.test_blocks[0].assertion_count, 1);
    }

    #[test]
    fn parse_single_describe_not_nested() {
        let dir = write_temp(
            r#"
import { describe, test, expect } from 'vitest';

describe('only one level', () => {
    test('not nested', () => {
        expect(1).toBe(1);
    });
});
"#,
            "single.test.ts",
        );
        let path = dir.path().join("single.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert!(!module.test_blocks[0].is_nested);
    }

    #[test]
    fn parse_line_number_correct() {
        let dir = write_temp(
            r#"import { test, expect } from 'vitest';

test('line check', () => {
    expect(1).toBe(1);
});"#,
            "line.test.ts",
        );
        let path = dir.path().join("line.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert_eq!(module.test_blocks[0].line, 3);
    }

    #[test]
    fn parse_single_assertion_not_multiple() {
        let dir = write_temp(
            r#"
import { test, expect } from 'vitest';

test('one assert', () => {
    expect(1).toBe(1);
});
"#,
            "one.test.ts",
        );
        let path = dir.path().join("one.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert_eq!(module.test_blocks[0].assertion_count, 1);
        assert!(!module.test_blocks[0].has_multiple_expects);
    }

    #[test]
    fn parse_two_assertions_is_multiple() {
        let dir = write_temp(
            r#"
import { test, expect } from 'vitest';

test('two asserts', () => {
    expect(1).toBe(1);
    expect(2).toBe(2);
});
"#,
            "two.test.ts",
        );
        let path = dir.path().join("two.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert_eq!(module.test_blocks[0].assertion_count, 2);
        assert!(module.test_blocks[0].has_multiple_expects);
    }

    #[test]
    fn parse_tsx_file_with_jsx() {
        let dir = write_temp(
            r#"
import { render, screen } from '@testing-library/react';
import { test, expect } from 'vitest';
import MyComponent from './MyComponent';

test('renders label', () => {
    render(<MyComponent />);
    expect(screen.getByText('hello')).toBeTruthy();
});
"#,
            "component.test.tsx",
        );
        let path = dir.path().join("component.test.tsx");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert_eq!(module.test_blocks[0].name, "renders label");
        assert!(module.test_blocks[0].has_assertions);
    }

    #[test]
    fn parse_deeply_nested_describe_with_extra_args() {
        let dir = write_temp(
            r#"
import { describe, test, expect } from 'vitest';

describe('level1', () => {
    describe('level2', () => {
        describe('level3', () => {
            describe('level4', () => {
                test('deep', () => {
                    expect(1).toBe(1);
                });
            });
        });
    });
}, config);
"#,
            "deep.test.ts",
        );
        let path = dir.path().join("deep.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.test_blocks.len(), 1);
        assert!(
            module.test_blocks[0].is_nested,
            "test inside 4-level nested describe should be is_nested"
        );
        assert!(module.test_blocks[0].has_assertions);
    }

    #[test]
    fn parse_vi_mock_module_scope() {
        let dir = write_temp(
            r#"
import { vi } from 'vitest';

vi.mock('../infrastructure/database', () => ({ db: {} }));
"#,
            "mock.test.ts",
        );
        let path = dir.path().join("mock.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.vi_mocks.len(), 1);
        assert_eq!(module.vi_mocks[0].source, "../infrastructure/database");
        assert_eq!(module.vi_mocks[0].scope, MockScope::Module);
    }

    #[test]
    fn parse_imports_structured_named_default_namespace() {
        let dir = write_temp(
            r#"
import { test, expect } from 'vitest';
import axios from 'axios';
import * as fs from 'fs';
import { progressPersistence } from './progress-persistence';
"#,
            "structured.test.ts",
        );
        let path = dir.path().join("structured.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        let vitest = module
            .imports_parsed
            .iter()
            .find(|e| e.source == "vitest")
            .unwrap();
        assert!(vitest.named.contains(&"test".to_string()));
        assert!(vitest.named.contains(&"expect".to_string()));

        let axios = module
            .imports_parsed
            .iter()
            .find(|e| e.source == "axios")
            .unwrap();
        assert_eq!(axios.default.as_deref(), Some("axios"));

        let fs_imp = module
            .imports_parsed
            .iter()
            .find(|e| e.source == "fs")
            .unwrap();
        assert_eq!(fs_imp.namespace.as_deref(), Some("fs"));

        let pp = module
            .imports_parsed
            .iter()
            .find(|e| e.source == "./progress-persistence")
            .unwrap();
        assert!(pp.named.contains(&"progressPersistence".to_string()));
    }

    #[test]
    fn parse_hook_calls_capture_vi_methods() {
        let dir = write_temp(
            r#"
import { beforeEach, afterEach, vi } from 'vitest';

beforeEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
});

afterEach(() => {
    vi.clearAllMocks();
});
"#,
            "hooks.test.ts",
        );
        let path = dir.path().join("hooks.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.hook_calls.len(), 2);
        let before = module
            .hook_calls
            .iter()
            .find(|h| h.kind == HookKind::BeforeEach)
            .unwrap();
        assert!(before.vi_calls.iter().any(|c| c == "vi.resetModules"));
        assert!(before.vi_calls.iter().any(|c| c == "vi.restoreAllMocks"));
        let after = module
            .hook_calls
            .iter()
            .find(|h| h.kind == HookKind::AfterEach)
            .unwrap();
        assert!(after.vi_calls.iter().any(|c| c == "vi.clearAllMocks"));
    }

    #[test]
    fn parse_vi_mock_dynamic_import() {
        let dir = write_temp(
            r#"
import { vi } from 'vitest';

vi.mock(import('../infrastructure/database'));
"#,
            "dynmock.test.ts",
        );
        let path = dir.path().join("dynmock.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.vi_mocks.len(), 1);
        assert_eq!(module.vi_mocks[0].source, "../infrastructure/database");
        assert_eq!(module.vi_mocks[0].scope, MockScope::Module);
    }

    #[test]
    fn parse_vi_mock_template_interpolation_ignored() {
        let dir = write_temp(
            r#"
import { vi } from 'vitest';

vi.mock(`../${name}`);
"#,
            "interp.test.ts",
        );
        let path = dir.path().join("interp.test.ts");
        let parser = TsParser::new().unwrap();
        let module = parser.parse_file(&path).unwrap();

        assert_eq!(module.vi_mocks.len(), 0);
    }
}
