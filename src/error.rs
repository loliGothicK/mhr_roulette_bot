/*
 * ISC License
 *
 * Copyright (c) 2021 Mitama Lab
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 *
 */

use std::fmt::Debug;
use strum_macros::ToString;
use thiserror::Error;

/// Error kinds
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, ToString)]
pub enum ErrorKind {
    /// [QueryError]
    QueryError,
    /// [LogicError]
    LogicError,
    /// [CommandError]
    CommandError,
    /// Other errors
    Others,
}

/// Error for SQLite query.
#[derive(Debug, Error)]
pub enum QueryError {
    /// Used for invalid weapon keys.
    #[error("No such weapon key: {param:?} => `{actual:?}`.")]
    InvalidWeapon { param: String, actual: String },
    /// Used for invalid date format
    #[error("{source:?}: {param:?} => `{actual:?}`")]
    InvalidDate {
        param: String,
        actual: String,
        #[source]
        source: chrono::format::ParseError,
    },
    /// Used at fail to INSERT or UPDATE.
    #[error("Failed to store data: {raw:?}\nwith query: {query:?}")]
    FailedToStore { raw: String, query: String },
    /// Used at fail to SELECT.
    #[error("Failed to aggregate statistics: {raw:?}\nwith query: {query:?}")]
    FailedToAggregate { raw: String, query: String },
}

/// Error for critical logic mistakes.
#[derive(Debug, Error)]
pub enum LogicError {
    /// Used at reaching unexpected match guard.
    #[error(
        "Reaching the unreachable match guard:\n with expression {{ {expr:?} => value }}\n{info:?}"
    )]
    UnreachableGuard {
        expr: String,
        value: String,
        info: String,
    },
}

/// Error for slash commands.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Used at time limit exceeded.
    #[error("Time limit exceeded during {command:?} (waiting for {})", .wait_for.as_millis())]
    TimeLimitExceeded {
        command: String,
        wait_for: std::time::Duration,
    },
    /// Used at fail to data sync.
    #[error("IO error occurred during {command:?}: {io_error:?}")]
    FailedToSync {
        command: String,
        #[source]
        io_error: std::io::Error,
    },
    /// Used for invalid command arguments.
    #[error("Invalid argument : {arg:?}")]
    InvalidArgument { arg: String },
}

/// Triage Sections for Error Level.
#[derive(Debug, Copy, Clone, ToString, Ord, PartialOrd, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum TriageTag {
    /// The BOT has a fatal problem
    Immediate,
    /// The BOT has anomalies visible to users
    Delayed,
    /// The BOT has an anomaly that is not visible to users
    Minor,
    /// Errors not caused by BOT
    /// For example, validation error, http error, etc.
    NotBad,
}

/// Extended Error trait to triage error level.
pub trait ErrorExt {
    fn triage(&self) -> Option<TriageTag>;
    fn kind(&self) -> ErrorKind;
}

/// Extended Error trait for QueryError.
impl ErrorExt for QueryError {
    /// # Error Level Definition for QueryError
    /// - InvalidWeapon: NotBad
    /// - InvalidDate: NotBad
    /// - FailedToStore: Delayed
    /// - FailedToAggregate: Immediate
    fn triage(&self) -> Option<TriageTag> {
        use QueryError::*;
        Some(match self {
            InvalidWeapon { .. } => TriageTag::NotBad,
            InvalidDate { .. } => TriageTag::NotBad,
            FailedToStore { .. } => TriageTag::Delayed,
            FailedToAggregate { .. } => TriageTag::Immediate,
        })
    }

    fn kind(&self) -> ErrorKind {
        ErrorKind::QueryError
    }
}

impl ErrorExt for LogicError {
    /// # Error Level Definition for LogicError
    /// All Immediate
    fn triage(&self) -> Option<TriageTag> {
        Some(TriageTag::Immediate)
    }

    fn kind(&self) -> ErrorKind {
        ErrorKind::LogicError
    }
}

impl ErrorExt for CommandError {
    /// # Error Level Definition for CommandError
    /// - TimeLimitExceeded: Immediate
    /// - FailedToSync: Immediate
    /// - InvalidArgument: NotBad
    fn triage(&self) -> Option<TriageTag> {
        use CommandError::*;
        Some(match self {
            TimeLimitExceeded { .. } => TriageTag::Immediate,
            FailedToSync { .. } => TriageTag::Immediate,
            InvalidArgument { .. } => TriageTag::NotBad,
        })
    }

    fn kind(&self) -> ErrorKind {
        ErrorKind::CommandError
    }
}

impl ErrorExt for anyhow::Error {
    /// Returns most fatal triage tag in error chain.
    fn triage(&self) -> Option<TriageTag> {
        self.chain()
            .flat_map(|cause| {
                [
                    cause.downcast_ref::<QueryError>().map(ErrorExt::triage),
                    cause.downcast_ref::<LogicError>().map(ErrorExt::triage),
                    cause.downcast_ref::<CommandError>().map(ErrorExt::triage),
                ]
            })
            .max()
            .flatten()
            .flatten()
    }

    /// Returns error kind of most fatal triage tag in error chain.
    fn kind(&self) -> ErrorKind {
        self.chain()
            .map(|cause| {
                match [
                    (
                        cause.downcast_ref::<QueryError>().map(ErrorExt::triage),
                        ErrorKind::QueryError,
                    ),
                    (
                        cause.downcast_ref::<LogicError>().map(ErrorExt::triage),
                        ErrorKind::LogicError,
                    ),
                    (
                        cause.downcast_ref::<CommandError>().map(ErrorExt::triage),
                        ErrorKind::CommandError,
                    ),
                ]
                .iter()
                .max_by_key(|(tag, _)| tag)
                {
                    Some((Some(tag), kind)) => (Some(*tag), *kind),
                    _ => (None, ErrorKind::Others),
                }
            })
            .max_by_key(|(tag, _)| *tag)
            .map(|(_, kind)| kind.to_owned())
            .unwrap_or(ErrorKind::Others)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono;
    #[test]
    fn triage_test() {
        let invalid_weapon = QueryError::InvalidWeapon {
            param: "".to_string(),
            actual: "".to_string(),
        };
        assert_eq!(invalid_weapon.triage(), Some(TriageTag::NotBad));

        let err = chrono::DateTime::parse_from_rfc3339("invalid");
        let invalid_date = QueryError::InvalidDate {
            param: "".to_string(),
            actual: "".to_string(),
            source: err.unwrap_err(),
        };
        assert_eq!(invalid_date.triage(), Some(TriageTag::NotBad));

        let failed_to_store = QueryError::FailedToStore {
            raw: "".to_string(),
            query: "".to_string(),
        };
        assert_eq!(failed_to_store.triage(), Some(TriageTag::Delayed));

        let failed_to_aggregate = QueryError::FailedToAggregate {
            raw: "".to_string(),
            query: "".to_string(),
        };
        assert_eq!(failed_to_aggregate.triage(), Some(TriageTag::Immediate));
        assert_eq!(
            anyhow::Error::from(failed_to_aggregate)
                .context("context")
                .triage(),
            Some(TriageTag::Immediate)
        );
    }

    #[test]
    fn kind_test() {
        let invalid_weapon = QueryError::InvalidWeapon {
            param: "".to_string(),
            actual: "".to_string(),
        };
        assert_eq!(invalid_weapon.kind(), ErrorKind::QueryError);

        let err = chrono::DateTime::parse_from_rfc3339("invalid");
        let invalid_date = QueryError::InvalidDate {
            param: "".to_string(),
            actual: "".to_string(),
            source: err.unwrap_err(),
        };
        assert_eq!(invalid_date.kind(), ErrorKind::QueryError);

        let failed_to_store = QueryError::FailedToStore {
            raw: "".to_string(),
            query: "".to_string(),
        };
        assert_eq!(failed_to_store.kind(), ErrorKind::QueryError);

        let failed_to_aggregate = QueryError::FailedToAggregate {
            raw: "".to_string(),
            query: "".to_string(),
        };
        assert_eq!(failed_to_aggregate.kind(), ErrorKind::QueryError);
    }
}
