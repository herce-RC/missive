<<<<<<< HEAD
# Tauri Email Client

Un client email moderne développé avec Tauri, **Nuxt 3** et Rust, utilisant SurrealDB comme base de données embarquée.

## 🚀 Fonctionnalités

- 📥 Réception d'emails via IMAP
- 📤 Envoi d'emails via SMTP
- 📁 Gestion des dossiers (Boîte de réception, Envoyés, Brouillons, Corbeille)
- ⭐ Marquage des emails importants
- 🔍 Recherche dans les emails
- 📎 Support des pièces jointes
- 🔐 Connexion sécurisée SSL/TLS
- 💾 Stockage local avec SurrealDB
- 🎨 Interface moderne et responsive

## 📋 Prérequis

- [Node.js](https://nodejs.org/) (v18 ou supérieur)
- [Rust](https://www.rust-lang.org/tools/install) (dernière version stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation des prérequis sur Linux (Debian/Ubuntu)

```bash
# Dépendances système pour Tauri
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Installation de Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Installation de Node.js (via nvm recommandé)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

## 🛠️ Installation

1. **Cloner le projet**
```bash
cd /home/lucile/Public/tauri-email-client
```

2. **Installer les dépendances Node.js**
```bash
npm install
```

3. **Lancer en mode développement**
```bash
npm run tauri dev
```

4. **Compiler pour la production**
```bash
npm run tauri build
```

## 📁 Structure du projet

```
tauri-email-client/
├── app.vue                  # Layout racine Nuxt
├── assets/                  # Styles CSS
├── components/              # Composants Vue
│   ├── Sidebar.vue          # Barre latérale
│   ├── Header.vue           # En-tête avec recherche
│   └── EmailList.vue        # Liste des emails
├── pages/                   # Pages Nuxt
│   ├── index.vue            # Boîte de réception
│   ├── compose.vue          # Composition d'email
│   ├── email/[id].vue       # Lecture d'email
│   └── settings.vue         # Paramètres
├── stores/                  # Pinia stores
│   └── emailStore.ts        # Store des emails
├── src-tauri/               # Backend Rust
│   ├── src/
│   │   ├── main.rs          # Point d'entrée Rust
│   │   ├── lib.rs           # Configuration Tauri
│   │   ├── commands.rs      # Commandes Tauri
│   │   ├── database.rs      # Couche SurrealDB
│   │   ├── email.rs         # Client IMAP/SMTP
│   │   └── models.rs        # Modèles de données
│   ├── Cargo.toml           # Dépendances Rust
│   └── tauri.conf.json      # Configuration Tauri
├── package.json             # Dépendances Node.js
├── nuxt.config.ts           # Configuration Nuxt
└── README.md
```

## 🔧 Configuration d'un compte email

1. Lancez l'application
2. Allez dans **Paramètres** (⚙️)
3. Cliquez sur **Ajouter un compte**
4. Remplissez les informations :
   - Adresse email
   - Serveur IMAP (ex: imap.gmail.com)
   - Port IMAP (généralement 993)
   - Serveur SMTP (ex: smtp.gmail.com)
   - Port SMTP (généralement 587)
   - Nom d'utilisateur
   - Mot de passe (ou mot de passe d'application)

### Configuration pour Gmail

Pour Gmail, vous devez :
1. Activer l'accès IMAP dans les paramètres Gmail
2. Créer un "Mot de passe d'application" dans les paramètres de sécurité Google
3. Utiliser ce mot de passe d'application dans le client

## 🗄️ Base de données

Le projet utilise **SurrealDB** en mode embarqué (in-memory) pour stocker :
- Les emails synchronisés
- Les comptes email configurés
- Les préférences utilisateur

Les données sont persistées localement et ne nécessitent pas de serveur externe.

## 🔐 Sécurité

- Les mots de passe sont stockés localement (considérez l'utilisation d'un gestionnaire de secrets pour la production)
- Les connexions IMAP/SMTP utilisent SSL/TLS par défaut
- Aucune donnée n'est envoyée à des serveurs tiers

## 📝 Technologies utilisées

### Frontend
- **Nuxt 3** - Framework Vue
- **Pinia** - Gestion d'état
- **TypeScript** - Typage statique

### Backend
- **Tauri 2** - Framework desktop
- **Rust** - Langage backend
- **SurrealDB** - Base de données embarquée
- **async-imap** - Client IMAP asynchrone
- **lettre** - Client SMTP
- **mail-parser** - Parsing des emails

## 🤝 Contribution

Les contributions sont les bienvenues ! N'hésitez pas à :
1. Fork le projet
2. Créer une branche (`git checkout -b feature/AmazingFeature`)
3. Commit vos changements (`git commit -m 'Add AmazingFeature'`)
4. Push sur la branche (`git push origin feature/AmazingFeature`)
5. Ouvrir une Pull Request

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier `LICENSE` pour plus de détails.

## 🐛 Problèmes connus

- La synchronisation initiale peut prendre du temps pour les boîtes mail volumineuses
- Certains formats d'email complexes peuvent ne pas s'afficher correctement

## 📞 Support

Pour toute question ou problème, ouvrez une issue sur le dépôt GitHub.
=======
# missive
an email client
>>>>>>> eec4383af452df986ec3437cf87a279abb2d463e
