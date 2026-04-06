//! Tests for combat/wounds.rs — wound definitions and triage system.

use saints_mile::combat::wounds;

#[test]
fn gunshot_wound_penalties() {
    let w = wounds::gunshot_wound();
    assert_eq!(w.id.0, "gunshot");
    assert!(w.treatable);
    assert_eq!(w.penalties.len(), 2);
    // Should reduce accuracy and speed
    let accuracy_penalty = w.penalties.iter().find(|p| p.stat == "accuracy").unwrap();
    assert!(accuracy_penalty.amount < 0, "gunshot should reduce accuracy");
    let speed_penalty = w.penalties.iter().find(|p| p.stat == "speed").unwrap();
    assert!(speed_penalty.amount < 0, "gunshot should reduce speed");
}

#[test]
fn nerve_shock_targets_nerve() {
    let w = wounds::nerve_shock();
    assert_eq!(w.id.0, "nerve_shock");
    assert!(w.treatable);
    assert_eq!(w.penalties.len(), 1);
    assert_eq!(w.penalties[0].stat, "nerve");
    assert_eq!(w.penalties[0].amount, -8, "nerve shock is the heaviest nerve penalty");
}

#[test]
fn exhaustion_affects_all_stats() {
    let w = wounds::exhaustion();
    assert_eq!(w.id.0, "exhaustion");
    assert_eq!(w.penalties.len(), 3);
    let stats: Vec<&str> = w.penalties.iter().map(|p| p.stat.as_str()).collect();
    assert!(stats.contains(&"accuracy"));
    assert!(stats.contains(&"speed"));
    assert!(stats.contains(&"nerve"));
}

#[test]
fn triage_heals_treatable_wounds() {
    let wounds_list = vec![
        wounds::gunshot_wound(),
        wounds::blunt_trauma(),
    ];
    let result = wounds::triage(&wounds_list, false);
    assert_eq!(result.healed.len(), 2, "both wounds are treatable");
    assert!(result.hp_restored > 0);
    assert!(result.nerve_restored > 0, "blunt_trauma has nerve penalty");
    assert_eq!(result.time_cost, 2, "quick triage = 1 per wound");
}

#[test]
fn thorough_triage_restores_more_costs_more() {
    let wounds_list = vec![wounds::gunshot_wound()];
    let quick = wounds::triage(&wounds_list, false);
    let thorough = wounds::triage(&wounds_list, true);

    assert!(thorough.hp_restored > quick.hp_restored,
        "thorough should restore more HP: {} vs {}", thorough.hp_restored, quick.hp_restored);
    assert!(thorough.time_cost > quick.time_cost,
        "thorough should cost more time");
}

#[test]
fn triage_empty_wounds_returns_nothing() {
    let result = wounds::triage(&[], true);
    assert!(result.healed.is_empty());
    assert_eq!(result.hp_restored, 0);
    assert_eq!(result.nerve_restored, 0);
    assert_eq!(result.time_cost, 0);
}
