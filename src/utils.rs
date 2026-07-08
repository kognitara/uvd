use crate::locales::LOCALES;
use crossterm::execute;
use crossterm::style::{Print, Stylize};
use crossterm::terminal::size;
use fluent_templates::Loader;
use std::io::stdout;
use unic_langid::LanguageIdentifier;

pub fn ok(lang: &LanguageIdentifier, key: &str) {
    let (x, _) = size().expect("failed to get term size");
    let description: String = LOCALES.try_lookup(lang, key).expect("no key found");
    let star: &str = " * ";
    let ok: &str = " ok ";
    let padding: u16 = x
        - description.chars().count() as u16
        - ok.chars().count() as u16
        - star.chars().count() as u16
        - 2;
    execute!(
        stdout(),
        Print(" * ".green().bold()),
        Print(description.white()),
        Print(" ".repeat(padding as usize)),
        Print("[ ".white().bold()),
        Print("ok".green().bold()),
        Print(" ]".white().bold()),
    )
    .expect("failed to print");
}
pub fn ko(lang: &LanguageIdentifier, key: &str) {
    let (x, _) = size().expect("failed to get term size");
    let description: String = LOCALES.try_lookup(lang, key).unwrap_or_default();
    let star: &str = " * ";
    let ok: &str = " ko ";
    let padding: u16 = x
        - description.chars().count() as u16
        - ok.chars().count() as u16
        - star.chars().count() as u16
        - 2;
    execute!(
        stdout(),
        Print(" ! ".red().bold()),
        Print(description.white()),
        Print(" ".repeat(padding as usize)),
        Print("[ ".white().bold()),
        Print("ko".red().bold()),
        Print(" ]".white().bold()),
    )
    .expect("faield to print");
}

pub fn tt(lang: &LanguageIdentifier, key: &str) -> String {
    LOCALES.try_lookup(lang, key).unwrap_or_default()
}
