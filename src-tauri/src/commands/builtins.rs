use std::collections::HashMap;
use std::sync::OnceLock;
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;

pub struct BuiltinRegexes {
    pub whitespace: Regex,
    pub spaces: Regex,
    pub line_numbers: Regex,
    pub md_link: Regex,
    pub md_heading: Regex,
    pub md_bold_inline: Regex,
    pub html_tags: Regex,
    pub code_blocks: Regex,
    pub comment_line: Regex,
    pub comment_block: Regex,
    pub url_component_unreserved: Regex,
    pub camel_boundary: Regex,
    pub non_alphanumeric_space_dash: Regex,
    pub whitespace_underscore: Regex,
    pub whitespace_dash: Regex,
    pub email: Regex,
    pub url: Regex,
    pub number: Regex,
    pub md_headings: Regex,
    pub extract_errors: Regex,
    pub redact_ip: Regex,
    pub redact_key: Regex,
    pub phone: Regex,
    pub md_inline_bold: Regex,
    pub md_inline_italic: Regex,
    pub md_inline_code: Regex,
}

static REGEXES: OnceLock<BuiltinRegexes> = OnceLock::new();

pub fn regexes() -> &'static BuiltinRegexes {
    REGEXES.get_or_init(|| BuiltinRegexes {
        whitespace: Regex::new(r"\s+").unwrap(),
        spaces: Regex::new(r" +").unwrap(),
        line_numbers: Regex::new(r"(?m)^\s*\d+[:.)\s]\s*").unwrap(),
        md_link: Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap(),
        md_heading: Regex::new(r"(?m)^#+\s+").unwrap(),
        md_bold_inline: Regex::new(r"\*\*([^*]+)\*\*|\*([^*]+)\*|`([^`]+)`").unwrap(),
        html_tags: Regex::new(r"<[^>]*>").unwrap(),
        code_blocks: Regex::new(r"(?s)```[a-zA-Z0-9_-]*\n?(.*?)```").unwrap(),
        comment_line: Regex::new(r"(?m)^\s*(//|#).*$").unwrap(),
        comment_block: Regex::new(r"(?s)/\*.*?\*/").unwrap(),
        url_component_unreserved: Regex::new(r"[^a-zA-Z0-9\-_.~]").unwrap(),
        camel_boundary: Regex::new(r"([a-z0-9])([A-Z])").unwrap(),
        non_alphanumeric_space_dash: Regex::new(r"[^a-zA-Z0-9\s-]").unwrap(),
        whitespace_underscore: Regex::new(r"[\s_]+").unwrap(),
        whitespace_dash: Regex::new(r"[\s-]+").unwrap(),
        email: Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").unwrap(),
        url: Regex::new(r#"https?://[^\s<>"']+"#).unwrap(),
        number: Regex::new(r"-?\d+(\.\d+)?").unwrap(),
        md_headings: Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap(),
        extract_errors: Regex::new(r"(?i)(error|exception|failed|fatal|traceback|panic|at\s+[\w\./\\]+:\d+)").unwrap(),
        redact_ip: Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
        redact_key: Regex::new(r"(?i)(api[_-]?key|token|secret|password)\s*[:=]\s*\S+").unwrap(),
        phone: Regex::new(r"\b[\+]?[(]?[0-9]{1,4}[)]?[-\s\.]?[0-9]{2,4}[-\s\.]?[0-9]{2,4}[-\s\.]?[0-9]{0,4}\b").unwrap(),
        md_inline_bold: Regex::new(r"\*\*(.+?)\*\*").unwrap(),
        md_inline_italic: Regex::new(r"\*(.+?)\*").unwrap(),
        md_inline_code: Regex::new(r"`(.+?)`").unwrap(),
    })
}

pub fn execute_builtin(id: &str, input: &str, params: &HashMap<String, String>) -> Result<String, String> {
    match id {
        "trim" => Ok(input.trim().to_string()),
        "remove_empty_lines" => Ok(input.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n")),
        "collapse_whitespace" => {
            Ok(regexes().whitespace.replace_all(input, " ").trim().to_string())
        }
        "normalize_whitespace" => {
            // Alle Whitespace-Varianten → reguläres Leerzeichen
            let result = input.chars().map(|c| if c.is_whitespace() && c != '\n' { ' ' } else { c }).collect::<String>();
            Ok(regexes().spaces.replace_all(&result, " ").to_string())
        }
        "remove_non_ascii" => Ok(input.chars().filter(|c| c.is_ascii()).collect()),
        "truncate" => {
            let max_len = params.get("length").and_then(|v| v.parse::<usize>().ok()).unwrap_or(100);
            if input.chars().count() > max_len {
                Ok(input.chars().take(max_len).collect::<String>() + "...")
            } else {
                Ok(input.to_string())
            }
        }
        "first_n_lines" => {
            let n = params.get("n").and_then(|v| v.parse::<usize>().ok()).unwrap_or(10);
            Ok(input.lines().take(n).collect::<Vec<_>>().join("\n"))
        }
        "last_n_lines" => {
            let n = params.get("n").and_then(|v| v.parse::<usize>().ok()).unwrap_or(10);
            let lines: Vec<&str> = input.lines().collect();
            let start = lines.len().saturating_sub(n);
            Ok(lines[start..].join("\n"))
        }
        "wrap_text" => {
            let width = params.get("width").and_then(|v| v.parse::<usize>().ok()).unwrap_or(80);
            let mut result = String::new();
            for line in input.lines() {
                let mut current_len = 0;
                for word in line.split_whitespace() {
                    let word_len = word.chars().count();
                    if current_len == 0 {
                        result.push_str(word);
                        current_len = word_len;
                    } else if current_len + 1 + word_len <= width {
                        result.push(' ');
                        result.push_str(word);
                        current_len += 1 + word_len;
                    } else {
                        result.push('\n');
                        result.push_str(word);
                        current_len = word_len;
                    }
                }
                result.push('\n');
            }
            Ok(result.trim_end_matches('\n').to_string())
        }

        // ── Groß-/Kleinschreibung ────────────────────────────────────────
        "uppercase" => Ok(input.to_uppercase()),
        "lowercase" => Ok(input.to_lowercase()),
        "title_case" => {
            let mut result = String::new();
            let mut capitalize_next = true;
            for c in input.chars() {
                if c.is_whitespace() || c == '_' || c == '-' {
                    capitalize_next = true;
                    result.push(c);
                } else if capitalize_next {
                    result.extend(c.to_uppercase());
                    capitalize_next = false;
                } else {
                    result.extend(c.to_lowercase());
                }
            }
            Ok(result)
        }
        "sentence_case" => {
            let mut result = String::new();
            let mut capitalize_next = true;
            for c in input.chars() {
                if capitalize_next && c.is_alphabetic() {
                    result.extend(c.to_uppercase());
                    capitalize_next = false;
                } else {
                    result.extend(c.to_lowercase())
                }
                if c == '.' || c == '!' || c == '?' {
                    capitalize_next = true;
                }
            }
            Ok(result)
        }
        "alternating_case" => {
            let mut result = String::new();
            let mut upper = false;
            for c in input.chars() {
                if c.is_alphabetic() {
                    if upper {
                        result.extend(c.to_uppercase());
                    } else {
                        result.extend(c.to_lowercase());
                    }
                    upper = !upper;
                } else {
                    result.push(c);
                }
            }
            Ok(result)
        }
        "rot13" => {
            Ok(input.chars().map(|c| match c {
                'a'..='m' | 'A'..='M' => (c as u8 + 13) as char,
                'n'..='z' | 'N'..='Z' => (c as u8 - 13) as char,
                _ => c,
            }).collect())
        }
        "reverse_text" => Ok(input.chars().rev().collect()),

        // ── Zeilenoperationen ────────────────────────────────────────────
        "sort_lines" => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.sort();
            Ok(lines.join("\n"))
        }
        "sort_lines_desc" => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.sort_by(|a, b| b.cmp(a));
            Ok(lines.join("\n"))
        }
        "sort_lines_by_length" => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.sort_by_key(|l| l.chars().count());
            Ok(lines.join("\n"))
        }
        "reverse_lines" => {
            let mut lines: Vec<&str> = input.lines().collect();
            lines.reverse();
            Ok(lines.join("\n"))
        }
        "unique_lines" => {
            let mut seen = std::collections::HashSet::new();
            let mut lines: Vec<&str> = Vec::new();
            for l in input.lines() {
                if seen.insert(l) {
                    lines.push(l);
                }
            }
            Ok(lines.join("\n"))
        }
        "shuffle_lines" => {
            // Deterministisches Mischen basierend auf Content-Hash
            let mut lines: Vec<&str> = input.lines().collect();
            let seed = lines.iter().fold(0u64, |acc, l| acc.wrapping_add(l.len() as u64));
            let n = lines.len();
            for i in (1..n).rev() {
                let j = (seed.wrapping_mul(i as u64 + 1).wrapping_add(42)) as usize % (i + 1);
                lines.swap(i, j);
            }
            Ok(lines.join("\n"))
        }
        "add_line_numbers" => {
            let lines: Vec<String> = input
                .lines()
                .enumerate()
                .map(|(i, line)| format!("{:4}: {}", i + 1, line))
                .collect();
            Ok(lines.join("\n"))
        }
        "remove_line_numbers" => {
            Ok(regexes().line_numbers.replace_all(input, "").to_string())
        }
        "prefix_lines" => {
            let prefix = params.get("prefix").map(|s| s.as_str()).unwrap_or("> ");
            Ok(input.lines().map(|l| format!("{}{}", prefix, l)).collect::<Vec<_>>().join("\n"))
        }
        "suffix_lines" => {
            let suffix = params.get("suffix").map(|s| s.as_str()).unwrap_or(" \\");
            Ok(input.lines().map(|l| format!("{}{}", l, suffix)).collect::<Vec<_>>().join("\n"))
        }
        "indent" => {
            let spaces = params.get("spaces").and_then(|v| v.parse::<usize>().ok()).unwrap_or(2);
            let pad = " ".repeat(spaces);
            Ok(input.lines().map(|l| format!("{}{}", pad, l)).collect::<Vec<_>>().join("\n"))
        }
        "dedent" => {
            let min_pad = input
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
                .min()
                .unwrap_or(0);
            let result: Vec<String> = input
                .lines()
                .map(|l| {
                    if l.len() >= min_pad { l[min_pad..].to_string() } else { l.to_string() }
                })
                .collect();
            Ok(result.join("\n"))
        }
        "join_lines" => {
            let separator = params.get("separator").map(|s| s.as_str()).unwrap_or(", ");
            Ok(input.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(separator))
        }

        // ── Code-Operationen ─────────────────────────────────────────────
        "wrap_markdown_block" => {
            let lang = params.get("language").map(|s| s.as_str()).unwrap_or("text");
            Ok(format!("```{}\n{}\n```", lang, input))
        }
        "strip_markdown" => {
            let s1 = regexes().md_link.replace_all(input, "$1");
            let s2 = regexes().md_heading.replace_all(&s1, "");
            Ok(regexes().md_bold_inline.replace_all(&s2, "$1$2$3").to_string())
        }
        "markdown_to_html" => {
            let mut html = String::new();
            let mut in_code_block = false;
            for line in input.lines() {
                if line.starts_with("```") {
                    if in_code_block {
                        html.push_str("</code></pre>\n");
                        in_code_block = false;
                    } else {
                        let lang = line.trim_start_matches('`').trim();
                        html.push_str(&format!("<pre><code class=\"language-{}\">", lang));
                        in_code_block = true;
                    }
                    continue;
                }
                if in_code_block {
                    html.push_str(&html_escape(line));
                    html.push('\n');
                    continue;
                }
                if line.starts_with("### ") { html.push_str(&format!("<h3>{}</h3>\n", &line[4..])); }
                else if line.starts_with("## ") { html.push_str(&format!("<h2>{}</h2>\n", &line[3..])); }
                else if line.starts_with("# ") { html.push_str(&format!("<h1>{}</h1>\n", &line[2..])); }
                else if line.trim().is_empty() { html.push_str("<br>\n"); }
                else {
                    let formatted = inline_markdown(line);
                    html.push_str(&format!("<p>{}</p>\n", formatted));
                }
            }
            Ok(html)
        }
        "strip_html_tags" => {
            Ok(regexes().html_tags.replace_all(input, "").to_string())
        }
        "pretty_json" => {
            let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
            serde_json::to_string_pretty(&parsed).map_err(|e| e.to_string())
        }
        "minify_json" => {
            let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
            serde_json::to_string(&parsed).map_err(|e| e.to_string())
        }
        "extract_json_keys" => {
            let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
            let mut keys = Vec::new();
            collect_json_keys(&parsed, &mut keys, "");
            Ok(keys.join("\n"))
        }
        "extract_code_blocks" => {
            let blocks: Vec<String> = regexes().code_blocks.captures_iter(input).map(|c| c[1].trim().to_string()).collect();
            Ok(blocks.join("\n\n---\n\n"))
        }
        "escape_json_string" => {
            Ok(serde_json::to_string(input).unwrap_or_else(|_| input.to_string())
                .trim_matches('"').to_string())
        }
        "unescape_json_string" => {
            let quoted = format!("\"{}\"", input);
            serde_json::from_str::<String>(&quoted).map_err(|e| e.to_string())
        }
        "remove_comments" => {
            let s1 = regexes().comment_line.replace_all(input, "");
            Ok(regexes().comment_block.replace_all(&s1, "").to_string())
        }

        // ── Kodierung/Dekodierung ─────────────────────────────────────────
        "base64_encode" => Ok(general_purpose::STANDARD.encode(input.as_bytes())),
        "base64_decode" => {
            let bytes = general_purpose::STANDARD.decode(input.trim()).map_err(|e| e.to_string())?;
            String::from_utf8(bytes).map_err(|e| e.to_string())
        }
        "url_encode" => Ok(urlencoding::encode(input).to_string()),
        "url_encode_component" => {
            Ok(regexes().url_component_unreserved.replace_all(input, |caps: &regex::Captures| {
                caps[0].bytes().map(|b| format!("%{:02X}", b)).collect::<String>()
            }).to_string())
        }
        "url_decode" => {
            urlencoding::decode(input).map(|cow| cow.to_string()).map_err(|e| e.to_string())
        }
        "html_entity_encode" => {
            Ok(input
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#39;"))
        }
        "html_entity_decode" => {
            Ok(input
                .replace("&quot;", "\"")
                .replace("&#39;", "'")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&"))
        }
        "hash_sha256" => {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }

        // ── Namenskonventionen ────────────────────────────────────────────
        "camel_to_snake" => {
            Ok(regexes().camel_boundary.replace_all(input, "${1}_${2}").to_lowercase())
        }
        "snake_to_camel" => {
            let mut result = String::new();
            let mut capitalize = false;
            for c in input.chars() {
                if c == '_' { capitalize = true; }
                else if capitalize { result.extend(c.to_uppercase()); capitalize = false; }
                else { result.push(c); }
            }
            Ok(result)
        }
        "snake_to_pascal" => {
            let mut result = String::new();
            let mut capitalize = true;
            for c in input.chars() {
                if c == '_' { capitalize = true; }
                else if capitalize { result.extend(c.to_uppercase()); capitalize = false; }
                else { result.push(c); }
            }
            Ok(result)
        }
        "to_slug" => {
            let clean = regexes().non_alphanumeric_space_dash.replace_all(input, "").to_lowercase();
            Ok(regexes().whitespace_underscore.replace_all(&clean, "-").trim_matches('-').to_string())
        }
        "to_kebab_case" => {
            let s = regexes().camel_boundary.replace_all(input, "${1}-${2}").to_lowercase();
            Ok(regexes().whitespace_underscore.replace_all(&s, "-").trim_matches('-').to_string())
        }
        "to_constant_case" => {
            let s = regexes().camel_boundary.replace_all(input, "${1}_${2}").to_uppercase();
            Ok(regexes().whitespace_dash.replace_all(&s, "_").trim_matches('_').to_string())
        }

        // ── Extraktion ────────────────────────────────────────────────────
        "extract_emails" => {
            let emails: Vec<&str> = regexes().email.find_iter(input).map(|m| m.as_str()).collect();
            Ok(emails.join("\n"))
        }
        "extract_urls" => {
            let urls: Vec<&str> = regexes().url.find_iter(input).map(|m| m.as_str()).collect();
            Ok(urls.join("\n"))
        }
        "extract_numbers" => {
            let nums: Vec<&str> = regexes().number.find_iter(input).map(|m| m.as_str()).collect();
            Ok(nums.join("\n"))
        }
        "extract_markdown_headings" => {
            let headings: Vec<String> = regexes().md_headings.captures_iter(input)
                .map(|c| format!("{} {}", &c[1], &c[2]))
                .collect();
            Ok(headings.join("\n"))
        }
        "extract_yaml_frontmatter" => {
            if input.starts_with("---") {
                if let Some(end) = input[3..].find("---") {
                    return Ok(input[3..end + 3].trim().to_string());
                }
            }
            Ok(String::new())
        }

        // ── Analyse & Statistik ───────────────────────────────────────────
        "with_stats" => {
            let words = input.split_whitespace().count();
            let chars = input.chars().count();
            let lines = input.lines().count();
            Ok(format!("{}\n\n--- Statistik ---\nZeichen: {} | Wörter: {} | Zeilen: {}", input, chars, words, lines))
        }
        "with_full_stats" => {
            // Vollständige Statistiken gemäß § 2.6 TextStats
            let char_count = input.chars().count();
            let char_no_space = input.chars().filter(|c| !c.is_whitespace()).count();
            let words: Vec<&str> = input.split_whitespace().collect();
            let word_count = words.len();
            let lines: Vec<&str> = input.lines().collect();
            let line_count = lines.len();
            let empty_line_count = lines.iter().filter(|l| l.trim().is_empty()).count();
            let non_empty_lines = line_count - empty_line_count;
            let paragraphs: Vec<&str> = input.split("\n\n").filter(|p| !p.trim().is_empty()).collect();
            let para_count = paragraphs.len();
            let sentences: Vec<&str> = input.split(|c| c == '.' || c == '!' || c == '?')
                .filter(|s| !s.trim().is_empty()).collect();
            let sentence_count = sentences.len();

            // Wortstatistiken
            let mut freq_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for w in &words {
                let clean = w.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect::<String>();
                if clean.len() > 2 {
                    *freq_map.entry(clean).or_insert(0) += 1;
                }
            }
            let unique_word_count = freq_map.len();
            let total_word_len: usize = words.iter().map(|w| w.len()).sum();
            let avg_word_length = if word_count > 0 { total_word_len as f64 / word_count as f64 } else { 0.0 };
            let longest_word = words.iter().max_by_key(|w| w.len()).unwrap_or(&"").to_string();

            let mut top_words: Vec<(String, usize)> = freq_map.into_iter().collect();
            top_words.sort_by(|a, b| b.1.cmp(&a.1));
            top_words.truncate(5);

            // Zeilenstatistiken
            let avg_line_length = if line_count > 0 { char_count as f64 / line_count as f64 } else { 0.0 };
            let longest_line = lines.iter().map(|l| l.len()).max().unwrap_or(0);

            // Token-Schätzung (cl100k_base)
            let estimated_tokens = std::cmp::max(char_count / 4, (word_count as f64 * 0.75) as usize);

            // Lesezeit (~200 WPM)
            let reading_secs = (word_count as f64 / 200.0) * 60.0;
            let reading_min = (reading_secs / 60.0).floor() as usize;
            let reading_sec_rem = reading_secs as usize % 60;

            // Flesch-Kincaid (nur bei >= 100 Wörter)
            let fk_str = if word_count >= 100 && sentence_count > 0 {
                let grade = 0.39 * (word_count as f64 / sentence_count as f64)
                    + 11.8 * (char_no_space as f64 / word_count as f64) - 15.59;
                format!("Flesch-Kincaid: {:.1}", grade.max(0.0))
            } else {
                "Flesch-Kincaid: n/a (< 100 Wörter)".to_string()
            };

            // Avg sentence length
            let avg_sent_len = if sentence_count > 0 { word_count as f64 / sentence_count as f64 } else { 0.0 };

            let top_words_str: Vec<String> = top_words.iter()
                .map(|(w, c)| format!("  {} ({}×)", w, c))
                .collect();

            let stats = format!(
                concat!(
                    "━━━ Vollständige Textstatistik ━━━\n",
                    "Zeichen gesamt:      {}\n",
                    "Zeichen (kein WS):   {}\n",
                    "Wörter:              {}\n",
                    "Eindeutige Wörter:   {} ({:.0}%)\n",
                    "Zeilen:              {} ({} leer, {} nicht leer)\n",
                    "Absätze:             {}\n",
                    "Sätze:               {}\n",
                    "Ø Satzlänge:         {:.1} Wörter\n",
                    "Ø Wortlänge:         {:.2} Zeichen\n",
                    "Längstes Wort:       {}\n",
                    "Ø Zeilenlänge:       {:.1} Zeichen\n",
                    "Längste Zeile:       {} Zeichen\n",
                    "Geschätzte Token:    ~{}\n",
                    "Lesezeit:            {}:{:02} min\n",
                    "{}\n",
                    "Häufigste Wörter:\n{}"
                ),
                char_count,
                char_no_space,
                word_count,
                unique_word_count,
                if word_count > 0 { unique_word_count as f64 / word_count as f64 * 100.0 } else { 0.0 },
                line_count, empty_line_count, non_empty_lines,
                para_count,
                sentence_count,
                avg_sent_len,
                avg_word_length,
                longest_word,
                avg_line_length,
                longest_line,
                estimated_tokens,
                reading_min, reading_sec_rem,
                fk_str,
                if top_words_str.is_empty() { "  (keine)".to_string() } else { top_words_str.join("\n") }
            );

            Ok(format!("{}\n\n{}", input, stats))
        }
        "count_occurrences" => {
            let pattern = params.get("pattern").map(|s| s.as_str()).unwrap_or("");
            if pattern.is_empty() {
                return Err("Parameter 'pattern' ist erforderlich.".to_string());
            }
            match Regex::new(pattern) {
                Ok(re) => {
                    let count = re.find_iter(input).count();
                    Ok(format!("{}\\n\\n--- Treffer für '{}': {} ---", input, pattern, count))
                }
                Err(_) => {
                    // Fallback: literale Suche
                    let count = input.matches(pattern).count();
                    Ok(format!("{}\n\n--- Treffer für '{}': {} ---", input, pattern, count))
                }
            }
        }
        "estimate_tokens" => {
            let tokens = (input.split_whitespace().count() as f32 * 0.75) as usize;
            Ok(format!("{}\n\n// Geschätzte Token: ~{}", input, tokens))
        }

        // ── Text-Erweiterungen (v2.0) ───────────────────────────────────
        "summary_cut" => {
            let n = params.get("n").and_then(|v| v.parse::<usize>().ok()).unwrap_or(200);
            let char_count = input.chars().count();
            if char_count <= n * 2 {
                Ok(input.to_string())
            } else {
                let start: String = input.chars().take(n).collect();
                let end: String = input.chars().skip(char_count - n).collect();
                let removed = char_count - n * 2;
                Ok(format!("{}\n…[{} Zeichen entfernt]…\n{}", start, removed, end))
            }
        }
        "normalize_unicode" => {
            use unicode_normalization::UnicodeNormalization;
            Ok(input.nfc().collect::<String>())
        }
        "remove_accents" => {
            use unicode_normalization::UnicodeNormalization;
            let nfd: String = input.nfd().collect();
            Ok(nfd.chars().filter(|c| !unicode_normalization::char::is_combining_mark(*c)).collect())
        }

        // ── Code-Erweiterungen (v2.0) ────────────────────────────────────
        "flatten_json" => {
            let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
            let mut keys = Vec::new();
            flatten_json_value(&parsed, &mut keys, "");
            Ok(keys.join("\n"))
        }
        "xml_pretty" => {
            use quick_xml::events::Event;
            use quick_xml::reader::Reader;
            use quick_xml::writer::Writer;
            use std::io::Cursor;

            let mut reader = Reader::from_str(input);
            reader.config_mut().trim_text(true);
            let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Eof) => break,
                    Ok(event) => {
                        writer.write_event(event).map_err(|e| format!("XML Write Error: {}", e))?;
                    }
                    Err(e) => return Err(format!("XML Parse Error: {}", e)),
                }
                buf.clear();
            }
            let result = writer.into_inner().into_inner();
            String::from_utf8(result).map_err(|e| e.to_string())
        }
        "xml_minify" => {
            use quick_xml::events::Event;
            use quick_xml::reader::Reader;
            use quick_xml::writer::Writer;
            use std::io::Cursor;

            let mut reader = Reader::from_str(input);
            reader.config_mut().trim_text(true);
            let mut writer = Writer::new(Cursor::new(Vec::new()));
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Eof) => break,
                    Ok(event) => {
                        writer.write_event(event).map_err(|e| format!("XML Write Error: {}", e))?;
                    }
                    Err(e) => return Err(format!("XML Parse Error: {}", e)),
                }
                buf.clear();
            }
            let result = writer.into_inner().into_inner();
            String::from_utf8(result).map_err(|e| e.to_string())
        }
        "minify_code" => {
            let s1 = input.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n");
            let s2 = regexes().comment_line.replace_all(&s1, "");
            Ok(regexes().comment_block.replace_all(&s2, "").to_string())
        }
        "extract_errors" => {
            let matches: Vec<&str> = input.lines().filter(|l| regexes().extract_errors.is_match(l)).collect();
            Ok(matches.join("\n"))
        }

        // ── Daten-Konvertierung ───────────────────────────────────────────
        "csv_to_json" => {
            let mut lines = input.lines();
            let header_line = lines.next().ok_or_else(|| "Empty CSV".to_string())?;
            let headers: Vec<&str> = header_line.split(',').map(|s| s.trim().trim_matches('"')).collect();
            
            let mut records = Vec::new();
            for line in lines {
                if line.trim().is_empty() { continue; }
                let values: Vec<&str> = line.split(',').map(|s| s.trim().trim_matches('"')).collect();
                let mut obj = serde_json::Map::new();
                for (i, &h) in headers.iter().enumerate() {
                    let val = values.get(i).copied().unwrap_or("");
                    obj.insert(h.to_string(), serde_json::Value::String(val.to_string()));
                }
                records.push(serde_json::Value::Object(obj));
            }
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())
        }
        "json_to_csv" => {
            let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
            let arr = parsed.as_array().ok_or_else(|| "JSON must be an array of objects".to_string())?;
            if arr.is_empty() {
                return Ok(String::new());
            }
            let mut headers: Vec<String> = Vec::new();
            if let Some(first) = arr.first().and_then(|v| v.as_object()) {
                for key in first.keys() {
                    headers.push(key.clone());
                }
            }
            let mut csv = String::new();
            csv.push_str(&headers.join(","));
            csv.push('\n');
            for item in arr {
                if let Some(obj) = item.as_object() {
                    let row: Vec<String> = headers.iter().map(|h| {
                        let val = obj.get(h).map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        }).unwrap_or_default();
                        if val.contains(',') || val.contains('"') {
                            format!("\"{}\"", val.replace('"', "\"\""))
                        } else {
                            val
                        }
                    }).collect();
                    csv.push_str(&row.join(","));
                    csv.push('\n');
                }
            }
            Ok(csv.trim_end().to_string())
        }
        "json_to_yaml" => {
            let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
            let mut out = String::new();
            json_to_yaml_fmt(&parsed, &mut out, 0);
            Ok(out.trim_end().to_string())
        }
        "yaml_to_json" => {
            // Simple key-value / line parser for basic YAML
            let mut obj = serde_json::Map::new();
            for line in input.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
                if let Some((k, v)) = trimmed.split_once(':') {
                    let val_str = v.trim().trim_matches('"').trim_matches('\'');
                    let json_val = if val_str == "true" {
                        serde_json::Value::Bool(true)
                    } else if val_str == "false" {
                        serde_json::Value::Bool(false)
                    } else if let Ok(num) = val_str.parse::<i64>() {
                        serde_json::Value::Number(num.into())
                    } else {
                        serde_json::Value::String(val_str.to_string())
                    };
                    obj.insert(k.trim().to_string(), json_val);
                }
            }
            serde_json::to_string_pretty(&serde_json::Value::Object(obj)).map_err(|e| e.to_string())
        }
        "table_to_markdown" => {
            let lines: Vec<&str> = input.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.is_empty() { return Ok(String::new()); }
            let mut result = String::new();
            for (i, line) in lines.iter().enumerate() {
                let cols: Vec<&str> = line.split_whitespace().collect();
                result.push_str("| ");
                result.push_str(&cols.join(" | "));
                result.push_str(" |\n");
                if i == 0 {
                    result.push_str("| ");
                    let dividers: Vec<&str> = cols.iter().map(|_| "---").collect();
                    result.push_str(&dividers.join(" | "));
                    result.push_str(" |\n");
                }
            }
            Ok(result.trim_end().to_string())
        }
        "align_columns" => {
            let sep = params.get("separator").map(|s| s.as_str()).unwrap_or(",");
            let lines: Vec<&str> = input.lines().collect();
            let rows: Vec<Vec<&str>> = lines.iter().map(|l| l.split(sep).map(|s| s.trim()).collect()).collect();
            let mut col_widths = Vec::new();
            for row in &rows {
                for (i, col) in row.iter().enumerate() {
                    if i >= col_widths.len() {
                        col_widths.push(col.chars().count());
                    } else {
                        col_widths[i] = col_widths[i].max(col.chars().count());
                    }
                }
            }
            let aligned: Vec<String> = rows.iter().map(|row| {
                row.iter().enumerate().map(|(i, col)| {
                    let width = col_widths.get(i).copied().unwrap_or(0);
                    format!("{:width$}", col, width = width)
                }).collect::<Vec<_>>().join(&format!(" {} ", sep))
            }).collect();
            Ok(aligned.join("\n"))
        }

        // ── Extraktion ────────────────────────────────────────────────────
        "extract_json_values" => {
            let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
            let mut values = Vec::new();
            collect_json_values(&parsed, &mut values);
            Ok(values.join("\n"))
        }

        // ── Template ──────────────────────────────────────────────────────
        "fill_template" => {
            let mut output = input.to_string();
            for (k, v) in params {
                let target = format!("{{{{{}}}}}", k);
                output = output.replace(&target, v);
            }
            Ok(output)
        }

        // ── Sicherheit ────────────────────────────────────────────────────
        "redact_sensitive" => {
            let s1 = regexes().redact_ip.replace_all(input, "[REDACTED-IP]");
            let s2 = regexes().redact_key.replace_all(&s1, "$1: [REDACTED]");
            Ok(s2.to_string())
        }
        "strip_pii" => {
            let s1 = regexes().email.replace_all(input, "[EMAIL]");
            let s2 = regexes().phone.replace_all(&s1, "[PHONE]");
            Ok(s2.to_string())
        }

        _ => Err(format!("Unknown builtin transformation: {}", id)),
    }
}

/// HTML-Escape für Sonderzeichen
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Inline-Markdown-Konvertierung für Fett, Kursiv, Code
fn inline_markdown(s: &str) -> String {
    let s1 = regexes().md_inline_bold.replace_all(s, "<strong>$1</strong>");
    let s2 = regexes().md_inline_italic.replace_all(&s1, "<em>$1</em>");
    regexes().md_inline_code.replace_all(&s2, "<code>$1</code>").to_string()
}

/// Rekursive JSON-Key-Extraktion
fn collect_json_keys(val: &serde_json::Value, keys: &mut Vec<String>, prefix: &str) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                keys.push(key.clone());
                collect_json_keys(v, keys, &key);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let key = format!("{}[{}]", prefix, i);
                collect_json_keys(v, keys, &key);
            }
        }
        _ => {}
    }
}

/// Rekursive JSON-Flachlegung (dot.notation: value)
fn flatten_json_value(val: &serde_json::Value, lines: &mut Vec<String>, prefix: &str) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                flatten_json_value(v, lines, &key);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let key = format!("{}[{}]", prefix, i);
                flatten_json_value(v, lines, &key);
            }
        }
        _ => {
            lines.push(format!("{}: {}", prefix, val));
        }
    }
}

/// Rekursive JSON-Werte-Extraktion
fn collect_json_values(val: &serde_json::Value, values: &mut Vec<String>) {
    match val {
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_json_values(v, values);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_json_values(v, values);
            }
        }
        serde_json::Value::String(s) => values.push(s.clone()),
        other => values.push(other.to_string()),
    }
}

/// Einfache Formatiersoftware JSON → YAML
fn json_to_yaml_fmt(val: &serde_json::Value, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                match v {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        out.push_str(&format!("{}{}:\n", pad, k));
                        json_to_yaml_fmt(v, out, indent + 2);
                    }
                    serde_json::Value::String(s) => {
                        out.push_str(&format!("{}{}: \"{}\"\n", pad, k, s));
                    }
                    _ => {
                        out.push_str(&format!("{}{}: {}\n", pad, k, v));
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                match item {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        out.push_str(&format!("{}-\n", pad));
                        json_to_yaml_fmt(item, out, indent + 2);
                    }
                    serde_json::Value::String(s) => {
                        out.push_str(&format!("{}- \"{}\"\n", pad, s));
                    }
                    _ => {
                        out.push_str(&format!("{}- {}\n", pad, item));
                    }
                }
            }
        }
        serde_json::Value::String(s) => out.push_str(&format!("{}\"{}\"\n", pad, s)),
        other => out.push_str(&format!("{}{}\n", pad, other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_static_regex_singleton_address() {
        let ptr1 = std::ptr::addr_of!(regexes().whitespace);
        let ptr2 = std::ptr::addr_of!(regexes().whitespace);
        assert_eq!(ptr1, ptr2, "Regex instance should be compiled only once and cached at the same memory address");

        let email_ptr1 = std::ptr::addr_of!(regexes().email);
        let email_ptr2 = std::ptr::addr_of!(regexes().email);
        assert_eq!(email_ptr1, email_ptr2);
    }

    #[test]
    fn test_builtin_transformations() {
        let params = HashMap::new();

        let res = execute_builtin("collapse_whitespace", "hello    world \n  test", &params).unwrap();
        assert_eq!(res, "hello world test");

        let res = execute_builtin("strip_markdown", "# Heading\n[Link](http://a.com) and **bold** text", &params).unwrap();
        assert_eq!(res, "Heading\nLink and bold text");

        let res = execute_builtin("redact_sensitive", "Server 192.168.1.100 api_key=secret123", &params).unwrap();
        assert_eq!(res, "Server [REDACTED-IP] api_key: [REDACTED]");

        let res = execute_builtin("strip_pii", "Mail user@test.com call 049 123 45678", &params).unwrap();
        assert_eq!(res, "Mail [EMAIL] call [PHONE]");

        let res = execute_builtin("camel_to_snake", "myCamelCaseVar", &params).unwrap();
        assert_eq!(res, "my_camel_case_var");

        let res = execute_builtin("extract_emails", "Contact me at alice@example.com or bob@test.org", &params).unwrap();
        assert_eq!(res, "alice@example.com\nbob@test.org");
    }

    #[test]
    fn test_benchmark_10k_regex_builtins() {
        let params = HashMap::new();
        let sample_text = "# Header\nContact admin@example.com at IP 10.0.0.1 or call 05550199. Check **important** link [Docs](https://docs.org) api_key=abc12345.";
        let iterations = 1_000;

        // 1. Measure without caching (re-compiling regexes on each call)
        let start_unoptimized = Instant::now();
        for _ in 0..iterations {
            let re_ws = regex::Regex::new(r"\s+").unwrap();
            let _ = re_ws.replace_all(sample_text, " ").trim().to_string();

            let re_link = regex::Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap();
            let s1 = re_link.replace_all(sample_text, "$1");
            let re_heading = regex::Regex::new(r"(?m)^#+\s+").unwrap();
            let s2 = re_heading.replace_all(&s1, "");
            let re_bold = regex::Regex::new(r"\*\*([^*]+)\*\*|\*([^*]+)\*|`([^`]+)`").unwrap();
            let _ = re_bold.replace_all(&s2, "$1$2$3").to_string();

            let re_ip = regex::Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
            let re_key = regex::Regex::new(r"(?i)(api[_-]?key|token|secret|password)\s*[:=]\s*\S+").unwrap();
            let s1 = re_ip.replace_all(sample_text, "[REDACTED-IP]");
            let _ = re_key.replace_all(&s1, "$1: [REDACTED]").to_string();
        }
        let elapsed_unoptimized = start_unoptimized.elapsed();

        // 2. Measure with static cached OnceLock regexes
        let start_cached = Instant::now();
        for _ in 0..iterations {
            let _ = execute_builtin("collapse_whitespace", sample_text, &params).unwrap();
            let _ = execute_builtin("strip_markdown", sample_text, &params).unwrap();
            let _ = execute_builtin("redact_sensitive", sample_text, &params).unwrap();
            let _ = execute_builtin("strip_pii", sample_text, &params).unwrap();
            let _ = execute_builtin("camel_to_snake", "sampleTextWithCamelCase", &params).unwrap();
        }
        let elapsed_cached = start_cached.elapsed();

        println!("\n=== REGEX PERFORMANCE BENCHMARK (1,000 Iterations) ===");
        println!("Without OnceLock (recompiling each call): {:?}", elapsed_unoptimized);
        println!("With OnceLock (precompiled static regexes):  {:?}", elapsed_cached);
        if elapsed_cached.as_nanos() > 0 {
            let speedup = elapsed_unoptimized.as_secs_f64() / elapsed_cached.as_secs_f64();
            println!("Speedup factor: {:.2}x faster", speedup);
        }
        println!("=========================================================\n");
    }
}
