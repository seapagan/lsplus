//! Directory-entry sorting helpers.

use std::cmp::Ordering;
use std::path::Path;

use crate::platform;
use crate::structs::{Params, SortMode};
use crate::utils::file::DirectoryEntryData;

/// Sort visible directory entries using the resolved runtime policy.
pub(crate) fn sort_entries(
    entries: &mut [DirectoryEntryData],
    params: &Params,
) {
    if params.sort == SortMode::None {
        return;
    }

    entries.sort_by(|left, right| {
        let ordering = mode_ordering(left, right, params.sort)
            .then_with(|| name_ordering(left, right));

        if params.reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

fn mode_ordering(
    left: &DirectoryEntryData,
    right: &DirectoryEntryData,
    mode: SortMode,
) -> Ordering {
    match mode {
        SortMode::Name | SortMode::None => name_ordering(left, right),
        SortMode::Size => {
            metadata_ordering(left, right, |metadata| metadata.len()).reverse()
        }
        SortMode::Time => modified_ordering(left, right),
        SortMode::Extension => extension_ordering(left, right),
        SortMode::Version => version_ordering(left, right),
    }
}

fn metadata_ordering<T: Ord>(
    left: &DirectoryEntryData,
    right: &DirectoryEntryData,
    key: impl Fn(&std::fs::Metadata) -> T,
) -> Ordering {
    left.metadata
        .as_ref()
        .zip(right.metadata.as_ref())
        .map(|(left, right)| key(left).cmp(&key(right)))
        .unwrap_or(Ordering::Equal)
}

fn modified_ordering(
    left: &DirectoryEntryData,
    right: &DirectoryEntryData,
) -> Ordering {
    left.metadata
        .as_ref()
        .and_then(|metadata| metadata.modified().ok())
        .zip(
            right
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.modified().ok()),
        )
        .map(|(left, right)| right.cmp(&left))
        .unwrap_or(Ordering::Equal)
}

fn extension_ordering(
    left: &DirectoryEntryData,
    right: &DirectoryEntryData,
) -> Ordering {
    let left = Path::new(&left.file_name).extension();
    let right = Path::new(&right.file_name).extension();

    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => {
            platform::compare_entry_names(left, right)
        }
    }
}

fn version_ordering(
    left: &DirectoryEntryData,
    right: &DirectoryEntryData,
) -> Ordering {
    left.file_name
        .to_str()
        .zip(right.file_name.to_str())
        .map(|(left, right)| vsort::compare(left, right))
        .unwrap_or_else(|| name_ordering(left, right))
}

fn name_ordering(
    left: &DirectoryEntryData,
    right: &DirectoryEntryData,
) -> Ordering {
    platform::compare_entry_names(&left.file_name, &right.file_name)
}
