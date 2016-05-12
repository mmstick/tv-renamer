#[derive(Clone, Debug, PartialEq)]
pub enum TemplateToken {
    Character(char),
    Series,
    Season,
    Episode,
    TVDB,
}

// The default template signature is `${SERIES} ${SEASON}x${EPISODE} ${TITLE}`
pub fn default_template() -> Vec<TemplateToken> {
    vec![TemplateToken::Series, TemplateToken::Character(' '), TemplateToken::Season, TemplateToken::Character('x'),
        TemplateToken::Episode, TemplateToken::Character(' '), TemplateToken::TVDB]
}

/// This tokenizer will take the template string as input and convert it into an ordered vector of tokens.
pub fn tokenize_template(template: &str) -> Vec<TemplateToken> {
    let mut tokens = Vec::new();
    let mut pattern = String::new();
    let mut matching = false;
    for character in template.chars() {
        if character == '$' && !matching {
            matching = true;
            pattern.push('$');
        } else if character == '$' && matching {
            matching = false;
            for character in pattern.chars() {
                tokens.push(TemplateToken::Character(character));
            }
            tokens.push(TemplateToken::Character('$'));
            pattern.clear();
        } else if character == '$' {
            tokens.push(TemplateToken::Character('$'));
        } else if character == '{' && matching && pattern.len() == 1 {
            pattern.push('{');
        } else if character == '{' && matching {
            matching = false;
            for character in pattern.chars() {
                tokens.push(TemplateToken::Character(character));
            }
            tokens.push(TemplateToken::Character('$'));
            pattern.clear();
        } else if character == '{'{
            tokens.push(TemplateToken::Character('{'));
        } else if character == '}' && matching {
            pattern.push('}');
            if let Some(value) = match_token(&pattern) {
                tokens.push(value);
            } else {
                for character in pattern.chars() {
                    tokens.push(TemplateToken::Character(character));
                }
                tokens.push(TemplateToken::Character('$'));
            }
            matching = false;
            pattern.clear();
        } else if matching {
            pattern.push(character);
        } else {
            tokens.push(TemplateToken::Character(character));
        }
    }
    tokens
}

/// Given a pattern, this function will attempt to match the pattern to a predefined token.
fn match_token(pattern: &str) -> Option<TemplateToken> {
    match pattern {
        "${Series}"     => Some(TemplateToken::Series),
        "${Season}"     => Some(TemplateToken::Season),
        "${Episode}"    => Some(TemplateToken::Episode),
        "${TVDB_Title}" => Some(TemplateToken::TVDB),
        _               => None
    }
}

#[test]
fn test_tokenize() {
    assert_eq!(default_template(), tokenize_template("${Series} ${Season}x${Episode} ${TVDB_Title}"));
}

#[test]
fn test_match_token() {
    assert_eq!(Some(TemplateToken::Series), match_token("${Series}"));
    assert_eq!(Some(TemplateToken::Season), match_token("${Season}"));
    assert_eq!(Some(TemplateToken::Episode), match_token("${Episode}"));
    assert_eq!(Some(TemplateToken::TVDB), match_token("${TVDB_Title}"));
    assert_eq!(None, match_token("${invalid}"));
}
