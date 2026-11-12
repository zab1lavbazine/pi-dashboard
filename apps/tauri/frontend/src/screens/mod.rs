pub mod bluetooth;
pub mod computer;
pub mod spotify;
pub mod system;
pub mod volume;
use leptos::prelude::*;

pub fn number(value: Option<f64>, suffix: &str, digits: usize) -> String {
    value
        .filter(|v| v.is_finite())
        .map(|v| format!("{v:.digits$}{suffix}"))
        .unwrap_or_else(|| "—".into())
}
pub fn storage(used: Option<u64>, total: Option<u64>) -> String {
    match (used, total) {
        (Some(u), Some(t)) if t > 0 => format!(
            "{:.1} / {:.1} GiB",
            u as f64 / 1073741824.0,
            t as f64 / 1073741824.0
        ),
        _ => "—".into(),
    }
}
pub fn ratio(used: Option<u64>, total: Option<u64>) -> Option<f64> {
    used.zip(total)
        .filter(|(_, t)| *t > 0)
        .map(|(u, t)| u as f64 / t as f64 * 100.0)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_and_numeric_readings() {
        assert_eq!(number(None, "%", 1), "—");
        assert_eq!(number(Some(f64::NAN), "%", 1), "—");
        assert_eq!(number(Some(f64::INFINITY), "%", 1), "—");
        assert_eq!(number(Some(12.25), "%", 1), "12.2%");
        assert_eq!(storage(Some(1073741824), Some(2147483648)), "1.0 / 2.0 GiB");
        assert_eq!(storage(Some(0), Some(0)), "—");
        assert_eq!(ratio(Some(1), Some(2)), Some(50.0));
        assert_eq!(ratio(Some(1), Some(0)), None);
        assert_eq!(ratio(None, Some(2)), None);
    }
}

#[component]
pub fn Meter(id: &'static str, label: &'static str, value: Signal<Option<f64>>) -> impl IntoView {
    view! { <progress id=id aria-label=label max="100" hidden=move || value.get().is_none_or(|v| !v.is_finite()) value=move || value.get().filter(|v| v.is_finite()).unwrap_or(0.0).clamp(0.0,100.0)></progress> }
}
#[component]
pub fn ErrorNotice(id: &'static str, message: Signal<String>) -> impl IntoView {
    view! { <p id=id class="error notice" role="alert" hidden=move || message.get().is_empty()>{move || message.get()}</p> }
}
