use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::{DateTime, Local};

#[derive(Parser, Debug)]
#[command(name = "scan", author = "Parťák v programování", version = "1.1")]
struct Args {
    /// Cesta k adresáři
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Seřadit podle: name, size, date, ext
    #[arg(short, long, default_value = "name")]
    order_by: String,

    /// Směr řazení: asc, desc
    #[arg(short, long, default_value = "asc")]
    direction: String,
}

struct FileEntry {
    name: String,
    extension: String, // Budeme ukládat skutečnou příponu
    is_dir: bool,
    size: u64,
    created: DateTime<Local>,
}

fn get_dir_size(path: &Path) -> u64 {
    fs::read_dir(path).ok().map(|entries| {
        entries.flatten().map(|entry| {
            let meta = entry.metadata().unwrap();
            if meta.is_dir() { get_dir_size(&entry.path()) } else { meta.len() }
        }).sum()
    }).unwrap_or(0)
}

fn main() {
    let args = Args::parse();
    
    let entries_raw = match fs::read_dir(&args.path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("❌ Chyba: {}", e);
            return;
        }
    };
    
    let mut files: Vec<FileEntry> = Vec::new();
    // Mapa pro sumář: Přípona -> (Počet, Celková velikost)
    let mut stats: HashMap<String, (u32, u64)> = HashMap::new();

    for entry in entries_raw.flatten() {
        let path = entry.path();
        let metadata = fs::metadata(&path).unwrap();
        let is_dir = path.is_dir();
        
        let ext = if is_dir {
            "Složka".to_string()
        } else {
            path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("Bez přípony")
                .to_lowercase()
        };

        let size = if is_dir { get_dir_size(&path) } else { metadata.len() };
        let created: DateTime<Local> = metadata.created().or_else(|_| metadata.modified()).unwrap().into();

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

    // Třídění (včetně možnosti podle přípony)
    files.sort_by(|a, b| {
        let cmp = match args.order_by.as_str() {
            "size" => a.size.cmp(&b.size),
            "date" => a.created.cmp(&b.created),
            "ext" => a.extension.cmp(&b.extension),
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        };
        if args.direction == "desc" { cmp.reverse() } else { cmp }
    });

    // Výpis
    println!("\n{:<5} {:<15} | {:<30} | {:<12} | {:<16}", "Typ", "Přípona", "Název", "Velikost", "Vytvořeno");
    println!("{:-<90}", "");

    for f in &files {
        let icon = if f.is_dir { "📁" } else { "📄" };
        let size_str = if f.size > 1_048_576 { format!("{:.2} MB", f.size as f64 / 1_048_576.0) } 
                       else { format!("{:.2} KB", f.size as f64 / 1024.0) };

        println!("{:<4} {:<15} | {:<30} | {:<12} | {:<16}",
            icon, f.extension, f.name, size_str, f.created.format("%d.%m.%Y %H:%M")
        );
    }

    println!("\n📊 --- SUMÁŘ PODLE PŘÍTOMNÝCH TYPŮ ---");
    for (ext, (count, size)) in &stats {
        println!("{:<15}: {:>3} položek, celkem {:>10.2} MB", ext, count, *size as f64 / 1_048_576.0);
    }
}