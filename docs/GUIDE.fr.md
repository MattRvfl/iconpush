# Guide iconpush (débutants bienvenus)

Ce guide t'explique pas à pas comment mettre des icônes personnalisées sur ton iPhone avec iconpush. Aucune connaissance technique n'est nécessaire.

## Sommaire

1. [Comment ça marche ?](#1-comment-ça-marche-)
2. [Ce qu'il te faut](#2-ce-quil-te-faut)
3. [Méthode 1 : l'application Windows (USB)](#3-méthode-1--lapplication-windows-usb)
4. [Méthode 2 : la version web (sans installer)](#4-méthode-2--la-version-web-sans-installer)
5. [Installer le profil sur l'iPhone](#5-installer-le-profil-sur-liphone)
6. [Ranger ton écran d'accueil](#6-ranger-ton-écran-daccueil)
7. [Mettre tes propres images](#7-mettre-tes-propres-images)
8. [Changer ou supprimer les icônes](#8-changer-ou-supprimer-les-icônes)
9. [Problèmes fréquents](#9-problèmes-fréquents)
10. [Lexique](#10-lexique)

---

## 1. Comment ça marche ?

Sans jailbreak, iOS ne permet pas de **remplacer** l'icône d'une app. Par contre, il permet d'**ajouter** des raccourcis sur l'écran d'accueil, avec l'image et le nom de ton choix. iconpush crée ces raccourcis pour toi : quand tu touches l'icône, elle ouvre l'app.

Ces raccourcis sont regroupés dans un **profil de configuration** : un petit fichier que l'iPhone sait installer, le même genre de fichier qu'utilisent les entreprises pour configurer leurs téléphones. Un seul profil peut contenir des dizaines d'icônes.

> 💡 Ensuite, tu caches les icônes d'origine dans la **Bibliothèque d'apps**, et ton écran d'accueil n'affiche plus que tes nouvelles icônes.

---

## 2. Ce qu'il te faut

| | Méthode 1 (USB) | Méthode 2 (web) |
| --- | --- | --- |
| Un iPhone sous iOS 14 ou plus récent | ✅ | ✅ |
| Un PC Windows 10 ou 11 | ✅ | ❌ (n'importe quel appareil) |
| Un câble USB pour l'iPhone | ✅ | ❌ |
| L'app « Appareils Apple » ou iTunes | ✅ | ❌ |

La méthode USB est la plus pratique : elle voit les apps installées sur ton iPhone et envoie les icônes directement.

---

## 3. Méthode 1 : l'application Windows (USB)

### Étape 1 : permettre à Windows de reconnaître l'iPhone

Windows a besoin du **pilote USB d'Apple**. iconpush le détecte tout seul : si la pastille en haut à droite affiche « Pilote Apple manquant », clique dessus (ou ouvre l'onglet **Pilotes**). Tu as deux choix :

- **iTunes 64 bits** : bouton « Télécharger et installer ». iconpush le télécharge depuis apple.com, vérifie qu'il est bien signé par Apple, puis lance l'installation (Windows te demande d'accepter).
- **Appareils Apple** : bouton « Ouvrir le Microsoft Store », puis « Installer ».

> Pourquoi ? Sans ce pilote, aucun logiciel ne peut voir un iPhone sur Windows. Il s'installe sur C:, comme tous les pilotes.

### Étape 2 : télécharger iconpush

1. Va sur la page des [versions](https://github.com/MattRvfl/iconpush/releases/latest).
2. Télécharge **iconpush.exe**.
3. Double-clique dessus. Pas besoin d'installer quoi que ce soit.

> ⚠️ Windows peut afficher « Windows a protégé votre ordinateur », parce que l'application est nouvelle et non signée. Clique sur **Informations complémentaires**, puis sur **Exécuter quand même**. Le code source est public sur GitHub, tu peux le vérifier.

La fenêtre d'iconpush s'ouvre. En haut à droite, une pastille indique l'état de la connexion avec ton iPhone.

### Étape 3 : brancher l'iPhone

1. Branche ton iPhone avec le câble.
2. **Déverrouille-le.**
3. S'il demande « **Se fier à cet ordinateur ?** », touche **Se fier** et entre ton code.

En haut à droite, tu dois voir le nom de ton iPhone et sa version d'iOS, avec un point vert.

### Étape 4 : choisir et envoyer

1. **Choisis un pack** à gauche. Tu peux chercher par style : « glass », « minimal », « néon »…
2. **Coche tes apps** au milieu. Par défaut, seules les apps installées sur ton iPhone sont affichées.
3. Vérifie l'**aperçu** à droite.
4. Clique sur **Envoyer sur [ton iPhone]**.

Pas de câble sous la main ? **Enregistrer le fichier .mobileconfig…** crée le profil, et tu l'envoies ensuite par AirDrop, iCloud Drive ou mail.

Passe ensuite à la partie 5.

---

## 4. Méthode 2 : la version web (sans installer)

1. Ouvre la [version web](https://mattrvfl.github.io/iconpush/).
   - **Le plus simple :** ouvre-la directement **sur ton iPhone, dans Safari**.
2. Choisis un pack et tes apps, comme ci-dessus.
3. Touche **⬇ Télécharger le profil**.
   - **Sur iPhone (Safari) :** touche **Autoriser** quand Safari le demande.
   - **Sur ordinateur :** envoie le fichier `.mobileconfig` sur ton iPhone (AirDrop, iCloud Drive ou mail), puis ouvre-le depuis l'app **Fichiers** ou **Mail**.

> ⚠️ Sur iPhone, utilise **Safari**. Les navigateurs intégrés aux apps (Instagram, WhatsApp…) et parfois Chrome bloquent l'installation des profils.

---

## 5. Installer le profil sur l'iPhone

Quelle que soit la méthode, il reste une étape **sur l'iPhone**. Apple l'impose pour ta sécurité, et aucune application ne peut la faire à ta place.

1. Ouvre **Réglages**.
2. Tout en haut, touche **Profil téléchargé**.
   - S'il n'y est pas : **Réglages → Général → VPN et gestion de l'appareil**.
3. Touche **Installer** (en haut à droite), entre ton code, puis **Installer** encore une fois.
4. L'avertissement « **Non vérifié** » est normal : le profil n'est pas signé par une entreprise.
5. Retourne sur l'écran d'accueil : tes nouvelles icônes sont là ! 🎉

> Le profil ne contient **que** des raccourcis d'icônes. Il ne peut pas lire tes données, ni installer d'app, ni changer tes réglages.

---

## 6. Ranger ton écran d'accueil

Pour ne garder que tes nouvelles icônes :

1. Appuie longuement sur l'icône d'**origine** d'une app.
2. Choisis **Supprimer l'app** puis **Retirer de l'écran d'accueil**. L'app n'est **pas** supprimée : elle reste dans la Bibliothèque d'apps (dernière page).
3. Déplace tes nouvelles icônes où tu veux, comme n'importe quelle icône.

**Astuce :** coche « Masquer les noms sous les icônes » dans iconpush pour un écran d'accueil sans texte.

---

## 7. Mettre tes propres images

### Les logos officiels

Dans l'onglet **Bibliothèque**, clique sur **Télécharger les logos**. iconpush récupère les vrais logos (Spotify, Instagram, WhatsApp…) depuis [Simple Icons](https://simpleicons.org), une bibliothèque open source, et ajoute 3 packs : **Logos officiels**, **Logos sur noir** et **Logos Liquid Glass**.

### Une image pour une app

Sur chaque app, clique sur **Image** (dans l'application Windows) ou 🖼 (dans la version web) et choisis une image (PNG, JPG ou WebP). Elle remplace l'icône générée par le pack.

### Tout un dossier (Figma, Canva…)

Dans l'onglet **Bibliothèque**, clique sur **Choisir un dossier…**. iconpush associe chaque image à son app grâce au nom du fichier : `spotify.png`, `Instagram.png`, `Google Maps.png`…

Depuis **Figma** : renomme chaque icône comme l'app, sélectionne-les, puis **Export → PNG → 3x**.

### Partager ton setup

Le bouton **Exporter l'aperçu** enregistre l'iPhone de l'aperçu en image PNG, prête à poster.

- Utilise de préférence une image **carrée** d'au moins 180 × 180 pixels.
- iOS arrondit les coins tout seul.
- Tu peux mélanger : des images à toi pour certaines apps, le pack pour les autres.

---

## 8. Changer ou supprimer les icônes

iOS ne permet pas de modifier un profil déjà installé. Pour changer de style :

1. **Supprime l'ancien profil :** Réglages → Général → **VPN et gestion de l'appareil** → choisis le profil iconpush → **Supprimer le profil**. Toutes ses icônes disparaissent.
2. **Envoie le nouveau** avec iconpush.

Tu peux aussi supprimer une seule icône : appui long → **Supprimer**.

---

## 9. Problèmes fréquents

**« Windows ne voit pas les iPhone »**
Installe l'app **Appareils Apple** (Microsoft Store) ou iTunes, puis relance iconpush.

**Le point reste orange, avec « Déverrouille ton iPhone et touche Se fier »**
Déverrouille l'iPhone. Si la question « Se fier à cet ordinateur ? » n'apparaît pas, débranche et rebranche le câble.

**« Ce PC n'est plus autorisé »**
Sur l'iPhone : Réglages → Général → Transférer ou réinitialiser → Réinitialiser → **Réinitialiser localisation et confidentialité**. Rebranche ensuite, et touche Se fier.

**L'icône ouvre Safari au lieu de l'app, ou ne fait rien**
- Vérifie que l'app est bien installée.
- Certaines apps ont changé leur « lien interne » (URL scheme). Dans iconpush, ces apps sont marquées « non testé ». [Signale-le](https://github.com/MattRvfl/iconpush/issues) en précisant ta version d'iOS.

**« Profil téléchargé » n'apparaît pas dans Réglages**
Un profil téléchargé expire au bout de 8 minutes s'il n'est pas installé. Renvoie-le depuis iconpush.

**Windows bloque iconpush.exe (SmartScreen)**
Clique sur « Informations complémentaires » puis « Exécuter quand même ». C'est normal pour une application récente et non signée.

---

## 10. Lexique

| Mot | Signification |
| --- | --- |
| **Profil de configuration** | Fichier `.mobileconfig` que l'iPhone sait installer. Il contient ici tes icônes. |
| **Web Clip** | Le nom technique d'un raccourci d'écran d'accueil défini dans un profil. |
| **URL scheme** | Le « lien interne » qui ouvre une app, par exemple `spotify://`. |
| **Bundle ID** | L'identifiant unique d'une app, par exemple `com.spotify.client`. Il sert à savoir si l'app est installée. |
| **Se fier à cet ordinateur** | L'autorisation que l'iPhone donne à ton PC pour communiquer avec lui en USB. |
| **Jailbreak** | Modification qui retire les protections d'iOS. iconpush n'en a **pas** besoin. |

---

Une question ou un bug ? [Ouvre une issue](https://github.com/MattRvfl/iconpush/issues) : toutes les questions sont les bienvenues.
