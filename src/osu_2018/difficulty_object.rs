use super::OsuObject;

pub(crate) struct DifficultyObject<'h> {
    pub(crate) base: &'h OsuObject,
    pub(crate) jump_dist: f32,
    pub(crate) travel_dist: f32,
    pub(crate) delta: f32,
    pub(crate) strain_time: f32,
}

impl<'h> DifficultyObject<'h> {
    pub(crate) fn new(
        base: &'h OsuObject,
        prev: &OsuObject,
        clock_rate: f32,
        scaling_factor: f32,
    ) -> Self {
        let delta = (base.time - prev.time) / clock_rate;

        let jump_dist = if base.is_spinner() {
            0.0
        } else {
            (base.pos * scaling_factor - prev.end_pos * scaling_factor).length()
        };

        let travel_dist = if base.is_slider {
            prev.travel_dist.unwrap_or(0.0) * scaling_factor
        } else {
            0.0
        };

        Self {
            base,
            jump_dist,
            travel_dist,
            delta,
            strain_time: delta.max(50.0),
        }
    }
}
