use crate::ai::classify::ResponseQuality;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocState {
    Probe,
    Clarify,
    Confront,
    Scaffold,
    Confirm,
}

impl fmt::Display for SocState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SocState::Probe => "Probe",
            SocState::Clarify => "Clarify",
            SocState::Confront => "Confront",
            SocState::Scaffold => "Scaffold",
            SocState::Confirm => "Confirm",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone)]
pub struct SocraticEngine {
    pub state: SocState,
    pub scaffold_turns: u32,
}

pub const MAX_SCAFFOLD_TURNS: u32 = 4;

fn next_state(current: SocState, q: ResponseQuality) -> SocState {
    match (current, q) {
        (SocState::Probe, ResponseQuality::Correct) => SocState::Confirm,
        (SocState::Probe, ResponseQuality::Partial) => SocState::Clarify,
        (SocState::Probe, ResponseQuality::Wrong) => SocState::Confront,
        (SocState::Probe, ResponseQuality::Confused) => SocState::Clarify,
        (SocState::Probe, ResponseQuality::NoResponse) => SocState::Scaffold,

        (SocState::Clarify, ResponseQuality::Correct) => SocState::Confirm,
        (SocState::Clarify, ResponseQuality::Partial) => SocState::Scaffold,
        (SocState::Clarify, ResponseQuality::Wrong) => SocState::Confront,
        (SocState::Clarify, ResponseQuality::Confused) => SocState::Scaffold,
        (SocState::Clarify, ResponseQuality::NoResponse) => SocState::Scaffold,

        (SocState::Confront, ResponseQuality::Correct) => SocState::Confirm,
        (SocState::Confront, ResponseQuality::Partial) => SocState::Clarify,
        (SocState::Confront, ResponseQuality::Wrong) => SocState::Scaffold,
        (SocState::Confront, ResponseQuality::Confused) => SocState::Scaffold,
        (SocState::Confront, ResponseQuality::NoResponse) => SocState::Scaffold,

        (SocState::Scaffold, ResponseQuality::Correct) => SocState::Confirm,
        (SocState::Scaffold, ResponseQuality::Partial) => SocState::Clarify,
        (SocState::Scaffold, ResponseQuality::Wrong) => SocState::Scaffold,
        (SocState::Scaffold, ResponseQuality::Confused) => SocState::Scaffold,
        (SocState::Scaffold, ResponseQuality::NoResponse) => SocState::Scaffold,

        (SocState::Confirm, ResponseQuality::Correct) => SocState::Probe,
        (SocState::Confirm, ResponseQuality::Partial) => SocState::Probe,
        (SocState::Confirm, ResponseQuality::Wrong) => SocState::Clarify,
        (SocState::Confirm, ResponseQuality::Confused) => SocState::Clarify,
        (SocState::Confirm, ResponseQuality::NoResponse) => SocState::Clarify,
    }
}

impl SocraticEngine {
    pub fn new() -> Self {
        SocraticEngine {
            state: SocState::Probe,
            scaffold_turns: 0,
        }
    }

    /// Static transition table on (state, quality); increments/resets
    /// scaffold_turns; Scaffold beyond MAX_SCAFFOLD_TURNS → Confirm escape hatch.
    pub fn transition(&mut self, q: ResponseQuality) -> SocState {
        let next = next_state(self.state, q);
        if matches!(next, SocState::Scaffold) {
            if matches!(self.state, SocState::Scaffold) {
                self.scaffold_turns += 1;
            } else {
                self.scaffold_turns = 0;
            }
            if self.scaffold_turns > MAX_SCAFFOLD_TURNS {
                self.scaffold_turns = 0;
                self.state = SocState::Confirm;
                return self.state;
            }
        } else {
            self.scaffold_turns = 0;
        }
        self.state = next;
        self.state
    }

    /// Per-state prompt directive used to build the LLM system prompt.
    pub fn directive(&self) -> &'static str {
        match self.state {
            SocState::Probe => {
                "Ask one open question that makes the student state what they already know. Never reveal the answer."
            }
            SocState::Clarify => {
                "Ask the student to restate their idea more precisely. Point at the fuzzy part with a question."
            }
            SocState::Confront => {
                "Present a counter-example that contradicts the student's claim and ask them to reconcile it."
            }
            SocState::Scaffold => {
                "Break the problem into one smaller step and ask only that step."
            }
            SocState::Confirm => {
                "Affirm what is now correct, summarize the concept in two sentences, then ask a fresh probe question."
            }
        }
    }
}

impl Default for SocraticEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::classify::ResponseQuality::*;

    #[test]
    fn table_driven_all_25_pairs() {
        let table: [((SocState, ResponseQuality), SocState); 25] = [
            ((SocState::Probe, Correct), SocState::Confirm),
            ((SocState::Probe, Partial), SocState::Clarify),
            ((SocState::Probe, Wrong), SocState::Confront),
            ((SocState::Probe, Confused), SocState::Clarify),
            ((SocState::Probe, NoResponse), SocState::Scaffold),
            ((SocState::Clarify, Correct), SocState::Confirm),
            ((SocState::Clarify, Partial), SocState::Scaffold),
            ((SocState::Clarify, Wrong), SocState::Confront),
            ((SocState::Clarify, Confused), SocState::Scaffold),
            ((SocState::Clarify, NoResponse), SocState::Scaffold),
            ((SocState::Confront, Correct), SocState::Confirm),
            ((SocState::Confront, Partial), SocState::Clarify),
            ((SocState::Confront, Wrong), SocState::Scaffold),
            ((SocState::Confront, Confused), SocState::Scaffold),
            ((SocState::Confront, NoResponse), SocState::Scaffold),
            ((SocState::Scaffold, Correct), SocState::Confirm),
            ((SocState::Scaffold, Partial), SocState::Clarify),
            ((SocState::Scaffold, Wrong), SocState::Scaffold),
            ((SocState::Scaffold, Confused), SocState::Scaffold),
            ((SocState::Scaffold, NoResponse), SocState::Scaffold),
            ((SocState::Confirm, Correct), SocState::Probe),
            ((SocState::Confirm, Partial), SocState::Probe),
            ((SocState::Confirm, Wrong), SocState::Clarify),
            ((SocState::Confirm, Confused), SocState::Clarify),
            ((SocState::Confirm, NoResponse), SocState::Clarify),
        ];
        for ((st, q), expected) in table {
            let mut e = SocraticEngine {
                state: st,
                scaffold_turns: 0,
            };
            assert_eq!(e.transition(q), expected, "({st:?}, {q:?})");
        }
    }

    #[test]
    fn scaffold_escape_hatch() {
        let mut e = SocraticEngine::new();
        e.transition(NoResponse); // -> Scaffold
        assert_eq!(e.state, SocState::Scaffold);
        for _ in 0..5 {
            let s = e.transition(Wrong); // stays Scaffold until escape
            if s == SocState::Confirm {
                assert_eq!(e.scaffold_turns, 0);
                return;
            }
        }
        panic!("expected escape to Confirm after 5 Wrong in Scaffold");
    }

    #[test]
    fn confirm_cycles_back_to_probe() {
        let mut e = SocraticEngine::new();
        e.state = SocState::Confirm;
        assert_eq!(e.transition(Correct), SocState::Probe);
    }
}
