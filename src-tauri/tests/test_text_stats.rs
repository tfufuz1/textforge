use textforge::commands::snippets::{compute_text_stats, count_syllables_heuristic};

#[tokio::test]
async fn test_flesch_kincaid_simple_text() {
    // "The cat sat on the mat." – einfacher Text, niedriger Grade
    let stats = compute_text_stats("The cat sat on the mat.".to_string()).await.unwrap();
    assert!(stats.flesch_kincaid_grade.unwrap() < 3.0, "Sehr einfacher Text sollte Grade < 3 haben");
}

#[tokio::test]
async fn test_flesch_kincaid_complex_text() {
    // Akademischer Text mit langen Sätzen und Polysyllabika
    let complex = "The implementation of algorithmic complexity measurement in computational linguistics demonstrates sophisticated theoretical understanding.";
    let stats = compute_text_stats(complex.to_string()).await.unwrap();
    assert!(stats.flesch_kincaid_grade.unwrap() > 10.0, "Komplexer Text sollte Grade > 10 haben");
}

#[tokio::test]
async fn test_syllable_counting() {
    assert_eq!(count_syllables_heuristic("table"), 2);
    assert_eq!(count_syllables_heuristic("make"), 1);
    assert_eq!(count_syllables_heuristic("simple"), 2);
    assert_eq!(count_syllables_heuristic("cat"), 1);
}
