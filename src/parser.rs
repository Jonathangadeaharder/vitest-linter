use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::models::{ParsedModule, TestBlock};

pub struct TsParser;

impl TsParser {
    #[allow(clippy::missing_errors_doc)]
    pub const fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn parse_file(&self, path: &Path) -> anyhow::Result<ParsedModule> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;

        let source = std::fs::read_to_string(path)?;
        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse file: {}", path.display()))?;

        let root = tree.root_node();
        let mut imports = Vec::new();
        let mut test_blocks = Vec::new();

        Self::collect(root, &source, path, 0, &mut imports, &mut test_blocks);

        let has_fake_timers = source.contains("useFakeTimers");

        Ok(ParsedModule {
            file_path: path.to_path_buf(),
            imports,
            test_blocks,
            has_fake_timers,
        })
    }

    fn collect(
        node: Node,
        source: &str,
        path: &Path,
        describe_depth: usize,
        imports: &mut Vec<String>,
        test_blocks: &mut Vec<TestBlock>,
    ) {
        for i in 0..node.named_child_count() {
            let Some(child) = node.named_child(i) else {
                continue;
            };
            match child.kind() {
                "import_statement" => {
                    let text = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    imports.push(text);
                }
                "call_expression" => {
                    Self::handle_call(child, source, path, describe_depth, imports, test_blocks);
                }
                _ => {
                    Self::collect(child, source, path, describe_depth, imports, test_blocks);
                }
            }
        }
    }

    fn handle_call(
        node: Node,
        source: &str,
        path: &Path,
        describe_depth: usize,
        imports: &mut Vec<String>,
        test_blocks: &mut Vec<TestBlock>,
    ) {
        let Some(func_node) = node.child_by_field_name("function") else {
            Self::collect(node, source, path, describe_depth, imports, test_blocks);
            return;
        };

        let (func_name, is_skip) = Self::parse_callee(func_node, source);

        match func_name.as_str() {
            "test" | "it" => {
                if let Some(tb) = Self::extract_test(node, source, path, describe_depth, is_skip) {
                    test_blocks.push(tb);
                }
            }
            "describe" => {
                if let Some(body) = Self::callback_body(node) {
                    Self::collect(body, source, path, describe_depth + 1, imports, test_blocks);
                } else {
                    Self::collect(node, source, path, describe_depth, imports, test_blocks);
                }
            }
            _ => {
                Self::collect(node, source, path, describe_depth, imports, test_blocks);
            }
        }
    }

    fn parse_callee(node: Node, source: &str) -> (String, bool) {
        match node.kind() {
            "identifier" => {
                let name = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                (name, false)
            }
            "member_expression" => {
                let full = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let is_skip = full.contains(".skip") || full.contains(".todo");
                let base = full.split('.').next().unwrap_or("").to_string();
                (base, is_skip)
            }
            _ => (String::new(), false),
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
            is_nested: describe_depth > 1,
            has_return_statement: st.has_return,
        })
    }

    fn string_value(node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "string" | "template_string" => {
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                Some(
                    text.trim_matches(|c| c == '"' || c == '\'' || c == '`')
                        .to_string(),
                )
            }
            _ => None,
        }
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
                }
                if text == "setTimeout" {
                    st.uses_settimeout = true;
                }
                if text.starts_with("Date.") {
                    st.uses_datemock = true;
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

describe('outer', () => {
    describe('inner', () => {
        test('nested', () => {
            expect(1).toBe(1);
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
    fn parse_deeply_nested_describe_with_extra_args() {
        let dir = write_temp(
            r#"
import { describe, test, expect } from 'vitest';

describe('outer', () => {
    describe('inner', () => {
        test('deep', () => {
            expect(1).toBe(1);
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
            "test inside nested describe should be is_nested"
        );
        assert!(module.test_blocks[0].has_assertions);
    }
}
