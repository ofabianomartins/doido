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
