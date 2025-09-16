pub fn hot_score(score: i32, age_hours: f64) -> f64 {
    let age = age_hours.max(1.0);
    (score as f64) / age.powf(1.5)
}