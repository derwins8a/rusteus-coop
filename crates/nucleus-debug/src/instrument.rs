//! Source-level JavaScript AST instrumentation for breakpoints, stepping, and scope inspection.

use boa_parser::{Parser, Source};
use nucleus_common::error::{NucleusError, NucleusResult};

/// Represents the instrumented script source code and line metadata.
#[derive(Debug, Clone)]
pub struct InstrumentedScript {
    /// File identifier (e.g., "handler.js").
    pub file_id: String,
    /// Fully instrumented JavaScript source code containing debug probes.
    pub instrumented_source: String,
    /// Original line count.
    pub original_line_count: usize,
    /// Lines that have active debug probes installed.
    pub instrumented_lines: Vec<u32>,
}

/// Instruments handler JavaScript source code with line/col pause probes.
pub struct JsInstrumentor;

impl JsInstrumentor {
    /// Checks whether a given raw JavaScript source is syntactically valid using Boa's parser.
    pub fn validate_syntax(source_code: &str) -> NucleusResult<()> {
        let mut interner = boa_interner::Interner::default();
        let source = Source::from_bytes(source_code.as_bytes());
        let mut parser = Parser::new(source);
        let scope = boa_ast::scope::Scope::new_global();

        parser.parse_script(&scope, &mut interner).map_err(|e| {
            NucleusError::Script(format!("JavaScript syntax validation error: {e}"))
        })?;

        Ok(())
    }

    /// Instruments `source_code` by injecting `__nucleus_dbg_stmt` probes at executable statement boundaries.
    /// Preserves exact 1:1 line numbers between original and instrumented source.
    pub fn instrument(file_id: &str, source_code: &str) -> NucleusResult<InstrumentedScript> {
        // Validate original source syntax first
        Self::validate_syntax(source_code)?;

        let mut instrumented_lines = Vec::new();
        let mut in_block_comment = false;
        let mut output_lines = Vec::new();

        let lines: Vec<&str> = source_code.lines().collect();
        let original_line_count = lines.len();

        for (idx, line) in lines.iter().enumerate() {
            let line_no = (idx + 1) as u32;
            let trimmed = line.trim();

            // Track multi-line comments
            if in_block_comment {
                if let Some(end_idx) = trimmed.find("*/") {
                    in_block_comment = false;
                    let remaining = trimmed[end_idx + 2..].trim();
                    if remaining.is_empty() {
                        output_lines.push(line.to_string());
                        continue;
                    }
                } else {
                    output_lines.push(line.to_string());
                    continue;
                }
            }

            if trimmed.starts_with("/*") && !trimmed.contains("*/") {
                in_block_comment = true;
                output_lines.push(line.to_string());
                continue;
            }

            // Determine if the line is a candidate for statement instrumentation
            if Self::is_statement_candidate(trimmed) {
                let leading_ws_count = line.chars().take_while(|c| c.is_whitespace()).count();
                let col = (leading_ws_count + 1) as u32;

                // Inject probe inline: preserves original line number exactly
                let probe = format!(
                    "/*#*/if (typeof __nucleus_dbg_stmt === 'function') __nucleus_dbg_stmt(\"{file_id}\", {line_no}, {col}, (__e) => eval(__e)); "
                );

                let indent = &line[..leading_ws_count];
                let rest = &line[leading_ws_count..];
                let instrumented_line = format!("{indent}{probe}{rest}");

                output_lines.push(instrumented_line);
                instrumented_lines.push(line_no);
            } else {
                output_lines.push(line.to_string());
            }
        }

        let instrumented_source = output_lines.join("\n");

        // Verify that the instrumented code remains 100% syntactically valid in Boa
        if let Err(e) = Self::validate_syntax(&instrumented_source) {
            tracing::warn!(
                "Instrumented source failed syntax check ({e}); falling back to original source without probes"
            );
            return Ok(InstrumentedScript {
                file_id: file_id.to_string(),
                instrumented_source: source_code.to_string(),
                original_line_count,
                instrumented_lines: Vec::new(),
            });
        }

        Ok(InstrumentedScript {
            file_id: file_id.to_string(),
            instrumented_source,
            original_line_count,
            instrumented_lines,
        })
    }

    /// Determines if a trimmed line is an executable statement start.
    fn is_statement_candidate(trimmed: &str) -> bool {
        if trimmed.is_empty() {
            return false;
        }

        // Comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            return false;
        }

        // Structural blocks, continuation lines, or lone punctuation
        if trimmed == "{"
            || trimmed == "}"
            || trimmed == "};"
            || trimmed == "]"
            || trimmed == "];"
            || trimmed == ")"
            || trimmed == ");"
        {
            return false;
        }

        // Control flow branches that must follow an if/try directly
        if trimmed.starts_with('}')
            || trimmed.starts_with("else ")
            || trimmed.starts_with("else{")
            || trimmed.starts_with("catch ")
            || trimmed.starts_with("catch(")
            || trimmed.starts_with("finally ")
            || trimmed.starts_with("finally{")
            || trimmed.starts_with("case ")
            || trimmed.starts_with("default:")
            || trimmed.starts_with('.')
            || trimmed.starts_with(',')
            || trimmed.starts_with('?')
            || trimmed.starts_with(':')
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_validation() {
        let valid_js = "var a = 10; function test() { return a + 5; }";
        assert!(JsInstrumentor::validate_syntax(valid_js).is_ok());

        let invalid_js = "var a = ; function ( {";
        assert!(JsInstrumentor::validate_syntax(invalid_js).is_err());
    }

    #[test]
    fn test_instrument_handler_script() {
        let script = r#"// Left 4 Dead 2 handler sample
Game.ExecutableName = "left4dead2.exe";
Game.KillMutex = ["hl2_singleton_mutex"];

Game.Play = function() {
    var p = Context.PlayerIndex;
    Context.Run();
};
"#;
        let instrumented = JsInstrumentor::instrument("handler.js", script).unwrap();
        assert_eq!(instrumented.file_id, "handler.js");
        assert!(instrumented.instrumented_lines.len() >= 4);

        // Verify 1:1 line count preservation
        assert_eq!(
            instrumented.instrumented_source.lines().count(),
            script.lines().count()
        );

        // Verify syntax validity of the instrumented output
        assert!(JsInstrumentor::validate_syntax(&instrumented.instrumented_source).is_ok());
    }
}
