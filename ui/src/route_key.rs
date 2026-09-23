use std::fmt;
use std::str::FromStr;

/// A table name, partition key or row key as a segment of the UI's URL.
///
/// A key is any text, while a URL segment can not hold a `/` (the router splits on it before it
/// decodes anything) or a `\` (the browser turns it into `/`), keeps a `%` it did not write itself
/// undecoded only by luck, and can not be `.` or `..` (the browser folds those away -
/// percent-encoded ones too). So the key is escaped with `~` before the router sees it, and
/// unescaped after the router has decoded the segment:
///
/// `~` → `~~`, `/` → `~s`, `\` → `~b`, `%` → `~p`, an empty key → `~e`, and the dots of a key
/// which is nothing but `.` or `..` → `~.`
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RouteKey(pub String);

impl RouteKey {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for RouteKey {
    fn from(src: String) -> Self {
        Self(src)
    }
}

impl From<&str> for RouteKey {
    fn from(src: &str) -> Self {
        Self(src.to_string())
    }
}

impl fmt::Display for RouteKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("~e");
        }

        let dots_only = self.0 == "." || self.0 == "..";

        for c in self.0.chars() {
            match c {
                '~' => f.write_str("~~")?,
                '/' => f.write_str("~s")?,
                '\\' => f.write_str("~b")?,
                '%' => f.write_str("~p")?,
                '.' if dots_only => f.write_str("~.")?,
                c => write!(f, "{}", c)?,
            }
        }

        Ok(())
    }
}

impl FromStr for RouteKey {
    type Err = String;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        if src == "~e" {
            return Ok(Self(String::new()));
        }

        let mut result = String::with_capacity(src.len());
        let mut chars = src.chars();

        while let Some(c) = chars.next() {
            if c != '~' {
                result.push(c);
                continue;
            }

            match chars.next() {
                Some('~') => result.push('~'),
                Some('s') => result.push('/'),
                Some('b') => result.push('\\'),
                Some('p') => result.push('%'),
                Some('.') => result.push('.'),
                Some(other) => return Err(format!("Unknown escape '~{}' in '{}'", other, src)),
                None => return Err(format!("Dangling '~' in '{}'", src)),
            }
        }

        Ok(Self(result))
    }
}

#[cfg(test)]
mod tests {
    use super::RouteKey;

    fn round_trip(src: &str) {
        let encoded = RouteKey::from(src).to_string();
        assert!(!encoded.contains('/'), "{encoded}");
        assert!(!encoded.contains('\\'), "{encoded}");
        assert!(!encoded.contains('%'), "{encoded}");
        assert_ne!(encoded, ".");
        assert_ne!(encoded, "..");
        assert!(!encoded.is_empty());
        assert_eq!(src, encoded.parse::<RouteKey>().unwrap().as_str());
    }

    #[test]
    fn test_round_trips() {
        for src in [
            "EUR/USD", "a/b/c", "100%", "%2F", "~", "~s", "..", ".", "...", "a.b", "", "sp ace",
            "q?x#y&z=1", "Ключ", "~~p/%", "a\\b", "\\", "~b",
        ] {
            round_trip(src);
        }
    }

    #[test]
    fn test_plain_keys_stay_readable() {
        assert_eq!("e2e-items", RouteKey::from("e2e-items").to_string());
        assert_eq!("EUR~sUSD", RouteKey::from("EUR/USD").to_string());
    }

    #[test]
    fn test_bad_escapes_are_refused() {
        assert!("a~x".parse::<RouteKey>().is_err());
        assert!("a~".parse::<RouteKey>().is_err());
    }
}
