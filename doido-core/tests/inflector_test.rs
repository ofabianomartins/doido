use doido_core::inflector::Inflections;

// ── pluralize ──────────────────────────────────────────────────────────────

#[test]
fn test_pluralize_regular_word() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("post"), "posts");
}

#[test]
fn test_pluralize_sibilant_ending() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("box"), "boxes");
    assert_eq!(i.pluralize("watch"), "watches");
    assert_eq!(i.pluralize("dish"), "dishes");
}

#[test]
fn test_pluralize_y_ending() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("city"), "cities");
    assert_eq!(i.pluralize("day"), "days"); // vowel+y: catch-all applies
}

#[test]
fn test_pluralize_irregular() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("person"), "people");
    assert_eq!(i.pluralize("man"), "men");
    assert_eq!(i.pluralize("child"), "children");
}

#[test]
fn test_pluralize_uncountable() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("sheep"), "sheep");
    assert_eq!(i.pluralize("money"), "money");
    assert_eq!(i.pluralize("fish"), "fish");
}

// ── singularize ────────────────────────────────────────────────────────────

#[test]
fn test_singularize_regular_word() {
    let i = Inflections::default();
    assert_eq!(i.singularize("posts"), "post");
}

#[test]
fn test_singularize_es_ending() {
    let i = Inflections::default();
    assert_eq!(i.singularize("boxes"), "box");
    assert_eq!(i.singularize("watches"), "watch");
    assert_eq!(i.singularize("dishes"), "dish");
}

#[test]
fn test_singularize_ies_ending() {
    let i = Inflections::default();
    assert_eq!(i.singularize("cities"), "city");
}

#[test]
fn test_singularize_irregular() {
    let i = Inflections::default();
    assert_eq!(i.singularize("people"), "person");
    assert_eq!(i.singularize("men"), "man");
    assert_eq!(i.singularize("children"), "child");
}

#[test]
fn test_singularize_uncountable() {
    let i = Inflections::default();
    assert_eq!(i.singularize("sheep"), "sheep");
    assert_eq!(i.singularize("money"), "money");
}

// ── custom rules override defaults ────────────────────────────────────────

#[test]
fn test_custom_irregular_overrides_default() {
    let mut i = Inflections::default();
    i.irregular("goose", "geese");
    assert_eq!(i.pluralize("goose"), "geese");
    assert_eq!(i.singularize("geese"), "goose");
}

#[test]
fn test_custom_uncountable() {
    let mut i = Inflections::default();
    i.uncountable("news");
    assert_eq!(i.pluralize("news"), "news");
    assert_eq!(i.singularize("news"), "news");
}

#[test]
fn test_custom_plural_regex_rule() {
    let mut i = Inflections::default();
    i.plural(r"(quiz)$", "${1}zes");
    assert_eq!(i.pluralize("quiz"), "quizzes");
}

#[test]
fn test_custom_singular_regex_rule() {
    let mut i = Inflections::default();
    i.singular(r"(quiz)zes$", "${1}");
    assert_eq!(i.singularize("quizzes"), "quiz");
}
