//! `init` subcommand

use abscissa_core::{Command, Runnable, Shutdown, status_err};
use anyhow::{Result, bail};
use dialoguer::Password;

use crate::{Application, RUSTIC_APP, repository::CliRepo};

use rustic_core::{ConfigOptions, KeyOptions, OpenStatus, Repository};

/// `init` subcommand
#[derive(clap::Parser, Command, Debug)]
pub(crate) struct InitCmd {
    /// Key options
    #[clap(flatten, next_help_heading = "Key options")]
    key_opts: KeyOptions,

    /// Config options
    #[clap(flatten, next_help_heading = "Config options")]
    config_opts: ConfigOptions,
}

impl Runnable for InitCmd {
    fn run(&self) {
        if let Err(err) = RUSTIC_APP
            .config()
            .repository
            .run(|repo| self.inner_run(repo))
        {
            status_err!("{}", err);
            RUSTIC_APP.shutdown(Shutdown::Crash);
        };
    }
}

impl InitCmd {
    fn inner_run(&self, repo: CliRepo) -> Result<()> {
        let config = RUSTIC_APP.config();

        // Note: This is again checked in repo.init_with_password(), however we want to inform
        // users before they are prompted to enter a password
        if repo.config_id()?.is_some() {
            bail!("Config file already exists. Aborting.");
        }

        // Handle dry-run mode
        if config.global.dry_run {
            bail!(
                "cannot initialize repository {} in dry-run mode!",
                repo.name
            );
        }

        let _ = init(repo.0, &self.key_opts, &self.config_opts)?;
        Ok(())
    }
}

/// Initialize repository
///
/// # Arguments
///
/// * `repo` - Repository to initialize
/// * `key_opts` - Key options
/// * `config_opts` - Config options
///
/// # Errors
///
///  * [`RepositoryErrorKind::OpeningPasswordFileFailed`] - If opening the password file failed
/// * [`RepositoryErrorKind::ReadingPasswordFromReaderFailed`] - If reading the password failed
/// * [`RepositoryErrorKind::FromSplitError`] - If splitting the password command failed
/// * [`RepositoryErrorKind::PasswordCommandExecutionFailed`] - If executing the password command failed
/// * [`RepositoryErrorKind::ReadingPasswordFromCommandFailed`] - If reading the password from the command failed
///
/// # Returns
///
/// Returns the initialized repository
///
/// [`RepositoryErrorKind::OpeningPasswordFileFailed`]: rustic_core::error::RepositoryErrorKind::OpeningPasswordFileFailed
/// [`RepositoryErrorKind::ReadingPasswordFromReaderFailed`]: rustic_core::error::RepositoryErrorKind::ReadingPasswordFromReaderFailed
/// [`RepositoryErrorKind::FromSplitError`]: rustic_core::error::RepositoryErrorKind::FromSplitError
/// [`RepositoryErrorKind::PasswordCommandExecutionFailed`]: rustic_core::error::RepositoryErrorKind::PasswordCommandExecutionFailed
/// [`RepositoryErrorKind::ReadingPasswordFromCommandFailed`]: rustic_core::error::RepositoryErrorKind::ReadingPasswordFromCommandFailed
pub(crate) fn init<P, S>(
    repo: Repository<P, S>,
    key_opts: &KeyOptions,
    config_opts: &ConfigOptions,
) -> Result<Repository<P, OpenStatus>> {
    let pass = init_password(&repo)?;
    Ok(repo.init_with_password(&pass, key_opts, config_opts)?)
}

pub(crate) fn init_password<P, S>(repo: &Repository<P, S>) -> Result<String> {
    let pass = repo.password()?.unwrap_or_else(|| {
        match Password::new()
            .with_prompt("enter password for new key")
            .allow_empty_password(true)
            .with_confirmation("confirm password", "passwords do not match")
            .interact()
        {
            Ok(it) => it,
            Err(err) => {
                status_err!("{}", err);
                RUSTIC_APP.shutdown(Shutdown::Crash);
            }
        }
    });

    Ok(pass)
}
