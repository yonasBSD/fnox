//! Clap command and argument ordering validation
//!
//! This module provides utilities to validate that clap commands and arguments
//! are ordered according to a standard convention:
//! - Commands/subcommands: alphabetically
//! - Arguments grouped and sorted as:
//!   1. Positional arguments (alphabetically)
//!   2. Flags with short options (alphabetically by short, then long)
//!   3. Flags with long-only options (alphabetically)
//!
//! Use `assert_command_order` in debug builds to validate ordering.

use clap::{Arg, Command};
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("CLI ordering error in '{command}': {details}")]
#[diagnostic(code(clap_sort::ordering_error))]
struct OrderingError {
    command: String,
    details: String,
}

/// Asserts that commands and arguments are ordered correctly.
/// This only runs in debug builds and is a no-op in release builds.
///
/// The expected order is:
/// - Subcommands: alphabetically by name
/// - Arguments:
///   1. Positional arguments (alphabetically by name)
///   2. Flags with short options (alphabetically by short, then long)
///   3. Flags with long-only options (alphabetically by long)
///
/// # Panics
/// Panics in debug builds if the ordering is incorrect.
///
/// # Example
/// ```no_run
/// use clap::{Command, Arg};
/// # use fnox::clap_sort::assert_command_order;
///
/// let cli = Command::new("myapp")
///     .arg(Arg::new("output").short('o').long("output"))
///     .arg(Arg::new("verbose").short('v').long("verbose"))
///     .subcommand(Command::new("alpha"))
///     .subcommand(Command::new("zebra"));
///
/// assert_command_order(&cli);
/// // Panics in debug if subcommands aren't alphabetical or args aren't grouped correctly
/// ```
pub fn assert_command_order(cmd: &Command) {
    assert_subcommands_sorted(cmd);
    assert_arguments_sorted(cmd);

    // Recursively check subcommands
    for subcmd in cmd.get_subcommands() {
        assert_command_order(subcmd);
    }
}

/// Formats a diff between current and expected order with color
fn format_diff<T: std::fmt::Display>(current: &[T], expected: &[T]) -> String {
    let mut output = String::new();

    output.push_str("\n\n");
    output.push_str("Current order:\n");
    for item in current {
        output.push_str(&format!("  • {}\n", item));
    }

    output.push_str("\nExpected order:\n");
    for item in expected {
        output.push_str(&format!("  • {}\n", item));
    }

    output
}

/// Asserts that subcommands are sorted alphabetically by name.
fn assert_subcommands_sorted(cmd: &Command) {
    let subcommand_names: Vec<&str> = cmd.get_subcommands().map(|c| c.get_name()).collect();

    let mut sorted_names = subcommand_names.clone();
    sorted_names.sort_unstable();

    if subcommand_names != sorted_names {
        let diff = format_diff(&subcommand_names, &sorted_names);
        let error = OrderingError {
            command: cmd.get_name().to_string(),
            details: format!("Subcommands must be sorted alphabetically.{}", diff),
        };
        panic!("{:?}", miette::Report::new(error));
    }
}

/// Asserts that arguments are sorted according to the grouping rules:
/// 1. Positional args (alphabetically)
/// 2. Flags with short options (alphabetically by short, then long)
/// 3. Flags with long-only options (alphabetically by long)
fn assert_arguments_sorted(cmd: &Command) {
    let args: Vec<&Arg> = cmd.get_arguments().collect();

    let mut positional = Vec::new();
    let mut with_short = Vec::new();
    let mut long_only = Vec::new();

    for arg in &args {
        if arg.is_positional() {
            positional.push(*arg);
        } else if arg.get_short().is_some() {
            with_short.push(*arg);
        } else if arg.get_long().is_some() {
            long_only.push(*arg);
        }
    }

    // Check positional args are sorted
    let positional_ids: Vec<&str> = positional.iter().map(|a| a.get_id().as_str()).collect();
    let mut sorted_positional = positional_ids.clone();
    sorted_positional.sort_unstable();

    if positional_ids != sorted_positional {
        let diff = format_diff(&positional_ids, &sorted_positional);
        let error = OrderingError {
            command: cmd.get_name().to_string(),
            details: format!(
                "Positional arguments must be sorted alphabetically.{}",
                diff
            ),
        };
        panic!("{:?}", miette::Report::new(error));
    }

    // Check short flags are sorted by short option (case-insensitive, lowercase before uppercase for same letter)
    let with_short_chars: Vec<Option<char>> = with_short.iter().map(|a| a.get_short()).collect();
    let mut sorted_short = with_short_chars.clone();
    sorted_short.sort_by(|a, b| {
        match (a, b) {
            (Some(a_char), Some(b_char)) => {
                // Compare case-insensitively first
                let a_lower = a_char.to_ascii_lowercase();
                let b_lower = b_char.to_ascii_lowercase();
                match a_lower.cmp(&b_lower) {
                    std::cmp::Ordering::Equal => {
                        // If same letter, lowercase comes before uppercase
                        if a_char.is_lowercase() && b_char.is_uppercase() {
                            std::cmp::Ordering::Less
                        } else if a_char.is_uppercase() && b_char.is_lowercase() {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    }
                    other => other,
                }
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    if with_short_chars != sorted_short {
        // Create more helpful output showing flag names with their short options
        let current: Vec<String> = with_short
            .iter()
            .map(|a| format!("-{} ({})", a.get_short().unwrap(), a.get_id().as_str()))
            .collect();
        let expected: Vec<String> = {
            let mut sorted = with_short.clone();
            sorted.sort_by(|a, b| {
                let a_char = a.get_short();
                let b_char = b.get_short();
                match (a_char, b_char) {
                    (Some(a_ch), Some(b_ch)) => {
                        let a_lower = a_ch.to_ascii_lowercase();
                        let b_lower = b_ch.to_ascii_lowercase();
                        match a_lower.cmp(&b_lower) {
                            std::cmp::Ordering::Equal => {
                                if a_ch.is_lowercase() && b_ch.is_uppercase() {
                                    std::cmp::Ordering::Less
                                } else if a_ch.is_uppercase() && b_ch.is_lowercase() {
                                    std::cmp::Ordering::Greater
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            }
                            other => other,
                        }
                    }
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            });
            sorted
                .iter()
                .map(|a| format!("-{} ({})", a.get_short().unwrap(), a.get_id().as_str()))
                .collect()
        };
        let diff = format_diff(&current, &expected);
        let error = OrderingError {
            command: cmd.get_name().to_string(),
            details: format!(
                "Flags with short options must be sorted alphabetically by short option.{}",
                diff
            ),
        };
        panic!("{:?}", miette::Report::new(error));
    }

    // Check long-only flags are sorted
    let long_only_names: Vec<Option<&str>> = long_only.iter().map(|a| a.get_long()).collect();
    let mut sorted_long = long_only_names.clone();
    sorted_long.sort_unstable();

    if long_only_names != sorted_long {
        let current: Vec<String> = long_only
            .iter()
            .map(|a| format!("--{}", a.get_long().unwrap()))
            .collect();
        let expected: Vec<String> = {
            let mut sorted = long_only.clone();
            sorted.sort_by_key(|a| a.get_long());
            sorted
                .iter()
                .map(|a| format!("--{}", a.get_long().unwrap()))
                .collect()
        };
        let diff = format_diff(&current, &expected);
        let error = OrderingError {
            command: cmd.get_name().to_string(),
            details: format!("Long-only flags must be sorted alphabetically.{}", diff),
        };
        panic!("{:?}", miette::Report::new(error));
    }

    // Check that groups appear in correct order
    let arg_ids: Vec<&str> = args.iter().map(|a| a.get_id().as_str()).collect();

    let mut expected_order = Vec::new();
    expected_order.extend(positional.iter().map(|a| a.get_id().as_str()));
    expected_order.extend(with_short.iter().map(|a| a.get_id().as_str()));
    expected_order.extend(long_only.iter().map(|a| a.get_id().as_str()));

    if arg_ids != expected_order {
        let diff = format_diff(&arg_ids, &expected_order);
        let error = OrderingError {
            command: cmd.get_name().to_string(),
            details: format!(
                "Arguments must be in the correct group order.\n\
                Expected: [positional, short flags, long-only flags]{}",
                diff
            ),
        };
        panic!("{:?}", miette::Report::new(error));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    #[test]
    fn test_correctly_sorted_subcommands_pass() {
        let cli = Command::new("test")
            .subcommand(Command::new("alpha"))
            .subcommand(Command::new("beta"))
            .subcommand(Command::new("zebra"));

        // Should not panic
        assert_command_order(&cli);
    }

    #[test]
    #[should_panic(expected = "Subcommands")]
    fn test_incorrectly_sorted_subcommands_panic() {
        let cli = Command::new("test")
            .subcommand(Command::new("zebra"))
            .subcommand(Command::new("alpha"))
            .subcommand(Command::new("beta"));

        assert_command_order(&cli);
    }

    #[test]
    fn test_correctly_sorted_arguments_pass() {
        let cli = Command::new("test")
            .arg(Arg::new("input")) // Positional
            .arg(
                Arg::new("debug")
                    .short('d')
                    .long("debug")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("output").short('o').long("output"))
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("age-key-file").long("age-key-file"));

        // Should not panic
        assert_command_order(&cli);
    }

    #[test]
    #[should_panic(expected = "Arguments must be in the correct group")]
    fn test_incorrectly_grouped_arguments_panic() {
        // Long-only flag before short flag (wrong order)
        let cli = Command::new("test")
            .arg(Arg::new("age-key-file").long("age-key-file"))
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            );

        assert_command_order(&cli);
    }

    #[test]
    #[should_panic(expected = "Flags with short options")]
    fn test_short_flags_not_sorted_panic() {
        let cli = Command::new("test")
            .arg(
                Arg::new("zebra")
                    .short('z')
                    .long("zebra")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("alpha")
                    .short('a')
                    .long("alpha")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("beta")
                    .short('b')
                    .long("beta")
                    .action(ArgAction::SetTrue),
            );

        assert_command_order(&cli);
    }

    #[test]
    #[should_panic(expected = "Long-only flags")]
    fn test_long_only_flags_not_sorted_panic() {
        let cli = Command::new("test")
            .arg(Arg::new("zebra").long("zebra").action(ArgAction::SetTrue))
            .arg(Arg::new("alpha").long("alpha").action(ArgAction::SetTrue))
            .arg(Arg::new("beta").long("beta").action(ArgAction::SetTrue));

        assert_command_order(&cli);
    }

    #[test]
    #[should_panic(expected = "Positional arguments")]
    fn test_positional_args_not_sorted_panic() {
        let cli = Command::new("test")
            .arg(Arg::new("pos2"))
            .arg(Arg::new("pos1"));

        assert_command_order(&cli);
    }

    #[test]
    fn test_recursive_subcommand_validation() {
        let cli = Command::new("test").subcommand(
            Command::new("parent")
                .subcommand(Command::new("child-a"))
                .subcommand(Command::new("child-z")),
        );

        // Should not panic - both levels are sorted
        assert_command_order(&cli);
    }

    #[test]
    #[should_panic(expected = "Subcommands")]
    fn test_recursive_subcommand_validation_fails_deep() {
        let cli = Command::new("test").subcommand(
            Command::new("parent")
                .subcommand(Command::new("child-z"))
                .subcommand(Command::new("child-a")), // Wrong order
        );

        assert_command_order(&cli);
    }

    #[test]
    fn test_complete_correctly_ordered_cli() {
        let cli = Command::new("test")
            .arg(Arg::new("file")) // Positional
            .arg(Arg::new("config").short('c').long("config"))
            .arg(Arg::new("profile").short('p').long("profile"))
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("age-key-file").long("age-key-file"))
            .arg(
                Arg::new("no-color")
                    .long("no-color")
                    .action(ArgAction::SetTrue),
            )
            .subcommand(Command::new("alpha"))
            .subcommand(Command::new("beta"));

        // Should not panic - everything is correctly ordered
        assert_command_order(&cli);
    }

    #[test]
    fn test_empty_command() {
        let cli = Command::new("test");

        // Should not panic - no subcommands or args to check
        assert_command_order(&cli);
    }

    #[test]
    fn test_only_subcommands() {
        let cli = Command::new("test")
            .subcommand(Command::new("a"))
            .subcommand(Command::new("b"))
            .subcommand(Command::new("c"));

        // Should not panic
        assert_command_order(&cli);
    }

    #[test]
    fn test_only_args() {
        let cli = Command::new("test")
            .arg(
                Arg::new("alpha")
                    .short('a')
                    .long("alpha")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("beta")
                    .short('b')
                    .long("beta")
                    .action(ArgAction::SetTrue),
            );

        // Should not panic
        assert_command_order(&cli);
    }
}
