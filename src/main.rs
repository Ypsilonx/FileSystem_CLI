use clap::{Parser, ValueEnum};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};

// ─────────────────────────────────────────────
//  CLI argumenty
// ─────────────────────────────────────────────

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum OrderBy {
    Name,
    Size,
    Date,
    Ext,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum Direction {
    Asc,
    Desc,
}

#[derive(Parser, Debug)]
#[command(
    name = "fscli",
    author = "Tomáš Cibulec",
    version = "2.0",
    about = "Interaktivní správce souborů v terminálu"
)]
struct Args {
    /// Cesta k adresáři (výchozí: aktuální)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Počáteční řazení: name, size, date, ext
    #[arg(short, long, default_value = "name", value_enum)]
    order_by: OrderBy,

    /// Směr řazení: asc, desc
    #[arg(short, long, default_value = "asc", value_enum)]
    direction: Direction,
}

// ─────────────────────────────────────────────
//  Datové struktury
// ─────────────────────────────────────────────

#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
    name: String,
    extension: String,
    is_dir: bool,
    size: u64,
    created: DateTime<Local>,
}

#[derive(Default, Clone)]
struct Filter {
    extension: Option<String>,
    name_contains: Option<String>,
    min_size_kb: Option<u64>,
    max_size_kb: Option<u64>,
}

impl Filter {
    fn is_empty(&self) -> bool {
        self.extension.is_none()
            && self.name_contains.is_none()
            && self.min_size_kb.is_none()
            && self.max_size_kb.is_none()
    }

    fn description(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(ext) = &self.extension {
            parts.push(format!("přípona={}", ext));
        }
        if let Some(name) = &self.name_contains {
            parts.push(format!("název⊃'{}'", name));
        }
        if let Some(min) = self.min_size_kb {
            parts.push(format!("min={}KB", min));
        }
        if let Some(max) = self.max_size_kb {
            parts.push(format!("max={}KB", max));
        }
        if parts.is_empty() {
            "–".to_string()
        } else {
            parts.join(", ")
        }
    }

    fn apply<'a>(&self, files: &'a [FileEntry]) -> Vec<&'a FileEntry> {
        files
            .iter()
            .filter(|f| {
                if let Some(ext) = &self.extension {
                    if f.extension.to_lowercase() != ext.to_lowercase() {
                        return false;
                    }
                }
                if let Some(name) = &self.name_contains {
                    if !f.name.to_lowercase().contains(&name.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(min) = self.min_size_kb {
                    if f.size < min * 1024 {
                        return false;
                    }
                }
                if let Some(max) = self.max_size_kb {
                    if f.size > max * 1024 {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

// ─────────────────────────────────────────────
//  Pomocné funkce
// ─────────────────────────────────────────────

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

    let mut order_by = args.order_by;
    let mut direction = args.direction;
    let mut filter = Filter::default();
    let mut files = scan_directory(&args.path);

    let theme = ColorfulTheme::default();

    loop {
        sort_files(&mut files, &order_by, &direction);
        display_files(&files, &filter, &order_by, &direction);

        let visible_count = filter.apply(&files).len();
        let filter_label = if filter.is_empty() {
            "–".to_string()
        } else {
            filter.description()
        };

        let menu_items = vec![
            format!("Filtrovat / Upravit filtr          [{}]", filter_label),
            format!(
                "Zrušit filtr                       [{}]",
                if filter.is_empty() { "–" } else { "aktivní" }
            ),
            format!("Hromadné přejmenování              [{} souborů]", visible_count),
            format!("Přesunout do složky                [{} souborů]", visible_count),
            format!("Zkopírovat do složky               [{} souborů]", visible_count),
            format!("Změnit řazení                      [{:?} {:?}]", order_by, direction),
            "Znovu skenovat adresář".to_string(),
            "Konec".to_string(),
        ];

        let sel = Select::with_theme(&theme)
            .with_prompt("Akce  (↑↓ = pohyb, Enter = potvrdit, Esc = konec)")
            .items(&menu_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match sel {
            Some(0) => menu_filter(&mut filter),
            Some(1) => filter = Filter::default(),
            Some(2) => {
                let changed = menu_rename(&files, &filter, &args.path)?;
                if changed > 0 {
                    files = scan_directory(&args.path);
                }
            }
            Some(3) => {
                let changed = menu_move_copy(&files, &filter, &args.path, true)?;
                if changed > 0 {
                    files = scan_directory(&args.path);
                }
            }
            Some(4) => {
                let _ = menu_move_copy(&files, &filter, &args.path, false)?;
            }
            Some(5) => menu_order(&mut order_by, &mut direction),
            Some(6) => {
                files = scan_directory(&args.path);
            }
            _ => break,
        }
    }

    println!("\n{}", style("Nashledanou! 👋").cyan().bold());
    Ok(())
}

// ─────────────────────────────────────────────
//  Skenování adresáře
// ─────────────────────────────────────────────

fn scan_directory(path: &Path) -> Vec<FileEntry> {
    let entries_raw = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("❌ Chyba při čtení adresáře: {}", e);
            return vec![];
        }
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut files: Vec<FileEntry> = Vec::new();

    for entry in entries_raw.flatten() {
        let entry_path = entry.path();
        spinner.set_message(format!("Skenuji: {}", entry.file_name().to_string_lossy()));

        let metadata = match fs::metadata(&entry_path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("⚠️  Přeskakuji {:?}: {}", entry_path, e);
                continue;
            }
        };

        let is_dir = metadata.is_dir();
        let ext = if is_dir {
            "Složka".to_string()
        } else {
            entry_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("Bez přípony")
                .to_lowercase()
        };

        let size = if is_dir { get_dir_size(&entry_path) } else { metadata.len() };
        let created: DateTime<Local> = metadata
            .created()
            .or_else(|_| metadata.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .into();

        files.push(FileEntry {
            path: entry_path,
            name: entry.file_name().to_string_lossy().into_owned(),
            extension: ext,
            is_dir,
            size,
            created,
        });
    }

    spinner.finish_and_clear();
    files
}

// ─────────────────────────────────────────────
//  Řazení
// ─────────────────────────────────────────────

fn sort_files(files: &mut [FileEntry], order_by: &OrderBy, direction: &Direction) {
    files.sort_by(|a, b| {
        let cmp = match order_by {
            OrderBy::Size => a.size.cmp(&b.size),
            OrderBy::Date => a.created.cmp(&b.created),
            OrderBy::Ext  => a.extension.cmp(&b.extension),
            OrderBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        };
        match direction {
            Direction::Desc => cmp.reverse(),
            Direction::Asc  => cmp,
        }
    });
}

// ─────────────────────────────────────────────
//  Výpis souborů
// ─────────────────────────────────────────────

fn display_files(files: &[FileEntry], filter: &Filter, order_by: &OrderBy, direction: &Direction) {
    let _ = Term::stdout().clear_screen();

    let visible: Vec<&FileEntry> = filter.apply(files);

    let filter_info = if filter.is_empty() {
        style("žádný filtr").dim().to_string()
    } else {
        style(filter.description()).yellow().bold().to_string()
    };

    println!(
        "{}  položky: {}{}  │  řazení: {} {}  │  filtr: {}",
        style("📂").bold(),
        style(visible.len()).cyan().bold(),
        style(format!("/{}", files.len())).dim(),
        style(format!("{:?}", order_by)).green(),
        style(format!("{:?}", direction)).green(),
        filter_info
    );
    println!("{:─<94}", "");
    println!(
        "{:<5} {:<14} │ {:<38} │ {:<12} │ {}",
        "Typ", "Přípona", "Název", "Velikost", "Vytvořeno"
    );
    println!("{:─<94}", "");

    for f in &visible {
        let icon = if f.is_dir { "📁" } else { "📄" };
        let name_display = if f.name.len() > 37 {
            format!("{}…", &f.name[..36])
        } else {
            f.name.clone()
        };
        println!(
            "{:<4} {:<14} │ {:<38} │ {:<12} │ {}",
            icon,
            f.extension,
            name_display,
            format_size(f.size),
            f.created.format("%d.%m.%Y %H:%M")
        );
    }

    println!("{:─<94}", "");

    // Sumář viditelných souborů
    let mut stats: BTreeMap<&str, (u32, u64)> = BTreeMap::new();
    for f in &visible {
        let s = stats.entry(f.extension.as_str()).or_insert((0, 0));
        s.0 += 1;
        s.1 += f.size;
    }
    let summary: Vec<String> = stats
        .iter()
        .map(|(ext, (count, size))| format!("{}: {} ({})", ext, count, format_size(*size)))
        .collect();
    println!("{} {}\n", style("📊").bold(), summary.join(" │ "));
}

// ─────────────────────────────────────────────
//  Menu: filtry
// ─────────────────────────────────────────────

fn menu_filter(filter: &mut Filter) {
    let theme = ColorfulTheme::default();
    loop {
        let options = vec![
            format!(
                "Přípona                   [{}]",
                filter.extension.as_deref().unwrap_or("–")
            ),
            format!(
                "Název obsahuje            [{}]",
                filter.name_contains.as_deref().unwrap_or("–")
            ),
            format!(
                "Min. velikost (KB)        [{}]",
                filter.min_size_kb.map(|v| v.to_string()).as_deref().unwrap_or("–")
            ),
            format!(
                "Max. velikost (KB)        [{}]",
                filter.max_size_kb.map(|v| v.to_string()).as_deref().unwrap_or("–")
            ),
            "Zrušit všechny filtry".to_string(),
            "◀ Zpět (uložit)".to_string(),
        ];

        let sel = Select::with_theme(&theme)
            .with_prompt("Nastavit filtr (Esc = zpět)")
            .items(&options)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match sel {
            Some(0) => {
                let val: String = Input::with_theme(&theme)
                    .with_prompt("Přípona (prázdné = zrušit filtr)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap_or_default();
                filter.extension = if val.trim().is_empty() {
                    None
                } else {
                    Some(val.trim().to_lowercase())
                };
            }
            Some(1) => {
                let val: String = Input::with_theme(&theme)
                    .with_prompt("Název obsahuje (prázdné = zrušit filtr)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap_or_default();
                filter.name_contains = if val.trim().is_empty() {
                    None
                } else {
                    Some(val.trim().to_string())
                };
            }
            Some(2) => {
                let val: String = Input::with_theme(&theme)
                    .with_prompt("Min. velikost v KB (prázdné = zrušit filtr)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap_or_default();
                filter.min_size_kb = val.trim().parse::<u64>().ok();
            }
            Some(3) => {
                let val: String = Input::with_theme(&theme)
                    .with_prompt("Max. velikost v KB (prázdné = zrušit filtr)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap_or_default();
                filter.max_size_kb = val.trim().parse::<u64>().ok();
            }
            Some(4) => {
                *filter = Filter::default();
            }
            _ => break,
        }
    }
}

// ─────────────────────────────────────────────
//  Menu: hromadné přejmenování
// ─────────────────────────────────────────────

fn menu_rename(
    files: &[FileEntry],
    filter: &Filter,
    base_path: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let theme = ColorfulTheme::default();
    let targets: Vec<&FileEntry> = filter
        .apply(files)
        .into_iter()
        .filter(|f| !f.is_dir)
        .collect();

    if targets.is_empty() {
        println!("⚠️  Žádné soubory pro přejmenování (složky jsou přeskočeny).");
        let _: String = Input::with_theme(&theme)
            .with_prompt("Stiskněte Enter")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();
        return Ok(0);
    }

    println!(
        "\n{}  Hromadné přejmenování – {} souborů\n",
        style("✏️").bold(),
        style(targets.len()).cyan()
    );

    let mode_options = vec![
        "Přidat předponu (prefix_nazev.ext)",
        "Přidat příponu k názvu (nazev_suffix.ext)",
        "Nahradit text v názvu",
        "Číslované přejmenování (šablona)",
        "◀ Zpět",
    ];

    let mode_idx = match Select::with_theme(&theme)
        .with_prompt("Způsob přejmenování")
        .items(&mode_options)
        .default(0)
        .interact_opt()
        .unwrap_or(None)
    {
        Some(i) if i < 4 => i,
        _ => return Ok(0),
    };

    let rename_pairs: Vec<(PathBuf, String)> = match mode_idx {
        0 => {
            let prefix: String = Input::with_theme(&theme)
                .with_prompt("Předpona")
                .interact_text()?;
            targets
                .iter()
                .map(|f| (f.path.clone(), format!("{}{}", prefix, f.name)))
                .collect()
        }
        1 => {
            let suffix: String = Input::with_theme(&theme)
                .with_prompt("Přípona k názvu (vkládá se před .ext)")
                .interact_text()?;
            targets
                .iter()
                .map(|f| {
                    let stem = Path::new(&f.name)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&f.name);
                    let new_name = if f.extension.is_empty() || f.extension == "bez přípony" {
                        format!("{}{}", stem, suffix)
                    } else {
                        format!("{}{}.{}", stem, suffix, f.extension)
                    };
                    (f.path.clone(), new_name)
                })
                .collect()
        }
        2 => {
            let old_text: String = Input::with_theme(&theme)
                .with_prompt("Hledaný text")
                .interact_text()?;
            let new_text: String = Input::with_theme(&theme)
                .with_prompt("Náhrada (prázdné = smazat výskyt)")
                .allow_empty(true)
                .interact_text()?;
            targets
                .iter()
                .map(|f| (f.path.clone(), f.name.replace(&old_text, &new_text)))
                .collect()
        }
        3 => {
            println!(
                "{}",
                style("  Proměnné: {name}, {num}, {num:02}, {num:03}, {num:04}, {ext}").dim()
            );
            let template: String = Input::with_theme(&theme)
                .with_prompt("Šablona")
                .default("{name}_{num:03}.{ext}".to_string())
                .interact_text()?;
            targets
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let stem = Path::new(&f.name)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&f.name);
                    let n = i + 1;
                    let new_name = template
                        .replace("{name}", stem)
                        .replace("{num:04}", &format!("{:04}", n))
                        .replace("{num:03}", &format!("{:03}", n))
                        .replace("{num:02}", &format!("{:02}", n))
                        .replace("{num}", &n.to_string())
                        .replace("{ext}", &f.extension);
                    (f.path.clone(), new_name)
                })
                .collect()
        }
        _ => return Ok(0),
    };

    // Náhled
    println!("\n{}", style("📋 Náhled přejmenování (prvních 8):").bold());
    println!(
        "{:<42} → {}",
        style("Původní název").dim(),
        style("Nový název").green()
    );
    println!("{:─<84}", "");
    for (old_path, new_name) in rename_pairs.iter().take(8) {
        let old_name = old_path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        println!("{:<42} → {}", old_name, style(new_name).green());
    }
    if rename_pairs.len() > 8 {
        println!("  … a dalších {}", rename_pairs.len() - 8);
    }
    println!();

    let confirm = Confirm::with_theme(&theme)
        .with_prompt(format!("Přejmenovat {} souborů?", rename_pairs.len()))
        .default(false)
        .interact()?;

    if !confirm {
        return Ok(0);
    }

    let mut count = 0usize;
    let mut errors = 0usize;
    for (old_path, new_name) in &rename_pairs {
        if new_name.is_empty()
            || new_name == old_path.file_name().and_then(|n| n.to_str()).unwrap_or("")
        {
            continue;
        }
        let new_path = old_path.parent().unwrap_or(base_path).join(new_name);
        match fs::rename(old_path, &new_path) {
            Ok(_) => count += 1,
            Err(e) => {
                eprintln!(
                    "⚠️  Chyba u '{}': {}",
                    old_path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                    e
                );
                errors += 1;
            }
        }
    }

    if errors > 0 {
        println!(
            "✅ Přejmenováno: {}  ⚠️ Chyby: {}",
            style(count).green(),
            style(errors).red()
        );
    } else {
        println!("{} Přejmenováno {} souborů.", style("✅").bold(), count);
    }
    Ok(count)
}

// ─────────────────────────────────────────────
//  Menu: přesun / kopírování
// ─────────────────────────────────────────────

fn menu_move_copy(
    files: &[FileEntry],
    filter: &Filter,
    _base_path: &Path,
    is_move: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let theme = ColorfulTheme::default();
    let targets: Vec<&FileEntry> = filter
        .apply(files)
        .into_iter()
        .filter(|f| !f.is_dir)
        .collect();

    let action = if is_move { "Přesunout" } else { "Zkopírovat" };

    if targets.is_empty() {
        println!("⚠️  Žádné soubory k operaci (složky jsou přeskočeny).");
        let _: String = Input::with_theme(&theme)
            .with_prompt("Stiskněte Enter")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();
        return Ok(0);
    }

    let target_str: String = Input::with_theme(&theme)
        .with_prompt(format!("{} {} souborů do cílové složky", action, targets.len()))
        .interact_text()?;

    let target_path = PathBuf::from(target_str.trim());

    if !target_path.exists() {
        let create = Confirm::with_theme(&theme)
            .with_prompt(format!(
                "Složka '{}' neexistuje. Vytvořit?",
                target_path.display()
            ))
            .default(true)
            .interact()?;
        if create {
            fs::create_dir_all(&target_path)?;
        } else {
            return Ok(0);
        }
    }

    let confirm = Confirm::with_theme(&theme)
        .with_prompt(format!(
            "{} {} souborů do '{}'?",
            action,
            targets.len(),
            target_path.display()
        ))
        .default(false)
        .interact()?;

    if !confirm {
        return Ok(0);
    }

    let mut count = 0usize;
    let mut errors = 0usize;
    for f in &targets {
        let dest = target_path.join(&f.name);
        let result = if is_move {
            // Pokus o přesun; při cross-device chybě fallback na kopii + smazání
            fs::rename(&f.path, &dest)
                .or_else(|_| fs::copy(&f.path, &dest).and_then(|_| fs::remove_file(&f.path)))
        } else {
            fs::copy(&f.path, &dest).map(|_| ())
        };
        match result {
            Ok(_) => count += 1,
            Err(e) => {
                eprintln!("⚠️  Chyba u '{}': {}", f.name, e);
                errors += 1;
            }
        }
    }

    if errors > 0 {
        println!(
            "{}: {}  ⚠️ Chyby: {}",
            action,
            style(count).green(),
            style(errors).red()
        );
    } else {
        println!("{} {}: {} souborů.", style("✅").bold(), action, count);
    }
    Ok(count)
}

// ─────────────────────────────────────────────
//  Menu: změna řazení
// ─────────────────────────────────────────────

fn menu_order(order_by: &mut OrderBy, direction: &mut Direction) {
    let theme = ColorfulTheme::default();

    let order_options = ["Název", "Velikost", "Datum", "Přípona"];
    let current_order = match order_by {
        OrderBy::Name => 0,
        OrderBy::Size => 1,
        OrderBy::Date => 2,
        OrderBy::Ext  => 3,
    };

    if let Some(s) = Select::with_theme(&theme)
        .with_prompt("Řadit podle")
        .items(&order_options)
        .default(current_order)
        .interact_opt()
        .unwrap_or(None)
    {
        *order_by = match s {
            0 => OrderBy::Name,
            1 => OrderBy::Size,
            2 => OrderBy::Date,
            _ => OrderBy::Ext,
        };
    }

    let dir_options = [
        "Vzestupně  (A→Z / malé→velké)",
        "Sestupně   (Z→A / velké→malé)",
    ];
    let current_dir = match direction {
        Direction::Asc  => 0,
        Direction::Desc => 1,
    };

    if let Some(s) = Select::with_theme(&theme)
        .with_prompt("Směr řazení")
        .items(&dir_options)
        .default(current_dir)
        .interact_opt()
        .unwrap_or(None)
    {
        *direction = if s == 0 { Direction::Asc } else { Direction::Desc };
    }
}