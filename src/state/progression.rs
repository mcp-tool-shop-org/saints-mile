//! Chapter progression validation — prerequisite checks for chapter entry.
//!
//! Guards the dev/quickstart system and future chapter-select features
//! by enforcing that the player has reached the correct point in the story.

use super::types::GameState;

/// The canonical chapter order. Each entry is a chapter ID as stored in GameState.
pub const CHAPTER_ORDER: &[&str] = &[
    "prologue",
    "cedar_wake",
    "saints_mile_convoy",
    "black_willow",
    "ropehouse_blood",
    "dust_revival",
    "fuse_country",
    "iron_ledger",
    "burned_mission",
    "long_wire",
    "deadwater_trial",
    "breakwater_junction",
    "names_in_dust",
    "fifteen_years_gone",
    "old_friends",
    "saints_mile_again",
];

/// Alternative chapter IDs that map to the canonical names.
/// Some code paths use "ch1", "ch2", etc. instead of the full name.
fn normalize_chapter_id(id: &str) -> &str {
    match id {
        "ch1" => "cedar_wake",
        "ch2" => "saints_mile_convoy",
        "ch3" => "black_willow",
        "ch4" => "ropehouse_blood",
        "ch5" => "dust_revival",
        "ch6" => "fuse_country",
        "ch7" => "iron_ledger",
        "ch8" => "burned_mission",
        "ch9" => "long_wire",
        "ch10" => "deadwater_trial",
        "ch11" => "breakwater_junction",
        "ch12" => "names_in_dust",
        "ch13" => "fifteen_years_gone",
        "ch14" => "old_friends",
        "ch15" => "saints_mile_again",
        other => other,
    }
}

/// Returns the index of a chapter in the canonical order, or None if unknown.
fn chapter_index(chapter: &str) -> Option<usize> {
    let normalized = normalize_chapter_id(chapter);
    CHAPTER_ORDER.iter().position(|&c| c == normalized)
}

/// Check whether the player's current state allows entering the given chapter.
///
/// Returns `true` if:
/// - The target chapter is the prologue (always allowed).
/// - The player's current chapter is at or one step before the target chapter.
///   (i.e., the player has already reached the prerequisite chapter.)
///
/// Returns `false` if:
/// - The target chapter is unknown.
/// - The player's current chapter is more than one step behind the target.
///   (i.e., the player would be skipping chapters.)
pub fn can_enter_chapter(state: &GameState, chapter: &str) -> bool {
    let target = match chapter_index(chapter) {
        Some(idx) => idx,
        None => return false, // unknown chapter
    };

    // Prologue is always reachable
    if target == 0 {
        return true;
    }

    let current = match chapter_index(&state.chapter.0) {
        Some(idx) => idx,
        None => return false, // current chapter unknown — block
    };

    // Allow entering the current chapter or the next one
    // (current >= target - 1) means the player is on the prerequisite or beyond
    current >= target.saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::GameState;
    use crate::types::ChapterId;

    #[test]
    fn prologue_always_reachable() {
        let state = GameState::new_game();
        assert!(can_enter_chapter(&state, "prologue"));
    }

    #[test]
    fn can_enter_next_chapter_from_current() {
        let mut state = GameState::new_game();
        state.chapter = ChapterId::new("prologue");
        assert!(can_enter_chapter(&state, "cedar_wake"));
    }

    #[test]
    fn can_reenter_current_chapter() {
        let mut state = GameState::new_game();
        state.chapter = ChapterId::new("cedar_wake");
        assert!(can_enter_chapter(&state, "cedar_wake"));
    }

    #[test]
    fn cannot_skip_chapters() {
        let state = GameState::new_game(); // prologue
        assert!(!can_enter_chapter(&state, "black_willow")); // ch3, skipping ch1+ch2
    }

    #[test]
    fn cannot_skip_to_endgame() {
        let state = GameState::new_game();
        assert!(!can_enter_chapter(&state, "saints_mile_again"));
    }

    #[test]
    fn numeric_chapter_ids_work() {
        let mut state = GameState::new_game();
        state.chapter = ChapterId::new("ch1"); // = cedar_wake
        assert!(can_enter_chapter(&state, "ch2")); // = saints_mile_convoy
        assert!(can_enter_chapter(&state, "saints_mile_convoy"));
        assert!(!can_enter_chapter(&state, "ch4")); // too far ahead
    }

    #[test]
    fn late_game_progression() {
        let mut state = GameState::new_game();
        state.chapter = ChapterId::new("ch14"); // old_friends
        assert!(can_enter_chapter(&state, "saints_mile_again"));
        assert!(can_enter_chapter(&state, "ch15"));
        assert!(can_enter_chapter(&state, "old_friends")); // current
    }

    #[test]
    fn unknown_chapter_blocked() {
        let state = GameState::new_game();
        assert!(!can_enter_chapter(&state, "nonexistent_chapter"));
    }

    #[test]
    fn midgame_cannot_go_back_requirement() {
        // This test verifies the function checks forward progression.
        // A player at ch7 can enter ch7 or ch8, but the function
        // also allows re-entering earlier chapters (which is fine —
        // the guard is about prerequisites, not preventing replay).
        let mut state = GameState::new_game();
        state.chapter = ChapterId::new("iron_ledger"); // ch7
        assert!(can_enter_chapter(&state, "burned_mission")); // ch8 — next
        assert!(can_enter_chapter(&state, "iron_ledger"));     // ch7 — current
        assert!(can_enter_chapter(&state, "prologue"));        // always ok
        assert!(can_enter_chapter(&state, "cedar_wake"));      // past — ok
        assert!(!can_enter_chapter(&state, "deadwater_trial")); // ch10 — too far
    }

    #[test]
    fn full_chapter_order_length() {
        assert_eq!(CHAPTER_ORDER.len(), 16);
    }
}
