<div align="center">

# auto-push

**Une commande pour commit et push — l'IA ecrit le message.**

Un outil CLI qui stage vos changements, genere un message de commit pertinent avec Claude, commit et push. Plus besoin d'ecrire vos messages de commit a la main.

![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-5C6BC0)
![License](https://img.shields.io/badge/License-MIT-green)
![Dependencies](https://img.shields.io/badge/Dependencies-5-7E57C2)

<a href="README.md"><img src="https://img.shields.io/badge/%F0%9F%87%AC%F0%9F%87%A7_Read_in_English-blue?style=for-the-badge" alt="Read in English"></a>

</div>

---

## Ce que ca fait

auto-push analyse votre diff git, collecte le contexte des README, l'envoie a l'API Claude, et genere un message de commit au format **Conventional Commits** automatiquement.

```
$ auto_push

Generated commit message:
feat: add user authentication via OAuth2

Implement Google OAuth2 login flow with token refresh.
Add session middleware and protect /dashboard routes.

Commit created successfully.
Changes pushed successfully.
```

### Fonctionnalites cles

- **Messages de commit generes par IA** via Claude (Haiku) et l'API Anthropic
- **Format Conventional Commits** — `feat:`, `fix:`, `refactor:`, `docs:`, etc.
- **Contextuel** — inclut le contenu des fichiers README pour des messages plus pertinents
- **Pipeline complet** — stage, commit et push en une seule commande
- **Persistance de la cle API** — sauvegarde votre cle dans `.zshrc`/`.bashrc` au premier lancement

---

## Demarrage rapide

```sh
cargo install --path .
auto_push
```

Au premier lancement, votre cle API Anthropic vous sera demandee. Elle sera sauvegardee automatiquement dans votre config shell.

Pour mettre a jour, relancez la meme commande. Pour desinstaller :

```sh
cargo uninstall auto_push
```

---

## Prerequis

- **Git** — doit etre lance dans un depot git
- **Cle API Anthropic** — obtenez-en une sur [console.anthropic.com](https://console.anthropic.com)

---

## Utilisation

```
auto_push
```

Lancez-le depuis n'importe quel depot git avec des changements. L'outil gere tout :

```
git add .  →  git diff --staged  →  API Claude  →  git commit  →  git push
```

---

## Comment ca marche

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Stage      │────▶│  Diff        │────▶│  Contexte    │────▶│  API Claude  │────▶│  Commit      │
│  git add .   │     │  --staged    │     │  + READMEs   │     │  (Haiku)     │     │  + Push      │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

| Etape | Fonction | Ce qu'elle fait |
|:------|:---------|:----------------|
| **Stage** | `main()` | Execute `git add .` pour stager tous les changements |
| **Diff** | `get_the_diff_for_github()` | Capture la sortie de `git diff --staged` |
| **Contexte** | `get_all_the_readmes()` | Collecte tous les fichiers `README*.md` pour le contexte du projet |
| **Generation** | `ask_claude()` | Envoie le diff + contexte a l'API Claude, retourne un message de commit |
| **Commit & Push** | `main()` | Execute `git commit -m` avec le message genere, puis `git push` |

---

## Stack technique

| | Crate | Utilisation |
|:-|:------|:------------|
| ![Reqwest](https://img.shields.io/badge/reqwest-0.13-5C6BC0?logoColor=white) | reqwest | Client HTTP pour les appels a l'API Claude |
| ![Serde](https://img.shields.io/badge/serde-1-7E57C2?logoColor=white) | serde + serde_json | Serialisation JSON pour les requetes/reponses API |
| ![Tokio](https://img.shields.io/badge/tokio-1-9575CD?logoColor=white) | tokio | Runtime async pour les requetes HTTP |
| ![Glob](https://img.shields.io/badge/glob-0.3-7986CB?logoColor=white) | glob | Pattern matching de fichiers pour la decouverte des README |

**5 dependances.** Architecture mono-fichier, aucun overhead de framework.

---

## Structure du projet

```
auto-push/
├── src/
│   └── main.rs        # Tout : appels API, commandes git, generation de prompt
├── Cargo.toml
├── Cargo.lock
├── README.md
├── README.fr.md
└── .gitignore
```

---

## Licence

MIT

---

<p align="center">
  <sub>Construit par Mateon — Propulse par Rust & Claude</sub>
</p>
