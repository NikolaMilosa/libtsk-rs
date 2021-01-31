extern crate tsk;
use std::path::PathBuf;
use std::io::{Read, Write, Seek, SeekFrom};
use tsk::tsk_img::TskImg;
use tsk::tsk_fs_dir::TskFsDir;
use tsk::tsk_fs_attr::TskFsAttr;
use tsk::bindings;
use std::fs::File;


#[cfg(target_os = "windows")]
fn get_windows_sample_file_parts() -> (String, String) {
    // Build path to test file
    let test_file_path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "samples",
        "test_file"
    ].iter().collect();

    // Get the root volume for correct TSK volume
    let root = test_file_path.components().nth(0).unwrap();
    let root_str = root.as_os_str().to_string_lossy();
    let volume_str = format!(r"\\.\{}", root_str);
    println!("volume_str => {}", volume_str);

    // Generate the TSK path to fetch
    let tsk_file_path_str = test_file_path
        .strip_prefix(root_str.to_string())
        .unwrap()
        .to_string_lossy()
        .replace("\\", "/");
    println!("tsk_file_path_str => {}", tsk_file_path_str);

    (volume_str, tsk_file_path_str)
}

#[cfg(target_os = "windows")]
#[test]
fn test_tsk_wrappers_dir() {
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    let root_fh = TskFsDir::from_meta(&tsk_fs, 5)
        .expect("Could not open root folder");
    println!("{:?}", root_fh);

    for name_attr in root_fh.get_name_iter() {
        println!("{:?}", name_attr);
    }

    let tsk_fs_name = root_fh.get_name(0)
        .expect("Error getting name at index 0");
    println!("{:?}", tsk_fs_name);
}


#[cfg(target_os = "windows")]
#[test]
fn test_tsk_iterate_root() {
    let source = r"\\.\C:";

    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    let file_iter = tsk_fs.iter_file_names()
        .expect("Could not get FsNameIter");

    let mut file_count = 0;
    for (path, f) in file_iter {
        if file_count == 512 {
            break;
        }
        
        if let Some(name) = f.name() {
            println!("{}/{}", path, name);
        }

        file_count += 1;
    }
}


#[cfg(target_os = "windows")]
#[test]
fn test_tsk_wrappers() {
    let source = r"\\.\PHYSICALDRIVE0";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");

    let tsk_vs = tsk_img.get_vs_from_offset(0)
        .expect("Could not open TskVs at offset 0");
    println!("{:?}", tsk_vs);

    let part_iter = tsk_vs.get_partition_iter()
        .expect("Could not get partition iterator for TskVs");
    for vs_part in part_iter {
        println!("{:?}", vs_part);
    }

    let tsk_vs_part = tsk_vs.get_partition_at_index(0)
        .expect("Could not open TskVsPart at offset 0");
    println!("{:?}", tsk_vs_part);

    let iter = tsk_vs_part.into_iter();
    for vs_part in iter {
        println!("{:?}", vs_part);
    }
    drop(tsk_vs);

    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");
    println!("{:?}", tsk_fs);

    let root_fh = tsk_fs.file_open_meta(5)
        .expect("Could not open root folder");
    println!("{:?}", root_fh);
    let root_fh_meta = root_fh.get_meta().unwrap();
    assert_eq!(false, root_fh_meta.is_unallocated());
    assert_eq!(5, root_fh_meta.addr());

    let mut tsk_attr = root_fh.get_attr_at_index(0)
        .expect("Unable to get attribute at index 0 for root node.");
    let mut buffer = vec![0; tsk_attr.size() as usize];
    let _bytes_read = tsk_attr.read(&mut buffer)
        .expect("Error reading attribute!");
    println!("Attribute 0 -> {:02x?}", buffer);

    drop(root_fh);

    let mft_fh = tsk_fs.file_open("/$MFT")
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);

    let mft_fh = tsk_fs.file_open_meta(0)
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);
    let mft_fh_meta = mft_fh.get_meta().unwrap();
    assert_eq!(false, mft_fh_meta.is_unallocated());

    let mut tsk_attr = mft_fh.get_attr()
        .expect("Unable to get default attribute.");
    let mut buffer = vec![0; 1024];
    let _bytes_read = tsk_attr.read(&mut buffer)
        .expect("Error reading attribute!");
    println!("MFT default attribute -> {:02x?}", buffer);
    

    let attr_0 = mft_fh.get_attr_at_index(0)
        .expect("Could not get attribute 0 for $MFT");
    println!("{:?}", attr_0);

    let attr_99 = mft_fh.get_attr_at_index(99);
    assert_eq!(attr_99.is_err(), true);
    println!("{:?}", attr_99);

    let attr_iter = mft_fh.get_attr_iter()
        .expect("Could not get attribute iterator for $MFT");

    for attr in attr_iter {
        println!("{:?}", attr);
    }
}

#[cfg(target_os = "windows")]
#[test]
fn test_tsk_attr_read_seek() {
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");
    println!("{:?}", tsk_fs);
    
    let mft_fh = tsk_fs.file_open_meta(0)
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);
    let mft_fh_meta = mft_fh.get_meta().unwrap();
    assert_eq!(false, mft_fh_meta.is_unallocated());

    let mut tsk_attr = mft_fh.get_attr()
        .expect("Unable to get default attribute.");
    let mut buffer = vec![0; 1024];
    let _pos = tsk_attr.seek(SeekFrom::Start(1024))
        .expect("Error seeking to pos 1024");
    let _bytes_read = tsk_attr.read(&mut buffer)
        .expect("Error reading attribute!");
    println!("MFT record at offset 1024 -> {:02x?}", buffer);
}

#[cfg(target_os = "windows")]
#[test]
fn test_tsk_file_handle_read_seek() {
    // Generate the TSK path to fetch
    let (volume_str, tsk_file_path_str) = get_windows_sample_file_parts();

    let tsk_img = TskImg::from_source(&volume_str)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");
    println!("{:?}", tsk_fs);

    println!("Opening '{}'...", tsk_file_path_str);
    let test_file = tsk_fs.file_open(&tsk_file_path_str)
        .expect(&format!("Could not open '{}'", tsk_file_path_str));
    println!("{:?}", test_file);

    // Get the default attribute
    let attr = TskFsAttr::from_default(&test_file).unwrap();
    println!("{:?}", attr);

    // Create a TskFsFileHandle from TskFsFile
    let mut test_file_handle = test_file.get_file_handle(
        attr, 
        bindings::TSK_FS_FILE_READ_FLAG_ENUM::TSK_FS_FILE_READ_FLAG_NONE
    ).expect("Unable to get default attribute.");

    let mut buf = [0; 1];

    // Read first byte
    test_file_handle.read(&mut buf).unwrap();
    assert_eq!(&buf, b"\x00");
    println!("{:?}", buf);

    // Seek to the last byte
    test_file_handle.seek(SeekFrom::End(-1)).unwrap();

    // Read last byte
    test_file_handle.read(&mut buf).unwrap();
    assert_eq!(&buf, b"\xff");
    println!("{:?}", buf);
}

#[cfg(target_os = "windows")]
#[test]
fn test_tsk_fs_meta(){
    let (volume_str, tsk_file_path_str) = get_windows_sample_file_parts();

    let tsk_img = TskImg::from_source(&volume_str)
        .expect("Could not create TskImg");

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    println!("Opening '{}'...", tsk_file_path_str);
    let root_fh = tsk_fs.file_open(&tsk_file_path_str)
        .expect("Could not open test_file");
    
    println!("Reading file metadata...");
    println!("{:?}",root_fh.get_meta());
}

#[cfg(target_os = "windows")]
#[test]
fn test_copy_file(){
    let (volume_str, tsk_file_path_str) = get_windows_sample_file_parts();

    let tsk_img = TskImg::from_source(&volume_str)
        .expect("Could not create TskImg");

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    println!("Opening '{}'...", tsk_file_path_str);
    let test_file = tsk_fs.file_open(&tsk_file_path_str)
        .expect("Could not open test_file");
    
    // Get the default attribute
    let attr = TskFsAttr::from_default(&test_file).unwrap();
    println!("{:?}", attr);

    // Create a TskFsFileHandle from TskFsFile
    let mut test_file_handle = test_file.get_file_handle(
        attr, 
        bindings::TSK_FS_FILE_READ_FLAG_ENUM::TSK_FS_FILE_READ_FLAG_NONE
    ).expect("Unable to get default attribute.");

    // Specify a buffer of 10 bytes
    let mut buf = [0;10];
    let outfile_path = format!("{}{}", tsk_file_path_str, "_copied_using_libtsk_rs");
    println!("Writing to '{}'...", outfile_path);
    let mut outfile = File::create(outfile_path).unwrap();
    loop {
        let bytes_read = test_file_handle.read(&mut buf).unwrap();
        if bytes_read == 0 {break;}
        let bw = outfile.write(&buf[..bytes_read]).unwrap();
        println!("Wrote '{}' bytes", bw);
    }
}