//! Tests for pressure/types.rs — pressure encounter definitions and state.

use saints_mile::types::*;
use saints_mile::pressure::types::*;
use saints_mile::scene::types::{Condition, StateEffect};

#[test]
fn escort_pressure_has_cargo() {
    let p = PressureEncounter {
        id: "test_escort".to_string(),
        pressure_type: PressureType::Escort {
            cargo: vec![
                CargoItem {
                    id: "medical_supplies".to_string(),
                    name: "Medical Supplies".to_string(),
                    integrity: 100,
                    max_integrity: 100,
                    loss_effect: vec![],
                },
            ],
        },
        pressure_bars: vec![
            PressureBar {
                id: "cargo_safety".to_string(),
                label: "Cargo Safety".to_string(),
                current: 100,
                max: 100,
                fail_at: 0,
                visible: true,
            },
        ],
        party_actions: vec![],
        success_threshold: PressureCondition::AllBarsAboveFail,
        failure_threshold: PressureCondition::BarReached {
            bar_id: "cargo_safety".to_string(),
            threshold: 0,
        },
        outcome_effects: vec![],
    };

    match &p.pressure_type {
        PressureType::Escort { cargo } => {
            assert_eq!(cargo.len(), 1);
            assert_eq!(cargo[0].integrity, 100);
        }
        _ => panic!("expected Escort"),
    }
    assert_eq!(p.pressure_bars.len(), 1);
}

#[test]
fn crowd_pressure_has_ringleaders() {
    let p = PressureEncounter {
        id: "test_crowd".to_string(),
        pressure_type: PressureType::Crowd {
            collective_nerve: 50,
            ringleaders: vec!["loud_man".to_string(), "angry_woman".to_string()],
        },
        pressure_bars: vec![],
        party_actions: vec![],
        success_threshold: PressureCondition::AllBarsAboveFail,
        failure_threshold: PressureCondition::TimeExpired,
        outcome_effects: vec![],
    };

    match &p.pressure_type {
        PressureType::Crowd { collective_nerve, ringleaders } => {
            assert_eq!(*collective_nerve, 50);
            assert_eq!(ringleaders.len(), 2);
        }
        _ => panic!("expected Crowd"),
    }
}

#[test]
fn transmission_race_has_channels() {
    let p = PressureEncounter {
        id: "test_transmission".to_string(),
        pressure_type: PressureType::TransmissionRace {
            channels: vec![
                Channel {
                    id: "wire_a".to_string(),
                    name: "Main Wire".to_string(),
                    controlled_by: ChannelOwner::Party,
                    relay_points: vec!["junction_a".to_string()],
                },
                Channel {
                    id: "wire_b".to_string(),
                    name: "Branch Wire".to_string(),
                    controlled_by: ChannelOwner::Enemy,
                    relay_points: vec!["junction_b".to_string()],
                },
            ],
            time_remaining: 10,
        },
        pressure_bars: vec![],
        party_actions: vec![],
        success_threshold: PressureCondition::FlagSet(FlagId::new("dispatch_sent")),
        failure_threshold: PressureCondition::TimeExpired,
        outcome_effects: vec![],
    };

    match &p.pressure_type {
        PressureType::TransmissionRace { channels, time_remaining } => {
            assert_eq!(channels.len(), 2);
            assert_eq!(*time_remaining, 10);
            assert_eq!(channels[0].controlled_by, ChannelOwner::Party);
            assert_eq!(channels[1].controlled_by, ChannelOwner::Enemy);
        }
        _ => panic!("expected TransmissionRace"),
    }
}

#[test]
fn pressure_bar_tracks_state() {
    let mut bar = PressureBar {
        id: "test_bar".to_string(),
        label: "Test".to_string(),
        current: 80,
        max: 100,
        fail_at: 20,
        visible: true,
    };
    assert!(bar.current > bar.fail_at, "bar should start above fail threshold");
    bar.current -= 30;
    assert_eq!(bar.current, 50);
    bar.current -= 40;
    assert!(bar.current < bar.fail_at, "bar should be below fail threshold now");
}

#[test]
fn pressure_action_has_delta() {
    let action = PressureAction {
        id: "brace".to_string(),
        label: "Brace the wagon".to_string(),
        description: "Rosa holds the line.".to_string(),
        target_bar: "cargo_safety".to_string(),
        delta: 10,
        conditions: vec![],
    };
    assert_eq!(action.delta, 10);
    assert_eq!(action.target_bar, "cargo_safety");
}

#[test]
fn public_reckoning_has_five_bars() {
    let p = PressureType::PublicReckoning {
        room_credibility: 50,
        crowd_nerve: 40,
        witness_integrity: 30,
        evidence_continuity: 80,
        procedural_control: 60,
    };
    match p {
        PressureType::PublicReckoning {
            room_credibility, crowd_nerve, witness_integrity,
            evidence_continuity, procedural_control,
        } => {
            assert_eq!(room_credibility, 50);
            assert_eq!(crowd_nerve, 40);
            assert_eq!(witness_integrity, 30);
            assert_eq!(evidence_continuity, 80);
            assert_eq!(procedural_control, 60);
        }
        _ => unreachable!(),
    }
}
