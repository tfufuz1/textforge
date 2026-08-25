use textforge::commands::snippets::render_template;
use std::collections::HashMap;

#[tokio::test]
async fn test_no_template_injection() {
    let content = "{{a}} und {{b}}";
    let mut context = HashMap::new();
    context.insert("a".to_string(), "{{b}}".to_string());
    context.insert("b".to_string(), "INJECTED".to_string());

    let result = render_template(content.to_string(), context, false).await.unwrap();
    // "a" soll zu "{{b}}" aufgelöst werden (Literal-String), NICHT zu "INJECTED"
    assert_eq!(result.output, "{{b}} und INJECTED");
    // NICHT: "INJECTED und INJECTED"
}
