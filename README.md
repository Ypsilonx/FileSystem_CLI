# 📁 FileSystem CLI

**Moderní nástroj pro analýzu a správu souborů v příkazové řádce napsaný v Rustu**

## 📋 Popis

FileSystem CLI je rychlý a efektivní nástroj pro skenování adresářů, analýzu souborů a zobrazení přehledných statistik. Aplikace umožňuje řadit soubory podle různých kritérií a poskytuje přehledný sumář o velikostech a typech souborů v adresáři.

## ✨ Funkce

### Aktuálně dostupné:
- 🔍 **Skenování adresářů** - Rychlé procházení souborového systému
- 📊 **Řazení souborů** podle:
  - Názvu (`name`)
  - Velikosti (`size`)
  - Data vytvoření (`date`)
  - Přípony (`ext`)
- ⬆️⬇️ **Směr řazení** - Vzestupně (`asc`) nebo sestupně (`desc`)
- 📈 **Statistický sumář** - Přehled typů souborů, jejich počtu a celkové velikosti
- 📁 **Rekurzivní měření** - Automatický výpočet velikosti složek včetně jejich obsahu
- 🎨 **Přehledný výstup** - Tabulkový formát s ikonami a jednotkami (KB/MB)

### 🚧 V plánu:
- 📂 Automatické třídění souborů do složek podle parametrů (přípona, velikost, datum)
- ✏️ Hromadné přejmenování souborů pomocí předpon a šablon

## 🚀 Instalace

### Předpoklady
- Nainstalovaný [Rust](https://www.rust-lang.org/tools/install) (verze 1.70 nebo novější)

### Kompilace

```bash
# Naklonujte repozitář (nebo stáhněte zdrojové kódy)
cd FileSystem_CLI

# Zkompilujte projekt
cargo build --release

# Spustitelný soubor najdete v:
# target/release/FileSystem_CLI.exe (Windows)
# target/release/FileSystem_CLI (Linux/macOS)
```

## 📖 Použití

### Základní syntaxe

```bash
FileSystem_CLI [CESTA] [PARAMETRY]
```

### Parametry

| Parametr | Zkrácená forma | Popis | Výchozí hodnota |
|----------|----------------|-------|-----------------|
| `path` | - | Cesta k adresáři, který chcete skenovat | `.` (aktuální adresář) |
| `--order-by` | `-o` | Způsob řazení: `name`, `size`, `date`, `ext` | `name` |
| `--direction` | `-d` | Směr řazení: `asc` (vzestupně), `desc` (sestupně) | `asc` |

### 💡 Příklady použití

#### 1. Skenování aktuálního adresáře
```bash
FileSystem_CLI
```

#### 2. Skenování konkrétního adresáře
```bash
FileSystem_CLI C:\Users\Dokumenty
```

#### 3. Řazení podle velikosti (od největších)
```bash
FileSystem_CLI -o size -d desc
```

#### 4. Řazení podle data vytvoření (od nejnovějších)
```bash
FileSystem_CLI --order-by date --direction desc
```

#### 5. Řazení podle přípony
```bash
FileSystem_CLI -o ext
```

#### 6. Komplexní příklad
```bash
FileSystem_CLI D:\Projekty -o size -d desc
```

### 📊 Ukázkový výstup

```
Typ   Přípona         | Název                          | Velikost     | Vytvořeno
------------------------------------------------------------------------------------------
📁    Složka          | target                         | 45.23 MB     | 01.01.2026 14:30
📁    Složka          | src                            | 1.45 MB      | 01.01.2026 14:25
📄    toml            | Cargo.toml                     | 0.25 KB      | 01.01.2026 14:20
📄    md              | README.md                      | 3.12 KB      | 02.01.2026 10:15

📊 --- SUMÁŘ PODLE PŘÍTOMNÝCH TYPŮ ---
Složka         :   2 položek, celkem      46.68 MB
toml           :   1 položek, celkem       0.00 MB
md             :   1 položek, celkem       0.00 MB
```

## 🛠️ Technické detaily

- **Jazyk**: Rust 2024 Edition
- **Závislosti**:
  - `clap 4.4` - Parsování argumentů příkazové řádky
  - `chrono 0.4` - Práce s datem a časem

### Struktura projektu

```
FileSystem_CLI/
├── src/
│   └── main.rs          # Hlavní logika aplikace
├── Cargo.toml           # Konfigurace projektu a závislostí
├── README.md            # Tento soubor
└── target/              # Zkompilované binárky (po build)
```

## 📝 Funkce v detailu

### Výpočet velikosti složek
Aplikace rekurzivně prochází všechny podsložky a soubory pro přesný výpočet celkové velikosti složky.

### Formátování velikostí
- Soubory menší než 1 MB se zobrazují v **KB** (kilobytech)
- Soubory větší než 1 MB se zobrazují v **MB** (megabytech)
- Sumář vždy zobrazuje velikosti v MB pro konzistenci

### Soubory bez přípony
Soubory bez přípony jsou označeny jako `"Bez přípony"` ve výpisu a statistikách.

## 🤝 Přispění

Projekt je ve vývoji! Pokud máte nápady na vylepšení nebo chcete přispět, budu rád za jakoukoliv zpětnou vazbu.

## 📄 Licence

Tento projekt je vyvíjen jako open-source nástroj pro osobní i komerční použití.

## 👨‍💻 Autor

**Ypsilonx** - verze 1.1

---

*Vytvořeno s ❤️ v Rustu*
