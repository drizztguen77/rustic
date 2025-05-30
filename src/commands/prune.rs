//! `prune` subcommand

use crate::{
    Application, RUSTIC_APP, helpers::bytes_size_to_string, repository::CliOpenRepo, status_err,
};
use abscissa_core::{Command, Runnable, Shutdown};
use log::debug;

use anyhow::Result;

use rustic_core::{PruneOptions, PruneStats};

/// `prune` subcommand
#[allow(clippy::struct_excessive_bools)]
#[derive(clap::Parser, Command, Debug, Clone)]
pub(crate) struct PruneCmd {
    /// Prune options
    #[clap(flatten)]
    pub(crate) opts: PruneOptions,
}

impl Runnable for PruneCmd {
    fn run(&self) {
        if let Err(err) = RUSTIC_APP
            .config()
            .repository
            .run_open(|repo| self.inner_run(repo))
        {
            status_err!("{}", err);
            RUSTIC_APP.shutdown(Shutdown::Crash);
        };
    }
}

impl PruneCmd {
    fn inner_run(&self, repo: CliOpenRepo) -> Result<()> {
        let config = RUSTIC_APP.config();

        let prune_plan = repo.prune_plan(&self.opts)?;

        print_stats(&prune_plan.stats);

        if config.global.dry_run {
            repo.warm_up(prune_plan.repack_packs().into_iter())?;
        } else {
            repo.prune(&self.opts, prune_plan)?;
        }

        Ok(())
    }
}

/// Print statistics about the prune operation
///
/// # Arguments
///
/// * `stats` - Statistics about the prune operation
#[allow(clippy::cast_precision_loss)]
fn print_stats(stats: &PruneStats) {
    let pack_stat = &stats.packs;
    let blob_stat = stats.blobs_sum();
    let size_stat = stats.size_sum();

    debug!("statistics:");
    debug!("{:#?}", stats.debug);

    debug!(
        "used:   {:>10} blobs, {:>10}",
        blob_stat.used,
        bytes_size_to_string(size_stat.used)
    );

    debug!(
        "unused: {:>10} blobs, {:>10}",
        blob_stat.unused,
        bytes_size_to_string(size_stat.unused)
    );
    debug!(
        "total:  {:>10} blobs, {:>10}",
        blob_stat.total(),
        bytes_size_to_string(size_stat.total())
    );

    println!(
        "to repack: {:>10} packs, {:>10} blobs, {:>10}",
        pack_stat.repack,
        blob_stat.repack,
        bytes_size_to_string(size_stat.repack)
    );
    println!(
        "this removes:                {:>10} blobs, {:>10}",
        blob_stat.repackrm,
        bytes_size_to_string(size_stat.repackrm)
    );
    println!(
        "to delete: {:>10} packs, {:>10} blobs, {:>10}",
        pack_stat.unused,
        blob_stat.remove,
        bytes_size_to_string(size_stat.remove)
    );
    if !stats.packs_unref > 0 {
        println!(
            "unindexed: {:>10} packs,         ?? blobs, {:>10}",
            stats.packs_unref,
            bytes_size_to_string(stats.size_unref)
        );
    }

    println!(
        "total prune:                 {:>10} blobs, {:>10}",
        blob_stat.repackrm + blob_stat.remove,
        bytes_size_to_string(size_stat.repackrm + size_stat.remove + stats.size_unref)
    );
    println!(
        "remaining:                   {:>10} blobs, {:>10}",
        blob_stat.total_after_prune(),
        bytes_size_to_string(size_stat.total_after_prune())
    );
    println!(
        "unused size after prune: {:>10} ({:.2}% of remaining size)",
        bytes_size_to_string(size_stat.unused_after_prune()),
        size_stat.unused_after_prune() as f64 / size_stat.total_after_prune() as f64 * 100.0
    );

    println!();

    println!(
        "packs marked for deletion: {:>10}, {:>10}",
        stats.packs_to_delete.total(),
        bytes_size_to_string(stats.size_to_delete.total()),
    );
    println!(
        " - complete deletion:      {:>10}, {:>10}",
        stats.packs_to_delete.remove,
        bytes_size_to_string(stats.size_to_delete.remove),
    );
    println!(
        " - keep marked:            {:>10}, {:>10}",
        stats.packs_to_delete.keep,
        bytes_size_to_string(stats.size_to_delete.keep),
    );
    println!(
        " - recover:                {:>10}, {:>10}",
        stats.packs_to_delete.recover,
        bytes_size_to_string(stats.size_to_delete.recover),
    );

    debug!(
        "index files to rebuild: {} / {}",
        stats.index_files_rebuild, stats.index_files
    );
}
