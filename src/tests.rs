use std::{fs::DirEntry, io, os::windows::prelude::MetadataExt, path::Path};


// TODO: These are equivalent and bs58 seems better. migrate.
// Test that our base58 encoder can encode/decode arbitrary bytes.
#[test]
fn base58_bytes() {
    use rust_base58::*;

    // Public keys and the private half of private keys are 32 bytes:
    let original_bytes = vec![
        0x00, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
    ];
    
    let encoded = original_bytes.to_base58();

    assert_eq!(encoded.as_str(), "11t6ZcNTcApL3s8ScmkCwNcFxJ47FfATY7cdg632k4X");
    let decoded = encoded.from_base58().unwrap();

    assert_eq!(original_bytes, decoded);

    // // Typos get decoded too?  Nope.
    // let decoded = "llLQX".from_base58().unwrap();
    // assert_eq!(original_bytes, decoded);
}

// The new one is better?
#[test]
fn bs58_bytes() {
    use bs58;

    // Public keys and the private half of private keys are 32 bytes:
    let original_bytes = vec![
        0x00, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
    ];
    
    let encoded = bs58::encode(&original_bytes)
        .with_alphabet(bs58::alphabet::BITCOIN)
        .into_string()
    ;

    assert_eq!(encoded.as_str(), "11t6ZcNTcApL3s8ScmkCwNcFxJ47FfATY7cdg632k4X");


    let decoded = bs58::decode(encoded)
        .into_vec()
        .unwrap()
    ;

    assert_eq!(original_bytes, decoded);


    let encoded = bs58::encode(&original_bytes)
        .with_check()
        .into_string()
    ;

    assert_eq!(encoded.as_str(), "116mMWNtNGD53Go3cNrwfPGW8dDEgLCGPdB7HAND77SUGmaWE");

    // // Should be able to fix up incorrect letters: But does not.
    // let decoded = bs58::decode("1l6mMWNtNGD53Go3cNrwfPGW8dDEgLCGPdB7HAND77SUGmaWE")
    //     .with_check(None)
    //     .into_vec()
    //     .unwrap();
    // assert_eq!(original_bytes, decoded);

}

// The time crate doesn't really specify what the max duration is. 
#[test]
fn time_duration() {
    use time::Duration;

    let max = Duration::max_value();
    assert_eq!(9223372036854775807999, max.whole_milliseconds());

    // Seems like enough years. 😆
    assert_eq!(292471208677, max.whole_days() / 365);

    // FeoBlog uses an i64 # ms since epoch, so its max is:
    let max_feo = Duration::milliseconds(i64::MAX);
    assert_eq!(292471208, max_feo.whole_days() / 365);
}

/// Test that Snowpack/Rollup didn't generate files with NTFS alternate data streams.
/// See: https://github.com/NfNitLoop/feoblog/issues/16
/// These are unintended side-effects of using file paths that include a : in the name.
/// In particular, they break RustEmbed being able to properly embed the file in the 
/// release build.
#[test]
fn no_ntfs_ads() -> Result<(), failure::Error> {
    use std::path::Path;
    use std::fs::{metadata};
    let client_build_dir = Path::new("web-client/build");

    // Must build the web client before this test runs.
    // The existence of index.html implies the build has run:
    let index = client_build_dir.join("index.html");
    assert!(metadata(index)?.file_type().is_file(), "Must build web-client before running tests.");


    let mut found_files = 0;
    for entry in walk_files(&client_build_dir)? {
        let entry = entry?;
        let md = metadata(entry.path())?;

        // I don't see any way in std:: to directly list NTFS data streams, but
        // Windows/Rust report file sizes w/o alternate data streams, so I'm
        // using this as a proxy for that.  
        if md.file_size() == 0 {
            println!("ERROR: 0 byte file: {}", entry.path().to_str().unwrap_or("Unknown file"));
            found_files += 1;
        }
    }

    assert!(found_files == 0, "Found {} files that seem to have NTFS data streams.", found_files);

    Ok(())
}


fn walk_files<P: AsRef<Path>>(dir_path: P) -> Result<impl Iterator<Item=io::Result<DirEntry>>, io::Error> {
    let mut top_level = std::fs::read_dir(dir_path)?;
    let mut inner: Option<Box<dyn Iterator<Item=io::Result<DirEntry>>>> = None;

    let iter = std::iter::from_fn(move || -> Option<io::Result<DirEntry>> {
        loop {
            if let Some(inner) = &mut inner {
                let next = inner.next();
                if next.is_some() {
                    return next;
                }
            }
            // else: we're done w/ inner:
            inner = None;

            let entry = match top_level.next() {
                None => return None,
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok(entry)) => entry,
            };

            let metadata = match entry.metadata() {
                Ok(d) => d,
                Err(err) => return Some(Err(err)),
            };

            if !metadata.is_dir() {
                return Some(Ok(entry))
            }

            let inner_walker = match walk_files(entry.path()) {
                Ok(x) => x,
                Err(err) => return Some(Err(err)),
            };
            inner = Some(Box::new(inner_walker));
        }
    });

    Ok(iter)
}