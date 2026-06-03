#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChunk {
    pub text: String,
    pub start_token: usize,
    pub end_token: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TokenSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn token_spans(text: &str) -> Vec<TokenSpan> {
    let mut out = Vec::new();
    let mut in_token = false;
    let mut token_start = 0usize;

    for (idx, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if in_token {
                out.push(TokenSpan {
                    start: token_start,
                    end: idx,
                });
                in_token = false;
            }
        } else if !in_token {
            token_start = idx;
            in_token = true;
        }
    }

    if in_token {
        out.push(TokenSpan {
            start: token_start,
            end: text.len(),
        });
    }

    out
}

pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<TextChunk> {
    let token_spans = token_spans(text);
    if token_spans.is_empty() {
        return Vec::new();
    }
    if chunk_size == 0 {
        return Vec::new();
    }
    let step = chunk_size.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let total_tokens = token_spans.len();
    let mut start_token = 0usize;

    while start_token < total_tokens {
        let end_token = (start_token + chunk_size).min(total_tokens);
        let byte_start = token_spans[start_token].start;
        let byte_end = token_spans[end_token - 1].end;
        chunks.push(TextChunk {
            text: text[byte_start..byte_end].to_string(),
            start_token,
            end_token,
        });

        if end_token == total_tokens {
            break;
        }
        start_token = (start_token + step).min(total_tokens);
    }

    chunks
}

