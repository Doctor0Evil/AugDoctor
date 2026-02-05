use crate::discipline::evolution_turn::EvolutionTurnDiscipline;

pub fn can_consume_turn(
    state: &DailyTurnState,
    now_utc: chrono::DateTime<chrono::Utc>,
    discipline: &EvolutionTurnDiscipline,
) -> bool {
    if state.turns_used >= discipline.max_turns_per_day {
        return false;
    }
    let seconds_since_last = /* existing logic */;
    if seconds_since_last < discipline.min_seconds_between_turns {
        return false;
    }
    true
}
