//! Tests for state/reassembly.rs — ally return modes and final approach validation.

use saints_mile::types::*;
use saints_mile::state::reassembly::*;

#[test]
fn chapter_14_reassembly_has_five_allies() {
    let returns = chapter_14_reassembly();
    assert_eq!(returns.len(), 5);

    let ids: Vec<&str> = returns.iter().map(|r| r.character.0.as_str()).collect();
    assert!(ids.contains(&"eli"));
    assert!(ids.contains(&"ada"));
    assert!(ids.contains(&"rosa"));
    assert!(ids.contains(&"miriam"));
    assert!(ids.contains(&"lucien"));
}

#[test]
fn return_modes_are_diverse() {
    let returns = chapter_14_reassembly();

    // Body returns
    let body_count = returns.iter().filter(|r| r.mode == ReturnMode::Body).count();
    assert!(body_count >= 2, "at least Eli, Ada, Miriam return in body");

    // Conditional returns
    let conditional_count = returns.iter().filter(|r| r.mode == ReturnMode::Conditional).count();
    assert!(conditional_count >= 1, "at least Rosa or Lucien are conditional");

    // Rosa is conditional — land and distrust
    let rosa = returns.iter().find(|r| r.character.0 == "rosa").unwrap();
    assert_eq!(rosa.mode, ReturnMode::Conditional);

    // Lucien is conditional — ugly but possibly necessary
    let lucien = returns.iter().find(|r| r.character.0 == "lucien").unwrap();
    assert_eq!(lucien.mode, ReturnMode::Conditional);
}

#[test]
fn can_approach_with_enough_body_returns() {
    let returns = chapter_14_reassembly();
    assert!(can_approach_saints_mile(&returns),
        "default reassembly should have enough returning allies");
}

#[test]
fn cannot_approach_with_too_few_body_returns() {
    // Only 2 body/conditional returns — not enough
    let returns = vec![
        AllyReturn {
            character: CharacterId::new("eli"),
            mode: ReturnMode::Body,
            truth_carried: "test".to_string(),
            change: "test".to_string(),
            tension: "test".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("ada"),
            mode: ReturnMode::Refusal,
            truth_carried: "test".to_string(),
            change: "test".to_string(),
            tension: "test".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("rosa"),
            mode: ReturnMode::MemoryOnly,
            truth_carried: "test".to_string(),
            change: "test".to_string(),
            tension: "test".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("miriam"),
            mode: ReturnMode::Testimony,
            truth_carried: "test".to_string(),
            change: "test".to_string(),
            tension: "test".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("lucien"),
            mode: ReturnMode::Body,
            truth_carried: "test".to_string(),
            change: "test".to_string(),
            tension: "test".to_string(),
        },
    ];
    assert!(!can_approach_saints_mile(&returns),
        "only 2 body/conditional returns is not enough for the final approach");
}

#[test]
fn each_ally_carries_truth_and_tension() {
    let returns = chapter_14_reassembly();
    for r in &returns {
        assert!(!r.truth_carried.is_empty(),
            "{} must carry a truth", r.character.0);
        assert!(!r.change.is_empty(),
            "{} must have changed", r.character.0);
        assert!(!r.tension.is_empty(),
            "{} must bring unresolved tension", r.character.0);
    }
}

#[test]
fn boundary_exactly_three_body_returns() {
    let returns = vec![
        AllyReturn {
            character: CharacterId::new("eli"),
            mode: ReturnMode::Body,
            truth_carried: "t".to_string(),
            change: "c".to_string(),
            tension: "n".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("ada"),
            mode: ReturnMode::Body,
            truth_carried: "t".to_string(),
            change: "c".to_string(),
            tension: "n".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("rosa"),
            mode: ReturnMode::Conditional,
            truth_carried: "t".to_string(),
            change: "c".to_string(),
            tension: "n".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("miriam"),
            mode: ReturnMode::Refusal,
            truth_carried: "t".to_string(),
            change: "c".to_string(),
            tension: "n".to_string(),
        },
    ];
    // 3 body/conditional = exactly at threshold
    assert!(can_approach_saints_mile(&returns),
        "exactly 3 body/conditional returns should be sufficient");
}
