use doido_core::inflector::Inflections;
use doido_core::inflector::Inflector;

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

// ── camelize ──────────────────────────────────────────────────────────────

#[test]
fn test_camelize_snake_case() {
    let i = Inflections::default();
    assert_eq!(i.camelize("post_comment"), "PostComment");
    assert_eq!(i.camelize("active_record"), "ActiveRecord");
}

#[test]
fn test_camelize_already_camel() {
    let i = Inflections::default();
    assert_eq!(i.camelize("PostComment"), "PostComment");
}

#[test]
fn test_camelize_lower() {
    let i = Inflections::default();
    assert_eq!(i.camelize_lower("post_comment"), "postComment");
    assert_eq!(i.camelize_lower("active_record"), "activeRecord");
}

// ── underscore ────────────────────────────────────────────────────────────

#[test]
fn test_underscore_camel_case() {
    let i = Inflections::default();
    assert_eq!(i.underscore("PostComment"), "post_comment");
    assert_eq!(i.underscore("ActiveRecord"), "active_record");
}

#[test]
fn test_underscore_already_snake() {
    let i = Inflections::default();
    assert_eq!(i.underscore("post_comment"), "post_comment");
}

// ── dasherize ─────────────────────────────────────────────────────────────

#[test]
fn test_dasherize() {
    let i = Inflections::default();
    assert_eq!(i.dasherize("post_comment"), "post-comment");
    assert_eq!(i.dasherize("active_record"), "active-record");
}

// ── humanize ─────────────────────────────────────────────────────────────

#[test]
fn test_humanize_snake_case() {
    let i = Inflections::default();
    assert_eq!(i.humanize("post_comment"), "Post comment");
}

#[test]
fn test_humanize_strips_id_suffix() {
    let i = Inflections::default();
    assert_eq!(i.humanize("author_id"), "Author");
}

// ── constantize ───────────────────────────────────────────────────────────

#[test]
fn test_constantize() {
    let i = Inflections::default();
    assert_eq!(i.constantize("post_comment"), "POST_COMMENT");
    assert_eq!(i.constantize("active-record"), "ACTIVE_RECORD");
}

// ── tableize and classify (inverses) ─────────────────────────────────────

#[test]
fn test_tableize() {
    let i = Inflections::default();
    assert_eq!(i.tableize("PostComment"), "post_comments");
    assert_eq!(i.tableize("Person"), "people");
}

#[test]
fn test_classify() {
    let i = Inflections::default();
    assert_eq!(i.classify("post_comments"), "PostComment");
}

#[test]
fn test_tableize_classify_are_inverses() {
    let i = Inflections::default();
    let original = "PostComment";
    assert_eq!(i.classify(&i.tableize(original)), original);
}

// ── foreign_key ───────────────────────────────────────────────────────────

#[test]
fn test_foreign_key() {
    let i = Inflections::default();
    assert_eq!(i.foreign_key("PostComment"), "post_comment_id");
    assert_eq!(i.foreign_key("Person"), "person_id");
}

// ── acronym support in camelize ───────────────────────────────────────────

#[test]
fn test_camelize_with_acronym() {
    let mut i = Inflections::default();
    i.acronym("API");
    i.acronym("HTML");
    assert_eq!(i.camelize("api_client"), "APIClient");
    assert_eq!(i.camelize("html_parser"), "HTMLParser");
}

// ── acronym support in underscore ────────────────────────────────────────

#[test]
fn test_underscore_acronym_sequence() {
    let i = Inflections::default();
    // No acronym registration needed — the algorithm handles uppercase runs.
    assert_eq!(i.underscore("APIClient"), "api_client");
    assert_eq!(i.underscore("HTMLParser"), "html_parser");
}

// ── static Inflector facade ───────────────────────────────────────────────

#[test]
fn test_inflector_static_pluralize() {
    assert_eq!(Inflector::pluralize("post"), "posts");
    assert_eq!(Inflector::pluralize("person"), "people");
    assert_eq!(Inflector::pluralize("sheep"), "sheep");
}

#[test]
fn test_inflector_static_singularize() {
    assert_eq!(Inflector::singularize("posts"), "post");
    assert_eq!(Inflector::singularize("people"), "person");
}

#[test]
fn test_inflector_static_camelize() {
    assert_eq!(Inflector::camelize("post_comment"), "PostComment");
}

#[test]
fn test_inflector_static_underscore() {
    assert_eq!(Inflector::underscore("PostComment"), "post_comment");
}

#[test]
fn test_inflector_static_tableize() {
    assert_eq!(Inflector::tableize("PostComment"), "post_comments");
}

#[test]
fn test_inflector_static_classify() {
    assert_eq!(Inflector::classify("post_comments"), "PostComment");
}

#[test]
fn test_inflector_static_foreign_key() {
    assert_eq!(Inflector::foreign_key("PostComment"), "post_comment_id");
}
