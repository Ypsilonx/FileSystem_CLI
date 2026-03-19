use clap::{Parser, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use chrono::{DateTime, Local};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(ValueEnum, Clone, Debug)]
enum OrderBy {
    Name,
    Size,
    Date,
    Ext,
}

#[derive(ValueEnum, Clone, Debug)]
enum Direction {
    Asc,
    Desc,
}

#[derive(Parser, Debug)]
#[command(name = "scan", author = "Parťák v programování", version = "1.1")]
struct Args {
    /// Cesta k adresáři
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Seřadit podle: name, size, date, ext
    #[arg(short, long, default_value = "name", value_enum)]
    order_by: OrderBy,

    /// Směr řazení: asc, desc
    #[arg(short, long, default_value = "asc", value_enum)]
    direction: Direction,
}

struct FileEntry {
    name: String,
    extension: String,
    is_dir: bool,
    size: u64,
    created: DateTime<Local>,
}

fn get_dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else { return 0 };
    entries.flatten().map(|entry| {
        match entry.metadata() {
            Ok(meta) if meta.is_dir() => get_dir_size(&entry.path()),
            Ok(meta) => meta.len(),
            Err(_) => 0,
        }
    }).sum()
}

fn format_size(bytes: u64) -> String {
    const GB: u64 = 1_073_741_824;
    const MB: u64 = 1_048_576;
    const KB: u64 = 1_024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let entries_raw = match fs::read_dir(&args.path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("❌ Chyba: {}", e);
            return Ok(());
        }
    };

    let mut files: Vec<FileEntry> = Vec::new();
    // BTreeMap zajistí abecední řazení klíčů ve výpisu sumáře
    let mut stats: BTreeMap<String, (u32, u64)> = BTreeMap::new();

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    for entry in entries_raw.flatten() {
        let path = entry.path();
        spinner.set_message(format!("Skenuji: {}", entry.file_name().to_string_lossy()));

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("⚠️  Přeskakuji {:?}: {}", path, e);
                continue;
            }
        };
        let is_dir = metadata.is_dir();

        let ext = if is_dir {
            "Složka".to_string()
        } else {
            path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("Bez přípony")
                .to_lowercase()
        };

        let size = if is_dir { get_dir_size(&path) } else { metadata.len() };
        let created: DateTime<Local> = metadata
            .created()
            .or_else(|_| metadata.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .into();

        files.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            extension: ext.clone(),
            is_dir,
            size,
            created,
        });

        let s = stats.entry(ext).or_insert((0, 0));
        s.0 += 1;
        s.1 += size;
    }

    spinner.finish_and_clear();

    files.sort_by(|a, b| {
        let cmp = match args.order_by {
            OrderBy::Size => a.size.cmp(&b.size),
            OrderBy::Date => a.created.cmp(&b.created),
            OrderBy::Ext  => a.extension.cmp(&b.extension),
            OrderBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        };
        match args.direction {
            Direction::Desc => cmp.reverse(),
            Direction::Asc  => cmp,
        }
    });

    println!("\n{:<5} {:<15} | {:<30} | {:<12} | {:<16}", "Typ", "Přípona", "Název", "Velikost", "Vytvořeno");
    println!("{:-<90}", "");

    for f in &files {
        let icon = if f.is_dir { "📁" } else { "📄" };
        println!("{:<4} {:<15} | {:<30} | {:<12} | {:<16}",
            icon, f.extension, f.name, format_size(f.size), f.created.format("%d.%m.%Y %H:%M")
        );
    }

    println!("\n📊 --- SUMÁŘ PODLE PŘÍTOMNÝCH TYPŮ ---");
    for (ext, (count, size)) in &stats {
        println!("{:<15}: {:>3} položek, celkem {:>10}", ext, count, format_size(*size));
    }

    Ok(())
}