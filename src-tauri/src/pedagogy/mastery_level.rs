#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MasteryLevel {
    NotStarted,
    Learning,
    Practiced,
    Mastered,
}

pub fn level_for(attempts: i64, p_known: f64) -> MasteryLevel {
    if attempts == 0 {
        MasteryLevel::NotStarted
    } else if p_known < 0.5 {
        MasteryLevel::Learning
    } else if p_known < 0.85 {
        MasteryLevel::Practiced
    } else {
        MasteryLevel::Mastered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_attempts_is_not_started() {
        assert_eq!(level_for(0, 0.2), MasteryLevel::NotStarted);
        assert_eq!(level_for(0, 0.9), MasteryLevel::NotStarted);
    }

    #[test]
    fn low_p_known_is_learning() {
        assert_eq!(level_for(1, 0.2), MasteryLevel::Learning);
        assert_eq!(level_for(5, 0.49), MasteryLevel::Learning);
    }

    #[test]
    fn moderate_p_known_is_practiced() {
        assert_eq!(level_for(1, 0.5), MasteryLevel::Practiced);
        assert_eq!(level_for(3, 0.84), MasteryLevel::Practiced);
    }

    #[test]
    fn high_p_known_is_mastered() {
        assert_eq!(level_for(1, 0.85), MasteryLevel::Mastered);
        assert_eq!(level_for(10, 0.99), MasteryLevel::Mastered);
    }
}
