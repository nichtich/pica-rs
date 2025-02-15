use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::create_dir;
use std::path::PathBuf;
use std::str::FromStr;

use bstr::ByteSlice;
use clap::Parser;
use pica_path::{Path, PathExt};
use pica_record::io::{
    ByteRecordWrite, ReaderBuilder, RecordsIterator, WriterBuilder,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::CliResult;
use crate::{gzip_flag, skip_invalid_flag, template_opt};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct PartitionConfig {
    /// Skip invalid records that can't be decoded
    pub(crate) skip_invalid: Option<bool>,

    /// Compress output in gzip format
    pub(crate) gzip: Option<bool>,

    /// Filename template
    pub(crate) template: Option<String>,
}

/// Partition records by subfield values
///
/// The files are written to the <outdir> directory with filenames
/// based on the values of the subfield, which is referenced by the
/// <PATH> expression.
///
/// If a record doesn't have the field/subfield, the record won't be
/// writte to a partition. A record with multiple values will be written
/// to each partition; thus the partitions may not be disjoint. In order
/// to prevent duplicate records in a partition , all duplicate values
/// of a record will be removed.
#[derive(Parser, Debug)]
pub(crate) struct Partition {
    /// Skip invalid records that can't be decoded as normalized PICA+
    #[arg(long, short)]
    skip_invalid: bool,

    /// Compress each partition in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Write partitions into <outdir>
    ///
    /// If the directory doesn't exists, it will be created
    /// automatically.
    #[arg(long, short, value_name = "outdir", default_value = ".")]
    outdir: PathBuf,

    /// Filename template ("{}" is replaced by subfield value)
    #[arg(long, short, value_name = "template")]
    template: Option<String>,

    /// A path expression (e.g. "002@.0")
    path: String,

    /// Read one or more files in normalized PICA+ format
    ///
    /// If no filenames where given or a filename is "-", data is read
    /// from standard input (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Partition {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let path = Path::from_str(&self.path)?;
        let gzip_compression = gzip_flag!(self.gzip, config.partition);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.partition,
            config.global
        );

        let filename_template = template_opt!(
            self.template,
            config.partition,
            if gzip_compression {
                "{}.dat.gz"
            } else {
                "{}.dat"
            }
        );

        if !self.outdir.exists() {
            create_dir(&self.outdir)?;
        }

        let mut writers: HashMap<Vec<u8>, Box<dyn ByteRecordWrite>> =
            HashMap::new();

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(e) => {
                        if e.is_invalid_record() && skip_invalid {
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                    Ok(record) => {
                        let mut values =
                            record.path(&path, &Default::default());
                        values.sort_unstable();
                        values.dedup();

                        for value in values {
                            let mut entry = writers
                                .entry(value.as_bytes().to_vec());
                            let writer = match entry {
                                Entry::Vacant(vacant) => {
                                    let filename = filename_template
                                        .replace(
                                            "{}",
                                            &value.to_str_lossy(),
                                        );

                                    let path = self
                                        .outdir
                                        .join(filename)
                                        .to_str()
                                        .unwrap()
                                        .to_owned();

                                    let writer = WriterBuilder::new()
                                        .gzip(gzip_compression)
                                        .from_path(path)?;

                                    vacant.insert(writer)
                                }
                                Entry::Occupied(ref mut occupied) => {
                                    occupied.get_mut()
                                }
                            };

                            writer.write_byte_record(&record)?;
                        }
                    }
                }
            }
        }

        for (_, mut writer) in writers {
            writer.finish()?;
        }

        Ok(())
    }
}
