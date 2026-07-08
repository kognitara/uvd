use crate::teams::{fetch_developers_list, fetch_managers_list, fetch_reviewers_list};
use crate::utils::{ko, ok, tt};
use inquire::{Confirm, Editor, Select, Text};
use lettre::{Message, SmtpTransport, Transport};
use std::path::Path;
use std::process::ExitCode;
use unic_langid::LanguageIdentifier;

pub async fn submit_archive(
    lang: &LanguageIdentifier,
    archive_path: &str,
    level: &i32,
) -> ExitCode {
    if !Path::new(archive_path).exists() {
        ko(lang, "archive-not-found");
        return ExitCode::FAILURE;
    }
    let result: (String, String, Vec<String>, Vec<String>);
    match level {
        0 => {
            // Un développeur (0) envoie à un reviewer (1)
            let devs = fetch_developers_list().await.unwrap_or_default();
            let revs = fetch_reviewers_list().await.unwrap_or_default();
            let who_question = tt(lang, "who-developper-are-you");
            let reviewer_question = tt(lang, "who-are-your-reviewer");
            result = (who_question, reviewer_question, devs, revs);
        }
        1 => {
            let revs = fetch_reviewers_list().await.unwrap_or_default();
            let mans = fetch_managers_list().await.unwrap_or_default();
            let who_question = tt(lang, "who-reviewer-are-you");
            let reviewer_question = tt(lang, "who-are-your-manager");
            result = (who_question, reviewer_question, revs, mans);
        }
        _ => {
            eprintln!("must be inferior to 2.");
            return ExitCode::FAILURE;
        }
    };
    let from = Select::new(result.0.as_str(), result.2).prompt();
    let from_text = match from {
        Ok(text) => text,
        Err(_) => {
            ko(lang, "submit-cancelled");
            return ExitCode::FAILURE;
        }
    };
    let to = Select::new(result.1.as_str(), result.3).prompt();
    let to_text = match to {
        Ok(text) => text,
        Err(_) => {
            ko(lang, "submit-cancelled");
            return ExitCode::FAILURE;
        }
    };

    let subject = Text::new("Subject:").prompt();
    let subject_text = match subject {
        Ok(text) => text,
        Err(_) => {
            ko(lang, "submit-cancelled");
            return ExitCode::FAILURE;
        }
    };

    let reason = Editor::new("Please explain why you submit archive?").prompt();
    let reason_text = match reason {
        Ok(text) => text,
        Err(_) => {
            ko(lang, "submit-cancelled");
            return ExitCode::FAILURE;
        }
    };

    let confirm = Confirm::new("Are you sure to send your work to the reviewer?")
        .with_default(false)
        .prompt();

    if let Ok(false) | Err(_) = confirm {
        ko(lang, "submit-cancelled");
        return ExitCode::FAILURE;
    }

    let email = Message::builder()
        .from(from_text.parse().unwrap())
        .to(to_text.parse().unwrap())
        .subject(subject_text)
        .body(reason_text)
        .unwrap();

    let mailer = SmtpTransport::starttls_relay("127.0.0.1").unwrap().build();

    match mailer.send(&email) {
        Ok(_) => {
            ok(lang, "submit-success");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Erreur SMTP : {:?}", e);
            ko(lang, "submit-failure");
            ExitCode::FAILURE
        }
    }
}
