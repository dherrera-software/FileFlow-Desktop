//! Analizador léxico y sintáctico para sentencias CREATE TABLE e INSERT INTO de SQL.

pub fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

pub fn strip_sql_comments(sql: &str) -> String {
    let mut output = String::with_capacity(sql.len());
    let mut characters = sql.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(character) = characters.next() {
        if in_single {
            output.push(character);
            if character == '\'' {
                if characters.peek() == Some(&'\'') {
                    if let Some(next) = characters.next() {
                        output.push(next);
                    }
                } else {
                    in_single = false;
                }
            }
            continue;
        }
        if in_double {
            output.push(character);
            if character == '"' {
                in_double = false;
            }
            continue;
        }
        match character {
            '\'' => {
                in_single = true;
                output.push(character);
            }
            '"' => {
                in_double = true;
                output.push(character);
            }
            '-' if characters.peek() == Some(&'-') => {
                while characters.next().is_some_and(|next| next != '\n') {}
                output.push('\n');
            }
            '/' if characters.peek() == Some(&'*') => {
                characters.next();
                let mut previous = '\0';
                for next in characters.by_ref() {
                    if previous == '*' && next == '/' {
                        break;
                    }
                    previous = next;
                }
                output.push(' ');
            }
            _ => output.push(character),
        }
    }
    output
}

pub fn split_statements(sql: &str) -> Vec<String> {
    split_top_level(sql, ';')
        .into_iter()
        .filter(|statement| !statement.trim().is_empty())
        .collect()
}

pub fn split_top_level(text: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backtick = false;
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if in_single {
            current.push(character);
            if character == '\'' {
                if characters.peek() == Some(&'\'') {
                    if let Some(next) = characters.next() {
                        output_escaped(&mut current, next);
                    }
                } else {
                    in_single = false;
                }
            }
            continue;
        }
        if in_double {
            current.push(character);
            if character == '"' {
                in_double = false;
            }
            continue;
        }
        if in_backtick {
            current.push(character);
            if character == '`' {
                in_backtick = false;
            }
            continue;
        }
        match character {
            '\'' => {
                in_single = true;
                current.push(character);
            }
            '"' => {
                in_double = true;
                current.push(character);
            }
            '`' => {
                in_backtick = true;
                current.push(character);
            }
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                depth -= 1;
                current.push(character);
            }
            _ if character == separator && depth == 0 => parts.push(current.trim().to_string()),
            _ => current.push(character),
        }
        if character == separator && depth == 0 {
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn output_escaped(current: &mut String, next: char) {
    current.push(next);
}

pub fn parse_create_table(statement: &str) -> Option<(String, Vec<String>)> {
    let rest = statement
        .trim_start()
        .get("CREATE TABLE".len()..)?
        .trim_start();
    let rest = strip_if_not_exists(rest);
    let (table, rest) = parse_identifier(rest)?;
    let open = rest.find('(')?;
    let close = rest.rfind(')')?;
    if close <= open {
        return None;
    }
    let body = &rest[open + 1..close];
    let mut columns = Vec::new();
    for part in split_top_level(body, ',') {
        let (name, _) = parse_identifier(part.trim())?;
        let upper = name.to_ascii_uppercase();
        if matches!(
            upper.as_str(),
            "PRIMARY" | "CONSTRAINT" | "FOREIGN" | "UNIQUE" | "KEY" | "INDEX" | "CHECK"
        ) {
            continue;
        }
        columns.push(name);
    }
    Some((table, columns))
}

fn strip_if_not_exists(input: &str) -> &str {
    let upper = input.to_ascii_uppercase();
    if upper.starts_with("IF NOT EXISTS") {
        input["IF NOT EXISTS".len()..].trim_start()
    } else {
        input
    }
}

pub fn parse_insert(statement: &str) -> Option<(String, Vec<String>, Vec<Vec<String>>)> {
    let rest = statement
        .trim_start()
        .get("INSERT INTO".len()..)?
        .trim_start();
    let (table, rest) = parse_identifier(rest)?;
    let rest = rest.trim_start();
    let mut columns = Vec::new();
    let rest = if rest.starts_with('(') {
        let close = matching_paren(rest)?;
        columns = split_top_level(&rest[1..close], ',')
            .into_iter()
            .filter_map(|column| parse_identifier(column.trim()).map(|(name, _)| name))
            .collect();
        rest[close + 1..].trim_start()
    } else {
        rest
    };
    let upper = rest.to_ascii_uppercase();
    if !upper.starts_with("VALUES") {
        return None;
    }
    let values = rest["VALUES".len()..].trim_start();
    let mut tuples = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    for character in values.chars() {
        if in_single {
            current.push(character);
            if character == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            current.push(character);
            if character == '"' {
                in_double = false;
            }
            continue;
        }
        match character {
            '\'' => {
                in_single = true;
                current.push(character);
            }
            '"' => {
                in_double = true;
                current.push(character);
            }
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                depth -= 1;
                current.push(character);
                if depth == 0 {
                    let tuple = current.trim();
                    let inner = &tuple[1..tuple.len().saturating_sub(1)];
                    tuples.push(
                        split_top_level(inner, ',')
                            .into_iter()
                            .map(|literal| parse_sql_literal(&literal))
                            .collect(),
                    );
                    current.clear();
                }
            }
            _ if depth > 0 => current.push(character),
            _ => {}
        }
    }
    if tuples.is_empty() {
        return None;
    }
    Some((table, columns, tuples))
}

fn matching_paren(input: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    for (index, character) in input.char_indices() {
        if in_single {
            if character == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            if character == '"' {
                in_double = false;
            }
            continue;
        }
        match character {
            '\'' => in_single = true,
            '"' => in_double = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn parse_identifier(input: &str) -> Option<(String, &str)> {
    let trimmed = input.trim_start();
    let mut characters = trimmed.char_indices();
    let (_, first) = characters.next()?;
    let closing = match first {
        '"' => Some('"'),
        '`' => Some('`'),
        '[' => Some(']'),
        _ => None,
    };
    if let Some(closing) = closing {
        let start = first.len_utf8();
        let end = trimmed[start..].find(closing)? + start;
        return Some((
            trimmed[start..end].to_string(),
            &trimmed[end + closing.len_utf8()..],
        ));
    }
    let end = trimmed
        .char_indices()
        .find(|(_, character)| {
            !(character.is_ascii_alphanumeric() || *character == '_' || *character == '.')
        })
        .map(|(index, _)| index)
        .unwrap_or(trimmed.len());
    if end == 0 {
        return None;
    }
    Some((trimmed[..end].to_string(), &trimmed[end..]))
}

pub fn parse_sql_literal(raw: &str) -> String {
    let value = raw.trim();
    if value.eq_ignore_ascii_case("NULL") {
        return String::new();
    }
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        return value[1..value.len() - 1].replace("''", "'");
    }
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}
