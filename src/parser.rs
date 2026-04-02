// src/parser.rs

use crate::ast::{
    ArithmeticOp, Backend, ComparisonOp, Expression, FilterCondition, LogicalOp, Statement,
};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }
    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token() {
            Some(Token::Use) => self.parse_use_statement(),
            Some(Token::Summarize) => self.parse_summarize_statement(),

            Some(Token::Regress) => self.parse_regress_statement(),

            Some(Token::Insight) => self.parse_insight_statement(),

            Some(Token::Lemmatize) => self.parse_lemmatize_statement(),
            Some(Token::Filter) => self.parse_filter_statement(),
            Some(Token::Load) => self.parse_load_statement(),
            Some(Token::Save) => self.parse_save_statement(),
            Some(Token::Print) => self.parse_print_statement(),
            Some(Token::Tokenize) => self.parse_tokenize_statement(),
            Some(Token::Doc2Bow) => self.parse_doc2bow_statement(),
            Some(Token::Split) => self.parse_split_statement(),
            Some(Token::Join) => self.parse_join_statement(),
            Some(Token::Replace) => self.parse_replace_statement(),
            Some(Token::Clean) => self.parse_clean_statement(),
            Some(Token::Normalize) => self.parse_normalize_statement(),
            Some(Token::Lowercase) => self.parse_lowercase_statement(),
            Some(Token::Uppercase) => self.parse_uppercase_statement(),
            Some(Token::Stem) => self.parse_stem_statement(),
            Some(Token::Count) => self.parse_count_statement(),
            Some(Token::WordCloud) => self.parse_wordcloud_statement(),
            Some(Token::Association) => self.parse_association_statement(),
            Some(Token::Similarity) => self.parse_similarity_statement(),
            Some(Token::SemanticSearch) => self.parse_semantic_search_statement(),
            Some(Token::Search) => self.parse_search_statement(),
            Some(Token::Compare) => self.parse_compare_statement(),
            Some(Token::Average) => self.parse_average_statement(),
            Some(Token::Check) if self.peek_token() == Some(&Token::Normality) => {
                self.parse_check_normality_statement()
            }
            Some(Token::Concordance) | Some(Token::KWIC) => self.parse_concordance_statement(),
            Some(Token::Tag) => self.parse_tag_statement(),
            Some(Token::Translate) => self.parse_translate_statement(),
            Some(Token::Sentiment) => self.parse_sentiment_statement(),

            Some(Token::TTest) | Some(Token::Anova) | Some(Token::MedianTest) => {
                self.parse_stat_test_statement()
            }
            Some(Token::Identifier(s))
                if ["one", "two", "paired"].contains(&s.to_lowercase().as_str()) =>
            {
                self.parse_stat_test_statement()
            }

            Some(Token::Shapiro) => self.parse_shapiro_statement(),
            Some(Token::Kolmogorov) => self.parse_kolmogorov_statement(),

            Some(Token::Mean) => self.parse_mean_statement(),
            Some(Token::Median) => self.parse_median_statement(),
            Some(Token::Mode) => self.parse_mode_statement(),
            Some(Token::Skewness) => self.parse_skewness_statement(),
            Some(Token::Kurtosis) => self.parse_kurtosis_statement(),

            Some(Token::Import) => self.parse_import_statement(),
            Some(Token::Install) => self.parse_install_statement(),
            Some(Token::CallMethod) => self.parse_call_method_statement(),
            Some(Token::Call) => self.parse_call_statement(),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::For) => self.parse_foreach_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::Break) => { self.advance(); Ok(Statement::Break) }
            Some(Token::Continue) => { self.advance(); Ok(Statement::Continue) }
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::Help) => {
                self.advance();
                Ok(Statement::Help)
            }
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case("openai") => {
                self.parse_openai_statement()
            }
            // --- Error handling ---
            Some(Token::Try) => self.parse_try_catch_statement(),
            // --- Functions ---
            Some(Token::Function) => self.parse_function_def_statement(),
            // `define the function …` — Define token followed by "the function"
            Some(Token::Define) => self.parse_define_or_function_statement(),
            // --- Parallelism ---
            Some(Token::Parallel) => self.parse_parallel_statement(),
            Some(Token::Spawn) => self.parse_spawn_statement(),
            Some(Token::Send) => self.parse_send_to_channel_statement(),
            Some(Token::Receive) => self.parse_recv_from_channel_statement(),
            Some(Token::Create) => self.parse_create_statement(),
            Some(Token::Collect) => self.parse_collect_statement(),
            // --- Memory ---
            Some(Token::Free) => self.parse_free_statement(),
            Some(Token::Memory) => self.parse_memory_statement(),
            // `with memory scope do … end`
            Some(Token::With) => self.parse_with_statement(),
            // --- Database ---
            Some(Token::Connect) => self.parse_db_connect_statement(),
            Some(Token::Query) => self.parse_db_query_statement(),
            Some(Token::Insert) => self.parse_db_insert_statement(),
            Some(Token::Run) => self.parse_db_execute_statement(),
            Some(Token::Close) => self.parse_db_close_statement(),
            // --- Arithmetic ---
            Some(Token::Add) => self.parse_arithmetic_statement(ArithmeticOp::Add),
            Some(Token::Subtract) => self.parse_arithmetic_statement(ArithmeticOp::Subtract),
            Some(Token::Multiply) => self.parse_arithmetic_statement(ArithmeticOp::Multiply),
            Some(Token::Divide) => self.parse_arithmetic_statement(ArithmeticOp::Divide),
            Some(Token::Modulo) => self.parse_arithmetic_statement(ArithmeticOp::Modulo),
            Some(Token::Power) => self.parse_arithmetic_statement(ArithmeticOp::Power),
            Some(Token::Exit) | Some(Token::Quit) => std::process::exit(0),
            Some(_) => {
                let expr = self.parse_expression()?;
                if let Some(tok) = self.current_token() {
                    if !matches!(tok, Token::Eof) {
                        return Err("Unexpected tokens after expression".to_string());
                    }
                }
                Ok(Statement::Expression(expr))
            }
            None => Err("Unexpected end of input".to_string()),
        }
    }

    // Command Parsers with Context-Aware Logic
    fn parse_load_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        if self.current_token() == Some(&Token::CSV) {
            self.advance();
            let path = self.parse_expression()?;
            self.consume(Token::As)?;
            let alias = self.parse_identifier_expression()?;
            Ok(Statement::LoadCsv { path, alias })
        } else {
            let source = self.parse_expression()?;
            self.consume(Token::As)?;
            let alias = self.parse_identifier_expression()?;
            Ok(Statement::Load { source, alias })
        }
    }

    fn parse_tokenize_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Tokenize {
            source,
            destination,
        })
    }

    fn parse_doc2bow_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Doc2Bow {
            source,
            destination,
        })
    }

    fn parse_split_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let unit = self.parse_unit_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Split {
            unit,
            source,
            destination,
        })
    }

    fn parse_join_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let unit = self.parse_unit_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_expression()?;
        self.consume(Token::With)?;
        let separator = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Join {
            unit,
            source,
            separator,
            destination,
        })
    }

    fn parse_replace_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let pattern = self.parse_expression()?;
        self.consume(Token::With)?;
        let replacement = self.parse_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Replace {
            pattern,
            replacement,
            source,
            destination,
        })
    }

    fn parse_clean_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Clean {
            source,
            destination,
        })
    }

    fn parse_lowercase_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Lowercase {
            source,
            destination,
        })
    }

    fn parse_uppercase_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Uppercase {
            source,
            destination,
        })
    }

    fn parse_stem_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Stem {
            source,
            destination,
        })
    }

    fn parse_normalize_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Normalize {
            source,
            destination,
        })
    }

    fn parse_tag_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'tag'
        let source = self.parse_expression()?;
        self.consume(Token::With)?;
        let method = self.parse_unit_expression()?;
        self.consume(Token::As)?;
        // FIX: The destination MUST be a variable name (an identifier).
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Tag {
            source,
            method,
            destination,
        })
    }

    fn parse_translate_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::To)?;
        let language = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Translate {
            source,
            language,
            destination,
        })
    }

    fn parse_sentiment_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Sentiment {
            source,
            destination,
        })
    }

    fn parse_stat_test_statement(&mut self) -> Result<Statement, String> {
        // optional sample descriptor
        let mut sample: Option<String> = None;
        if let Some(Token::Identifier(s)) = self.current_token().cloned() {
            if ["one", "two", "paired"].contains(&s.to_lowercase().as_str()) {
                sample = Some(s.to_lowercase());
                self.advance();
                self.consume(Token::Sample)?;
            }
        }
        match self.current_token() {
            Some(Token::TTest) => {
                self.advance();
                self.consume_identifier("column")?;
                let column = self.parse_identifier_expression()?;
                self.consume_identifier("between")?;
                let group1 = self.parse_identifier_expression()?;
                self.consume_identifier("and")?;
                let group2 = self.parse_identifier_expression()?;
                self.consume(Token::As)?;
                let destination = self.parse_identifier_expression()?;
                Ok(Statement::TTest {
                    sample,
                    column,
                    group1,
                    group2,
                    destination,
                })
            }
            Some(Token::Anova) => {
                self.advance();
                self.consume_identifier("column")?;
                let column = self.parse_identifier_expression()?;
                self.consume_identifier("between")?;
                let group1 = self.parse_identifier_expression()?;
                self.consume_identifier("and")?;
                let group2 = self.parse_identifier_expression()?;
                self.consume(Token::As)?;
                let destination = self.parse_identifier_expression()?;
                Ok(Statement::Anova {
                    column,
                    group1,
                    group2,
                    destination,
                })
            }
            Some(Token::MedianTest) => {
                self.advance();
                self.consume_identifier("column")?;
                let column = self.parse_identifier_expression()?;
                self.consume_identifier("between")?;
                let group1 = self.parse_identifier_expression()?;
                self.consume_identifier("and")?;
                let group2 = self.parse_identifier_expression()?;
                self.consume(Token::As)?;
                let destination = self.parse_identifier_expression()?;
                Ok(Statement::MedianTest {
                    column,
                    group1,
                    group2,
                    destination,
                })
            }
            _ => Err("Expected statistical test".to_string()),
        }
    }

    fn parse_shapiro_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        match self.current_token() {
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case("column") => self.advance(),
            _ => return Err("Expected 'column' after 'shapiro'".to_string()),
        }
        let column = self.parse_identifier_expression()?;
        self.consume(Token::In)?;
        let data = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Shapiro {
            column,
            data,
            destination,
        })
    }

    fn parse_mean_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let column = self.parse_identifier_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Mean {
            column,
            source,

            destination,
        })
    }

    fn parse_kolmogorov_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        match self.current_token() {
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case("column") => self.advance(),
            _ => return Err("Expected 'column' after 'kolmogorov'".to_string()),
        }
        let column = self.parse_identifier_expression()?;
        self.consume(Token::In)?;
        let data = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Kolmogorov {
            column,
            data,
            destination,
        })
    }

    fn parse_median_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let column = self.parse_identifier_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Median {
            column,
            source,
            destination,
        })
    }

    fn parse_mode_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let column = self.parse_identifier_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Mode {
            column,
            source,
            destination,
        })
    }

    fn parse_skewness_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let column = self.parse_identifier_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Skewness {
            column,
            source,
            destination,
        })
    }

    fn parse_kurtosis_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let column = self.parse_identifier_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Kurtosis {
            column,
            source,

            destination,
        })
    }

    fn parse_count_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let unit = self.parse_unit_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Count {
            unit,
            source,
            destination,
        })
    }

    fn parse_wordcloud_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::WordCloud {
            source,
            destination,
        })
    }

    fn parse_association_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::WordAssociation {
            source,
            destination,
        })
    }

    fn parse_similarity_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let text1 = self.parse_expression()?;
        self.consume(Token::To)?;
        let text2 = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::SemanticSimilarity {
            text1,
            text2,
            destination,
        })
    }

    fn parse_semantic_search_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let query = self.parse_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::SemanticSearch {
            query,
            source,
            destination,
        })
    }

    fn parse_search_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let query = self.parse_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::KeywordSearch {
            query,
            source,
            destination,
        })
    }

    fn parse_compare_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let left = self.parse_expression()?;
        self.consume(Token::To)?;
        let right = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Compare {
            left,
            right,
            destination,
        })
    }

    fn parse_average_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Average {
            source,
            destination,
        })
    }

    fn parse_check_normality_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'check'
        self.consume(Token::Normality)?;
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::CheckNormality {
            source,
            destination,
        })
    }

    fn parse_concordance_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let word = self.parse_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Concordance {
            word,
            source,
            destination,
        })
    }

    fn parse_define_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        if let Some(Token::Identifier(s)) = self.current_token() {
            if s.eq_ignore_ascii_case("env") || s.eq_ignore_ascii_case("environment") {
                self.advance();
                if let Some(Token::Identifier(s2)) = self.current_token() {
                    if s2.eq_ignore_ascii_case("variable") {
                        self.advance();
                    }
                }
                let name_expr = self.parse_identifier_expression()?;
                let var_name = if let Expression::Identifier(n) = name_expr {
                    n
                } else {
                    return Err("Expected environment variable name".to_string());
                };
                self.consume(Token::To)?;
                let value = self.parse_expression()?;
                return Ok(Statement::SetEnv {
                    name: var_name,
                    value,
                });
            }
        }
        let name = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let value = self.parse_expression()?;
        Ok(Statement::Define { name, value })
    }

    fn parse_import_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'import'
        let backend = match self.current_token() {
            Some(Token::Python) => Backend::Python,
            Some(Token::R) => Backend::R,
            _ => return Err("Expected 'python' or 'r' after 'import'".to_string()),
        };
        self.advance();
        let module = self.parse_expression()?;
        self.consume(Token::As)?;
        let alias = self.parse_identifier_expression()?;
        Ok(Statement::Import {
            backend,
            module,
            alias,
        })
    }

    fn parse_install_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'install'
        let backend = match self.current_token() {
            Some(Token::Python) => Backend::Python,
            Some(Token::R) => Backend::R,
            _ => return Err("Expected 'python' or 'r' after 'install'".to_string()),
        };
        self.advance();
        let module = self.parse_expression()?;
        Ok(Statement::Install { backend, module })
    }

    fn parse_call_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'call'

        // Peek ahead: if the next identifier is NOT followed by another
        // identifier before an 'as', treat it as a user-function call.
        // Pattern: `call FUNCNAME with arg1, arg2 as dest`
        //           `call FUNCNAME arg1 arg2 as dest`  (library.function form)
        let first = self.parse_identifier_expression()?;

        // Check if this looks like a user-defined function call (single name, then 'with' or 'as')
        let is_user_func = matches!(
            self.current_token(),
            Some(Token::With) | Some(Token::As) | Some(Token::Eof) | None
        );

        if is_user_func {
            let func_name = if let Expression::Identifier(n) = first { n }
                else { return Err("Expected function name".to_string()); };
            // consume optional 'with'
            if self.current_token() == Some(&Token::With) { self.advance(); }
            let mut args = Vec::new();
            while let Some(tok) = self.current_token() {
                if matches!(tok, Token::As | Token::Eof) { break; }
                args.push(self.parse_expression()?);
                if self.current_token() == Some(&Token::Comma) { self.advance(); }
            }
            let destination = if self.current_token() == Some(&Token::As) {
                self.advance();
                Some(self.parse_identifier_expression()?)
            } else { None };
            return Ok(Statement::FunctionCall { name: func_name, args, destination });
        }

        // Otherwise fall through to library.function call
        let library = first;
        let function = self.parse_identifier_expression()?;
        let mut args = Vec::new();
        while let Some(tok) = self.current_token() {
            if matches!(tok, Token::As) {
                break;
            }
            args.push(self.parse_expression()?);
        }
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Call {
            library,
            function,
            args,
            destination,
        })
    }

    fn parse_call_method_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'callmethod'
        let object = self.parse_identifier_expression()?;
        let method = self.parse_identifier_expression()?;
        let mut args = Vec::new();
        while let Some(tok) = self.current_token() {
            if matches!(tok, Token::As) {
                break;
            }
            args.push(self.parse_expression()?);
        }
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::CallMethod {
            object,
            method,
            args,
            destination,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        self.consume(Token::Then)?;
        let mut then_body = Vec::new();
        while let Some(tok) = self.current_token() {
            if matches!(tok, Token::Else | Token::End) {
                break;
            }
            then_body.push(self.parse_statement()?);
        }
        let mut else_body = Vec::new();
        if self.current_token() == Some(&Token::Else) {
            self.advance(); // consume 'else'
            while let Some(tok) = self.current_token() {
                if matches!(tok, Token::End) {
                    break;
                }
                else_body.push(self.parse_statement()?);
            }
        }
        self.consume(Token::End)?;
        Ok(Statement::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_foreach_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'for'
        self.consume(Token::Each)?;
        let item_expr = self.parse_identifier_expression()?;
        let item = if let Expression::Identifier(name) = item_expr {
            name
        } else {
            return Err("Expected loop variable".to_string());
        };
        self.consume(Token::In)?;
        let list = self.parse_expression()?;
        self.consume(Token::Do)?;
        let mut body = Vec::new();
        while !matches!(self.current_token(), Some(Token::End)) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::End)?;
        Ok(Statement::ForEach { item, list, body })
    }

    fn parse_openai_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'openai'
        let prompt = self.parse_expression()?;

        // Optional model specification
        let model = if self.current_token() == Some(&Token::With) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Destination can be a variable (introduced by 'as') and/or a file path
        let mut destination = None;
        let mut file_path = None;
        match self.current_token() {
            Some(Token::As) => {
                self.advance();
                destination = Some(self.parse_identifier_expression()?);
                if self.current_token() == Some(&Token::To) {
                    self.advance();
                    file_path = Some(self.parse_expression()?);
                }
            }
            Some(Token::To) => {
                self.advance();
                file_path = Some(self.parse_expression()?);
            }
            _ => {
                return Err("Expected 'as' or 'to' after OpenAI prompt".to_string());
            }
        }

        Ok(Statement::OpenAI {
            prompt,
            model,
            destination,
            file_path,
        })
    }

    fn parse_arithmetic_statement(&mut self, op: ArithmeticOp) -> Result<Statement, String> {
        self.advance();
        let (target, value) = match op {
            ArithmeticOp::Add | ArithmeticOp::Subtract => {
                let val = self.parse_expression()?;
                if self.current_token() == Some(&Token::To)
                    || self.current_token() == Some(&Token::From)
                {
                    self.advance();
                } else {
                    return Err("Expected 'to' or 'from'".to_string());
                }
                let tar = self.parse_expression()?;
                (tar, val)
            }
            ArithmeticOp::Multiply | ArithmeticOp::Divide => {
                let tar = self.parse_expression()?;
                self.consume(Token::By)?;
                let val = self.parse_expression()?;
                (tar, val)
            }
        };
        let destination = if self.current_token() == Some(&Token::As) {
            self.advance();
            Some(self.parse_identifier_expression()?)
        } else {
            None
        };
        Ok(Statement::Arithmetic {
            op,
            value,
            target,
            destination,
        })
    }

    // Other existing parsers
    fn parse_use_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let backend = match self.current_token() {
            Some(Token::Python) => Backend::Python,
            Some(Token::R) => Backend::R,
            _ => return Err("Expected 'python' or 'r' after 'use'".to_string()),
        };
        self.advance();
        Ok(Statement::Use(backend))
    }
    fn parse_regress_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let target = self.parse_identifier_expression()?;
        self.consume(Token::Using)?;
        let mut features = Vec::new();
        loop {
            features.push(self.parse_identifier_expression()?);
            if let Some(Token::Comma) = self.current_token() {
                self.advance();
            } else {
                break;
            }
        }
        self.consume(Token::In)?;
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        let mut with_openai = false;
        if let Some(Token::With) = self.current_token() {
            self.advance();
            if let Some(Token::Identifier(name)) = self.current_token().cloned() {
                if name.eq_ignore_ascii_case("openai") {
                    self.advance();
                    with_openai = true;
                }
            }
        }
        Ok(Statement::Regress {
            target,
            features,
            source,
            destination,
            with_openai,
        })
    }
    fn parse_summarize_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Summarize {
            source,
            destination,
        })
    }
    fn parse_insight_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::With)?;
        // Expect the identifier 'openai'
        match self.current_token() {
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case("openai") => {
                self.advance();
            }
            _ => return Err("Expected 'openai' after 'using'".to_string()),
        }
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Insight {
            source,
            destination,
        })
    }
    fn parse_lemmatize_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Lemmatize {
            source,
            destination,
        })
    }
    fn parse_filter_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        let condition = match self.current_token() {
            Some(Token::Containing) => {
                self.advance();
                FilterCondition::Containing(self.parse_expression()?)
            }
            Some(Token::StartingWith) => {
                self.advance();
                FilterCondition::StartingWith(self.parse_expression()?)
            }
            Some(Token::EndingWith) => {
                self.advance();
                FilterCondition::EndingWith(self.parse_expression()?)
            }
            _ => return Err("Expected a filter condition".to_string()),
        };
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Filter {
            source,
            condition,
            destination,
        })
    }
    fn parse_save_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::To)?;
        let destination = self.parse_expression()?;
        Ok(Statement::Save {
            source,
            destination,
        })
    }
    fn parse_print_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let expr = self.parse_expression()?;
        Ok(Statement::Print(expr))
    }

    // Expression parsing logic
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;
        while let Some(Token::LBracket) = self.current_token() {
            self.advance();
            let index_expr = self.parse_expression()?;
            self.consume(Token::RBracket)?;
            expr = Expression::Index(Box::new(expr), Box::new(index_expr));
        }
        expr = self.parse_comparison(expr)?;
        Ok(expr)
    }

    fn parse_comparison(&mut self, left: Expression) -> Result<Expression, String> {
        let mut expr = left;

        // optional 'is' connector
        if self.current_token() == Some(&Token::Is) {
            self.advance();
        }
        // optional 'not' for `is not equal`
        let negate = if self.current_token() == Some(&Token::Not) {
            self.advance();
            true
        } else {
            false
        };

        let op = if self.current_token() == Some(&Token::Greater)
            && self.peek_token() == Some(&Token::Than)
        {
            self.advance(); self.advance();
            Some(ComparisonOp::GreaterThan)
        } else if self.current_token() == Some(&Token::Less)
            && self.peek_token() == Some(&Token::Than)
        {
            self.advance(); self.advance();
            Some(ComparisonOp::LessThan)
        } else if self.current_token() == Some(&Token::Equals) {
            self.advance();
            Some(ComparisonOp::Equals)
        } else if self.current_token() == Some(&Token::NotEquals) {
            self.advance();
            Some(ComparisonOp::NotEquals)
        } else {
            None
        };

        if let Some(op) = op {
            let right = self.parse_primary()?;
            let cmp = Expression::Comparison(Box::new(expr), op, Box::new(right));
            expr = if negate { Expression::Not(Box::new(cmp)) } else { cmp };
        }

        // Chain logical `and` / `or`
        loop {
            let logical_op = match self.current_token() {
                Some(Token::And) => { self.advance(); Some(LogicalOp::And) }
                Some(Token::Or)  => { self.advance(); Some(LogicalOp::Or) }
                _ => None,
            };
            match logical_op {
                Some(op) => {
                    let right = self.parse_primary()?;
                    let right = self.parse_comparison(right)?;
                    expr = Expression::Logical(Box::new(expr), op, Box::new(right));
                }
                None => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.current_token().cloned() {
            Some(Token::String(v)) => {
                self.advance();
                Ok(Expression::StringLiteral(v))
            }
            Some(Token::Number(v)) => {
                self.advance();
                Ok(Expression::Number(v))
            }
            // Bool literals
            Some(Token::True) => {
                self.advance();
                Ok(Expression::Bool(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expression::Bool(false))
            }
            // Null literal: `nothing` / `null` / `none`
            Some(Token::Nothing) => {
                self.advance();
                Ok(Expression::Null)
            }
            // `not EXPR`
            Some(Token::Not) => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Expression::Not(Box::new(expr)))
            }
            Some(Token::LBracket) => self.parse_list_literal(),
            Some(_) => self.parse_identifier_expression(),
            None => Err("Expected an expression, but found nothing.".to_string()),
        }
    }

    fn parse_list_literal(&mut self) -> Result<Expression, String> {
        self.consume(Token::LBracket)?;
        let mut elems = Vec::new();
        while let Some(tok) = self.current_token() {
            if matches!(tok, Token::RBracket) {
                break;
            }
            let expr = self.parse_expression()?;
            elems.push(expr);
            if let Some(Token::Comma) = self.current_token() {
                self.advance();
            }
        }
        self.consume(Token::RBracket)?;
        Ok(Expression::ListLiteral(elems))
    }

    // This function now correctly handles keywords by turning them into identifier strings.
    fn parse_identifier_expression(&mut self) -> Result<Expression, String> {
        match self.current_token().cloned() {
            Some(Token::Identifier(name)) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Some(tok) => {
                let identifier_string = format!("{:?}", tok).to_lowercase();
                self.advance();
                Ok(Expression::Identifier(identifier_string))
            }
            None => Err("Expected a variable name, but found nothing.".to_string()),
        }
    }

    // This function specifically looks for unit keywords.
    fn parse_unit_expression(&mut self) -> Result<Expression, String> {
        match self.current_token() {
            Some(tok) if self.is_unit_token(tok) => {
                let unit_token = tok.clone();
                self.advance();
                Ok(Expression::Unit(unit_token))
            }
            _ => Err(format!(
                "Expected a unit (like words, ner, etc), but found {:?}",
                self.current_token()
            )),
        }
    }

    fn is_unit_token(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::Words
                | Token::Sentences
                | Token::Lines
                | Token::Paragraphs
                | Token::Characters
                | Token::Tokens
                | Token::Types
                | Token::Uniques
                | Token::Length
                | Token::POS
                | Token::NER
                | Token::Entities
        )
    }

    fn consume_identifier(&mut self, expected: &str) -> Result<(), String> {
        match self.current_token() {
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case(expected) => {
                self.advance();
                Ok(())
            }
            Some(tok) => Err(format!(
                "Expected '{}' but found {}",
                expected,
                token_to_string(tok)
            )),
            None => Err(format!("Expected '{}' but found end of input", expected)),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: While loop
    // ─────────────────────────────────────────────────────────────────────
    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        self.consume(Token::Do)?;
        let mut body = Vec::new();
        while !matches!(self.current_token(), Some(Token::End) | None) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::End)?;
        Ok(Statement::While { condition, body })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: Return statement
    // ─────────────────────────────────────────────────────────────────────
    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'return'
        // 'return' with no expression → return nothing
        if matches!(self.current_token(), Some(Token::Eof) | Some(Token::End) | None) {
            return Ok(Statement::Return { value: None });
        }
        let value = self.parse_expression()?;
        Ok(Statement::Return { value: Some(value) })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: try … catch error_var … end
    // ─────────────────────────────────────────────────────────────────────
    fn parse_try_catch_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'try'
        let mut try_body = Vec::new();
        while !matches!(self.current_token(), Some(Token::Catch) | None) {
            try_body.push(self.parse_statement()?);
        }
        self.consume(Token::Catch)?;
        // the next token is the error variable name
        let error_var = if let Some(Token::Identifier(name)) = self.current_token() {
            let n = name.clone();
            self.advance();
            n
        } else {
            "error".to_string() // default name if omitted
        };
        let mut catch_body = Vec::new();
        while !matches!(self.current_token(), Some(Token::End) | None) {
            catch_body.push(self.parse_statement()?);
        }
        self.consume(Token::End)?;
        Ok(Statement::TryCatch { try_body, error_var, catch_body })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: function definition
    //   `function greet with inputs name do … end`
    // ─────────────────────────────────────────────────────────────────────
    fn parse_function_def_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'function'
        let name = if let Some(Token::Identifier(n)) = self.current_token() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err("Expected function name after 'function'".to_string());
        };
        let mut params = Vec::new();
        if self.current_token() == Some(&Token::With) || self.current_token() == Some(&Token::Inputs) {
            self.advance(); // consume 'with'
            // consume optional 'inputs'
            if self.current_token() == Some(&Token::Inputs) {
                self.advance();
            }
            while let Some(Token::Identifier(p)) = self.current_token() {
                params.push(p.clone());
                self.advance();
                if self.current_token() == Some(&Token::Comma) { self.advance(); }
                else { break; }
            }
        }
        // consume optional 'do' or 'as'
        if matches!(self.current_token(), Some(Token::Do) | Some(Token::As)) {
            self.advance();
        }
        let mut body = Vec::new();
        while !matches!(self.current_token(), Some(Token::End) | None) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::End)?;
        Ok(Statement::FunctionDef { name, params, body })
    }

    // Dispatches either a variable `define` or a `define the function` form.
    fn parse_define_or_function_statement(&mut self) -> Result<Statement, String> {
        // peek: Define → skip glue words → if we see 'function', parse FunctionDef
        let saved_pos = self.position;
        self.advance(); // skip 'define'
        // skip optional glue: 'the', 'a', identifiers
        while let Some(Token::Identifier(_)) = self.current_token() {
            let word = if let Some(Token::Identifier(w)) = self.current_token() { w.clone() } else { break; };
            if word.eq_ignore_ascii_case("the") || word.eq_ignore_ascii_case("a") {
                self.advance();
            } else {
                break;
            }
        }
        if self.current_token() == Some(&Token::Function) {
            // rewind and let parse_function_def handle it
            self.position = saved_pos;
            self.advance(); // skip 'define'
            // skip glue
            while let Some(Token::Identifier(w)) = self.current_token() {
                if w.eq_ignore_ascii_case("the") || w.eq_ignore_ascii_case("a") { self.advance(); }
                else { break; }
            }
            self.parse_function_def_statement()
        } else {
            // restore and parse normal define
            self.position = saved_pos;
            self.parse_define_statement()
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: parallel map / parallel for
    //   `parallel map each ITEM in LIST using N threads do … end and call it DEST`
    //   `parallel for each ITEM in LIST using N threads do … end`
    // ─────────────────────────────────────────────────────────────────────
    fn parse_parallel_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'parallel'
        match self.current_token() {
            Some(Token::Map) => {
                self.advance(); // consume 'map'
                // optional 'each'
                if self.current_token() == Some(&Token::Each) { self.advance(); }
                let item_expr = self.parse_identifier_expression()?;
                let item = if let Expression::Identifier(n) = item_expr { n }
                    else { return Err("Expected loop variable in parallel map".to_string()); };
                self.consume(Token::In)?;
                let list = self.parse_expression()?;
                let threads = self.parse_optional_threads()?;
                self.consume(Token::Do)?;
                let mut body = Vec::new();
                while !matches!(self.current_token(), Some(Token::End) | None) {
                    body.push(self.parse_statement()?);
                }
                self.consume(Token::End)?;
                // optional `and call it DEST`
                self.skip_glue_words();
                let destination = if self.current_token() == Some(&Token::As) {
                    self.advance();
                    self.parse_identifier_expression()?
                } else {
                    Expression::Identifier("_parallel_result".to_string())
                };
                Ok(Statement::ParallelMap { item, list, body, threads, destination })
            }
            Some(Token::For) => {
                self.advance(); // consume 'for'
                if self.current_token() == Some(&Token::Each) { self.advance(); }
                let item_expr = self.parse_identifier_expression()?;
                let item = if let Expression::Identifier(n) = item_expr { n }
                    else { return Err("Expected loop variable in parallel for".to_string()); };
                self.consume(Token::In)?;
                let list = self.parse_expression()?;
                let threads = self.parse_optional_threads()?;
                self.consume(Token::Do)?;
                let mut body = Vec::new();
                while !matches!(self.current_token(), Some(Token::End) | None) {
                    body.push(self.parse_statement()?);
                }
                self.consume(Token::End)?;
                Ok(Statement::ParallelFor { item, list, body, threads })
            }
            _ => Err("Expected 'map' or 'for' after 'parallel'".to_string()),
        }
    }

    /// Parse optional `using N threads` clause.
    fn parse_optional_threads(&mut self) -> Result<Option<Expression>, String> {
        if matches!(self.current_token(), Some(Token::Using)) {
            self.advance();
            let n = self.parse_expression()?;
            // consume optional 'threads'
            if self.current_token() == Some(&Token::Threads) { self.advance(); }
            Ok(Some(n))
        } else {
            Ok(None)
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: spawn do … end  /  spawn do … end and join as HANDLE
    // ─────────────────────────────────────────────────────────────────────
    fn parse_spawn_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'spawn'
        self.consume(Token::Do)?;
        let mut body = Vec::new();
        while !matches!(self.current_token(), Some(Token::End) | None) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::End)?;
        // optional `and join as HANDLE`
        let join_as = if matches!(self.current_token(), Some(Token::Join)) {
            self.advance();
            if self.current_token() == Some(&Token::As) { self.advance(); }
            if let Some(Token::Identifier(name)) = self.current_token() {
                let n = name.clone();
                self.advance();
                Some(n)
            } else { None }
        } else { None };
        Ok(Statement::Spawn { body, join_as })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: create channel NAME with capacity N
    //      create channel NAME
    // ─────────────────────────────────────────────────────────────────────
    fn parse_create_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'create'
        if self.current_token() == Some(&Token::Channel) {
            self.advance(); // consume 'channel'
            let name = if let Some(Token::Identifier(n)) = self.current_token() {
                let n = n.clone(); self.advance(); n
            } else {
                return Err("Expected channel name after 'create channel'".to_string());
            };
            let capacity = if self.current_token() == Some(&Token::With) {
                self.advance();
                if self.current_token() == Some(&Token::Capacity) { self.advance(); }
                Some(self.parse_expression()?)
            } else { None };
            Ok(Statement::CreateChannel { name, capacity })
        } else {
            Err("Expected 'channel' after 'create'".to_string())
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: send VALUE to channel NAME
    // ─────────────────────────────────────────────────────────────────────
    fn parse_send_to_channel_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'send'
        let value = self.parse_expression()?;
        self.consume(Token::To)?;
        // optional 'channel'
        if self.current_token() == Some(&Token::Channel) { self.advance(); }
        let channel = self.parse_identifier_expression()?;
        Ok(Statement::SendToChannel { channel, value })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: receive from channel NAME and call it DEST
    // ─────────────────────────────────────────────────────────────────────
    fn parse_recv_from_channel_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'receive'
        self.consume(Token::From)?;
        if self.current_token() == Some(&Token::Channel) { self.advance(); }
        let channel = self.parse_identifier_expression()?;
        self.skip_glue_words();
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::RecvFromChannel { channel, destination })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: collect CHANNEL as DEST  (drain all items from channel into list)
    // ─────────────────────────────────────────────────────────────────────
    fn parse_collect_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'collect'
        let channel = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        // reuse RecvFromChannel — interpreter handles drain
        Ok(Statement::RecvFromChannel { channel, destination })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: free VARIABLE
    // ─────────────────────────────────────────────────────────────────────
    fn parse_free_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'free'
        let name = self.parse_identifier_expression()?;
        Ok(Statement::Free { name })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: set memory limit to 512 megabytes
    //      print memory usage
    // ─────────────────────────────────────────────────────────────────────
    fn parse_memory_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'memory'
        match self.current_token() {
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case("usage") => {
                self.advance();
                Ok(Statement::PrintMemoryUsage)
            }
            Some(Token::Limit) => {
                self.advance(); // consume 'limit'
                self.consume(Token::To)?;
                let mb = self.parse_expression()?;
                if matches!(self.current_token(), Some(Token::Megabytes) | Some(Token::Gigabytes)) {
                    self.advance();
                }
                Ok(Statement::SetMemoryLimit { megabytes: mb })
            }
            _ => Err("Expected 'limit' or 'usage' after 'memory'".to_string()),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: with memory scope do … end
    // ─────────────────────────────────────────────────────────────────────
    fn parse_with_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'with'
        // expect 'memory'
        if self.current_token() == Some(&Token::Memory) { self.advance(); }
        // expect 'scope'
        if self.current_token() == Some(&Token::Scope) { self.advance(); }
        self.consume(Token::Do)?;
        let mut body = Vec::new();
        while !matches!(self.current_token(), Some(Token::End) | None) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::End)?;
        Ok(Statement::MemoryScope { body })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: connect to postgres "url" as db
    // ─────────────────────────────────────────────────────────────────────
    fn parse_db_connect_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'connect'
        self.consume(Token::To)?;
        let db_type = self.parse_expression()?;
        let url = self.parse_expression()?;
        self.consume(Token::As)?;
        let alias_expr = self.parse_identifier_expression()?;
        let alias = if let Expression::Identifier(n) = alias_expr { n }
            else { return Err("Expected alias name for connection".to_string()); };
        Ok(Statement::DbConnect { db_type, url, alias })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: query "SQL" in CONN with parameters [x, y] as DEST
    // ─────────────────────────────────────────────────────────────────────
    fn parse_db_query_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'query'
        let sql = self.parse_expression()?;
        self.consume(Token::In)?;
        let connection = self.parse_identifier_expression()?;
        let mut params = Vec::new();
        if self.current_token() == Some(&Token::With) {
            self.advance();
            if self.current_token() == Some(&Token::Parameters) { self.advance(); }
            // parse list literal [x, y]
            if self.current_token() == Some(&Token::LBracket) {
                self.advance();
                while !matches!(self.current_token(), Some(Token::RBracket) | None) {
                    params.push(self.parse_expression()?);
                    if self.current_token() == Some(&Token::Comma) { self.advance(); }
                }
                self.consume(Token::RBracket)?;
            }
        }
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::DbQuery { connection, sql, params, destination })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: run "SQL" in CONN with parameters [x]
    // ─────────────────────────────────────────────────────────────────────
    fn parse_db_execute_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'run'
        let sql = self.parse_expression()?;
        self.consume(Token::In)?;
        let connection = self.parse_identifier_expression()?;
        let mut params = Vec::new();
        if self.current_token() == Some(&Token::With) {
            self.advance();
            if self.current_token() == Some(&Token::Parameters) { self.advance(); }
            if self.current_token() == Some(&Token::LBracket) {
                self.advance();
                while !matches!(self.current_token(), Some(Token::RBracket) | None) {
                    params.push(self.parse_expression()?);
                    if self.current_token() == Some(&Token::Comma) { self.advance(); }
                }
                self.consume(Token::RBracket)?;
            }
        }
        Ok(Statement::DbExecute { connection, sql, params })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: insert DATA into CONN table "TABLE_NAME"
    // ─────────────────────────────────────────────────────────────────────
    fn parse_db_insert_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'insert'
        let data = self.parse_expression()?;
        self.consume(Token::Into)?;
        let connection = self.parse_identifier_expression()?;
        // consume optional 'table'
        if self.current_token() == Some(&Token::Table) { self.advance(); }
        let table = self.parse_expression()?;
        Ok(Statement::DbInsert { connection, table, data })
    }

    // ─────────────────────────────────────────────────────────────────────
    // NEW: close CONN
    // ─────────────────────────────────────────────────────────────────────
    fn parse_db_close_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'close'
        let connection = self.parse_identifier_expression()?;
        Ok(Statement::DbClose { connection })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Helper: skip optional English glue identifiers ("and", "call", "it",
    // "the", "named", "called") between syntactic tokens.
    // ─────────────────────────────────────────────────────────────────────
    fn skip_glue_words(&mut self) {
        const GLUE: &[&str] = &[
            "and", "call", "it", "the", "a", "an", "named",
            "called", "name", "store", "put", "then",
        ];
        loop {
            match self.current_token() {
                Some(Token::Identifier(w)) if GLUE.contains(&w.to_lowercase().as_str()) => {
                    self.advance();
                }
                Some(Token::And) => { self.advance(); }
                _ => break,
            }
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || self.current_token() == Some(&Token::Eof)
    }
    fn consume(&mut self, expected: Token) -> Result<(), String> {
        if let Some(t) = self.current_token() {
            if std::mem::discriminant(t) == std::mem::discriminant(&expected) {
                self.advance();
                return Ok(());
            }
        }
        Err(format_user_error(&expected, self.current_token()))
    }
}

fn token_to_string(token: &Token) -> String {
    match token {
        Token::Identifier(s) => format!("identifier '{}'", s),
        Token::String(s) => format!("string literal \"{}\"", s),
        Token::Number(n) => format!("number {}", n),
        Token::As => "\"as\"".to_string(),
        Token::In => "\"in\"".to_string(),
        Token::To => "\"to\"".to_string(),
        Token::With => "\"with\"".to_string(),
        Token::LBracket => "'['".to_string(),
        Token::RBracket => "']'".to_string(),
        Token::Comma => "','".to_string(),
        Token::Eof => "end of input".to_string(),
        other => format!("{other:?}").to_lowercase(),
    }
}

fn format_user_error(expected: &Token, found: Option<&Token>) -> String {
    let expected_str = token_to_string(expected);
    let found_str = match found {
        Some(t) => token_to_string(t),
        None => "end of input".to_string(),
    };
    let mut message = format!("Expected {}, but found {}.", expected_str, found_str);
    if matches!(expected, Token::As) {
        message.push_str(" Did you mean `load \"file.txt\" as article`?");
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn missing_as_suggests_correction() {
        let mut lexer = Lexer::new("load \"file.txt\" article");
        let tokens = lexer.all_tokens();
        let mut parser = Parser::new(tokens);
        let err = parser.parse().unwrap_err();
        assert!(
            err.contains("Did you mean `load \"file.txt\" as article`?"),
            "{}",
            err
        );
    }

    #[test]
    fn parse_insight_command() {
        let mut lexer = Lexer::new("insight result with openai as explanation");
        let tokens = lexer.all_tokens();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().expect("failed to parse");
        assert!(matches!(
            &stmts[0],
            Statement::Insight { source, destination }
            if matches!(source, Expression::Identifier(s) if s == "result") &&
               matches!(destination, Expression::Identifier(s) if s == "explanation")
        ));
    }
}
