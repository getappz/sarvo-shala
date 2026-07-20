/// FSRS-4.5 scheduler (17-parameter model).
///
/// Ported from the MIT-licensed OpenTutor reference
/// (`apps/api/services/spaced_repetition/fsrs.py`). Weight constants `w0..w16`
/// are taken verbatim from that file's `DEFAULT_W` FSRS-4.5 segment.

const W: [f64; 17] = [
    0.4, 0.6, 2.4, 5.8, // w0-w3: initial stability per rating (Again..Easy)
    4.93, 0.94, // w4-w5: difficulty parameters
    0.86, 0.01, // w6-w7: difficulty delta / mean reversion
    1.49, 0.14, 0.94, // w8-w10: stability parameters
    2.18, 0.05, 0.34, 1.26, // w11-w14: lapse parameters
    0.29, 2.61, // w15-w16: hard penalty / easy bonus
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}

impl Rating {
    fn as_u8(self) -> u8 {
        match self {
            Rating::Again => 1,
            Rating::Hard => 2,
            Rating::Good => 3,
            Rating::Easy => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub stability: f64,
    pub difficulty: f64,
    pub reps: u32,
    pub lapses: u32,
    pub last_review: Option<i64>,
    pub due: Option<i64>,
}

fn clamp_d(d: f64) -> f64 {
    d.clamp(1.0, 10.0)
}

fn initial_stability(rating: u8) -> f64 {
    (W[(rating - 1) as usize]).max(0.1)
}

fn initial_difficulty(rating: u8) -> f64 {
    clamp_d(W[4] - (rating as f64 - 3.0) * W[5])
}

fn next_difficulty(d: f64, rating: u8) -> f64 {
    let delta = -W[6] * (rating as f64 - 3.0);
    let mut d_new = d + delta * (10.0 - d) / 9.0;
    d_new = W[7] * initial_difficulty(4) + (1.0 - W[7]) * d_new;
    clamp_d(d_new)
}

fn retrievability(elapsed_days: f64, stability: f64) -> f64 {
    if stability <= 0.0 {
        return 0.0;
    }
    1.0 / (1.0 + elapsed_days / (9.0 * stability))
}

fn next_stability(d: f64, s: f64, r: f64, rating: u8) -> f64 {
    if rating == 1 {
        // Again — lapse
        let v = W[11] * d.powf(-W[12]) * (s + 1.0).powf(W[13]) - 1.0;
        return (v * ((1.0 - r) * W[14]).exp()).max(0.1);
    }
    let hard_penalty = if rating == 2 { W[15] } else { 1.0 };
    let easy_bonus = if rating == 4 { W[16] } else { 1.0 };
    let new_s = s
        * (1.0
            + W[8].exp()
                * (11.0 - d)
                * s.powf(-W[9])
                * (((1.0 - r) * W[10]).exp() - 1.0)
                * hard_penalty
            * easy_bonus);
    new_s.max(0.1)
}

const DAY: f64 = 86400.0;

/// Process a review. `now` is a unix timestamp in seconds.
pub fn review(card: &Card, rating: Rating, now: i64) -> Card {
    let rating = rating.as_u8();
    let elapsed_days = match card.last_review {
        Some(lr) => (((now - lr) as f64) / DAY).max(0.0),
        None => 0.0,
    };

    let mut new_card = card.clone();

    if card.reps == 0 {
        new_card.difficulty = initial_difficulty(rating);
        new_card.stability = initial_stability(rating);
    } else {
        let r = retrievability(elapsed_days, card.stability);
        let new_d = next_difficulty(card.difficulty, rating);
        let new_s = if elapsed_days < 1.0 && card.reps > 0 {
            // Same-day review: prevent stability inflation.
            (card.stability * (rating as f64 - 3.0 + 1.0).exp()).max(0.1)
        } else {
            next_stability(card.difficulty, card.stability, r, rating)
        };
        new_card.difficulty = new_d;
        new_card.stability = new_s;
        if rating == 1 {
            new_card.lapses += 1;
        }
    }

    new_card.reps += 1;
    new_card.last_review = Some(now);

    let scheduled_days = if rating == 1 {
        1
    } else {
        new_card.stability.round().max(1.0) as i64
    };
    new_card.due = Some(now + scheduled_days * DAY as i64);

    new_card
}

/// Interval (days) until 90% retention — for FSRS-4.5 that is ~stability.
pub fn next_interval_days(stability: f64) -> f64 {
    stability
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_card() -> Card {
        Card {
            stability: 0.0,
            difficulty: 0.0,
            reps: 0,
            lapses: 0,
            last_review: None,
            due: None,
        }
    }

    #[test]
    fn first_review_sets_positive_stability() {
        let c = review(&new_card(), Rating::Good, 1_000_000);
        assert!(c.stability > 0.0);
        assert_eq!(c.reps, 1);
        assert!(c.due.is_some());
    }

    #[test]
    fn good_grows_interval() {
        let c0 = new_card();
        let c1 = review(&c0, Rating::Good, 1_000_000);
        let interval1 = (c1.due.unwrap() - 1_000_000) as f64 / DAY;
        let c2 = review(&c1, Rating::Good, c1.due.unwrap());
        let interval2 = (c2.due.unwrap() - c1.due.unwrap()) as f64 / DAY;
        assert!(interval2 > interval1, "expected {interval2} > {interval1}");
    }

    #[test]
    fn again_increments_lapses_and_shrinks_stability() {
        let c1 = review(&new_card(), Rating::Good, 1_000_000);
        let c2 = review(&c1, Rating::Again, c1.due.unwrap());
        assert_eq!(c2.lapses, 1);
        assert!(c2.stability < c1.stability, "stability should shrink on Again");
    }

    #[test]
    fn difficulty_bounded() {
        let mut c = new_card();
        let mut now = 1_000_000;
        let ratings = [Rating::Good, Rating::Easy, Rating::Hard, Rating::Again];
        for i in 0..50 {
            let r = ratings[i % ratings.len()];
            c = review(&c, r, now);
            assert!(
                (1.0..=10.0).contains(&c.difficulty),
                "difficulty {} out of bounds",
                c.difficulty
            );
            now += (c.stability.round().max(1.0) as i64) * DAY as i64;
        }
    }
}
