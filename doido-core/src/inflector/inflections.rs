use regex::Regex;
use std::collections::HashSet;

pub struct Inflections {
    pub(crate) plurals: Vec<(Regex, String)>,
    pub(crate) singulars: Vec<(Regex, String)>,
    pub(crate) irregulars: Vec<(String, String)>,
    pub(crate) uncountables: HashSet<String>,
    pub(crate) acronyms: HashSet<String>,
}

impl Inflections {
    pub fn new() -> Self {
        Self {
            plurals: Vec::new(),
            singulars: Vec::new(),
            irregulars: Vec::new(),
            uncountables: HashSet::new(),
            acronyms: HashSet::new(),
        }
    }

    pub fn plural(&mut self, pattern: &str, replacement: &str) {
        self.plurals.push((
            Regex::new(pattern).unwrap_or_else(|e| panic!("invalid plural pattern `{pattern}`: {e}")),
            replacement.to_string(),
        ));
    }

    pub fn singular(&mut self, pattern: &str, replacement: &str) {
        self.singulars.push((
            Regex::new(pattern).unwrap_or_else(|e| panic!("invalid singular pattern `{pattern}`: {e}")),
            replacement.to_string(),
        ));
    }

    pub fn irregular(&mut self, singular: &str, plural: &str) {
        self.irregulars.push((singular.to_lowercase(), plural.to_lowercase()));
    }

    pub fn uncountable(&mut self, word: &str) {
        self.uncountables.insert(word.to_lowercase());
    }

    pub fn acronym(&mut self, word: &str) {
        self.acronyms.insert(word.to_uppercase());
    }

    /// Returns the plural form of `word` according to the configured rules.
    pub fn pluralize(&self, word: &str) -> String {
        let lower = word.to_lowercase();

        // 1. Uncountables are returned unchanged.
        if self.uncountables.contains(&lower) {
            return word.to_string();
        }

        // 2. Irregulars take priority over regex rules.
        for (singular, plural) in &self.irregulars {
            if singular == &lower {
                return plural.clone();
            }
            if plural == &lower {
                // Already plural — return as-is.
                return plural.clone();
            }
        }

        // 3. Regex rules — tried in reverse add-order (last added = highest priority).
        for (pattern, replacement) in self.plurals.iter().rev() {
            if pattern.is_match(&lower) {
                return pattern.replace(&lower, replacement.as_str()).to_string();
            }
        }

        word.to_string()
    }

    /// Returns the singular form of `word` according to the configured rules.
    pub fn singularize(&self, word: &str) -> String {
        let lower = word.to_lowercase();

        // 1. Uncountables.
        if self.uncountables.contains(&lower) {
            return word.to_string();
        }

        // 2. Irregulars — lookup by plural form.
        for (singular, plural) in &self.irregulars {
            if plural == &lower {
                return singular.clone();
            }
            if singular == &lower {
                // Already singular — return as-is.
                return singular.clone();
            }
        }

        // 3. Regex rules.
        for (pattern, replacement) in self.singulars.iter().rev() {
            if pattern.is_match(&lower) {
                return pattern.replace(&lower, replacement.as_str()).to_string();
            }
        }

        word.to_string()
    }
}

impl Default for Inflections {
    fn default() -> Self {
        crate::inflector::rules::defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plural_adds_rule() {
        let mut i = Inflections::new();
        i.plural(r"s$", "ses");
        assert_eq!(i.plurals.len(), 1);
    }

    #[test]
    fn test_singular_adds_rule() {
        let mut i = Inflections::new();
        i.singular(r"ses$", "s");
        assert_eq!(i.singulars.len(), 1);
    }

    #[test]
    fn test_irregular_stores_lowercase() {
        let mut i = Inflections::new();
        i.irregular("Person", "People");
        assert_eq!(i.irregulars[0], ("person".to_string(), "people".to_string()));
    }

    #[test]
    fn test_uncountable_stores_lowercase() {
        let mut i = Inflections::new();
        i.uncountable("Sheep");
        assert!(i.uncountables.contains("sheep"));
    }

    #[test]
    fn test_acronym_stores_uppercase() {
        let mut i = Inflections::new();
        i.acronym("api");
        assert!(i.acronyms.contains("API"));
    }
}
