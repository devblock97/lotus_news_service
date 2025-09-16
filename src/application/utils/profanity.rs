/// Extremely small, conservative bad-word list as a placeholder.
/// Replace with a better library/dictionary as needed.
static BAD_WORDS: &[&str] = &[
    "damn", "hell", "shit", "fuck" // keep minimal to avoid false positives
];

pub fn contains_profanity(s: &str) -> bool {
    let lower = s.to_lowercase();
    BAD_WORDS.iter().any(|w| lower.contains(w))
}

/// Optional sanitizer: stars out matched words (very naive).
pub fn sanitize(mut s: String) -> String {
    let lower = s.to_lowercase();
    for &w in BAD_WORDS {
        if lower.contains(w) {
            let stars = "*".repeat(w.len());
            s = s.replace(w, &stars);
        }
    }
    s
}
