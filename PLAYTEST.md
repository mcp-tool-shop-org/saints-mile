# Saint's Mile — Playtest Protocol

**Build:** v1.0.1
**Date:** 2026-03-22
**Status:** Content frozen. Only bug fixes, readability fixes, balance tuning, and canon-protection fixes allowed.

---

## Round 1: Opening Arc (3-5 players)

### What they play
Prologue through Chapter 2 end. No jump points. No skipping. Start at the title screen, press N.

**Expected path:** prologue_poster → eli_intro → morrow_square → ride_to_arroyo → Glass Arroyo standoff + combat → campfire_choice → return → Cedar Wake (Ch1) → convoy (Ch2) → relay

**Estimated time:** 60-90 minutes

### Setup
- Terminal at least 100x30 (larger is fine, 80x24 minimum supported)
- Event log enabled: `RUST_LOG=saints_mile=info cargo run 2> playtest_log.txt`
- No explanation of the design thesis. No context about age phases, standoff mechanics, or the pressure system. Let the game teach itself.
- Tell them: "Play until you stop wanting to or until it ends. Say what you're thinking when you feel like it."

### Questions after play

**Do not ask these during play. Ask after they stop.**

1. What do you remember about Cedar Wake?
2. What happened at Bitter Cut?
3. Did you notice the standoff doing anything before the fight started?
4. What did you think of Eli?
5. Was there a choice you regretted?
6. Was there a moment where you didn't know what to do?
7. Was there a moment where you stopped reading?
8. Did the game feel different when Galen was 19 versus 34?
9. Would you keep playing?

**Do not prompt answers. Write down what they say, not what you wanted them to say.**

### What to watch for (observer notes)

| Signal | What it means |
|--------|--------------|
| Player slows down at a scene | Engagement or confusion — note which |
| Player speeds through text | Text too long, pacing wrong, or not invested |
| Player tries a locked choice | They wanted it — the lock worked |
| Player ignores a locked choice | They didn't notice it — the lock is invisible |
| Player asks "what does this mean?" at a pressure bar | Readability failure |
| Player asks "what does this mean?" at a choice | Wording failure |
| Player laughs or reacts at Eli's lines | Tone is landing |
| Player doesn't react at Bitter Cut | The moral shift didn't carry through play |
| Player talks about what they chose at the campfire | The choice was hard — this is success |
| Player describes the campfire choice as obvious | The constraint wasn't felt — this needs work |

### What counts as what

| Category | Example | Response |
|----------|---------|----------|
| **Bug** | Crash, broken transition, state not carrying | Fix immediately |
| **Readability** | "I didn't understand what that bar meant" | Fix before Round 2 |
| **Balance** | "I ran out of ammo and couldn't do anything" | Note, fix in tuning pass |
| **Canon drift** | Eli sounds cool instead of divided | Fix before Round 2 |
| **Working as intended** | "That choice was unfair" | Exactly |

---

## Round 2: Mid-game jump points (2-3 players)

### Run after Round 1 fixes are applied.

**Jump point A:** Chapter 5 start (Dust Revival) — 4-person party, crowd pressure
- Tests: crowd screen legibility, Miriam's entry, five-character roster management
- Ask: "What was the crowd encounter like? What did you do?"

**Jump point B:** Chapter 10 start (Deadwater Trial) — reckoning
- Tests: five-bar legibility, testimony sequence, Eli's defining act
- Ask: "What happened at the trial? What did Eli do?"

**Jump point C:** Chapter 13 start (Fifteen Years Gone) — older Galen
- Tests: age-phase transition feel, older menu identity, return arc
- Ask: "Did Galen feel different? How?"

### Setup for jump points
Use the dev quickstart system to load state at chapter boundaries. Players should not know they're at a jump point — present it as "here's a save from partway through."

---

## Round 3: Full run (1-2 players)

### Run after Round 2 fixes.

One complete Prologue-to-Chapter 15 run. No jump points. Let them save and return across sessions if needed.

**The only question that matters:** Does it feel like one life?

---

## Log collection

### Event log
`RUST_LOG=saints_mile=info cargo run 2> playtest_[name]_[date].txt`

Captures: scene transitions, choice executions, state effects applied, combat actions, standoff resolutions, objective evaluations, saves/loads.

### Observer notes
Markdown file per player. Record:
- Time stamps at chapter boundaries
- Moments of visible engagement or disengagement
- Questions they ask unprompted
- Choices they visibly deliberate on
- Things they say out loud

### Post-play interview
Record or transcribe. Keep it short. The nine questions above, plus one open: "What would you tell someone about this game?"

---

## What we're validating

From the build constitution's success tests:

- [ ] Players love Cedar Wake
- [ ] Players feel sick at Bitter Cut
- [ ] Players argue about Eli after the relay
- [ ] Players understand why the poster exists
- [ ] Players want to keep going because the world feels alive and wrong

From the presentation bet:

- [ ] Terminal form feels evocative rather than cheap

From the combat thesis:

- [ ] Players describe the standoff as distinct from standard turn-based combat
- [ ] Players understand that party composition matters

From the pressure systems:

- [ ] Crowd pressure does not feel like combat with a crowd skin
- [ ] Reckoning feels like a battlefield made of testimony
- [ ] Escort feels like fragile motion, not a bar-filling exercise

---

## Content freeze rules

**Allowed:**
- Bug fixes (crashes, broken transitions, state not carrying)
- Readability fixes (bar labels, lock reason wording, scan speed)
- Balance tuning (ammo pressure, nerve thresholds, surge timing)
- Canon-protection fixes (character voice drift, tonal drift, mystery collapse)

**Not allowed:**
- New systems
- New content
- New screens
- New mechanics
- Polishing detours that avoid truth

---

## When playtest is complete

Cut v1.0.0 when:
1. One full no-jump run completes without crashers
2. No broken carry states across chapters
3. No unreadable pressure screens
4. No chapter with placeholder-feeling presentation
5. At least 3/5 success test criteria validated by player behavior (not self-report)

The game is done when a full run feels like one life, one wound, one argument with history — not 16 sections that happen to be good.
