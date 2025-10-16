# snapshotter

Petit outil (lib + binaire) pour **créer des snapshots découpés** d’un dépôt Rust. (utiles quand on veut copier coller dans chatgpt)
Il produit des fichiers “part” contenant des blocs « header + contenu », limités en nombre de lignes pour faciliter la revue ou l’archivage.

---

## But

Prendre le code important d’un projet Rust et le couper en morceaux lisibles (fichiers `*_partN.txt`) sans charger tout en mémoire.

---

## Public visé

Débutants Linux / développeurs débutants Rust.

---

## Prérequis

* Linux (ou macOS). Fonctionne aussi sous Windows mais les commandes d’exemples ci-dessous sont pour Linux.
* Rust toolchain (rustc + cargo). Si absent :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# puis
source "$HOME/.cargo/env"
```

* (optionnel) droits en écriture sur le répertoire de destination.

---

## Construire et installer

Depuis la racine du projet (là où se trouve `Cargo.toml`) :

Compiler en debug (rapide) :

```bash
cargo build
```

Compiler en release (binaire optimisé) :

```bash
cargo build --release
```

Installer le binaire localement (dans `~/.cargo/bin`) :

```bash
cargo install --path .
# ou pour forcer la réinstallation
cargo install --path . --force
```

Vérifier le binaire :

```bash
snapshotter --help
# si le binaire est installé, ceci affiche l'aide (ou tester `cargo run --bin snapshotter -- --help`)
```

> Remarque : le projet a une **lib** réutilisable et un **bin** (`src/bin/cli.rs`). Si vous préférez exécuter sans installer : `cargo run --release --bin snapshotter -- <options>`.

---

## Utilisation (rapide, programmatique)

La bibliothèque expose `SnapshotConfig` et `Snapshotter`. Exemple simple (extrait de la doc) — peut servir dans un petit programme Rust :

```rust
use snapshotter::{SnapshotConfig, Snapshotter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // workspace_root = répertoire du projet à snapshotter
    let cfg = SnapshotConfig::new(".", "snapshot", 1500)?;
    let snapshotter = Snapshotter::new(cfg);
    let res = snapshotter.run()?;
    println!("Parts: {}", res.parts.len());
    Ok(())
}
```

Options importantes de `SnapshotConfig` :

* `workspace_root` : chemin vers le projet à traiter.
* `prefix` : préfixe des fichiers de sortie (`<prefix>_part1.txt`).
* `max_lines` : nombre maximal de lignes par part (entier > 0).
* `with_dest(path)` : définit le répertoire de sortie.
* `with_dry_run(true)` : n’écrit rien, simule l’exécution.

---

## Exemple d’usage CLI (générique)

Le CLI peut accepter des options — si le binaire expose des flags, vérifiez avec `snapshotter --help` ou `cargo run --bin snapshotter -- --help`.
Exemple d’usage possible (adaptez selon les flags réels) :

```bash
# exécution réelle (écrit dans ./out)
snapshotter --root . --prefix snap --max-lines 1500 --dest out

# mode simulation (ne crée aucun fichier)
snapshotter --root . --prefix snap --max-lines 1500 --dest out --dry-run
```

> Si les noms de flags diffèrent, utilisez `--help` pour connaître les options exactes.

---

## Ce que l’outil collecte (par défaut)

Le collecteur suit une politique simple et déterministe :

* `./src` : uniquement les fichiers `*.rs` (non récursif).
* `./tests` : tous les fichiers (récursif).
* `./migrations` : tous les fichiers (récursif).
* fichiers racine : `askama.toml`, `Cargo.toml` (s’ils existent).

Les chemins sont triés pour donner un ordre déterministe.

---

## Logique de découpage — explication simple

* Pour chaque fichier collecté on calcule `block_lines = 1 (header) + file_lines + 2` (deux lignes vides).
* Si `block_lines >= max_lines` → le fichier est isolé dans **sa propre** part (pour ne pas casser le seuil).
* Sinon on ajoute le bloc à la part courante. Si l’ajout dépasse `max_lines`, on finalise la part et on ouvre une nouvelle part.
* L’écriture se fait **en streaming** (flux) : le contenu est copié sans charger tout le fichier en mémoire.

---

## Fichiers générés

Pour `prefix = snapshot` :

* `snapshot_part1.txt`, `snapshot_part2.txt`, ...
* `snapshot_summary.txt` — résumé listant chaque part, nombre de fichiers et de lignes, total.

En `dry-run`, aucun fichier n’est créé (utile pour tester sans toucher le disque).

---

## Erreurs courantes & dépannage

* `NoFiles` : le collecteur n’a trouvé aucun fichier correspondant. Vérifiez que le projet a `src` ou `tests` ou les fichiers ciblés.
* `IO error` (permission denied) : vérifiez les droits sur le répertoire de destination.
* `max_lines must be > 0` : `SnapshotConfig::new` refuse 0 — passez un entier positif.
* Pour tester sans écrire : activez `dry_run`.

---

## Pour les développeurs

* MSRV (minimum supported Rust version) : **1.90.0**.
* Lancer les tests :

```bash
cargo test
```

* Dépendances utiles en dev : `tempfile` (utilisé dans les tests d’intégration).
* Feature optionnelle `completion` : permet de compiler la génération de scripts de complétion (zsh/bash/fish) si le binaire expose ce comportement. Compiler :

```bash
cargo build --features completion
```

Consultez l’aide du binaire pour la commande exacte de génération des scripts.

---

## Contribution rapide

1. Fork → clone → nouvelle branche.
2. Respecter l’édition Rust et la MSRV.
3. Tests unitaires/integration ajoutés pour toute logique nouvelle.
4. PR claire + description.

---

## Licence

MIT OR Apache-2.0 (dual). Voir `Cargo.toml`.

---

## Résumé en 3 lignes (si vous n’avez pas le temps)

1. Installez Rust (`rustup`), puis `cargo build --release` ou `cargo install --path .`.
2. Configurez un `SnapshotConfig` (ou utilisez le binaire) avec `prefix` et `max_lines`.
3. Exécutez : le projet est découpé en `*_partN.txt` + `*_summary.txt`. Utilisez `dry_run` pour tester.
