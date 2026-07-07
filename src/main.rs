use fluent_templates::{Loader, static_loader};
use sys_locale::get_locale;
use unic_langid::langid;

static_loader! {
  pub static LOCALES = {
        locales: "./locales", // Il va chercher les dossiers enfants
        fallback_language: "en-US",
        // ...
    };
}
fn main() {
    let locale = get_locale().unwrap_or_else(|| "en-US".to_string());

    let lang = locale.parse().unwrap_or(langid!("en-US"));
    println!(
        "{}",
        LOCALES.try_lookup(&lang, "hello-world").unwrap_or_default()
    );
}
