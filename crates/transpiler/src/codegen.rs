use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("cannot dedent below zero indentation")]
    InvalidDedent,
    #[error("template '{0}' was not found")]
    TemplateNotFound(String),
}

/// Code generation core that manages output buffering, indentation, and templates.
#[derive(Debug, Clone)]
pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    indent_unit: String,
    at_line_start: bool,
    templates: HashMap<String, String>,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    /// Create a code generator with a 4-space indentation style.
    pub fn new() -> Self {
        let mut generator = Self {
            output: String::new(),
            indent_level: 0,
            indent_unit: "    ".to_string(),
            at_line_start: true,
            templates: HashMap::new(),
        };
        generator.register_default_templates();
        generator
    }

    /// Create a code generator with a custom indentation unit (e.g., "\t").
    pub fn with_indent_unit(indent_unit: impl Into<String>) -> Self {
        let mut generator = Self {
            indent_unit: indent_unit.into(),
            ..Self::new()
        };
        generator.register_default_templates();
        generator
    }

    /// Return generated source as a string slice.
    pub fn as_str(&self) -> &str {
        &self.output
    }

    /// Consume this generator and return the final generated source.
    pub fn into_string(self) -> String {
        self.output
    }

    /// Clear generated output and reset indentation state.
    pub fn clear(&mut self) {
        self.output.clear();
        self.indent_level = 0;
        self.at_line_start = true;
    }

    /// Increase indentation level by one.
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level by one.
    pub fn dedent(&mut self) -> Result<(), CodegenError> {
        if self.indent_level == 0 {
            return Err(CodegenError::InvalidDedent);
        }
        self.indent_level -= 1;
        Ok(())
    }

    /// Emit raw text into the output buffer while respecting indentation.
    pub fn emit(&mut self, text: &str) {
        for ch in text.chars() {
            if self.at_line_start && ch != '\n' {
                self.write_current_indent();
            }

            self.output.push(ch);
            self.at_line_start = ch == '\n';
        }
    }

    /// Emit one line and append a trailing newline.
    pub fn emit_line(&mut self, line: &str) {
        self.emit(line);
        self.emit("\n");
    }

    /// Emit a blank line.
    pub fn emit_empty_line(&mut self) {
        self.emit("\n");
    }

    /// Emit an opening Rust block and increase indentation.
    pub fn open_block(&mut self, header: &str) {
        self.emit_line(&format!("{} {{", header));
        self.indent();
    }

    /// Emit a closing Rust block and decrease indentation.
    pub fn close_block(&mut self) -> Result<(), CodegenError> {
        self.dedent()?;
        self.emit_line("}");
        Ok(())
    }

    /// Register or overwrite a named template.
    pub fn register_template(&mut self, name: impl Into<String>, content: impl Into<String>) {
        self.templates.insert(name.into(), content.into());
    }

    /// Render a registered template by replacing `{{key}}` placeholders.
    pub fn render_template(
        &self,
        name: &str,
        values: &HashMap<String, String>,
    ) -> Result<String, CodegenError> {
        let template = self
            .templates
            .get(name)
            .ok_or_else(|| CodegenError::TemplateNotFound(name.to_string()))?;

        let mut rendered = template.clone();
        for (key, value) in values {
            let marker = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&marker, value);
        }

        Ok(rendered)
    }

    /// Render a template and append it to output.
    pub fn emit_template(
        &mut self,
        name: &str,
        values: &HashMap<String, String>,
    ) -> Result<(), CodegenError> {
        let rendered = self.render_template(name, values)?;
        self.emit(&rendered);
        Ok(())
    }

    fn write_current_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str(&self.indent_unit);
        }
        self.at_line_start = false;
    }

    fn register_default_templates(&mut self) {
        self.templates.insert(
            "function_signature".to_string(),
            "fn {{name}}({{params}}) -> {{return_type}}".to_string(),
        );
        self.templates.insert(
            "let_binding".to_string(),
            "let {{name}} = {{value}};".to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{CodeGenerator, CodegenError};
    use std::collections::HashMap;

    #[test]
    fn emits_basic_lines() {
        let mut generator = CodeGenerator::new();
        generator.emit_line("fn main() {");
        generator.emit_line("}");

        assert_eq!(generator.as_str(), "fn main() {\n}\n");
    }

    #[test]
    fn manages_indentation() {
        let mut generator = CodeGenerator::new();
        generator.emit_line("fn main() {");
        generator.indent();
        generator.emit_line("let x = 1;");
        generator.dedent().expect("dedent should work");
        generator.emit_line("}");

        assert_eq!(generator.as_str(), "fn main() {\n    let x = 1;\n}\n");
    }

    #[test]
    fn block_helpers_work() {
        let mut generator = CodeGenerator::new();
        generator.open_block("fn add(a: i64, b: i64) -> i64");
        generator.emit_line("a + b");
        generator.close_block().expect("close block should work");

        assert_eq!(
            generator.as_str(),
            "fn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n"
        );
    }

    #[test]
    fn dedent_below_zero_errors() {
        let mut generator = CodeGenerator::new();
        let result = generator.dedent();
        assert!(matches!(result, Err(CodegenError::InvalidDedent)));
    }

    #[test]
    fn templates_render() {
        let mut generator = CodeGenerator::new();
        let mut values = HashMap::new();
        values.insert("name".to_string(), "value".to_string());
        values.insert("value".to_string(), "42".to_string());

        generator
            .emit_template("let_binding", &values)
            .expect("template should render");

        assert_eq!(generator.as_str(), "let value = 42;");
    }

    #[test]
    fn missing_template_errors() {
        let generator = CodeGenerator::new();
        let values = HashMap::new();
        let result = generator.render_template("unknown", &values);

        assert!(matches!(
            result,
            Err(CodegenError::TemplateNotFound(name)) if name == "unknown"
        ));
    }

    #[test]
    fn supports_custom_indent_unit() {
        let mut generator = CodeGenerator::with_indent_unit("\t");
        generator.open_block("fn main()");
        generator.emit_line("let x = 1;");
        generator.close_block().expect("close block should work");

        assert_eq!(generator.as_str(), "fn main() {\n\tlet x = 1;\n}\n");
    }

    #[test]
    fn clear_resets_state() {
        let mut generator = CodeGenerator::new();
        generator.open_block("fn main()");
        generator.emit_line("let x = 1;");
        generator.close_block().expect("close block should work");

        generator.clear();
        generator.emit_line("let y = 2;");

        assert_eq!(generator.as_str(), "let y = 2;\n");
    }
}
