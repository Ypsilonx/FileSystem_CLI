# 📁 FileSystem CLI

**Moderní interaktivní správce souborů v příkazové řádce napsaný v Rustu**

## 📋 Popis

FileSystem CLI je rychlý a efektivní nástroj pro skenování adresářů, analýzu souborů a zobrazení přehledných statistik. Po naskenování se program nepřepne zpět do shellu, ale zobrazí interaktivní menu, kde lze filtrovat, přesouvat, kopírovat i hromadně přejmenovávat soubory – vše bez opuštění aplikace.

## ✨ Funkce

### Aktuálně dostupné:
- 🔍 **Skenování adresářů** – Rychlé procházení souborového systému s progress spinnerem
- 📊 **Řazení souborů** podle názvu, velikosti, data vytvoření nebo přípony (vzestupně / sestupně)
- 🎛️ **Filtry** – kombinovatelné filtry podle přípony, části názvu, minimální a maximální velikosti (KB)
- ✏️ **Hromadné přejmenování** – prefix, suffix, náhrada textu nebo číslovaná šablona (`{name}_{num:03}.{ext}`) s náhledem před potvrzením
- 📂 **Přesun do složky** – přesune vyfiltrované soubory, automaticky vytvoří cílovou složku, cross-device fallback
- 📋 **Kopírování do složky** – zkopíruje vyfiltrované soubory
- 📁 **Rekurzivní měření** – automatický výpočet velikosti složek včetně jejich obsahu
- 🎨 **Přehledný výstup** – tabulkový formát s ikonami a jednotkami (B / KB / MB / GB), barevně zvýrazněný
- 🛡️ **Bezpečný chod** – nedostupné soubory jsou přeskočeny s varováním, program nekrachuje

## 🚀 Instalace

### Předpoklady
- Nainstalovaný [Rust](https://www.rust-lang.org/tools/install) (verze 1.70 nebo novější)

### Kompilace

```bash
cd FileSystem_CLI

cargo build --release

# Spustitelný soubor:
# target/release/FileSystem_CLI.exe  (Windows)
# target/release/FileSystem_CLI      (Linux/macOS)
```

## 📖 Použití

### Základní syntaxe

```bash
FileSystem_CLI [CESTA] [PARAMETRY]
```

### Parametry

| Parametr | Zkrácená forma | Popis | Výchozí hodnota |
|----------|----------------|-------|-----------------|
| `path` | – | Cesta k adresáři, který chcete skenovat | `.` (aktuální adresář) |
| `--order-by` | `-o` | Počáteční řazení: `name`, `size`, `date`, `ext` | `name` |
| `--direction` | `-d` | Směr řazení: `asc`, `desc` | `asc` |

Řazení i filtry lze kdykoli změnit přímo v interaktivním menu.

### 💡 Příklady spuštění

```bash
# Aktuální adresář
FileSystem_CLI

# Konkrétní cesta
FileSystem_CLI C:\Users\Dokumenty

# Seřadit od největších souborů
FileSystem_CLI D:\Projekty -o size -d desc
```

### 📊 Ukázkový výstup

```
📂  položky: 4/4  │  řazení: Name Asc  │  filtr: žádný filtr
──────────────────────────────────────────────────────────────────────────────────────────────
Typ   Přípona        │ Název                                  │ Velikost     │ Vytvořeno
──────────────────────────────────────────────────────────────────────────────────────────────
📁    Složka         │ src                                    │ 1.45 KB      │ 01.01.2026 14:25
📁    Složka         │ target                                 │ 45.23 MB     │ 01.01.2026 14:30
📄    md             │ README.md                              │ 3.12 KB      │ 02.01.2026 10:15
📄    toml           │ Cargo.toml                             │ 265 B        │ 01.01.2026 14:20
──────────────────────────────────────────────────────────────────────────────────────────────
📊 md: 1 (3.12 KB) │ Složka: 2 (46.68 MB) │ toml: 1 (265 B)

Akce  (↑↓ = pohyb, Enter = potvrdit, Esc = konec)
> Filtrovat / Upravit filtr          [–]
  Zrušit filtr                       [–]
  Hromadné přejmenování              [4 souborů]
  Přesunout do složky                [4 souborů]
  Zkopírovat do složky               [4 souborů]
  Změnit řazení                      [Name Asc]
  Znovu skenovat adresář
  Konec
```

## 🛠️ Technické detaily

- **Jazyk**: Rust 2024 Edition
- **Závislosti**:
  - `clap 4.6` – Parsování argumentů příkazové řádky
  - `chrono 0.4.44` – Práce s datem a časem
  - `indicatif 0.18` – Progress bar / spinner
  - `dialoguer 0.12` – Interaktivní menu, vstupy, potvrzení
  - `console 0.16` – Barevný výstup, clear screen

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

### Interaktivní menu
Po naskenování adresáře program zobrazí tabulku souborů a spustí smyčku s menu. Veškeré akce (filtrování, přejmenování, přesun) probíhají bez restartu – po dokončení operace se adresář automaticky znovu naskenuje.

### Filtry
Filtry se kombinují (AND logika). Aktivní filtr je zobrazen v hlavičce tabulky. Lze filtrovat současně podle přípony, části názvu i rozsahu velikosti.

### Hromadné přejmenování
Čtyři režimy s náhledem prvních 8 souborů před potvrzením:
- **Prefix** – `prefix_nazev.ext`
- **Suffix** – `nazev_suffix.ext`
- **Nahradit text** – jednoduchý find & replace v názvu
- **Šablona** – proměnné `{name}`, `{num}`, `{num:02}`, `{num:03}`, `{num:04}`, `{ext}`

### Přesun / kopírování
Při přesunu na jiný disk (cross-device) program automaticky použije fallback: zkopíruje soubor a smaže originál. Pokud cílová složka neexistuje, nabídne její vytvoření.

### Výpočet velikosti složek
Rekurzivně prochází všechny podsložky pro přesný výpočet celkové velikosti.

### Formátování velikostí
`< 1 024 B` → **B** · `< 1 MB` → **KB** · `< 1 GB` → **MB** · `≥ 1 GB` → **GB**

## 🤝 Přispění

Projekt je ve vývoji! Pokud máte nápady na vylepšení nebo chcete přispět, budu rád za jakoukoliv zpětnou vazbu.

## 📄 Licence

Tento projekt je vyvíjen jako open-source nástroj pro osobní i komerční použití.

## 👨‍💻 Autor

**Ypsilonx** – verze 2.0

---

*Vytvořeno s ❤️ v Rustu*
